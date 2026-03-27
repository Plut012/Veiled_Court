# Visual Direction — Spirit Animal Character Brief

## Style Foundation

All nine characters share a single visual language. This foundation is established first and locked before any individual animal is generated.

**Aesthetic** — Dark stylized animation. Reference points: Castlevania (Netflix series), Arcane, Into the Spider-Verse. Bold silhouettes, deliberate linework, rich shadow. Not cute. Not realistic. Somewhere between graphic novel and animated film concept art.

**Rendering quality** — Stylized but weighted. These animals have mass and soul. Lines suggest form without rendering every detail. Shadow does as much work as light. Negative space finishes the thought.

**Color philosophy** — Each character is built from its own palette. Animals are not colored naturalistically — they are colored atmospherically. The jaguar does not look like a jaguar in sunlight. It looks like a jaguar that has always lived in darkness.

**Format** — Single character portrait. No background scene. No props except where noted. The creature alone, in one definitive pose, against deep negative space.

---

## Fixed Prompt Foundation

Use these strings verbatim in every generation to enforce cohesion across the roster:

```
dark stylized animation, character portrait, bold clean silhouette,
dramatic lighting, concept art, brooding atmosphere, transparent background
```

```
negative: cute, kawaii, realistic photo, busy background, text, watermark,
multiple animals, childish, soft, pastel, bright, colorful, happy
```

---

## CivitAI Generation Settings

- **Base model** — Dreamshaper XL, Juggernaut XL, or AnimagineXL (test all three against the Jaguar first — whichever produces the right mood for the hardest subject is the right model for the roster)
- **LoRA stack** — layer `dark cartoon`, `Castlevania style`, and `bold linework` LoRAs at 0.5–0.8 weight
- **Resolution** — 1024×1024 minimum
- **CFG scale** — 6–8
- **Steps** — 30–40 final, 20 exploration
- **Sampler** — DPM++ 2M Karras or Euler a
- **Seed** — lock seed once a composition is right, then iterate prompt and LoRA weights from that seed

**Cohesion pass** — Once all nine are generated, lay them side by side. Run outliers through img2img at denoise 0.3–0.4 with a shared style reference image to nudge toward a common rendering quality without destroying individual compositions.

---

## Generation Order

1. **Jaguar first** — hardest subject, two renders required, sets the quality bar
2. **Mantis Shrimp second** — tests the model's ability to handle unusual subjects with graphic precision
3. **Dragon, Spider, Crow, Eagle, Lion, Praying Mantis** — in any order
4. **Crane last** — separate tonal session, light palette, different energy entirely

---

## The Nine

---

### 🐉 Dragon — Living Go

**Palette** — Deep ink black-green `#0D1F1A` / Aged jade `#2E6B55` / Living gold `#C9A84C`

**Vibe** — Ancient, continuous, alive. Not threatening — transcendent. The dragon does not need to intimidate. It simply exists, and its existence reorganizes the space around it. The eye contact is calm and total.

**Pose** — Coiling upward through the frame. Body enters lower left, spirals through the composition, head emerging upper right — turning back to look directly at the viewer. The tail disappears into darkness below; the full body is never entirely visible. Something this large cannot be fully seen. Gold accent traces the spine line only — a single luminous thread through the composition.

**Linework** — Fluid, continuous. Scales suggested rather than individually rendered. The spine is the most defined element.

**Light** — Emanates faintly from within. Not glowing dramatically — just the sense that the darkness around it is slightly warmer than darkness elsewhere.

**Mood** — ancient, continuous, aware, generous, inevitable

```
eastern dragon, jade and gold, coiling upward, making eye contact,
glowing spine, emerges from darkness, partially visible,
deep green atmosphere, living, serene power
```

---

### 🦐 Mantis Shrimp — The AI Disciple

**Palette** — Absolute deep black `#080B10` / Deep spectral teal `#0D4D5E` / UV violet `#9B5DE5`

**Vibe** — The most unsettling portrait on the roster. Not because it is aggressive — because it is completely still and looking directly at you with eyes that perceive things yours cannot. Clinical. Alien. Unhurried.

**Pose** — Dead center, full frontal, perfectly symmetrical. The only character presented with absolute bilateral symmetry — reinforcing computational precision. Raptorial claws slightly forward and open, not raised in threat but present. The compound eyes are the entire focal point.

**Linework** — Sharp, precise, crystalline. The only character where linework feels almost geometric. Each body segment rendered with the same weight — nothing emphasized over anything else. Unsettlingly even.

**Light** — The compound eyes catch the violet accent and fracture it into spectral fragments. This is the only light source in the image. Everything else is teal darkness.

**Mood** — alien, precise, still, perceptive, cold, complete

