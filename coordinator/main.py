"""
Veiled Court — Coordinator

Serves the frontend, manages RunPod GPU pods on demand.
Uses stop/resume for fast restarts instead of create/terminate.
"""

import asyncio
import time
import os
import httpx
from contextlib import asynccontextmanager
from fastapi import FastAPI
from fastapi.responses import HTMLResponse, JSONResponse, Response
from dotenv import load_dotenv

load_dotenv(os.path.join(os.path.dirname(__file__), "..", ".env"))

# ── config ──

RUNPOD_API_KEY = os.environ["RUNPOD_API_KEY"]
DOCKER_IMAGE = os.getenv("DOCKER_IMAGE", "ghcr.io/plut012/kitsune:latest")
GPU_TYPES = os.getenv("RUNPOD_GPU_TYPES",
    "NVIDIA RTX 4000 Ada Generation,"
    "NVIDIA GeForce RTX 3090,"
    "NVIDIA RTX A4000,"
    "NVIDIA GeForce RTX 4070 Ti,"
    "NVIDIA RTX A5000"
).split(",")
IDLE_TIMEOUT = int(os.getenv("IDLE_TIMEOUT", "600"))  # 10 min
POLL_INTERVAL = 30

RUNPOD_API = "https://api.runpod.io/graphql"

# ── state ──

pod_state = {
    "id": None,         # persisted pod ID (survives stop/resume)
    "url": None,
    "status": "off",    # off | starting | ready
    "ready_at": None,   # when pod became ready (for grace period)
}


# ── RunPod API ──

async def runpod_gql(query: str) -> dict:
    async with httpx.AsyncClient() as client:
        resp = await client.post(
            RUNPOD_API,
            json={"query": query},
            headers={"Authorization": f"Bearer {RUNPOD_API_KEY}"},
            timeout=30,
        )
        return resp.json()


