# Deployment Review — current state (2026-10-10)

A snapshot of how Veiled Court is run and deployed today, and where it is
fragile. Starting point for defining the target setup.

**Constraints:** no local GPU, so KataGo runs on RunPod. Goal is a simple,
robust setup: run the frontend locally, deploy easily.

## How it works today

```
Browser ──▶ Coordinator (FastAPI)          ──RunPod API──▶  Game pod (Rust + KataGo, GPU)
            serves frontend, /summon,                      serves its OWN copy of the
            /status, idle watchdog                         frontend + /ws, /health, /activity
   │                                                                 ▲
   └──── once pod is healthy, browser is redirected to pod URL ──────┘
```

- Coordinator: `coordinator/main.py`. Runs locally (`scripts/run-coordinator.sh`)
  or on a VM (`scripts/deploy-oracle.sh`, Oracle free tier, port 80).
- Pod: `Dockerfile.runpod` → `ghcr.io/plut012/kitsune:latest`, standard net
  baked in. Created on demand, **stopped** (not terminated) after 10 min idle,
  resumed on next summon.
- Fully local alternative: `./run-local.sh` — CPU KataGo in a container
  (`Dockerfile.cpu`), dev configs (~3s/move), or `MOCK=1` for random moves.

## Problems

1. **Two copies of the frontend.** After redirect the player runs the pod's
   baked-in frontend. Local edits and coordinator redeploys don't reach the
   game screen until the pod image is rebuilt and pushed. "New game" /
   "Session expired" navigate to `/` on the pod, so the player never returns
   to the coordinator — when the pod idles out, the page is dead.
2. **Coordinator forgets its pod on restart.** The pod ID lives only in
   memory. A restart/redeploy creates a fresh pod and orphans the stopped one
   (stopped pods still bill for disk). A pod that was running during the
   restart is never watched or stopped. `docs/deploy-oracle.md` claims the
   opposite.
3. **Two coordinators = two pods.** Running locally and on the VM at the same
   time each creates its own pod.
4. **Open `/summon` on a public IP.** Anyone who finds the coordinator can
   start GPU pods. No auth, plain HTTP.
5. **Script and config sprawl.** Several overlapping, partly stale entry
   points: `run`, `scripts/run-dev.sh`, `scripts/run_dev.sh`,
   `scripts/run_prod.sh`, `scripts/deploy_server.sh`, `railway.json`,
   `coordinator/Procfile`, `docker-compose.yml` (needs local GPU),
   `build-frontend.sh`, `play` (Termux). Docs (`architecture.md` hosting
   section, `deploy.md`, `scripts/README.md`) describe setups that no longer
   apply.
6. **Analysis config ignores dev mode.** `src/ws.rs` hardcodes
   `configs/analysis.cfg`, so `configs-dev/analysis.cfg` is unused and
   post-game review runs at 200 visits even on CPU.
7. **Smaller:** `run-local.sh` scrapes katagotraining.org for the net URL
   (breaks if markup changes); `deploy-oracle.sh` relies on
   `netfilter-persistent` without installing it (port 80 rule may not survive
   reboot); `target/` is gitignored but partly tracked.

## Candidate direction (to discuss)

- **Frontend lives only on the coordinator.** The game page opens its
  WebSocket directly to the pod URL instead of navigating there. One origin,
  one copy; pod image only changes when Rust/configs change.
- **Coordinator discovers its pod by name** (`kitsune-game`) via the RunPod
  API on startup — survives restarts, shared by local and deployed
  coordinators.
- **One documented path per job:** local CPU (`run-local.sh`), local
  frontend + RunPod (`run-coordinator.sh`), deploy (`deploy-oracle.sh`),
  build/push pod image. Remove the rest.
- **Shared secret on `/summon`.**