```
mantis shrimp, frontal symmetrical portrait, compound eyes glowing violet,
spectral iridescence, crystalline linework, deep black background,
teal shadow, perfectly still, clinical, otherworldly, bilateral symmetry
```

---

### 🕊️ Crane — The Classicist

**Palette** — Ash white `#E8E4DC` / Graphite `#4A4A52` / Cold silver `#A8B8C8`

**Vibe** — The only light world. Built from white and silence rather than darkness and depth. The crane does not need drama — its form is already perfect. Maximum negative space. Every line placed as if it could not have gone elsewhere.

**Pose** — Standing, one leg raised in classical crane stance. Head tilted very slightly downward — the one character that does not make eye contact. Looking at something just below the frame. This self-possession — absorbed in its own world — is what separates it from every other portrait. Wings slightly open, not in flight, not folded. Suspended between states.

**Linework** — The sparest on the roster. Minimum lines to convey the crane completely. Graphite on ash. Flat, even, diffused light — no dramatic shadows. The crane exists in clarity, not contrast.

**Mood** — still, ancient, self-possessed, perfect, unhurried, sparse

```
japanese crane, classical stance one leg raised, minimal linework,
graphite on white, ash background, wings slightly open,
not making eye contact, extreme negative space,
sparse elegant, looking slightly downward, absorbed
```

---

### 🕷️ Spider — The Trickster

**Palette** — Deep forest black `#0F1410` / Smoke `#3A3F38` / Amber thread `#C17D2A`

**Vibe** — Patient, geometric, slightly eerie. Does not look dangerous — looks inevitable. The composition is itself a trap: beautiful, precise, and you only understand what you're looking at after you've already looked.

**Pose** — Suspended from the top of the frame on a single amber thread. Legs arranged in perfect geometric symmetry below the body — architectural, not chaotic. Looking downward. The viewer is below, looking up at something that is looking down at them. The thread is the most important line in the image. Everything else is secondary to it.

**Linework** — The amber accent runs along the thread and catches faintly at leg joints. Faint shadow geometry below echoes web structure.

**Light** — From below, very faint. Creates shadow geometry from the legs that feels like the beginning of a web.

**Mood** — patient, geometric, watching, precise, inevitable, beautiful and eerie

```
spider hanging from single thread, view from below looking up,
legs geometrically arranged, amber silk thread, deep forest black,
watching downward, architectural symmetry,
faint web geometry in shadows, eerie elegant, suspended
```

---

### 🦅 Eagle — The Moyo Artist

**Palette** — Deep dusk sky `#1A2535` / Cloud grey `#6B7A8D` / Dawn gold `#D4A843`

**Vibe** — Vast. The eagle's presence is atmospheric rather than intimate. The viewer feels it before they see it. Scale achieved through composition, not aggression. The board is below. Everything is below.

**Pose** — Wings at absolute full spread, seen from directly below — looking up at the underside at altitude. Full wingspan fills the frame edge to edge. Head turns very slightly. Gold accent catches the leading edge of wings and the eye. The body is a dark silhouette against dusk sky — almost more shadow than eagle.

**Linework** — Bold outer silhouette, minimal internal detail. Wing feathers suggested at edges only. The power of this image is in the shape, not the rendering. Dawn light from below catches wing edges in gold.

**Mood** — vast, elevated, atmospheric, inevitable, sovereign, unhurried

```
eagle wings fully spread, view from directly below,
silhouette against deep blue-grey sky, dawn gold wing edges,
fills entire frame, slightly turning head, altitude,
shadow and silhouette dominant, minimal internal linework
```

---

### 🦁 Lion — The Territorial Builder

**Palette** — Burnt savanna `#2A1E0F` / Ochre dust `#8B6914` / Heat amber `#E8A020`

**Vibe** — Absolute presence without effort. Does not perform authority — simply has it. The portrait should feel like looking at something that was here before you arrived and will remain after you leave. No aggression. No display. Complete, unambiguous ownership of the space it occupies.

**Pose** — Seated, perfectly upright, full frontal. The stillest pose on the roster. No movement implied. The mane frames the face like a crown that was never put on because it was never not there. Eyes open, direct, neither warm nor cold. The mane is the most complex element — textured, layered, ochre and amber interweaving. The face is cleaner. Contrast between mane complexity and face clarity draws the eye to the gaze.

**Light** — Warm, low, from slightly below — late savanna sun. Amber accent on mane edges and eyes.

**Mood** — sovereign, still, complete, unambiguous, present, unhurried

```
lion seated upright, full frontal, direct gaze,
warm amber savanna light, detailed mane with ochre and gold,
burnt warm dark background, regal stillness,
not aggressive, simply present, crown-like mane, ancient
```

---

### 🦗 Praying Mantis — The Blood Warrior