async def create_pod() -> str:
    """Create a new pod. Tries multiple GPU types."""
    last_error = "No GPU types configured"
    for gpu_type in GPU_TYPES:
        gpu_type = gpu_type.strip()
        query = f"""mutation {{
            podFindAndDeployOnDemand(input: {{
                cloudType: COMMUNITY,
                gpuCount: 1,
                gpuTypeId: "{gpu_type}",
                volumeInGb: 0,
                containerDiskInGb: 15,
                minVcpuCount: 2,
                minMemoryInGb: 8,
                name: "kitsune-game",
                imageName: "{DOCKER_IMAGE}",
                ports: "3000/http",
                env: [{{ key: "RUST_LOG", value: "info" }}]
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
        f'mutation {{ podResume(input: {{ podId: "{pod_id}", gpuCount: 1 }}) {{ id desiredStatus }} }}'
    )
    if "errors" in result:
        print(f"[coordinator] resume failed: {result['errors'][0]['message']}")
        return False
    print(f"[coordinator] resuming pod {pod_id}")
    return True


async def stop_pod(pod_id: str):
    """Stop a pod (preserves it for fast resume)."""
    await runpod_gql(f'mutation {{ podStop(input: {{ podId: "{pod_id}" }}) {{ id desiredStatus }} }}')
    print(f"[coordinator] stopped pod {pod_id}")


async def terminate_pod(pod_id: str):
    """Terminate a pod permanently."""
    await runpod_gql(f'mutation {{ podTerminate(input: {{ podId: "{pod_id}" }}) }}')
    print(f"[coordinator] terminated pod {pod_id}")


async def get_pod_status(pod_id: str) -> str | None:
    """Get pod's desired status. Returns None if pod doesn't exist."""
    result = await runpod_gql(
        f'{{ pod(input: {{ podId: "{pod_id}" }}) {{ desiredStatus }} }}'
    )
    pod = result.get("data", {}).get("pod")
    return pod["desiredStatus"] if pod else None


async def check_pod_health(pod_id: str) -> str | None:
    """Returns proxy URL if pod is healthy, None otherwise."""
    url = f"https://{pod_id}-3000.proxy.runpod.net"
    try:
        async with httpx.AsyncClient() as client:
            resp = await client.get(f"{url}/health", timeout=5)
            if resp.status_code == 200:
                return url
    except (httpx.ConnectError, httpx.ReadTimeout):
        pass
    return None


# ── pod lifecycle ──

async def ensure_pod_running() -> str:
    """Get a running pod — resume if stopped, create if needed. Returns pod ID."""

    # If we have a pod, try resuming it
    if pod_state["id"]:
        status = await get_pod_status(pod_state["id"])
        if status == "RUNNING":
            return pod_state["id"]
        if status == "EXITED":
            if await resume_pod(pod_state["id"]):
                return pod_state["id"]
            # Resume failed — pod's host is gone. Terminate and create fresh.
            await terminate_pod(pod_state["id"])
            pod_state["id"] = None

    # Create a new pod
    pod_id = await create_pod()
    pod_state["id"] = pod_id
    return pod_id


# ── watchdog ──

async def idle_watchdog():
    """Stop (not terminate) pods that have been idle too long."""
    while True:
        await asyncio.sleep(POLL_INTERVAL)
        if pod_state["status"] != "ready" or not pod_state["url"]:
            continue

        # Grace period: don't touch a pod for 5 min after it became ready
        ready_at = pod_state.get("ready_at") or 0
        if time.time() - ready_at < 300:
            continue

        try:
            async with httpx.AsyncClient() as client:
                resp = await client.get(f"{pod_state['url']}/activity", timeout=5)
                idle_seconds = resp.json().get("idle_seconds", 0)
        except Exception:
            idle_seconds = IDLE_TIMEOUT + 1

        if idle_seconds >= IDLE_TIMEOUT:
            pod_id = pod_state["id"]
            pod_state["url"] = None
            pod_state["status"] = "off"
            pod_state["ready_at"] = None
            # Stop instead of terminate — preserves for fast resume
            if pod_id:
                await stop_pod(pod_id)


# ── startup / shutdown ──

@asynccontextmanager
async def lifespan(app: FastAPI):
    task = asyncio.create_task(idle_watchdog())
    print("[coordinator] started, watchdog running")
    yield
    task.cancel()
    # Stop (not terminate) on shutdown — pod can be resumed next time
    if pod_state["id"] and pod_state["status"] == "ready":
        await stop_pod(pod_state["id"])
        print(f"[coordinator] stopped pod {pod_state['id']} on shutdown")


# ── app ──

app = FastAPI(lifespan=lifespan)


@app.post("/summon")
async def summon():
    """Start or resume a GPU pod."""
    if pod_state["status"] == "ready" and pod_state["url"]:
        return {"status": "ready", "url": pod_state["url"]}

    if pod_state["status"] == "starting":
        return {"status": "starting"}

    pod_state["status"] = "starting"

    try:
        pod_id = await ensure_pod_running()
    except RuntimeError as e:
        pod_state["status"] = "off"
        return JSONResponse({"status": "error", "message": str(e)}, status_code=503)

    asyncio.create_task(poll_pod_ready(pod_id))
    return {"status": "starting"}


async def poll_pod_ready(pod_id: str):
    """Poll until pod is healthy."""
    for _ in range(120):  # 10 min max
        if pod_state["id"] != pod_id:
            return
        url = await check_pod_health(pod_id)
        if url:
            pod_state["url"] = url
            pod_state["status"] = "ready"
            pod_state["ready_at"] = time.time()
            print(f"[coordinator] pod {pod_id} ready at {url}")
            return
        await asyncio.sleep(5)

    print(f"[coordinator] pod {pod_id} timed out waiting for health")
    pod_state["status"] = "off"
    await stop_pod(pod_id)


@app.get("/status")
async def status():
    """Pod status for frontend polling."""
    resp = {
        "status": pod_state["status"],
        "ready": pod_state["status"] == "ready",
    }
    if pod_state["status"] == "ready" and pod_state["url"]:
        resp["url"] = pod_state["url"]
    return resp


# ── serve frontend ──

FRONTEND_DIR = os.path.join(os.path.dirname(__file__), "..", "frontend")
if not os.path.isdir(FRONTEND_DIR):
    FRONTEND_DIR = os.path.join(os.getcwd(), "frontend")


@app.get("/", response_class=HTMLResponse)
async def entrance():
    path = os.path.join(FRONTEND_DIR, "entrance.html")
    if not os.path.exists(path):
        return HTMLResponse("<h1>Veiled Court</h1><p>Frontend not found</p>", status_code=500)
    with open(path) as f:
        return f.read()


@app.get("/{path:path}")
async def serve_frontend(path: str):
    file_path = os.path.join(FRONTEND_DIR, path)
    if os.path.isfile(file_path):
        import mimetypes
        content_type = mimetypes.guess_type(file_path)[0] or "application/octet-stream"
        with open(file_path, "rb") as f:
            return Response(content=f.read(), media_type=content_type)
    return JSONResponse({"error": "not found"}, status_code=404)
