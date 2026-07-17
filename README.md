# ॐ संस्कृता (Sanskrita) v0.3 — फलम्

The Sanskrit programming language. Website: enable GitHub Pages → `docs/`. CI runs `परीक्षा.py` on every push.

## Install

**Prerequisite:** Python 3.9+ — check with `python3 --version`. Mac/Linux usually have it; on Mac, a popup may offer to install Apple's command line tools (click Install). Windows: get Python from [python.org](https://www.python.org/downloads/) (tick "Add to PATH" during setup).

**macOS / Linux (2 commands):**

```bash
git clone https://github.com/BlackSparrow1506/sanskrita
cd sanskrita && bash install.sh
```

(No git? Click the green **Code ▾** button on GitHub → **Download ZIP** → unzip → run `bash install.sh` inside the folder.)

Then from anywhere:

```bash
sanskrita examples/नमस्ते.सं     # run a program  →  नमस्ते जगत्
sanskrita                        # interactive REPL
sanskrita-playground             # browser playground (perfect Devanagari)
```

If `sanskrita` isn't found, the installer printed a PATH line to add to `~/.zshrc` — paste it, open a new terminal.

**Windows:** download/clone the repo, then run programs directly (no installer needed):

```
python sanskrita.py examples\नमस्ते.सं
python playground.py
```

**Useful flags:** `--roman` (ASCII digit output) • `--convert file` (rewrite a roman-typed file into Devanagari).

**Verify your install:** `python3 परीक्षा.py` should end with `सर्वं शुद्धम् ✓`.

**New to संस्कृता?** Start with `docs/TUTORIAL.md` — your first 10 programs in an hour. Read `docs/AI-SPEC.md` — or paste it into any AI assistant and it becomes your संस्कृता tutor. Design patterns guide: `docs/अभिकल्पनप्रतिमानानि.md`. License: MIT. Contributions: see `CONTRIBUTING.md`.

## Phase 1 features

- Variables `मानय` / constants `ध्रुव`, assignment
- Numbers (Devanagari ०-९ and ASCII digits), **exact decimal arithmetic** (०.१ + ०.२ = ०.३), strings, `सत्यम्`/`असत्यम्`, `शून्यम्`
- `वद(...)` print, `पृच्छ(...)` input, `वाक्यम्(...)` to-text, `सङ्ख्या(...)` to-number
- `यदि / अथ यदि / अन्यथा` conditions, `यावत्` loops with `विरम` (break) / `अनुवर्त` (continue)
- Logic: `च` (and), `वा` (or), `न` (not)
- Optional type annotations: `मानय क : पूर्णाङ्कः = ५।` — enforced on every assignment
- `प्रकारः(x)` type-of, `दैर्घ्यम्(x)` text length
- Statements end with danda `।` (or `|` in roman mode)
- Every keyword has a roman alias (`vada`, `yadi`, `yaavat`…) — one language, two ways to type
- Bilingual (Sanskrit + English) error messages with line numbers
- `#` comments

## Phase 2 features (वृक्षः) — NEW

- **विधि functions** — first-class values, closures, recursion; `फलम्` returns
- **Kāraka-labeled arguments** — `प्रेषय(कर्म: "नमस्ते", सम्प्रदान: "रामः")` — roles in any order, checked by the engine. No other language has this.
- **सूची lists & कोशः maps** — literals `[१, २]` / `{"नाम": "गौरी"}`, **1-based indexing** (प्रथमः = १, as Sanskrit counts)
- **प्रत्येकम् … इति** for-each over lists, maps, and text
- **वर्गः classes** — `सृज` creates, `आरम्भ` constructs, `अयम्` is "this"
- **प्रयत/दोषे** — try/catch error handling
- **आनय Python-bridge** — `आनय "python:math" इति गणितम्।` — 5 lakh libraries
- New builtins: योजय (append), अपनय (remove), कुञ्जिकाः (keys), क्रमय (sort)

## New in v0.3 (फलम्)

- **Import your own files:** `आनय "सहायः.सं" इति सहायः।` — build your own libraries in संस्कृता
- **वाक्यकर्म** string module: विभज (split), संयोजय (join), खोज (find, 1-based), प्रतिस्थापय (replace), अंश (substring)
- **परिधिः(१, १००)** — inclusive range for easy counting loops
- **अपनय(कोशः, कुञ्जिका)** — remove a key from a map
- **Safety:** NFC normalization mandatory; mixed-script identifiers (नामx) rejected
- **Process:** CI on every push, `मापनम्.py` benchmarks (see BENCHMARKS.md), landing page in `docs/`

## The संस्कृतम् library — unique in the world

```
आनय "संस्कृतम्" इति सं।
```

- `सं.अक्षराणि(text)` — syllable splitting; `सं.अक्षरगणना(text)` — count
- `सं.मात्राः(text)` — laghu/guru (ल/ग) weights; `सं.छन्दः(verse)` — meter detection (गायत्री, अनुष्टुभ्, त्रिष्टुभ्, जगती)
- `सं.रोमनय(text)` — Devanagari → IAST; `सं.देवनागरय(text)` — IAST → Devanagari
- `सं.संधय(a, b)` — vowel sandhi joining (dīrgha, guṇa, vṛddhi, yaṇ, avagraha)

Reserved-word note: `न`, `फलम्`, `इति` etc. are keywords — don't use them as variable names (the engine will tell you if you do).

जयतु संस्कृतम् ।
