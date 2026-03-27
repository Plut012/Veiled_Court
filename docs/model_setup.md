# Model Setup

All bots run on **KataGo** via GTP (Go Text Protocol), managed as local subprocesses by the game server. Two neural nets are used across the roster:

- **Standard net** — `kata1-b28c512nbt` — strongest distributed training net, no human influence
- **Human style net** — `b18c384nbt-humanv0` — trained to predict human moves by rank and historical era

Configurations requiring the human style net load both nets simultaneously via:
```
katago gtp -model kata1-b28c512nbt.gz -human-model b18c384nbt-humanv0.bin.gz -config [spirit].cfg
```

---

## Key Parameters

| Parameter | Effect |
|---|---|
| `maxVisits` | Depth of MCTS search — primary strength dial |
| `humanSLProfile` | Rank (25k → 9d) and era (1800 → present) to imitate |
| `rootPolicyTemperature` | Higher = more varied move selection at root |
| `chosenMoveTemperature` | Higher = more randomness in final chosen move |
| `winLossUtilityFactor` | Weight given to winning outcome vs. score margin |
| `staticScoreUtilityFactor` | Weight given to point margins |
| `rootNoiseEnabled` | Injects Dirichlet noise — increases creative unpredictability |

---

## Configurations

### 🐉 Dragon — Living Go
```
net:              humanv0 + standard (dual load)
humanSLProfile:   rank=7d, era=1950–1970 (Japanese professional)
maxVisits:        2000–5000
rootPolicyTemp:   1.0
noise:            disabled
notes:            Moderate visits preserve human flow over AI rationality.
                  Mid-century Japanese era captures thickness-first,
                  direction-of-play philosophy naturally.
```

### 🦐 Mantis Shrimp — The AI Disciple
```
net:              kata1-b28c512nbt (standard only)
humanSLProfile:   none
maxVisits:        50000+
rootPolicyTemp:   1.0 (pure policy, no softening)
winLoss:          high
noise:            disabled
notes:            No human influence. Maximum visits on largest net.
                  Full alien. The absence of humanSL is the configuration.
```

### 🕊️ Crane — The Classicist
```
net:              humanv0 + standard (dual load)
humanSLProfile:   rank=7d, era=1940–1970 (pre-Chinese opening revolution)
maxVisits:        3000
rootPolicyTemp:   0.8
noise:            disabled
notes:            Low temperature enforces classical conviction.
                  Pre-1970 era suppresses 3-3 invasions and AI-influenced
                  joseki naturally. Each move feels inevitable.
```

### 🕷️ Spider — The Trickster
```
net:              humanv0 + standard (dual load)
humanSLProfile:   rank=5d, era=1980–1990 (peak human creativity pre-AI)
maxVisits:        1000–2000
rootPolicyTemp:   1.4
chosenMoveTemp:   high early game, dropping toward late game
noise:            enabled
notes:            High temperature + noise creates genuine unpredictability.
                  Lower visits means it sets threads rather than solving.
                  The unusual early moves are the trap, not mistakes.
```

### 🦅 Eagle — The Moyo Artist
```
net:              humanv0 + standard (dual load)
humanSLProfile:   rank=8d, era=1970–1990 (Chinese/Korean school)
maxVisits:        4000
rootPolicyTemp:   1.1
staticScore:      low
winLoss:          high
notes:            Low score utility drives toward influence over profit.
                  Chinese school era amplifies framework-building tendency.
                  Cares about winning the whole board, not counting corners.
```

### 🦁 Lion — The Territorial Builder
```
net:              humanv0 + standard (dual load)
humanSLProfile:   rank=7d, era=1970–1980 (Japanese territorial school)
maxVisits:        3000–5000
rootPolicyTemp:   0.7
staticScore:      high
winLoss:          low
noise:            disabled
notes:            High score utility = always takes safe concrete profit.
                  Low temperature = total commitment, no adventuring.
                  Japanese territorial era suppresses fighting instincts.
```

### 🦗 Praying Mantis — The Blood Warrior
```
net:              kata1-b15c192 or b20c256 (mid-size standard)
humanSLProfile:   none
maxVisits:        1500–3000
rootPolicyTemp:   1.2
staticScore:      very low
winLoss:          very high
noise:            enabled
notes:            Score utility stripped — it does not collect safe points.
                  Win/loss maximized — creates fighting positions by nature.
                  Mid-size net chosen so it does not avoid complexity.
```

### 🐆 Jaguar — The Endgame Assassin
```
net:              kata1-b28c512nbt (standard)
humanSLProfile:   none
maxVisits:        500 (opening) → 20000+ (endgame, dynamic ramp)
chosenMoveTemp:   high early, approaching 0 late game
staticScore:      very high (endgame phase)
notes:            ⚠ Requires custom visit-scaling wrapper keyed to move number.
                  Low visits early = casual, seemingly approachable.
                  Dramatic ramp after move ~120 = transformation.
                  The character arc IS the configuration.
                  Two palette renders required: warm (early) and cold (late).
```

### 🐦‍⬛ Crow — The Ko Monster
```
net:              mid-size standard
humanSLProfile:   none
maxVisits:        2000
rootPolicyTemp:   1.4
winLoss:          high
noise:            enabled
notes:            ⚠ Requires thin GTP wrapper that biases move selection
                  toward ko-prone positions and tracks ko threat inventory.
                  High temperature surfaces unconventional threats.
                  The chaos is intentional and managed.
```

---

## Implementation Notes

**Dual-net memory** — Loading humanv0 alongside the standard net increases VRAM usage. The RTX 2070 (8GB) handles this comfortably for one active game. Monitor VRAM carefully if running concurrent sessions.

**Spirit config files** — Each spirit maps to a dedicated `.cfg` file passed to KataGo at subprocess spawn. The game server selects the config based on the spirit chosen at the selection screen.

**Custom wrappers** — Jaguar and Crow require logic at the GTP middleware layer, not inside KataGo config. These are implemented in the game server's KataGo process manager, not in KataGo itself.

**Suggested config file naming convention:**
```
configs/
  dragon.cfg
  mantis_shrimp.cfg
  crane.cfg
  spider.cfg
  eagle.cfg
  lion.cfg
  praying_mantis.cfg
  jaguar.cfg
  crow.cfg
```
