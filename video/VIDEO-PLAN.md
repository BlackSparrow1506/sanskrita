# संस्कृता — Video Inventory & Plan
_Audited 26 July 2026_

---

## PART 1 — What already exists

**11 .mp4 files = 7 distinct videos** (4 of them have a v2 re-cut). **0 published.**

| # | Video | File(s) | Length | Format | Topic |
|---|-------|---------|--------|--------|-------|
| 1 | Intro short | `sanskrita_short_v1/v2.mp4` | 32s / **43s** | 1080×1920 vertical | "A language written entirely in Sanskrit" — नमस्ते जगत्, 0.1+0.2, Gītā meter |
| 2 | Origin story | `sanskrita_story_v1/v2.mp4` | 89s / **94s** | 1920×1080 landscape | Why संस्कृता was built |
| 3 | Tutorial #1 | `sanskrita_tutorial_v1.mp4` | 84s | 1920×1080 | First program walkthrough |
| 4 | Gītā meter short | `short_gita_meter_v1/v2.mp4` | 27s / **65s** | vertical | छन्दः detection → अनुष्टुभ् |
| 5 | Kāraka short | `short_karaka_v1/v2.mp4` | 38s / **74s** | vertical | कर्म:/करण:/सम्प्रदान: labelled arguments |
| 6 | Sanskrit heritage | `sanskrit_heritage.mp4` | **148s** | 1920×1080 | Pāṇini / Sanskrit's computational legacy |
| 7 | v0.3 update | `sanskrita_v03_update.mp4` | **65s** | 1920×1080 | Release notes for v0.3 |

**Also ready:** 2 Instagram carousels (`instagram/story_carousel/` 7 slides, `instagram/tutorial_carousel/` 6 slides), 3 SRT sets (hi/en/sa), `thumbnail_v1.png`, `UPLOAD-KIT.md` (full SEO kit for video #1).

**Pipeline in place:** `make_voice_v1…v9.sh` (edge-tts, Swara hi-IN) + `gen_*.py` (PIL, NFC font) + ffmpeg. v6/v7 voice tracks already rendered.

### The real bottleneck
Not production — **distribution.** 7 finished videos are sitting unpublished. Everything below assumes video #1 goes live first.

---

## PART 2 — The gap: what's shipped but never filmed

Last video covered **v0.3 (18 July)**. Since then, three large milestones shipped with zero coverage:

- **phase3b** — lambdas, class inheritance, visarga sandhi, `pip install .`, 51 conformance tests
- **द्रुतम्** — संस्कृता→C transpiler, measured **1000×+** over the interpreter
- **वेगः (Rust engine)** — slices 1–7 **all complete**: lexer, parser, evaluator, functions, exact BigInt/Decimal arithmetic, collections + classes, and the **entire native stdlib rewritten in Rust** (संस्कृतम्/गणितम्/वाक्यकर्म/यादृच्छिकम्/कालः). Measured 5–15× over the Python engine, differential-tested against the reference.

That last one is the biggest story the channel has and it is completely untold.

---

## PART 3 — Candidate videos

### A. Development progress (the build-in-public arc)

| ID | Title | Length | Format | Why | Priority |
|----|-------|--------|--------|-----|----------|
| A1 | **"I rewrote my language in Rust — 15× faster"** (वेगः) | 60–90s | vertical | Biggest untold milestone. Side-by-side timer: 120ms → 8ms. Devs share speed numbers. | ★★★ |
| A2 | **"Sanskrit → C: 1000× faster"** (द्रुतम्) | 45–60s | vertical | Pure spectacle. Show the generated C, then the timer. | ★★★ |
| A3 | **v0.3.1 / phase3b update** — lambdas, inheritance, `pip install sanskrita` | 60s | landscape | Keeps the release cadence; ends with a one-line install CTA. | ★★ |
| A4 | **"How I test a language against itself"** — तुल्यता differential harness | 60s | landscape | Credibility with serious devs; proves this isn't a toy. | ★★ |
| A5 | **The danda bug** — why U+0964 broke the Rust lexer | 45s | vertical | Great standalone story: a punctuation mark living inside the Devanagari block. Bug-story videos travel. | ★★ |
| A6 | **Roadmap: what v1.0 looks like** | 60s | landscape | Sets expectations, invites contributors. | ★ |

### B. Informative — about Sanskrit itself (widest audience)

| ID | Title | Length | Format | Why | Priority |
|----|-------|--------|--------|-----|----------|
| B1 | **"Pāṇini wrote the first programming language — in 500 BCE"** | 60s | vertical | The single most shareable idea you own. Aṣṭādhyāyī as ~4000 rewrite rules. | ★★★ |
| B2 | **सन्धि is string concatenation with rules** | 45s | vertical | Show `संधय` running live. Sandhi finally *clicks* for learners. | ★★★ |
| B3 | **कारक = named function parameters, 2500 years early** | 45s | vertical | Partly covered by #5 — this is the *linguistics-first* cut, not the code-first one. | ★★ |
| B4 | **लघु-गुरु: Sanskrit poetry runs on binary** | 60s | vertical | Laghu/guru → 1/0 → Piṅgala's binary → meter detection. Stunning connection. | ★★★ |
| B5 | **Why Sanskrit is unambiguous** — and why that matters to computers | 60–90s | landscape | The NASA-paper claim, told honestly (this is where accuracy matters most). | ★★ |
| B6 | **Devanagari & Unicode** — NFC, ligatures, why coding in it is hard | 60s | landscape | Real engineering, real explanation. Niche but respected. | ★ |
| B7 | **Read your first Sanskrit line as code** (गीता 1.1 broken down) | 90s | landscape | Bridges the two audiences: literature people meet code. | ★★ |

### C. Teaching संस्कृता (retention & conversion)

| ID | Title | Length | Format | Why | Priority |
|----|-------|--------|--------|-----|----------|
| C1 | **Tutorial #2: variables, यदि, यावत्** | 3–5 min | landscape | Tutorial #1 exists with no sequel. Series = subscribers. | ★★★ |
| C2 | **Tutorial #3: विधि functions + kāraka args** | 3–5 min | landscape | | ★★ |
| C3 | **Tutorial #4: सूची / कोशः (1-based indexing)** | 3–5 min | landscape | The 1-based choice is itself a talking point. | ★★ |
| C4 | **Tutorial #5: वर्गः classes + inheritance** | 4–6 min | landscape | | ★★ |
| C5 | **Install in 60 seconds** | 60s | vertical | Removes the #1 drop-off. Should exist before any video goes viral. | ★★★ |
| C6 | **Build something real** — a छन्दः checker in 30 lines | 3 min | landscape | Proves it's a usable language, not a demo. | ★★ |
| C7 | **"0.1 + 0.2 = 0.3"** — why Python gets it wrong | 45s | vertical | Classic dev-bait. Currently buried mid-video in #1; deserves its own. | ★★★ |

---

## PART 4 — Suggested order

**Before making anything new:** publish video #1 using `UPLOAD-KIT.md`, then release #2–#7 on a schedule (2/week). Seven videos = seven weeks of channel activity already banked.

Then the next four to *produce*, in order:

1. **A1 — Rust engine, 15×** (biggest news, and it's stale already)
2. **B1 — Pāṇini, the first programming language** (widest reach; feeds everything else)
3. **C5 — Install in 60 seconds** (conversion floor, cheap to make)
4. **C1 — Tutorial #2** (starts the series that builds subscribers)

Reusable assets: v6/v7 voice tracks already exist; `make_voice_v9.sh` is the newest pipeline; SRT workflow is proven in 3 languages.
