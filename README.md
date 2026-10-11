# Veiled Court

Play Go against nine spirits, each with a different philosophy and playing style.

## The Nine

| Spirit | Name | Nature |
|--------|------|--------|
| 🕊️ Crane | Tsuru | Each move placed as if it were the last |
| 🦅 Eagle | Garuda | The territory is the sky |
| 🦁 Lion | Sekhmet | Was here before you arrived |
| 🦗 Praying Mantis | Cǎotáng | Stillness. Then not. |
| 🕷️ Spider | Anansi | The web was finished before you noticed the thread |
| 🐦‍⬛ Crow | Morrígan | Takes something small. Returns for the rest. |
| 🐆 Jaguar | Balam | What you cannot see is most of what is there |
| 🐉 Dragon | Ryūjin | Older than the board. Older than the game. |
| 🦐 Mantis Shrimp | Zhìyǎn | Sees colors you cannot name |

## Stack

- **Game server:** Rust + Axum + KataGo, on an on-demand RunPod GPU pod (stops itself when idle)
- **Coordinator:** Python FastAPI — serves the frontend, summons the pod (password), hosts friend rooms
- **Frontend:** Vanilla HTML/CSS/JS, canvas board renderer

How it fits together: [`docs/deploy-review.md`](docs/deploy-review.md).

## Play — coordinator + RunPod GPU

Needs `.env` with `RUNPOD_API_KEY=...` and `COURT_PASSWORD=...`:

```bash
./scripts/run-coordinator.sh                     # http://localhost:8000
cloudflared tunnel --url http://localhost:8000   # optional: public HTTPS link to share
```

Bot games ask for the password and wake the GPU pod (~1–3 min cold). "Play a friend" makes a shareable room link and needs no GPU.

## Run everything locally (no GPU)

Builds a CPU container (podman or docker), downloads nets on first run (~600MB), serves on port 3000:

```bash
./run-local.sh
# http://localhost:3000
```

Uses KataGo's Eigen/AVX2 CPU build with the lightweight `configs-dev` visit counts (~3s/move). Options: `MOCK=1 ./run-local.sh` (instant random-move engine, no nets), `CONFIGS=prod ./run-local.sh` (full-strength search — slow on CPU).

## Update the GPU pod image

Only needed when `src/` or `configs/` change:

```bash
./scripts/runpod-push.sh   # build + push ghcr.io image; then terminate the old pod once
```

## Docs

See [`docs/`](docs/) for architecture, deployment, spirit roster, theming, and KataGo configuration.

## License

Built with Claude Code
