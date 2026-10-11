//! Pod self-stop watchdog.
//!
//! On RunPod the pod stops itself after IDLE_TIMEOUT seconds without
//! WebSocket activity, so a GPU never keeps billing when the coordinator is
//! asleep, crashed or gone. Only active when RUNPOD_POD_ID is set (RunPod
//! injects it), so local CPU/mock runs are unaffected.

use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};

use crate::state::AppState;

const RUNPOD_GRAPHQL_URL: &str = "https://api.runpod.io/graphql";
const DEFAULT_IDLE_TIMEOUT_SECS: i64 = 600;
const CHECK_INTERVAL: Duration = Duration::from_secs(30);
/// After a successful podStop, wait this long before asking again (in case
/// RunPod accepted the request but the container is somehow still running)
const RESEND_AFTER_SECS: i64 = 300;

/// Start the watchdog task if we're running on RunPod
pub fn spawn(state: Arc<AppState>) {
    let pod_id = match std::env::var("RUNPOD_POD_ID") {
        Ok(id) if !id.is_empty() => id,
        _ => return, // Not on RunPod — nothing to stop
    };

    let api_key = match std::env::var("RUNPOD_API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => {
            println!(
                "[watchdog] WARNING: RUNPOD_POD_ID is set but RUNPOD_API_KEY is missing — \
                 idle self-stop DISABLED, this pod will bill until stopped externally"
            );
            return;
        }
    };

    let timeout = parse_timeout(std::env::var("IDLE_TIMEOUT").ok().as_deref());
    println!("[watchdog] pod {} will stop itself after {}s idle", pod_id, timeout);

    tokio::spawn(run(state, pod_id, api_key, timeout));
}

async fn run(state: Arc<AppState>, pod_id: String, api_key: String, timeout: i64) {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            println!("[watchdog] ERROR: could not build HTTP client, self-stop disabled: {}", e);
            return;
        }
    };

    let mut ticker = tokio::time::interval(CHECK_INTERVAL);
    let mut stop_requested_at: Option<i64> = None;

    loop {
        ticker.tick().await;

        let now = chrono::Utc::now().timestamp();
        let last = state.last_activity_ts();

        if !is_idle(now, last, timeout) {
            // Activity resumed (or never stopped) — forget any earlier request
            stop_requested_at = None;
            continue;
        }

        if let Some(at) = stop_requested_at {
            if now - at < RESEND_AFTER_SECS {
                continue;
            }
        }

        println!("[watchdog] idle for {}s (limit {}s) — requesting podStop", now - last, timeout);
        match stop_pod(&client, &pod_id, &api_key).await {
            Ok(body) => {
                println!("[watchdog] podStop accepted: {}", body);
                stop_requested_at = Some(now);
            }
            // Retry on the next tick
            Err(e) => println!("[watchdog] podStop failed, will retry: {}", e),
        }
    }
}

/// Idle when no activity for at least `timeout` seconds
fn is_idle(now: i64, last_activity: i64, timeout: i64) -> bool {
    now - last_activity >= timeout
}

/// Parse IDLE_TIMEOUT (seconds); falls back to the default on missing/invalid
fn parse_timeout(raw: Option<&str>) -> i64 {
    match raw.map(|s| s.trim().parse::<i64>()) {
        Some(Ok(secs)) if secs > 0 => secs,
        Some(_) => {
            println!(
                "[watchdog] invalid IDLE_TIMEOUT {:?}, using {}s",
                raw.unwrap_or_default(),
                DEFAULT_IDLE_TIMEOUT_SECS
            );
            DEFAULT_IDLE_TIMEOUT_SECS
        }
        None => DEFAULT_IDLE_TIMEOUT_SECS,
    }
}

/// Ask RunPod to stop this pod. Returns the response body on success.
async fn stop_pod(client: &reqwest::Client, pod_id: &str, api_key: &str) -> Result<String, String> {
    let resp = client
        .post(RUNPOD_GRAPHQL_URL)
        .bearer_auth(api_key)
        .json(&stop_request_body(pod_id))
        .send()
        .await
        .map_err(|e| format!("request error: {}", e))?;

    let status = resp.status();
    let body = resp.text().await.map_err(|e| format!("reading body: {}", e))?;
    check_response(status.as_u16(), &body)?;
    Ok(body)
}

fn stop_request_body(pod_id: &str) -> Value {
    json!({
        "query": format!(
            "mutation {{ podStop(input: {{podId: \"{}\"}}) {{ id desiredStatus }} }}",
            pod_id
        )
    })
}

/// GraphQL reports failures as HTTP 200 with an `errors` array, so check both
fn check_response(status: u16, body: &str) -> Result<(), String> {
    if !(200..300).contains(&status) {
        return Err(format!("HTTP {}: {}", status, body));
    }
    let parsed: Value = serde_json::from_str(body).map_err(|e| format!("bad JSON ({}): {}", e, body))?;
    if parsed.get("errors").is_some_and(|e| !e.is_null()) {
        return Err(format!("GraphQL errors: {}", body));
    }
    if parsed.pointer("/data/podStop").map_or(true, Value::is_null) {
        return Err(format!("no podStop in response: {}", body));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_only_after_full_timeout() {
        assert!(!is_idle(1000, 1000, 600));
        assert!(!is_idle(1599, 1000, 600));
        assert!(is_idle(1600, 1000, 600));
        assert!(is_idle(5000, 1000, 600));
    }

    #[test]
    fn timeout_parsing() {
        assert_eq!(parse_timeout(None), 600);
        assert_eq!(parse_timeout(Some("120")), 120);
        assert_eq!(parse_timeout(Some(" 90 ")), 90);
        assert_eq!(parse_timeout(Some("0")), 600);
        assert_eq!(parse_timeout(Some("-5")), 600);
        assert_eq!(parse_timeout(Some("ten")), 600);
    }

    #[test]
    fn request_body_shape() {
        let body = stop_request_body("abc123");
        assert_eq!(
            body["query"],
            "mutation { podStop(input: {podId: \"abc123\"}) { id desiredStatus } }"
        );
    }

    #[test]
    fn response_checking() {
        let ok = r#"{"data":{"podStop":{"id":"abc123","desiredStatus":"EXITED"}}}"#;
        assert!(check_response(200, ok).is_ok());

        let gql_err = r#"{"errors":[{"message":"not authorized"}],"data":{"podStop":null}}"#;
        assert!(check_response(200, gql_err).is_err());

        assert!(check_response(401, "Unauthorized").is_err());
        assert!(check_response(200, "<html>").is_err());
        assert!(check_response(200, r#"{"data":null}"#).is_err());
    }
}
