# STATUS — what संस्कृता actually does, promise by promise

*Last audited: 25 July 2026 · engine v0.4.0 "फलम्" · Phases 1–3*

This file exists so nobody has to guess. Every commitment made in the design
document for Phases 1–3 is listed below with an honest verdict: **shipped**,
**partial**, or **deferred** (with the phase it belongs to). Items marked
deferred are *not* failures — they belong to a later phase and are listed so the
boundary is visible rather than blurred.

Verification for every "shipped" row: `python3 परीक्षा.py` (conformance),
`cargo test` in `rust-engine/` (unit), `python3 तुल्यता.py` (both engines must
produce byte-identical output).

---

## 1. Language core (Phases 1–2)

| Promise | Status | Where to see it |
|---|---|---|
| Devanagari keywords, `।` statement terminator | shipped | every example |
| Devanagari digits ०–९ everywhere ASCII digits work | shipped | `examples/गणना.सं` |
| Roman aliases, permanent and first-class | shipped | `sanskrita.py --roman`, `--convert` |
| Explicit `{ }` blocks — formatting never changes meaning | shipped | all examples |
| मानय / ध्रुव, यदि / अथ यदि / अन्यथा, यावत्, विरम / अनुवर्त | shipped | `examples/नियन्त्रणम्.सं` |
| विधि functions, first-class, closures, recursion, फलम् | shipped | `examples/विधयः.सं` |
| **Kāraka-labelled arguments in any order** | shipped | `examples/विधयः.सं` |
| सूची and कोशः, 1-based indexing (प्रथमः = १) | shipped | `examples/सूचीकोशौ.सं` |
| प्रत्येकम् … इति for-each | shipped | `examples/नियन्त्रणम्.सं` |
| वर्गः classes, सृज, अयम्, inheritance | shipped | `examples/वर्गाः.सं` |
| प्रयत / दोषे error handling | shipped | `examples/दोषनिवारणम्.सं` |
| क्षिप — raise your own error | shipped (v0.4) | `examples/स्वपरीक्षा.सं` |
| Lambdas (anonymous विधि) | shipped | `examples/प्रतिमानानि.सं` |
| आनय — import your own `.सं` files | shipped | `examples/स्वपरीक्षा.सं` |
| आनय "python:…" — the Python bridge | shipped | `examples/सेतुः.सं` |
| Bilingual errors with line numbers and "did you mean?" hints | shipped | `परीक्षा.py` error cases |

## 2. The Python/Java flaws we said we would fix (design §2b)

| Flaw | संस्कृता's answer | Status |
|---|---|---|
| Java boilerplate | `वद("नमस्ते")।` is a complete program | shipped |
| **NullPointerException** | शून्यम्-safety: a typed variable cannot hold शून्यम् unless declared `नाम?` | shipped (v0.4) |
| Type errors hide until runtime | **प्राक्परीक्षा** — annotations and text/number mixing are proved *before* the program runs; nothing executes if a problem is found | shipped (v0.4) |
| Indentation as syntax | explicit `{ }` | shipped |
| **Mutable default arguments** | defaults store the *expression*, re-evaluated fresh on every call — the classic Python bug cannot occur | shipped (v0.4) |
| `०.१ + ०.२ ≠ ०.३` | exact दशमांशः by default; **द्रुतदशमांशः** is the opt-in fast binary float | shipped (v0.4) |
| Slow interpreted loops | native वेगः engine (Rust), `--druta` compiled mode | shipped (see §5) |
| Checked-exception clutter / silent failures | one प्रयत/दोषे model, errors carry line + suggestion | shipped |
| Cryptic beginner errors | Sanskrit + English on every error | shipped |

`प्राक्परीक्षा` is deliberately conservative: it reports only what it can
**prove** from the source (literal-typed declarations, provable text/number
mixing). Everything it cannot prove is still checked at runtime. A full
whole-program type inference pass is Phase 4 work, and is listed as such below.

## 3. Standard library (design §6)

| Module | Contents | Status |
|---|---|---|
| **गणितम्** | वर्गमूलम्, घातः, लघुगणकः, ज्या, कोज्या, स्पर्शज्या, तलम्, उपरितलम्, निरपेक्षम्, पाई, ई | shipped |
| **वाक्यकर्म** | विभज, संयोजय, खोज, प्रतिस्थापय, अंश, उच्च, निम्न, परिष्कार, आरभते, अन्तयति, अन्तर्भवति | shipped |
| **सूचीकर्म** | छानय, प्रतिचित्रय, न्यूनीकरण, विपर्यय, अन्तर्भवति, अनुक्रमः, योगः, महत्तमम्, लघुत्तमम्, अद्वितीयम् | shipped (v0.4) |
| **कालः** | अद्य, संप्रति, वर्षः, क्षणविरामः | shipped |
| **सञ्चिका** | पठ, लिख, योजय, अस्ति, निष्कासय, पङ्क्तयः, सूचिका | shipped (v0.4) |
| **जेसन** | विश्लेषय, पाठय — a decimal survives the round trip exactly, scale and all, because it never becomes a float | shipped (v0.4) |
| **यादृच्छिकम्** | अन्तरे, वरय, भिन्नम् | shipped |
| **संस्कृतम्** | अक्षराणि, अक्षरगणना, मात्राः, छन्दः, रोमनय, देवनागरय, संधय | shipped |
| **कृत्रिमबुद्धिः** (AI/ML) | — | deferred to Phase 3–4 (today: via the Python bridge) |

