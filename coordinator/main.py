"""
Veiled Court — Coordinator

Serves the frontend, manages RunPod GPU pods on demand.
Once a pod is ready, redirects the player to it.
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
GPU_TYPES = os.getenv("RUNPOD_GPU_TYPES", "NVIDIA RTX 4000 Ada Generation,NVIDIA GeForce RTX 3090,NVIDIA RTX A4000,NVIDIA GeForce RTX 4070 Ti,NVIDIA RTX A5000").split(",")
IDLE_TIMEOUT = int(os.getenv("IDLE_TIMEOUT", "600"))  # 10 minutes
POLL_INTERVAL = 30  # seconds between idle checks

RUNPOD_API = "https://api.runpod.io/graphql"

# ── state ──

pod_state = {
    "id": None,
    "url": None,        # e.g. "https://xyz-3000.proxy.runpod.net"
    "status": "off",    # off | starting | ready
    "last_summon": None,
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


async def start_pod() -> str:
    """Start a new game pod. Tries multiple GPU types for availability."""
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
            print(f"[coordinator] started pod {pod_id} on {gpu_type}")
            return pod_id
        last_error = result["errors"][0]["message"]
        print(f"[coordinator] {gpu_type}: {last_error}, trying next...")
    raise RuntimeError(last_error)


async def terminate_pod(pod_id: str):
    """Terminate a pod."""
    await runpod_gql(f'mutation {{ podTerminate(input: {{ podId: "{pod_id}" }}) }}')


async def check_pod_health(pod_id: str) -> str | None:
    """Check if pod is ready. Returns proxy URL if healthy, None otherwise."""
    url = f"https://{pod_id}-3000.proxy.runpod.net"
    try:
        async with httpx.AsyncClient() as client:
            resp = await client.get(f"{url}/health", timeout=5)
            if resp.status_code == 200:
                return url
    except (httpx.ConnectError, httpx.ReadTimeout):
        pass
    return None


# ── watchdog ──

async def idle_watchdog():
    """Kill pods that have been idle too long."""
    while True:
        await asyncio.sleep(POLL_INTERVAL)
        if pod_state["status"] != "ready" or not pod_state["url"]:
            continue

        # Query the pod's own activity endpoint
        try:
            async with httpx.AsyncClient() as client:
                resp = await client.get(f"{pod_state['url']}/activity", timeout=5)
                data = resp.json()
                idle_seconds = data.get("idle_seconds", 0)
        except Exception:
            idle_seconds = IDLE_TIMEOUT + 1  # can't reach → kill it

        if idle_seconds >= IDLE_TIMEOUT:
            pod_id = pod_state["id"]
            pod_state["id"] = None
            pod_state["url"] = None
            pod_state["status"] = "off"
            if pod_id:
                await terminate_pod(pod_id)
                print(f"[watchdog] terminated idle pod {pod_id} ({idle_seconds:.0f}s idle)")


# ── startup / shutdown ──

@asynccontextmanager
async def lifespan(app: FastAPI):
    task = asyncio.create_task(idle_watchdog())
    print("[coordinator] started, watchdog running")
    yield
    task.cancel()
    if pod_state["id"]:
        await terminate_pod(pod_state["id"])
        print(f"[coordinator] terminated pod {pod_state['id']} on shutdown")


# ── app ──

app = FastAPI(lifespan=lifespan)


@app.post("/summon")
async def summon():
    """Start a GPU pod. Called when player enters the court."""
    if pod_state["status"] == "ready":
        return {"status": "ready", "url": pod_state["url"]}

    if pod_state["status"] == "starting":
        return {"status": "starting", "pod_id": pod_state["id"]}

    try:
        pod_id = await start_pod()
    except RuntimeError as e:
        return JSONResponse({"status": "error", "message": str(e)}, status_code=503)

    pod_state["id"] = pod_id
    pod_state["status"] = "starting"
    pod_state["last_summon"] = time.time()

    asyncio.create_task(poll_pod_ready(pod_id))

    return {"status": "starting", "pod_id": pod_id}


async def poll_pod_ready(pod_id: str):
    """Poll until pod is healthy, then update state."""
    for _ in range(120):  # 10 minutes max
        if pod_state["id"] != pod_id:
            return
        url = await check_pod_health(pod_id)
        if url:
            pod_state["url"] = url
            pod_state["status"] = "ready"
            print(f"[coordinator] pod {pod_id} ready at {url}")
            return
        await asyncio.sleep(5)

    print(f"[coordinator] pod {pod_id} failed to become healthy, terminating")
    pod_state["status"] = "off"
    pod_state["id"] = None
    await terminate_pod(pod_id)


@app.get("/status")
async def status():
    """Pod status for frontend polling. Returns pod URL when ready."""
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
    """Serve the entrance page at root."""
    path = os.path.join(FRONTEND_DIR, "entrance.html")
    if not os.path.exists(path):
        return HTMLResponse("<h1>Veiled Court</h1><p>Frontend not found</p>", status_code=500)
    with open(path) as f:
        return f.read()


@app.get("/{path:path}")
async def serve_frontend(path: str):
    """Serve frontend files."""
    file_path = os.path.join(FRONTEND_DIR, path)
    if os.path.isfile(file_path):
        import mimetypes
        content_type = mimetypes.guess_type(file_path)[0] or "application/octet-stream"
        with open(file_path, "rb") as f:
            return Response(content=f.read(), media_type=content_type)
    return JSONResponse({"error": "not found"}, status_code=404)
