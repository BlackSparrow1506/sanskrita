# ॐ संस्कृता (Sanskrita) v0.5 — फलम् (Phase 3 complete)

The Sanskrit programming language: Devanagari keywords, Pāṇinian kāraka
arguments, exact decimal arithmetic, and a native Rust engine.

**Two engines, one language.** `sanskrita.py` is the *reference* engine — it
defines the language. **वेगः** (`rust-engine/`) is the native engine — a single
fast binary with no runtime. Both must produce byte-identical output for every
program; `तुल्यता.py` is what enforces that, on every push.

📋 **[STATUS.md](STATUS.md)** — every design promise for Phases 1–3, with an
honest verdict: shipped, partial, or deferred (and to which phase).
📐 **[docs/GRAMMAR.md](docs/GRAMMAR.md)** — the complete formal grammar ·
🔒 **[docs/STABILITY.md](docs/STABILITY.md)** — what we promise not to break ·
📝 **[CHANGELOG.md](CHANGELOG.md)**
Website: enable GitHub Pages → `docs/`. CI runs `परीक्षा.py` on every push.

**हिन्दी में पढ़ें:** [docs/README.hi.md](docs/README.hi.md) ·
**पारिभाषिकशब्दकोशः:** [docs/शब्दकोशः.md](docs/शब्दकोशः.md)

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
sanskrita examples/नमस्ते.सं          # run a program  →  नमस्ते जगत्
sanskrita                             # interactive REPL
sanskrita --veg examples/नमस्ते.सं    # same program on the native वेगः engine
sanskrita-playground                  # browser playground (perfect Devanagari)
```

The first `--veg` run builds the native engine (needs [Rust](https://rustup.rs));
after that it is a single binary and starts instantly.

If `sanskrita` isn't found, the installer printed a PATH line to add to `~/.zshrc` — paste it, open a new terminal.

**Windows:** download/clone the repo, then run programs directly (no installer needed):

```
python sanskrita.py examples\नमस्ते.सं
python playground.py
```

**Useful flags:** `--veg` (native engine) • `--roman` (ASCII digit output) •
`--convert file` (rewrite a roman-typed file into Devanagari) • `--druta`
(experimental compiled mode) • `--version`.

**Verify your install:**

```bash
python3 परीक्षा.py             # conformance — should end with सर्वं शुद्धम् ✓
python3 तुल्यता.py             # both engines, byte-identical output required
python3 यादृच्छिकपरीक्षा.py    # random programs — both engines must agree
cd rust-engine && cargo test  # वेगः unit tests
```

Four layers, all in CI. The last one generates programs nobody wrote: it found
two real bugs in exact-decimal arithmetic on its first run.

**New to संस्कृता?** Start with `docs/TUTORIAL.md` — your first 10 programs in an hour. Read `docs/AI-SPEC.md` — or paste it into any AI assistant and it becomes your संस्कृता tutor. Design patterns guide: `docs/अभिकल्पनप्रतिमानानि.md`. License: MIT. Contributions: see `CONTRIBUTING.md` and `CODE_OF_CONDUCT.md`.
Security: `SECURITY.md` (please do not open a public issue).

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

## Three ways to run the same program

```bash
sanskrita प्रोग्राम.सं            # reference engine — the specification, REPL & playground
sanskrita --veg प्रोग्राम.सं      # वेगः — native Rust engine, single binary, no runtime
sanskrita --druta प्रोग्राम.सं    # EXPERIMENTAL compiled mode — 1000×+ faster
```

`--druta` (द्रुत = "fast") transpiles to C, compiles, and caches the binary
(instant on re-run). It covers a subset — integers, loops, functions, यदि, वद —
enough to *prove* the compiled path (§7b यन्त्रसङ्कलकः); anything outside the
subset runs in the interpreter. Same parser, same language, verified identical
output. Honest benchmarks: `python3 मापनम्.py` → BENCHMARKS.md.

## New in v0.4 — Phase 3 complete

**The engine**

- **वेगः, the native Rust engine** — the whole language in a dependency-free
  binary: hand-rolled arbitrary-precision integers, exact decimals, NFC
  normalization, and a stack-measuring recursion guard. `sanskrita --veg prog.सं`,
  or a REPL with no arguments. Every program is differential-tested against the
  reference engine; the divergence ledger is public and currently empty.

**The language — the last four §2b promises, delivered**

- **शून्यम्-safety (the billion-dollar mistake, refused).** A typed variable
  cannot hold शून्यम् unless you opt in:
  `मानय नाम? : वाक्यम् = शून्यम्।`
- **प्राक्परीक्षा — checked *before* the program runs.** Annotation violations
  and provable text/number mixing are reported together, and nothing executes.
  It is conservative on purpose: it reports only what it can prove.
- **Default arguments that cannot bite you.** `विधि नम(क, ख = ५) { … }` stores
  the *expression*, re-evaluated fresh on every call — Python's classic
  mutable-default bug simply cannot happen here.
- **द्रुतदशमांशः — speed, opt-in and honest.** Exact दशमांशः stays the default;
  ask for `द्रुतदशमांशः(x)` and you get an IEEE-754 binary float that prints
  exactly what it holds, `०.३०००००००००००००००४` and all. See
  `examples/शुद्धिवेगौ.सं`.
- **क्षिप** — raise your own error, catchable by प्रयत/दोषे.
- **Method chaining (§7d sandhi-style composition).**
  `अङ्काः.छानय(f).प्रतिचित्रय(g).योगः()` — `.नाम` on a सूची/वाक्यम्/कोशः is the
  same stdlib function with the receiver first. See `examples/शृङ्खला.सं`.

**The standard library — §6 complete**

- **सूचीकर्म** — छानय (filter), प्रतिचित्रय (map), न्यूनीकरण (reduce), विपर्यय,
  अन्तर्भवति, अनुक्रमः, योगः, महत्तमम्, लघुत्तमम्, अद्वितीयम्
- **सञ्चिका** — पठ, लिख, योजय, अस्ति, निष्कासय, पङ्क्तयः, सूचिका (UTF-8, always)
- **जेसन** — विश्लेषय / पाठय, and a decimal survives the round trip *exactly*,
  because it never becomes a float
- **वाक्यकर्म** grows — उच्च, निम्न, परिष्कार, आरभते, अन्तयति, अन्तर्भवति
- **कालः** grows — क्षणविरामः
- **आदेशचराः()** — command-line arguments given to your program

**Testing in संस्कृता itself** — `examples/परीक्षणम्.सं` is a test framework
written in the language, needing nothing outside the core; `examples/स्वपरीक्षा.सं`
uses it.

**Docs** — [STATUS.md](STATUS.md) (promise-by-promise audit),
[docs/README.hi.md](docs/README.hi.md) (Hindi),
[docs/शब्दकोशः.md](docs/शब्दकोशः.md) (technical glossary).

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

## Real programs — and both engines give the same answer

These are not demos that print things. Each one is a job somebody actually has
to do, and each runs on the **native वेगः engine** as well as the reference one,
byte for byte — `तुल्यता.py` checks that on every push.

| Example | The job | What it leans on |
|---|---|---|
| `examples/वेतनपत्रम्.सं` | **payroll run** — prorated salary, PF, slab tax, rejected rows with reasons, CSV + JSON out, audit digest | exact `दशमांशः`, `ग.परिवृत्त` to the paisa, `सारणी`, `जेसन`, `गूढ`, error kinds |
| `examples/लेखापरीक्षा.सं` | **server log triage** — parse, group by level, slowest requests, per-route averages | `वाक्यकर्म`, `सूचीकर्म.क्रमय` with a key, `कालः`, `लेखनी` |
| `examples/आदेशसाधनम्.सं` | **a CLI utility** you could put in a cron job — arguments, env var, stdout report, stderr complaints, real exit codes | `आदेशचराः`, `परिवेशः`, `सञ्चिका`, `लेखनी` |
| `examples/कोशागारम्.सं` | CSV in → validate → exact money → fingerprint → CSV out | most of the standard library at once |
| `examples/दोषविवरणम्.सं` | what an error *is* here, and how to branch on it | error objects, kinds, tracebacks |

```bash
sanskrita       examples/वेतनपत्रम्.सं     # reference engine
sanskrita --veg examples/वेतनपत्रम्.सं     # native engine — identical output
```

The payroll one is worth reading if you only read one. Totals reconcile to the
paisa because every money value is rounded exactly once, where it becomes
money — and because a `दशमांशः` never passed through a float on the way in from
the CSV or out to the JSON.

## Errors you can work with

```
प्रयत {
    वेतनगणना(कर्मी)।
} दोषे (त्रु) {
    यदि (त्रु.प्रकारः == "गणितदोषः") { वद("शून्येन भागः")। }
    अन्यथा { क्षिप त्रु। }          # यत् न जानीमः तत् न गिलामः
}
```

An error is a value: `त्रु.सन्देशः`, `त्रु.आङ्ग्लसन्देशः`, `त्रु.पङ्क्तिः`,
`त्रु.प्रकारः`, `त्रु.अनुरेखा`. It still prints as its message, so old code is
unaffected — but now you can branch on the **kind**, log the details, or
re-raise it unchanged.

Uncaught, it prints the whole chain rather than one line:

```
दोषः पङ्क्तौ ६० — शून्येन भागो न शक्यः
Error at line 60 — division by zero

