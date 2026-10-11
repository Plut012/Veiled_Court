"""
Veiled Court — Coordinator

Serves the frontend and wakes the RunPod GPU pod on demand.

Stateless: the pod is looked up by name on every call, so restarts and
multiple coordinators share one pod. The pod stops itself when idle.
"""

import asyncio
import hmac
import json
import mimetypes
import os
import sys
import time

import httpx
from fastapi import FastAPI, Request
from fastapi.responses import HTMLResponse, JSONResponse, Response
from dotenv import load_dotenv

load_dotenv(os.path.join(os.path.dirname(__file__), "..", ".env"))

# ── config ──

def require_env(name: str) -> str:
    value = os.getenv(name, "")
    if not value:
        sys.exit(f"[coordinator] {name} is not set. Add it to .env (see scripts/run-coordinator.sh).")
    return value


RUNPOD_API_KEY = require_env("RUNPOD_API_KEY")
COURT_PASSWORD = require_env("COURT_PASSWORD")
DOCKER_IMAGE = os.getenv("DOCKER_IMAGE", "ghcr.io/plut012/kitsune:latest")
GPU_TYPES = os.getenv("RUNPOD_GPU_TYPES",
    "NVIDIA RTX 4000 Ada Generation,"
    "NVIDIA GeForce RTX 3090,"
    "NVIDIA RTX A4000,"
    "NVIDIA GeForce RTX 4070 Ti,"
    "NVIDIA RTX A5000"
).split(",")
IDLE_TIMEOUT = os.getenv("IDLE_TIMEOUT", "600")  # seconds; enforced by the pod itself
BOOT_TIMEOUT = 600   # stop a pod that never becomes healthy after this long
STATUS_CACHE = 3     # seconds

POD_NAME = "kitsune-game"
RUNPOD_API = "https://api.runpod.io/graphql"


# ── RunPod API ──

def gql_str(value: str) -> str:
    """Quote a value as a GraphQL string literal."""
    return json.dumps(value)


async def runpod_gql(query: str) -> dict:
    """Run a GraphQL query. Transport/HTTP failures come back as `errors` too."""
    try:
        async with httpx.AsyncClient() as client:
            resp = await client.post(
                RUNPOD_API,
                json={"query": query},
                headers={"Authorization": f"Bearer {RUNPOD_API_KEY}"},
                timeout=30,
            )
        result = resp.json()
    except (httpx.HTTPError, ValueError) as e:
        return {"errors": [{"message": f"RunPod API unreachable: {e}"}]}
    if resp.status_code != 200 and "errors" not in result:
        return {"errors": [{"message": f"RunPod API returned {resp.status_code}"}]}
    return result


async def find_pods() -> list[dict]:
    """All of our (non-terminated) pods, by name."""
    result = await runpod_gql("query { myself { pods { id name desiredStatus } } }")
    if "errors" in result:
        raise RuntimeError(result["errors"][0]["message"])
    return [
        p for p in ((result.get("data") or {}).get("myself") or {}).get("pods") or []
        if p["name"] == POD_NAME and p["desiredStatus"] != "TERMINATED"
    ]


async def find_pod() -> dict | None:
    """Look up our pod by name. Prefers a RUNNING one if several exist."""
    pods = await find_pods()
    if not pods:
        return None
    pods.sort(key=lambda p: p["desiredStatus"] != "RUNNING")
    if len(pods) > 1:
        extra = ", ".join(f"{p['id']} ({p['desiredStatus']})" for p in pods[1:])
        print(f"[coordinator] several '{POD_NAME}' pods, using {pods[0]['id']}; others: {extra}")
    return pods[0]