Sandhi-style composition (design §7d #2) is shipped: `.नाम` on a सूची, वाक्यम्
or कोशः resolves to the same stdlib function, receiver first — so
`अङ्काः.छानय(f).प्रतिचित्रय(g).योगः()` and the module spelling are the same code.
See `examples/शृङ्खला.सं`.

## 4. Phase 3 deliverables (design §8)

| Deliverable | Status |
|---|---|
| **Rust engine (वेगः)** — complete language, 104 unit tests, differential-tested against the reference | shipped |
| **Standard library** | shipped (table above) |
| **Installer** | shipped — `install.sh`, `pip install sanskrita`, `--veg` builds the native engine on first use |
| **One CLI, two engines** — `sanskrita prog.सं` (reference) / `sanskrita --veg prog.सं` (native) | shipped (v0.4) |
| **REPL on both engines** | shipped (v0.4) |
| **Docs — Sanskrit + English** | shipped (README, TUTORIAL, AI-SPEC, design-patterns guide) |
| **Docs — Hindi** | shipped (v0.4) — `docs/README.hi.md` |
| **Technical glossary** (risk #11) | shipped (v0.4) — `docs/शब्दकोशः.md` |
| **CI, benchmarks, landing page** | shipped |
| Paṇḍit review of every keyword (risk #12) | **open** — budgeted before v1.0; the grammatical rationale is written, the external review is not done |
| Institutional anchor (risk #16) | **open** — a Phase 3–4 outreach task, not a code task |

## 5. Speed and memory (design §7c) — the honest numbers

| Mode | What it is | Measured |
|---|---|---|
| Reference engine (`sanskrita.py`) | Python-hosted tree-walker. The specification, not the product. | 29–402× slower than CPython on `मापनम्.py` |
| **वेगः** (`--veg`, Rust) | native tree-walker, single binary, no runtime | see `BENCHMARKS.md` |
| `--druta` | संस्कृता → C transpile + binary cache, subset of the language | 1000–2000× faster than the reference engine, measured |

We publish the unflattering number as loudly as the flattering one. A
Python-hosted tree-walker *cannot* beat Python; that is exactly why वेगः exists.
The §7c memory-benchmark rule (a table vs Python/Java/Rust every release) is
honoured in `BENCHMARKS.md`.

## 6. Deferred by design — and to which phase

These were never Phase-3 promises. They are listed so the roadmap boundary is
explicit rather than implied.

| Item | Phase | Note |
|---|---|---|
| Bytecode VM + JIT | 4 | the Java/Python road; वेगः is the prerequisite |
| WASM compiler (browser) | 4 | today: the JS playground |
| Native AI/ML library | 3–4 | today: NumPy/pandas/scikit-learn through the bridge |
| Package manager | 4 | today: `.सं` file imports + pip |
| Whole-program type inference | 4 | today: प्राक्परीक्षा proves what it can, runtime checks the rest |
| Parallelism / no-GIL story | 4 | वेगः has no GIL, but no threading API is exposed yet |
| AOT compiler → mobile / games / embedded | 5 | |
| No-GC strict mode (OS, drivers, real-time) | 6 | |
| Python bridge in वेगः | 4 | by design: the bridge is a bootstrap, not a foundation (risk #21). वेगः refuses `python:` imports with a clear message rather than pretending |

## 7. Known, tracked divergences between the two engines

The differential harness (`तुल्यता.py`) is the contract: every listed program
must produce byte-identical output from both engines. Divergences are recorded
openly, never hidden.

**Current ledger: empty.** All three original divergences (integer range,
decimal division, division result type) were closed in slice 5 and promoted into
the always-checked set.

One *intentional* difference remains, documented rather than "fixed":

- `आनय "python:…"` works only in the reference engine. वेगः reports a clear
  bilingual error telling you which engine to use. This is by design — the
  bridge is a bootstrap, not a foundation (risk #21).

---

## How to verify all of this yourself

```bash
python3 परीक्षा.py                 # conformance: micro-tests, error cases, every example
cd rust-engine && cargo test       # वेगः unit tests
python3 तुल्यता.py                 # both engines, byte-identical output required
python3 मापनम्.py                  # benchmarks (§7c)
```

If any of these fails on `main`, that is a bug — please open an issue.
