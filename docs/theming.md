# Theming

Go is already beautiful. The theming does not redesign it — it dresses it.

The board, the stones, the grid — these do not change. The *atmosphere* changes. Each spirit places the same perfect game inside a different world.

## The Rule of 60/30/10

Every palette is structured around three color roles:

- **60% — Board field** — the dominant color. The world you are sitting inside. This single change does most of the atmospheric work.
- **30% — Lines & hoshi** — grid lines and star points. Structural, present, understated. The skeleton of the world.
- **10% — Accent** — last move indicator, territory fill, active intersection. Used sparingly. The only element that breathes against the depth.

**Stones** — black picks up a whisper of the palette. White responds subtly in the opposite direction. Their contrast relationship never breaks. Applied as a subtle CSS blend mode over existing stone assets — the natural sheen and geometry of the stone stays intact.

---

## Color Philosophy

Matte, deep, rich, slightly desaturated. Colors that feel like physical materials — lacquer, stone, moss, ink, old wood, twilight sky. Not colors on a screen. Colors you feel you could touch.

---

## 🐉 Dragon — Living Go

*A still pond at night. Somewhere beneath the surface, something moves.*

| Element | Color | Hex |
|---|---|---|
| Board (60%) | Deep ink black-green | `#0D1F1A` |
| Lines & hoshi (30%) | Aged jade | `#2E6B55` |
| Black stone tint | Ink, cold and deep | — |
| White stone tint | Pale celadon whisper | — |
| Accent (10%) | Living gold | `#C9A84C` |

---

## 🦐 Mantis Shrimp — The AI Disciple

*Void. Then a flash of something your eyes were never built to see.*

| Element | Color | Hex |
|---|---|---|
| Board (60%) | Absolute deep black | `#080B10` |
| Lines & hoshi (30%) | Deep spectral teal | `#0D4D5E` |
| Black stone tint | Pure void, no warmth | — |
| White stone tint | Cold, clinical white | — |
| Accent (10%) | UV violet | `#9B5DE5` |

The accent is the most saturated color in the roster by a significant margin. Against `#080B10` it will feel like a rupture. Use it exactly that way.

---

## 🕊️ Crane — The Classicist

*The only light world in the roster. Silence made visible.*

| Element | Color | Hex |
|---|---|---|
| Board (60%) | Ash white | `#E8E4DC` |
| Lines & hoshi (30%) | Graphite ink | `#4A4A52` |
| Black stone tint | True ink black | — |
| White stone tint | Warm ivory | — |
| Accent (10%) | Cold silver | `#A8B8C8` |

The Crane is the only light-dominant palette. On a selection screen surrounded by dark worlds it commands attention through contrast alone — which is exactly right for its character.

---

## 🕷️ Spider — The Trickster

*Almost nothing to see. Until suddenly everything is connected.*

| Element | Color | Hex |
|---|---|---|
| Board (60%) | Deep forest black | `#0F1410` |
| Lines & hoshi (30%) | Smoke | `#3A3F38` |
| Black stone tint | Green-black, living | — |
| White stone tint | Silk grey | — |
| Accent (10%) | Amber thread | `#C17D2A` |

---

## 🦅 Eagle — The Moyo Artist

*Seen from altitude. The whole board is one shape.*

| Element | Color | Hex |
|---|---|---|
| Board (60%) | Deep dusk sky | `#1A2535` |
| Lines & hoshi (30%) | Cloud grey | `#6B7A8D` |
| Black stone tint | Midnight blue-black | — |
| White stone tint | Cool cloud white | — |
| Accent (10%) | Dawn gold | `#D4A843` |

---

## 🦁 Lion — The Territorial Builder

*Dry heat. Ancient ground. Nothing moves that hasn't decided to.*

| Element | Color | Hex |
|---|---|---|
| Board (60%) | Burnt savanna | `#2A1E0F` |
| Lines & hoshi (30%) | Ochre dust | `#8B6914` |
| Black stone tint | Dark earth | — |
| White stone tint | Dry bone | — |
| Accent (10%) | Heat amber | `#E8A020` |

---

## 🦗 Praying Mantis — The Blood Warrior

*Perfect stillness. Then the accent color earns its name.*

| Element | Color | Hex |
|---|---|---|
| Board (60%) | Obsidian | `#0C0F0C` |
| Lines & hoshi (30%) | Deep emerald | `#1A4030` |
| Black stone tint | Black with green blood | — |
| White stone tint | Cold white | — |
| Accent (10%) | Arterial red | `#8B1A1A` |

The accent appears only on captures. Nowhere else. The restraint is the point. An entire game of obsidian and emerald — and then, only when a stone dies, the red appears. Every capture feels significant.

---

## 🐆 Jaguar — The Endgame Assassin

*The board itself transforms. The warmth leaves gradually. By the endgame, a different world.*

This is the only palette that moves. A slow continuous drift keyed to move number — no hard transition. The player experiences the transformation before they consciously understand it.

| Element | Early game | Late game |
|---|---|---|
| Board (60%) | Dusk canopy `#1C1508` | Cold black `#0A0C10` |
| Lines & hoshi (30%) | Amber shadow `#5C3D1A` | Steel shadow `#2A3040` |
| Black stone tint | Warm dark | Cold void |
| White stone tint | Soft warm white | Hard cold white |
| Accent (10%) | Warm gold `#C9A870` | Cold silver `#B0C4D8` |

The accent shift is the most visible signal of the transformation. The board palette drifts slowly — the accent snaps at a defined threshold around move 120. The player feels the change before they see it.

---

## 🐦‍⬛ Crow — The Ko Monster

*Storm light. Everything dims except what the crow is watching.*

| Element | Color | Hex |
|---|---|---|
| Board (60%) | Storm blue-black | `#12141A` |
| Lines & hoshi (30%) | Deep slate | `#2E3340` |
| Black stone tint | Blue-black iridescence | — |
| White stone tint | Sharp cool white | — |
| Accent (10%) | Sharp white | `#E8EEF4` |

During ko fights — board opacity dims globally, the active ko and its active threats remain at full brightness. The crow's attention made visible through the board itself. This is the one moment where the board becomes behaviorally expressive rather than just atmospheric.

---

## Implementation Notes

**Stone tinting** — Apply as a subtle CSS `mix-blend-mode` or `filter` over existing stone assets. Target effect is felt, not obvious. The stone's natural sheen and geometry must stay intact.

**Jaguar drift** — Interpolate board and line colors linearly from move 0 to move 120, then hold cold palette. Accent transitions at move 120 threshold. Implement as a CSS custom property updated on each move event.

**Crow ko-dim** — On ko detection, apply a global board opacity reduction (e.g., `0.6`) with `pointer-events: none` exclusion on highlighted intersections. Remove on ko resolution.

**Palette as CSS variables** — All six color values per spirit stored as CSS custom properties on the root element, swapped atomically on spirit selection. This keeps theming decoupled from board rendering logic.

```css
:root {
  --board-primary: #0D1F1A;
  --board-secondary: #2E6B55;
  --accent: #C9A84C;
  --stone-black-tint: ...;
  --stone-white-tint: ...;
}
```