async def create_pod() -> str:
    """Create a new pod. Tries multiple GPU types."""
    env = ", ".join(
        f"{{ key: {gql_str(k)}, value: {gql_str(v)} }}"
        for k, v in [("RUST_LOG", "info"), ("RUNPOD_API_KEY", RUNPOD_API_KEY), ("IDLE_TIMEOUT", IDLE_TIMEOUT)]
    )
    last_error = "No GPU types configured"
    for gpu_type in GPU_TYPES:
        gpu_type = gpu_type.strip()
        query = f"""mutation {{
            podFindAndDeployOnDemand(input: {{
                cloudType: COMMUNITY,
                gpuCount: 1,
                gpuTypeId: {gql_str(gpu_type)},
                volumeInGb: 0,
                containerDiskInGb: 15,
                minVcpuCount: 2,
                minMemoryInGb: 8,
                name: {gql_str(POD_NAME)},
                imageName: {gql_str(DOCKER_IMAGE)},
                ports: "3000/http",
                env: [{env}]
            }}) {{ id desiredStatus }}
        }}"""
        result = await runpod_gql(query)
        if "errors" not in result:
            pod_id = result["data"]["podFindAndDeployOnDemand"]["id"]
            print(f"[coordinator] created pod {pod_id} on {gpu_type}")
            return pod_id
        last_error = result["errors"][0]["message"]
        print(f"[coordinator] {gpu_type}: {last_error}")
    raise RuntimeError(last_error)


async def resume_pod(pod_id: str) -> bool:
    """Resume a stopped pod. Returns True if successful."""
    result = await runpod_gql(
        f'mutation {{ podResume(input: {{ podId: {gql_str(pod_id)}, gpuCount: 1 }}) {{ id desiredStatus }} }}'
    )
    if "errors" in result:
        print(f"[coordinator] resume failed: {result['errors'][0]['message']}")
        return False
    print(f"[coordinator] resuming pod {pod_id}")
    return True


async def stop_pod(pod_id: str) -> bool:
    """Stop a pod (preserves it for fast resume). Returns True if RunPod accepted."""
    result = await runpod_gql(f'mutation {{ podStop(input: {{ podId: {gql_str(pod_id)} }}) {{ id desiredStatus }} }}')
    if "errors" in result:
        print(f"[coordinator] stop failed for {pod_id}: {result['errors'][0]['message']}")
        return False
    print(f"[coordinator] stopped pod {pod_id}")
    return True


async def terminate_pod(pod_id: str):
    """Terminate a pod permanently."""
    await runpod_gql(f'mutation {{ podTerminate(input: {{ podId: {gql_str(pod_id)} }}) }}')
    print(f"[coordinator] terminated pod {pod_id}")


def pod_url(pod_id: str) -> str:
    return f"https://{pod_id}-3000.proxy.runpod.net"


async def is_healthy(pod_id: str) -> bool:
    try:
        async with httpx.AsyncClient() as client:
            resp = await client.get(f"{pod_url(pod_id)}/health", timeout=5)
            return resp.status_code == 200
    except httpx.HTTPError:
        return False


async def idle_seconds(pod_id: str) -> int | None:
    """Seconds since the pod's last game activity, or None if it isn't answering."""
    try:
        async with httpx.AsyncClient() as client:
            resp = await client.get(f"{pod_url(pod_id)}/activity", timeout=5)
            if resp.status_code == 200:
                return int(resp.json().get("idle_seconds", 0))
    except (httpx.HTTPError, ValueError):
        pass
    return None


# ── pod state ──

async def pod_status() -> dict:
    """Current pod state: {status, ready, url?, rests_in?}.

    rests_in = seconds until the pod stops itself for idleness (approximate:
    the pod checks every 30s)."""
    pod = await find_pod()
    if not pod or pod["desiredStatus"] != "RUNNING":
        return {"status": "off", "ready": False}
    idle = await idle_seconds(pod["id"])
    if idle is not None:
        return {"status": "ready", "ready": True, "url": pod_url(pod["id"]),
                "rests_in": max(0, int(IDLE_TIMEOUT) - idle)}
    return {"status": "starting", "ready": False}


_status_cache = {"at": 0.0, "value": None}


async def cached_pod_status() -> dict:
    if _status_cache["value"] and time.monotonic() - _status_cache["at"] < STATUS_CACHE:
        return _status_cache["value"]
    value = await pod_status()
    _status_cache.update(at=time.monotonic(), value=value)
    return value


async def wake_pod() -> dict:
    """Resume or create the pod. Returns the resulting status."""
    pod = await find_pod()

    if pod and pod["desiredStatus"] == "RUNNING":
        if await is_healthy(pod["id"]):
            return {"status": "ready", "ready": True, "url": pod_url(pod["id"])}
        watch_boot(pod["id"])
        return {"status": "starting", "ready": False}

    if pod and pod["desiredStatus"] == "EXITED":
        if await resume_pod(pod["id"]):
            watch_boot(pod["id"])
            return {"status": "starting", "ready": False}
        # Resume failed — the pod's host is gone. Start over.
        await terminate_pod(pod["id"])

    pod_id = await create_pod()
    watch_boot(pod_id)
    return {"status": "starting", "ready": False}