अनुरेखा (नवीनतमम् आह्वानम् अन्ते) / traceback (most recent call last):
    विधि 'वेतनम्' — पङ्क्तिः ६३
    विधि 'करः' — पङ्क्तिः ५८
    विधि 'प्रतिशतम्' — पङ्क्तिः ५९
    → पङ्क्तिः ६०: शून्येन भागो न शक्यः
```

See `examples/दोषविवरणम्.सं`.

## Standard library at a glance

```
आनय "गणितम्"     इति ग।    # वर्गमूलम्, घातः, ज्या, कोज्या, तलम्, परिवृत्त, पाई, ई …
आनय "वाक्यकर्म"  इति वाक।  # विभज, संयोजय, खोज, प्रतिस्थापय, उच्च, आकारय, पूरय …
आनय "सूचीकर्म"   इति सू।   # छानय, प्रतिचित्रय, न्यूनीकरण, क्रमय, योगः, सङ्गमः …
आनय "सञ्चिका"    इति स।    # पठ, लिख, योजय, अस्ति, निष्कासय, पङ्क्तयः, सूचिका
आनय "जेसन"       इति ज।    # विश्लेषय, पाठय
आनय "सारणी"      इति सा।   # CSV: विश्लेषय, कोशाः, पाठय
आनय "कालः"       इति का।   # अद्य, वासरः, दिनयोगः, अन्तरम्, रूपय, शुद्धः, अधिवर्षः …
आनय "नियमितम्"   इति नि।   # regex: मेलति, खोज, सर्वाणि, प्रतिस्थापय, समूहाः …
आनय "गूढ"        इति गू।   # सङ्क्षेपः (SHA-256), एकाकी (UUID), गूढय/प्रकटय (base64)
आनय "परिवेशः"    इति प।    # चरः, चराः, निर्गम, दोषवद, कार्यसूचिका, मञ्चः
आनय "लेखनी"      इति ले।   # logging: विवरणम्, सूचना, चेतावनी, दोषः, महादोषः
आनय "यादृच्छिकम्" इति य।    # अन्तरे, वरय, भिन्नम्
आनय "संस्कृतम्"   इति सं।   # अक्षराणि, मात्राः, छन्दः, रोमनय, देवनागरय, संधय
```

See `examples/कोशागारम्.सं` — one program that reads a CSV, validates the dates,
computes exact money, fingerprints the result and writes it back out, using
nothing outside the standard library.

Reserved-word note: `न`, `फलम्`, `इति` etc. are keywords — don't use them as variable names (the engine will tell you if you do).

जयतु संस्कृतम् ।
