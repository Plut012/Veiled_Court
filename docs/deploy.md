# Deployment

## Architecture

```
┌──────────────────────┐         ┌──────────────────────────┐
│  Coordinator (24/7)  │         │  Game Pod (on-demand)    │
│  Cheap CPU VPS       │ ──API──▶│  RunPod GPU, Rust+KataGo │
│  Landing + Summon    │ ◀──WS── │  Existing app, unchanged │
│  Idle watchdog       │         │  core game logic         │
└──────────────────────┘         └──────────────────────────┘
        │                                    ▲
        └────── Player redirected once ──────┘
                  pod is healthy
```

**Coordinator** — always-on cheap VPS (Hetzner CAX11 ~$4/mo). Serves landing page, manages pod lifecycle via RunPod API, kills idle pods.

**Game Pod** — existing Dockerized server on RunPod GPU (RTX 3090 ~$0.19/hr). Spun up on demand, terminated after idle timeout. KataGo models stored on a RunPod Network Volume to avoid re-downloading on cold start.

## Cost

| Component | Monthly |
|---|---|
| Coordinator VPS | ~$4 |
| RunPod Network Volume (1GB) | ~$0.07 |
| GPU compute (~2 hr/day avg) | ~$12 |
| Domain | ~$1 |
| **Total** | **~$17** |

## Cold Start

60-90s from summon to play. Themed as a loading sequence:

1. *Pod boot* (0-30s)
2. *Container start + model load* (30-60s)
3. *KataGo ready, redirect to game* (60-90s)

## Game Server Endpoints (to add)

- `GET /health` — 200 only after KataGo has loaded the neural net
- `GET /activity` — timestamp of last WebSocket message
- Optional: `POST /shutdown` — graceful termination

## Status

Not yet implemented. See `docs/implementation_plan.md` for the original phased build plan. Current priority: get the Docker image validated on RunPod manually before building the coordinator.

## Local Development

For running locally without cloud infrastructure, see [dev-mode.md](dev-mode.md).