# Safety net for pods that never boot (bad image, broken host): the pod can only
# stop itself once the game server runs, so stop it here if it never gets healthy.
_watched: set[str] = set()


def watch_boot(pod_id: str):
    if pod_id not in _watched:
        _watched.add(pod_id)
        asyncio.create_task(_watch_boot(pod_id))


async def _watch_boot(pod_id: str):
    try:
        deadline = time.monotonic() + BOOT_TIMEOUT
        while time.monotonic() < deadline:
            if await is_healthy(pod_id):
                print(f"[coordinator] pod {pod_id} ready at {pod_url(pod_id)}")
                return
            await asyncio.sleep(5)
        print(f"[coordinator] pod {pod_id} never became healthy, stopping it")
        await stop_pod(pod_id)
    finally:
        _watched.discard(pod_id)


# ── app ──

app = FastAPI()
summon_lock = asyncio.Lock()


async def check_password(request: Request) -> JSONResponse | None:
    """None if the body carries the court password, else a 401 response."""
    try:
        body = await request.json()
        password = str(body.get("password", "")) if isinstance(body, dict) else ""
    except ValueError:
        password = ""
    if hmac.compare_digest(password.encode(), COURT_PASSWORD.encode()):
        return None
    if password:
        await asyncio.sleep(1)
    return JSONResponse({"status": "error", "message": "wrong password"}, status_code=401)


@app.post("/summon")
async def summon(request: Request):
    """Wake the GPU pod. Requires the court password."""
    if denied := await check_password(request):
        return denied

    async with summon_lock:
        try:
            result = await wake_pod()
        except RuntimeError as e:
            print(f"[coordinator] summon failed: {e}")
            return JSONResponse({"status": "error", "message": str(e)}, status_code=503)
        _status_cache["value"] = None
    return result


@app.post("/rest")
async def rest(request: Request):
    """Stop the GPU pod now (ends any game in progress). Requires the court password."""
    if denied := await check_password(request):
        return denied

    async with summon_lock:
        _status_cache["value"] = None
        try:
            running = [p for p in await find_pods() if p["desiredStatus"] == "RUNNING"]
        except RuntimeError as e:
            return JSONResponse({"status": "error", "message": str(e)}, status_code=503)
        stopped = [await stop_pod(p["id"]) for p in running]
        if not all(stopped):
            return JSONResponse({"status": "error", "message": "RunPod did not accept the stop"}, status_code=502)
    return {"status": "off", "ready": False}


@app.get("/status")
async def status():
    """Pod status for frontend polling."""
    try:
        return await cached_pod_status()
    except RuntimeError as e:
        print(f"[coordinator] status lookup failed: {e}")
        return {"status": "off", "ready": False, "error": str(e)}


# ── friend rooms (optional module) ──

try:
    from coordinator.rooms import router as rooms_router
    app.include_router(rooms_router)
except ImportError:
    pass


# ── serve frontend ──

FRONTEND_DIR = os.path.join(os.path.dirname(__file__), "..", "frontend")
if not os.path.isdir(FRONTEND_DIR):
    FRONTEND_DIR = os.path.join(os.getcwd(), "frontend")
FRONTEND_ROOT = os.path.realpath(FRONTEND_DIR)


@app.get("/", response_class=HTMLResponse)
async def entrance():
    path = os.path.join(FRONTEND_DIR, "entrance.html")
    if not os.path.exists(path):
        return HTMLResponse("<h1>Veiled Court</h1><p>Frontend not found</p>", status_code=500)
    with open(path) as f:
        return f.read()


@app.get("/{path:path}")
async def serve_frontend(path: str):
    file_path = os.path.realpath(os.path.join(FRONTEND_DIR, path))
    if file_path.startswith(FRONTEND_ROOT + os.sep) and os.path.isfile(file_path):
        content_type = mimetypes.guess_type(file_path)[0] or "application/octet-stream"
        with open(file_path, "rb") as f:
            return Response(content=f.read(), media_type=content_type)
    return JSONResponse({"error": "not found"}, status_code=404)
