use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde::{Deserialize, Serialize};

/// Analysis result for a single position in the game
#[derive(Debug, Clone, Serialize)]
pub struct PositionAnalysis {
    pub move_number: usize,
    pub winrate: f64,
    pub score_lead: f64,
    pub ownership: Vec<f64>,
    pub best_move: String,
    pub played_move: String,
}

/// KataGo process running in analysis mode (JSON protocol)
pub struct KataGoAnalysis {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

/// JSON query sent to KataGo analysis engine
#[derive(Serialize)]
struct AnalysisQuery {
    id: String,
    moves: Vec<(String, String)>, // [["B","D4"],["W","Q16"],...]
    rules: String,
    komi: f64,
    #[serde(rename = "boardXSize")]
    board_x_size: usize,
    #[serde(rename = "boardYSize")]
    board_y_size: usize,
    #[serde(rename = "includeOwnership")]
    include_ownership: bool,
}

/// JSON response from KataGo analysis engine
#[derive(Deserialize)]
struct AnalysisResponse {
    id: String,
    #[serde(rename = "moveInfos")]
    move_infos: Option<Vec<MoveInfo>>,
    #[serde(rename = "rootInfo")]
    root_info: Option<RootInfo>,
    ownership: Option<Vec<f64>>,
}

#[derive(Deserialize)]
struct MoveInfo {
    #[serde(rename = "move")]
    move_coord: String,
    // other fields exist but we only need the top move
}

#[derive(Deserialize)]
struct RootInfo {
    winrate: f64,
    #[serde(rename = "scoreLead")]
    score_lead: f64,
}

impl KataGoAnalysis {
    /// Spawn a KataGo process in analysis mode
    pub fn spawn(config_path: &str) -> Result<Self, String> {
        let binary_path = PathBuf::from(
            std::env::var("KATAGO_BINARY").unwrap_or_else(|_| "assets/katago/katago".to_string()),
        );
        let model_path = PathBuf::from(
            std::env::var("KATAGO_MODEL")
                .unwrap_or_else(|_| "assets/katago/model.bin.gz".to_string()),
        );

        if !binary_path.exists() {
            return Err(format!("KataGo binary not found at {:?}", binary_path));
        }
        if !model_path.exists() {
            return Err(format!("KataGo model not found at {:?}", model_path));
        }

        let config = PathBuf::from(config_path);
        if !config.exists() {
            return Err(format!("Analysis config not found at {:?}", config));
        }

        let mut child = Command::new(&binary_path)
            .arg("analysis")
            .arg("-model")
            .arg(&model_path)
            .arg("-config")
            .arg(&config)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to spawn KataGo analysis: {}", e))?;

        let stdin = child
            .stdin
            .take()
            .ok_or("Failed to open KataGo analysis stdin")?;
        let stdout = child
            .stdout
            .take()
            .ok_or("Failed to open KataGo analysis stdout")?;

        Ok(Self {
            process: child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    /// Analyze a complete game. Returns analysis for each position (move 0 through N).
    ///
    /// `moves` is a list of (color, gtp_coord) pairs, e.g. [("B","D4"), ("W","Q16"), ...]
    pub fn analyze_game(
        &mut self,
        moves: &[(String, String)],
        board_size: usize,
    ) -> Result<Vec<PositionAnalysis>, String> {
        let total_positions = moves.len() + 1; // include the empty board

        // Batch-write all queries — KataGo pipelines them
        for k in 0..total_positions {
            let query = AnalysisQuery {
                id: format!("pos_{}", k),
                moves: moves[..k].to_vec(),
                rules: "chinese".to_string(),
                komi: 7.5,
                board_x_size: board_size,
                board_y_size: board_size,
                include_ownership: true,
            };

            let json = serde_json::to_string(&query)
                .map_err(|e| format!("Failed to serialize query: {}", e))?;

            writeln!(self.stdin, "{}", json)
                .map_err(|e| format!("Failed to write to KataGo analysis: {}", e))?;
        }

        self.stdin
            .flush()
            .map_err(|e| format!("Failed to flush analysis stdin: {}", e))?;

        // Read all responses — they may arrive out of order
        let mut results: HashMap<usize, AnalysisResponse> = HashMap::new();

        while results.len() < total_positions {
            let mut line = String::new();
            self.stdout
                .read_line(&mut line)
                .map_err(|e| format!("Failed to read from KataGo analysis: {}", e))?;

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Skip non-JSON lines (startup banners, warnings)
            let response: AnalysisResponse = match serde_json::from_str(line) {
                Ok(r) => r,
                Err(_) => continue,
            };

            // Parse move number from id "pos_N"
            if let Some(num_str) = response.id.strip_prefix("pos_") {
                if let Ok(n) = num_str.parse::<usize>() {
                    results.insert(n, response);
                }
            }
        }

        // Assemble into ordered Vec<PositionAnalysis>
        let mut analysis = Vec::with_capacity(total_positions);

        for k in 0..total_positions {
            let resp = results
                .remove(&k)
                .ok_or(format!("Missing analysis for position {}", k))?;

            let root = resp
                .root_info
                .ok_or(format!("Missing rootInfo for position {}", k))?;

            let best_move = resp
                .move_infos
                .as_ref()
                .and_then(|infos| infos.first())
                .map(|m| m.move_coord.clone())
                .unwrap_or_else(|| "pass".to_string());

            let played_move = if k > 0 {
                moves[k - 1].1.clone()
            } else {
                String::new()
            };

            let ownership = resp.ownership.unwrap_or_default();

            analysis.push(PositionAnalysis {
                move_number: k,
                winrate: root.winrate,
                score_lead: root.score_lead,
                ownership,
                best_move,
                played_move,
            });
        }

        Ok(analysis)
    }
}

impl Drop for KataGoAnalysis {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}
