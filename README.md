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

- **Backend:** Rust + Axum (WebSocket server)
- **AI:** KataGo with per-spirit configurations
- **Frontend:** Vanilla HTML/CSS/JS, canvas board renderer
- **Deploy:** On-demand RunPod GPU behind a coordinator VPS

## Development

```bash
./scripts/setup-dev-mode.sh   # one time
./scripts/run-dev.sh           # run locally
# http://localhost:3000
```

## Docs

See [`docs/`](docs/) for architecture, deployment, spirit roster, theming, and KataGo configuration.

## License

Built with Claude Code
