# CHANGELOG — परिवर्तनसूची

All notable changes to संस्कृता are recorded here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows the policy in [docs/STABILITY.md](docs/STABILITY.md) —
before v1.0, a **minor** bump may contain breaking changes, and every one of
them is listed under **Breaking** below.

---

## [Unreleased]

### Added

- **`गणितम्.परिवृत्त(x, स्थानानि)`** — round to a fixed number of decimal places,
  **half away from zero** (the commercial convention, not banker's rounding),
  keeping exactly that many places so `१००` prints as `१००.००`. Division still
  uses half-even at 28 significant digits; this is the explicit *"and now make it
  money"* step. Without it, a payroll total ran to 28 digits — which is exact,
  and useless on a payslip.
- **`वाक्यकर्म.पूरय(पाठः, विस्तारः)`** — pad to a width; a negative width pads on
  the left. Counts characters, not display columns, and says so.
- **Three programs that are jobs, not demos**, each running identically on both
  engines and each in the differential harness where its output is deterministic:
  - `examples/वेतनपत्रम्.सं` — a payroll run. Prorated salary, PF, slab tax,
    rows rejected *with reasons*, CSV and JSON output, and an audit digest.
    Totals reconcile to the paisa.
  - `examples/लेखापरीक्षा.सं` — server log triage: parse, group by level, find
    the slowest requests, per-route averages. No regex, so वेगः runs it too.
  - `examples/आदेशसाधनम्.सं` — a CLI utility fit for a cron job: arguments,
    an environment variable, report on stdout, complaints on stderr, and real
    exit codes. `परीक्षा.py` now checks its exit code rather than skipping it.

## [0.5.0] — 2026-07-25 — "enterprise readiness"

Three tiers of work from an honest audit of what "industrial grade" actually
requires. Nothing here is a new headline feature; all of it is what a language
needs before someone can responsibly build on it.

**Breaking:** none. Every program that ran on 0.4.0 runs unchanged — the error
value still prints as its message, and trailing commas only *add* what was
previously rejected.

### Added

- **`यादृच्छिकपरीक्षा.py` — property-based differential testing.** Generates
  random valid संस्कृता, runs it on both engines, and demands identical output
  and no crashes; shrinks any failure to the smallest reproducing program. Now
  in CI. It found two real bugs on its first run (below).
- **Trailing commas** are accepted everywhere a comma-separated list appears —
  literals, parameter lists, argument lists. (वेगः already allowed them in
  literals; the reference did not. That silent divergence is now closed.)
- `SECURITY.md`, `CODE_OF_CONDUCT.md`, `CHANGELOG.md`, issue and pull-request
  templates, dependabot config
- `docs/GRAMMAR.md` — the complete formal grammar (EBNF), normative for both
  engines
- `docs/STABILITY.md` — versioning policy, the list of decisions that are frozen
  forever, and the checklist that has to be true before v1.0
- **Five new standard-library modules**, in both engines unless noted:
  - **कालः grows into a real date library** — वासरः (weekday), दिनयोगः (add
    days), अन्तरम् (days between), पूर्वम् (before?), शुद्धः (valid?), रूपय
    (format), अधिवर्षः (leap year), मासः, दिनम्, कालमुद्रा. Accepts Devanagari
    or ASCII digits, and rejects ३०-February rather than quietly accepting it.
  - **सारणी** — CSV to RFC 4180, including quoted fields with embedded commas
    and newlines. Every field arrives as वाक्यम्: numbers are never guessed at,
    because a parser guessing is how leading zeros and IDs get destroyed.
  - **गूढ** — SHA-256, UUID4, base64. One hash algorithm on purpose; no
    encryption at all, deliberately.
  - **परिवेशः** — environment variables, exit codes, stderr, working directory,
    platform. This is what makes a संस्कृता program usable in a pipeline.
  - **लेखनी** — five-level logging to stderr or a file, so logs never pollute a
    program's real output.
  - **नियमितम्** (regex) — reference engine only for now; वेगः says so plainly
    rather than diverging. See STATUS.md §7.
- सूचीकर्म gains **क्रमय** (sort, with an optional key function) and the set
  operations सङ्गमः, सम्पातः, भेदः. वाक्यकर्म gains **आकारय** (formatting) —
  and a दशमांशः keeps its exact digits through it.

- **Errors are values now, and they remember where they came from.**
  - An uncaught error prints the **whole call stack**, outermost first, instead
    of a single innermost line. In a five-deep chain that is the difference
    between a usable report and a shrug.
  - `दोषे (त्रु)` binds an error object: `त्रु.सन्देशः`, `त्रु.आङ्ग्लसन्देशः`,
    `त्रु.पङ्क्तिः`, `त्रु.प्रकारः`, `त्रु.अनुरेखा`. It still *prints* as its
    Sanskrit message, so every program written before this change behaves
    exactly as it did.
  - Errors carry a **kind** you can branch on — `गणितदोषः`, `नामदोषः`,
    `प्रकारदोषः`, `सीमादोषः`, `आयातदोषः`, `व्याकरणदोषः`, `स्वयंदोषः`, `दोषः`.
    One catch block, different responses.
  - `क्षिप त्रु।` re-raises a caught error **unchanged** — same kind, same line,
    same traceback. Handling what you understand and passing on what you do not
    is finally expressible.

### Fixed

- **Decimal `+`, `-`, `*` were silently rounded to 28 significant digits** in the
  reference engine, so `१०५९२७०२७९०६७५४१३१७७२७०००४.३३३३ + ०.०००१` lost the
  addend. Exact at any size now. (वेगः was already correct — this was a
  divergence as well as a bug.)
- **Decimal `%` crashed** with an unhandled `decimal.InvalidOperation` on large
  operands. Both engines now take the floored remainder on the unscaled
  integers: no precision ceiling, no rounding step. `(०-७.५) % ३` is `१.५`.

### Changed

- `मापनम्.py` rewritten: measures both संस्कृता engines, Python, Java, Rust and C
  — wall-clock time **and** peak RSS, each in its own process — and regenerates
  `BENCHMARKS.md`. Absent toolchains are written in as *not measured*, never
  estimated. Design §7c is now honoured in full.

## [0.4.0] — 2026-07-25 — "फलम् complete" (Phase 3)

The phase that was supposed to deliver a native engine and a standard library.
Both landed — and an audit against the design document found five promises that
had never actually been built, plus one real correctness bug. All six are fixed
here. The promise-by-promise verdict is in [STATUS.md](STATUS.md).

### Added — engine

- **वेगः, the native Rust engine.** The complete language in a dependency-free
  binary: hand-rolled arbitrary-precision integers, exact decimals, NFC
  normalization, and a stack-measuring recursion guard. Run it with
  `sanskrita --veg prog.सं`, or with no argument for a REPL.
- **One CLI, two engines.** `--veg` builds the native engine on first use and
  hands the program to it. `तुल्यता.py` requires byte-identical output from both
  on every push.

### Added — language

- **`क्षिप`** — raise your own error, catchable by `प्रयत/दोषे`.
- **शून्यम्-safety** (design §2b). A typed variable cannot hold `शून्यम्` unless
  declared nullable: `मानय नाम? : वाक्यम् = शून्यम्।`
- **प्राक्परीक्षा — checks before the program runs** (design §2b). Annotation
  violations and provable text/number mixing are reported *together*, and
  nothing executes. Deliberately conservative: it reports only what it can prove
  from the source; the runtime check still stands behind it.
- **Default parameter values.** `विधि नम(क, ख = ५) { … }`. The *expression* is
  stored and re-evaluated in the callee's scope on every call, so Python's
  mutable-default bug is structurally impossible.
- **द्रुतदशमांशः** — the opt-in IEEE-754 binary float (design §2b). Exact
  `दशमांशः` remains the default; ask for speed by name and you get a value that
  prints exactly what it holds, `०.३०००००००००००००००४` included.
- **Method chaining** on built-in types (design §7d #2 "sandhi-style
  composition"). `अङ्काः.छानय(f).प्रतिचित्रय(g).योगः()` — `.नाम` on a
  `सूची`/`वाक्यम्`/`कोशः` resolves to the same stdlib function with the receiver
  as its first argument.
- **`आदेशचराः()`** — the arguments given to your program, as a `सूची`.

### Added — standard library (design §6 now complete)

- **सूचीकर्म** — छानय, प्रतिचित्रय, न्यूनीकरण, विपर्यय, अन्तर्भवति, अनुक्रमः,
  योगः, महत्तमम्, लघुत्तमम्, अद्वितीयम्
- **सञ्चिका** — पठ, लिख, योजय, अस्ति, निष्कासय, पङ्क्तयः, सूचिका (UTF-8, always)
- **जेसन** — विश्लेषय, पाठय. Hand-written on both sides so a decimal survives the
  round trip *exactly*, scale and all
- **वाक्यकर्म** grows — उच्च, निम्न, परिष्कार, आरभते, अन्तयति, अन्तर्भवति
- **कालः** grows — क्षणविरामः

### Added — docs and tooling

- [STATUS.md](STATUS.md) — every Phase 1–3 promise with an honest verdict
- [docs/README.hi.md](docs/README.hi.md) — Hindi documentation (design §8 asked
  for Sanskrit + English + Hindi)
- [docs/शब्दकोशः.md](docs/शब्दकोशः.md) — the technical glossary risk #11 asked
  for, each term with its derivation, offered for paṇḍit review
- [docs/GRAMMAR.md](docs/GRAMMAR.md) — the complete formal grammar (EBNF)
- [docs/STABILITY.md](docs/STABILITY.md) — versioning and compatibility policy
- [SECURITY.md](SECURITY.md), [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), issue
  and pull-request templates
- `examples/परीक्षणम्.सं` — a test framework written **in** संस्कृता, needing
  nothing outside the core; `examples/स्वपरीक्षा.सं` uses it
- New examples: `सूचीकर्मोदाहरणम्.सं`, `जेसनोदाहरणम्.सं`, `शृङ्खला.सं`,
  `शुद्धिवेगौ.सं`

### Fixed

- **Exactness was lost at the native-module boundary.** `to_sk()` did not
  recognise `Decimal`, and `to_py()` converted decimals to floats on the way
  into native modules. `सू.योगः([०.१, ०.२])` answered `०.३०००००००००००००००४`;
  `ज.पाठय({"क": ०.१०})` wrote `0.1`. The language's headline promise was false
  inside its own standard library. Fixed with a `RawFn` marker: native functions
  that touch numbers now receive संस्कृता values untouched.
- वेगः accepted any identifier as a type name after `:`; it now validates
  against the same seven names the reference does.
- `प्रकारः(instance)` returned `वस्तु` in वेगः and the class name in the
  reference. वेगः now returns the class name; divergence closed.
- 62 MB of `rust-engine/target/` build artefacts were tracked in git despite a
  later `.gitignore` rule. Untracked; `build/` and `*.egg-info/` too. Tracked
  files drop from 825 to 211.

### Known limitations (stated, not hidden)

- Errors are still plain text — no stack trace, no error object. This is the
  largest remaining gap and is the next thing being built.
- No networking, concurrency, regex, CSV, or date arithmetic.
- `BENCHMARKS.md` carries v0.3 numbers with no वेगः, Java or Rust columns, so
  design §7c's benchmark rule is **partially** honoured, not fully.
- Paṇḍit review of the keyword set (risk #12) has not happened.

## [0.3.1] — 2026-07-20

### Added

- `--druta` experimental compiled mode: संस्कृता → C, compiled and cached
  (instant re-run). Subset only — integers, loops, functions, `यदि`, `वद`.
  Measured 1000–2000× faster than the reference engine on that subset.
- Interpreter hot-path optimizations: `__slots__`, inlined variable lookup and
  call paths, integer fast-path arithmetic. Loops ~1.4× faster.
- Lambdas (anonymous `विधि`), class inheritance (`वर्गः X : Y`), visarga sandhi.
- pip packaging (`pyproject.toml`), `--version`.

## [0.3.0] — 2026-07-16 — "फलम् begins"

### Added

- Import your own `.सं` files: `आनय "सहायः.सं" इति सहायः।`
- **वाक्यकर्म** string module; `परिधिः(a, b)` inclusive range;
  `अपनय(कोशः, कुञ्जिका)`
- GitHub Actions CI on every push; `मापनम्.py` benchmarks; landing page in
  `docs/`

### Changed

- NFC normalization is now **mandatory** — visually identical Devanagari is
  identical to the engine (design §10 #8)
- Mixed-script identifiers (`नामx`) are rejected at lex time (design §10 #7)

## [0.2.0] — 2026-07-07 — "वृक्षः" (Phase 2)

### Added

- `विधि` functions — first-class, closures, recursion, `फलम्` returns
- **Kāraka-labelled arguments** in any order — the language's signature feature
- `सूची` lists and `कोशः` maps, **1-based** (प्रथमः = १)
- `प्रत्येकम् … इति` for-each; `वर्गः` classes with `सृज` and `अयम्`
- `प्रयत/दोषे` error handling; `आनय "python:…"` bridge
- `संस्कृतम्` library: sandhi, metre detection, syllable counting,
  Devanagari ↔ IAST

## [0.1.0] — 2026-07-06 — "अङ्कुरः" (Phase 1)

### Added

- The first working engine: `मानय`/`ध्रुव`, exact decimal arithmetic,
  `यदि`/`अथ यदि`/`अन्यथा`, `यावत्` with `विरम`/`अनुवर्त`, `वद`, `पृच्छ`
- Devanagari digits ०–९ alongside ASCII; danda `।` as statement terminator
- Roman aliases for every keyword; `--convert` to canonical Devanagari
- Bilingual (Sanskrit + English) errors with line numbers and "did you mean?"
- Browser playground, VS Code extension, conformance suite, 10 examples

[0.5.0]: https://github.com/BlackSparrow1506/sanskrita/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/BlackSparrow1506/sanskrita/compare/v0.3.0...v0.4.0
[0.3.1]: https://github.com/BlackSparrow1506/sanskrita/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/BlackSparrow1506/sanskrita/releases/tag/v0.3.0