**Palette** — Obsidian `#0C0F0C` / Deep emerald `#1A4030` / Arterial red `#8B1A1A`

**Vibe** — The only character with implied motion. Everything else on the roster is still. The mantis is mid-strike — the single moment of commitment after infinite patience. The red is used here, on the strike surfaces only, and almost nowhere else in the entire theme. When it appears, it feels earned.

**Pose** — Three-quarter view, body slightly coiled and twisted, front strike legs fully extended forward. Not posed — caught in the act. One moment after stillness became violence. The triangular head faces the viewer at an angle. Compound eyes cold and focused. Angular, decisive linework — lines that feel made quickly and correctly.

**Light** — Red accent catches only on the inner strike surfaces of the forelegs. Everywhere else, deep emerald shadow. The restriction makes the red feel revealed, not displayed.

**Mood** — committed, precise, sudden, cold, decisive, dangerous without cruelty

```
praying mantis mid-strike, forelegs fully extended,
three-quarter view, angular linework, deep emerald and obsidian,
red accent on strike surfaces only, compound eyes focused,
frozen in moment of commitment, sharp geometric body segments
```

---

### 🐆 Jaguar — The Endgame Assassin

**Palette (Early)** — Dusk canopy `#1C1508` / Amber shadow `#5C3D1A` / Warm gold `#C9A870`
**Palette (Late)** — Cold black `#0A0C10` / Steel shadow `#2A3040` / Cold silver `#B0C4D8`

**Vibe** — Presence that is almost absence. The instinct is to show more. The truth is the opposite. What makes the jaguar terrifying is what you cannot see. The portrait should feel like something vast is mostly hidden.

**Pose** — Low, horizontal, pressed close to the ground. Body extends across the lower portion of the frame nearly edge to edge, but the head is the only fully resolved element. Eyes directly forward — the only thing in sharp focus. The body fades into darkness at the edges. Spots implied as texture, not pattern. The eye contact is the whole image.

**Two renders required** — Identical pose and composition. Only the color world shifts. Name them `jaguar_warm.png` and `jaguar_cold.png`. The warm version lives on the selection screen. Both exist in the game, with the cold version gradually replacing the warm as the board palette transforms.

**Light** — Eyes carry the only light. Warm gold in the early version, cold silver in the late version. Everything else exists in shadow.

**Mood** — patient, hidden, watchful, precise, transformation, cold intelligence

```
jaguar low to ground, horizontal composition, head only fully visible,
body fades into darkness, eyes in sharp focus making eye contact,
spots implied as texture not pattern, minimal linework,
vast mostly hidden, [warm amber / cold silver] eye glow
```
*Generate twice — once warm palette, once cold.*

---

### 🐦‍⬛ Crow — The Ko Monster

**Palette** — Storm blue-black `#12141A` / Deep slate `#2E3340` / Sharp white `#E8EEF4`

**Vibe** — Intelligence that is slightly off. Not sinister — knowing, which is sometimes harder to sit with. The head tilt is the entire character. Something small and luminous in the beak. The eye toward you contains more than you expected.

**Pose** — Perched, body three-quarters to viewer, head turned sharply to look directly at the camera. The head tilt angle matters — not a casual tilt, slightly too far. The kind of angle that makes you aware the bird is looking at you more deliberately than birds usually do. One eye fully toward the viewer, catching white accent light. Something small and white — a stone, a fragment — held in the beak. Iridescent plumage quality suggested through subtle blue-black variation in the dark tones, not through color.

**Light** — White accent catches on the eye and the object in the beak. Two points of sharp white light in an otherwise dark composition. Small, precise, and slightly unsettling in their clarity.

**Mood** — knowing, watchful, slightly wrong, intelligent, collected, amused

```
crow perched, head tilted toward viewer at sharp angle,
one eye directly facing camera, white accent on eye,
small white object in beak, blue-black iridescent feathers,
storm dark background, too-aware expression,
three-quarter body frontal head, deliberate unsettling gaze
```

---

## Corner Icon Crops

Each portrait is designed to have one visually essential element that survives aggressive cropping to 64×64px. Identify and protect this element before finalizing each portrait.

| Animal | Corner crop element |
|---|---|
| 🐉 Dragon | Eye and partial head emerging from darkness |
| 🦐 Mantis Shrimp | Compound eyes — full frontal, perfectly centered |
| 🕊️ Crane | Head and neck line |
| 🕷️ Spider | Body and two central legs on thread |
| 🦅 Eagle | Eye, beak, and wing edge in gold |
| 🦁 Lion | Eyes and mane crown |
| 🦗 Praying Mantis | Head and extended strike legs |
| 🐆 Jaguar | Eyes only — everything else is darkness |
| 🐦‍⬛ Crow | Eye and beak with white object |

The eye is almost always the answer. When in doubt, crop to the eyes.
