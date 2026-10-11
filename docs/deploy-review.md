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

## Target setup (agreed 2026-10-10)

```
Browser ──HTTPS──▶ cloudflared tunnel ──▶ Coordinator (your machine; later any small host)
                                          • serves the ONLY copy of the frontend
                                          • POST /summon (password) → resume/create pod
                                          • GET /status → pod lookup + health + rests_in
                                          • POST /rest (password) → stop the pod now
                                          • /room/<code> WebSocket → friend games (no GPU)
   │
   └── bot games: page opens WS directly ──▶ RunPod GPU pod (Rust + KataGo)
                                             • stops ITSELF after 10 min idle
```

Principles:

- **Pod stops itself.** The pod calls the RunPod API to stop itself when idle,
  using its own pod ID + API key from env. A sleeping laptop, crashed or
  absent coordinator can never leave a GPU billing.
- **Stateless coordinator.** No in-memory pod ID: every call looks the pod up
  by name (`kitsune-game`). Restarts, redeploys and multiple coordinators are
  safe and share one pod.
- **One frontend copy**, served by the coordinator. Bot games connect their
  WebSocket straight to `wss://<pod>-3000.proxy.runpod.net/ws`. The pod image
  only changes when Rust/configs change. The Rust server still serves the
  frontend itself for `run-local.sh` (CPU/mock), with no coordinator.
- **Password only on `/summon`** — the only thing that costs money. One shared
  password (`COURT_PASSWORD` in `.env`), remembered by the browser. Friend
  games and static pages are open.
- **Friend games are anonymous rooms.** Share a link `/room/<code>`. The
  coordinator relays moves and keeps the move list (so reconnects resume);
  both clients apply the existing JS rules from `local.js`. No GPU, no
  accounts.
- **Hosting:** run the coordinator locally and share it with
  `cloudflared tunnel --url http://localhost:8000` (free, HTTPS, no account;
  URL changes per run). Move the same coordinator to a small host or a named
  tunnel + domain when that gets annoying — no code changes.
- **Summon flames (selection screen).** No words: the title sits between two
  thin-line art nouveau sconces, each holding a wick. Touch either flame:
  summon (the word input appears only if not remembered); both flames
  flicker while the pod wakes, then burn easily once ready. Touch twice: they
  go out (`POST /rest`) and smoke rises until `/status` says the pod is off —
  i.e. until nothing is billed. Without a coordinator the sconces stay as
  ornament, with no wick.
- **Presence ember + end button.** A small light in the top-right corner of
  the game, review and room pages while the pod runs (flickers while waking, breathes when
  awake, absent when nothing is billed). Tap it: "the spirit is here · rests
  on its own in ~N min", and *let it rest* (tap twice to confirm) stops the
  pod immediately via `POST /rest`.
- **Idle timeout:** the pod stops itself 10 min after the last game message
  (only real game messages count, not protocol pings). The coordinator also
  stops a pod that never becomes healthy within 10 min of a summon.
- **Summon screen:** pick a spirit → the screen takes its palette → one line,
  *"speak the word"*, single underline input. Correct: the line dissolves into
  the breathing status text + thin progress bar while the pod wakes. Wrong:
  the line shivers and fades. Minimal, matches the existing aesthetic.

## Implementation plan

Status: steps 1–6 implemented and validated against RunPod 2026-10-10 (create, reuse, cross-origin WS game, self-stop after 10 min idle, resume in ~30s). Browser walkthrough pending.

1. **Pod self-stop** (Rust): idle watchdog inside the game server, active only
   when `RUNPOD_POD_ID` is set. Also: analysis engine respects
   `KITSUNE_CONFIG_DIR` (fixes problem 6).
2. **Stateless coordinator**: pod lookup by name, resume/create, health-based
   status; passes `RUNPOD_API_KEY` to the pod for self-stop.
3. **One frontend copy**: game page takes the pod URL as a param and connects
   its WebSocket there; navigation always returns to the coordinator.
4. **Password + summon screen.**
5. **Friend rooms.**
6. **Cleanup**: remove stale scripts/configs/docs (problem 5), untrack
   `target/`, update README to the three paths: `run-local.sh` (all CPU),
   `scripts/run-coordinator.sh` (+ tunnel), pod image build/push.
