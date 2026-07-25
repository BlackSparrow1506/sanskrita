# वेगः — audit log

*Newest section first. §1 is the Phase-3 completion audit; §2 is the
original slice 1–3 audit that established the method.*

---

# 0. Enterprise-readiness audit (v0.5.0, 25 July 2026)

Question asked: *is this industrial and enterprise ready?* Answer at the time:
no — and here is exactly why. Three tiers of gaps were found and closed.

## Tier 1 — errors (the largest gap)

An uncaught error printed **one line**: the innermost one. In a five-deep call
chain that is true and useless. And `दोषे (त्रु)` bound a plain `वाक्यम्`, so a
program could not tell a division by zero from a missing file, could not read
the line number, and could not re-raise what it did not understand.

Fixed in both engines: errors carry a **kind**, a line, both messages and a
**call stack**; `दोषे` binds a value that still *prints* as its message (so no
existing program changed) but answers `.सन्देशः .आङ्ग्लसन्देशः .पङ्क्तिः
.प्रकारः .अनुरेखा`; `क्षिप त्रु।` re-raises unchanged.

Implementation note worth remembering: the वेगः error channel is
`Result<_, String>` at over a hundred sites. Rather than refactor all of them,
`err.rs` **encodes** the structure into that string with a marker no source file
contains, and decodes at the three places that need it — प्रयत, the call
boundary, and the top level. Eight touch points instead of a hundred and twenty.

## Tier 2 — the standard library people actually need

`कालः` was four functions with no arithmetic; there was no regex, no CSV, no
hashing, no environment access, no logging. Added: `कालः` as a real date
library, `सारणी` (CSV, RFC 4180), `गूढ` (SHA-256, UUID, base64), `परिवेशः`
(env, exit codes, stderr), `लेखनी` (logging), plus `सूचीकर्म.क्रमय` with a key
function, set operations, and `वाक्यकर्म.आकारय`.

**`नियमितम्` (regex) is reference-engine only, and वेगः says so.** वेगः has no
external crates by design, and a hand-written engine that was not byte-identical
to Python's `re` would make the same pattern mean two things. An honest gap beats
a silent divergence. Tracked as the next वेगः slice.

## Tier 3 — the things a serious project has

`SECURITY.md`, `CODE_OF_CONDUCT.md`, `CHANGELOG.md`, issue and PR templates,
dependabot, `docs/GRAMMAR.md` (the full EBNF), `docs/STABILITY.md` (versioning,
the frozen decisions, the v1.0 checklist), and a rewritten `मापनम्.py` that
measures both engines against Python, Java, Rust and C — time **and** peak RSS,
each in its own process — so design §7c is finally honoured rather than claimed.

## Issues found and fixed in this pass

| # | Severity | Issue | Fix |
|---|---|---|---|
| 16 | **High** | Decimal `+`, `-`, `*` in the reference engine were silently rounded to 28 significant digits by Python's default context. `१०५९२७०२७९०६७५४१३१७७२७०००४.३३३३ + ०.०००१` lost the addend — the language's headline promise, false above 28 digits. वेगः was already correct, so this was a divergence too. | Exact at any size, in both engines. Found by the new property tester. |
| 17 | **High** | Decimal `%` raised an unhandled `decimal.InvalidOperation` on large operands — an engine crash, not a संस्कृता error. वेगः computed it as `a - floor(a/b)*b`, which depends on how the division rounds. | Both engines now take the floored remainder on the **unscaled integers**: no precision ceiling, no rounding step. `(०-७.५) % ३` is `१.५`. |
| 18 | Medium | वेगः accepted trailing commas in list and map literals; the reference rejected them. A silent divergence no test had noticed. | Trailing commas are now accepted everywhere a comma-separated list appears, in both engines, and are in the differential harness. |
| 19 | Medium | `STATUS.md` claimed design §7c's benchmark rule was honoured. `BENCHMARKS.md` carried v0.3 numbers with no वेगः, Java or Rust column. | The claim was corrected first, then made true: `मापनम्.py` rewritten to measure everything present and to write *not measured* — never an estimate — for what is absent. |

## The new test layer

`यादृच्छिकपरीक्षा.py` generates random valid संस्कृता, runs it on both engines,
and demands identical output and no crashes, shrinking any failure to the
smallest reproducing program. **It found issues 16 and 17 on its first run** —
after four layers of hand-written tests had passed. It is now in CI.

---

# 1. Phase-3 completion audit (v0.4.0, 25 July 2026)

Method, unchanged from slice 3: read the design document promise by promise,
read the Rust against the Python reference, and encode every finding as a test
before calling anything done.

## What was audited

Every commitment in the design document for Phases 1–3 — §2b (the Python/Java
flaws), §6 (standard library), §7d (what Sanskrit itself gives us), §8 (Phase 3
deliverables), and the §10 risk register. The full promise-by-promise verdict is
in `../STATUS.md`; this section records only what the audit *found wrong*.

## Issues found and fixed

| # | Severity | Issue | Fix |
|---|---|---|---|
| 8 | **High** | `to_sk()` in the reference engine did not recognise `Decimal`, so every number coming back from a native module was wrapped as an opaque `python-वस्तु`. `जेसन.विश्लेषय("{\"x\": 0.1}")["x"] + ०.२` raised "expected numbers" — the exact-decimal promise silently broke at the module boundary. | `to_sk` now returns `Decimal` unchanged. Regression test + differential case added. |
| 9 | **High** | §2b promised annotations "checked *before* the program runs"; both engines checked them at execution time, so a program could print output and *then* fail on a provable type error. | New `precheck.rs` / `precheck()` pass in both engines. Provable problems are reported together and nothing executes. Deliberately conservative — it reports only what it can prove. |
| 10 | Medium | §2b promised frozen default arguments; the language had **no default arguments at all**, so the promise was vacuous. | `Param.default` stores the *expression*, evaluated fresh in the callee's scope on every call. Python's mutable-default bug is structurally impossible. Tests in both engines. |
| 11 | Medium | §7d #2 promised sandhi-style composition (`सूची.क्रमय().विपर्यय()`); attribute access on built-in types was a hard error. | Method tables in `stdlib.rs` + `Value::BoundNative`. `.नाम` resolves to the same stdlib function with the receiver first — one implementation, two spellings. |
| 12 | Medium | §2b promised `द्रुतदशमांशः` as the explicit fast path; it did not exist, so "exact by default" had no counterpart to be default *over*. | `Value::Flt` + the `द्रुतदशमांशः()` builtin in both engines, contagious through arithmetic, printed exactly as IEEE-754 holds it. `fmt_f64` reproduces Python's `repr(float)`; verified against 12,000 random values. |
| 13 | Low | वेगः accepted any identifier as a type name after `:`; the reference rejects unknown types. Silent divergence. | The Rust parser now validates against the same seven type names. |
| 14 | Low | `प्रकारः(instance)` returned `वस्तु` in वेगः and the class name in the reference. | वेगः now returns the class name. Divergence closed rather than documented. |
| 15a | **High** | `सूचीकर्म` and `जेसन` in the reference engine ran their arguments through `to_py`, which turns a `Decimal` into a **float**. `सू.योगः([०.१, ०.२])` answered `०.३०००००००००००००००४`, `सू.अद्वितीयम्([०.१०, ०.१])` lost the scale, and `ज.पाठय({"क": ०.१०})` wrote `0.10` as `0.1`. The language's headline promise was false at the library boundary. | New `RawFn` marker: native functions that touch numbers receive संस्कृता values untouched. सूचीकर्म rewritten on exact values; जेसन hand-written on both sides (`parse_float=Decimal` in, digit-preserving writer out). वेगः gained exponent expansion (`1e5` → `100000`) so JSON exponents stay exact there too. Four conformance cases + four differential cases added. |
| 15 | Low | 62 MB of `rust-engine/target/` build artefacts were tracked in git despite a `.gitignore` rule added later; `build/` and `*.egg-info/` too. | `git rm -r --cached`, `.gitignore` extended. Repo drops from 825 to 211 tracked files. |

## Not fixed — recorded instead

| Item | Why |
|---|---|
| Paṇḍit review of every keyword (risk #12) | An external review, not a code change. Budgeted before v1.0; `docs/शब्दकोशः.md` now gives the reviewer a single document to mark up, with each term's derivation stated. |
| `आनय "python:…"` in वेगः | Intentional. The bridge is a bootstrap, not a foundation (risk #21). वेगः reports a clear bilingual error naming the engine to use. |
| Whole-program type inference | Phase 4. `प्राक्परीक्षा` proves what it can from the source; the runtime check still stands behind it. Claiming more would be dishonest. |

## Quality gates after this audit

- `cargo test` — 104 unit tests across lexer, NFC, bignum, decimal, precheck, evaluator, stdlib
- `तुल्यता.py` — differential harness: 68 programs byte-identical, 15 must fail
  in both, 20 whole example programs diffed
- `परीक्षा.py` — 67 micro-tests + 14 error cases + every example + converter sync
- CI runs `cargo test`, `clippy`, `fmt`, `परीक्षा.py` and the differential
  harness on every push

## Honest status

The language is **feature-complete for Phases 1–3** and defended at three levels
(unit, differential, conformance). What that does *not* mean: it has not been run
in production by anyone, the standard library is small next to Python's, and the
AI/ML, WASM, mobile and bare-metal stories are Phase 4–6 work that has not
started. `../STATUS.md` states the boundary explicitly so nobody has to infer it.

---

# 2. Audit of slices 1–3

Reviewed before beginning slice 4. Method: read the Rust against the Python
reference line by line, hunt for semantic divergence, then encode every finding
as a test.

## Issues found and fixed

| # | Severity | Issue | Fix |
|---|---|---|---|
| 1 | **High** | The danda `।` (U+0964) sits *inside* the Devanagari Unicode block, so range-based identifier scanning swallowed statement terminators (`इ।` became one identifier). | Exclude U+0964/U+0965 from identifier chars; 3 regression tests. |
| 2 | **High** | Integer overflow: Rust panics in debug and **silently wraps in release**, where the Python reference has arbitrary precision. A wrong answer with no error is the worst failure mode. | All arithmetic uses `checked_*`; numeric literals too. Overflow now raises a bilingual error. |
| 3 | **High** | `%` semantics: Rust truncates (`-७ % ३ == -१`), Python floors (`== २`). Silent wrong answers on negative operands. | Floored remainder implemented to match the reference; test added. |
| 4 | Medium | No Unicode normalization, so `क़` typed as one codepoint ≠ `क + ़` typed as two — the "ghost bug" §10 #8 forbids. | `nfc.rs`: dependency-free NFC for the Devanagari block (the 8 nukta composition exclusions); test added. |
| 5 | Low | Error format didn't match the reference (single line). | Messages now use the reference's two-line bilingual shape. **Partially done:** the line number still prints in ASCII on the Sanskrit line (५ vs 5); `err.rs` holds the correct formatter and becomes the error type in slice 5. |
| 7 | **High** | Recursion guard was set above what the stack could hold, so runaway recursion crashed the process (stack overflow) instead of erroring — caught by our own test. | Depth limit is now conservative by default (400, safe for a 2 MB test thread on any build profile) and configurable; the binary runs the program on a 256 MB stack with a 20,000 limit. Deep recursion works; runaway recursion always errors cleanly. |
| 6 | Low | No `--version`/`--help`; dead-code warnings. | Added; warnings silenced with intent documented. |

## Known, tracked divergences — **all resolved in slice 5** ✓

The three entries below were tracked openly rather than hidden; slice 5 closed
each, and they are now ordinary must-agree cases in `तुल्यता.py`:

1. ~~**Integer range**~~ → वेगः has arbitrary-precision integers (`bigint.rs`).
   `२५!` and `९२२३३७२०३६८५४७७५८०७ + १` are exact in both engines.
2. ~~**Decimal division**~~ → `१ / ४` is exact `०.२५` in both (`decimal.rs`).
3. ~~**Type of exact division**~~ → `/` always yields दशमांशः in both, so a
   result's *type* never depends on its runtime values.

**Ledger status: empty.** Any future divergence gets added here the moment it
is found.

### Notes from slice 5 (worth remembering)

- Two "failures" turned out to be **wrong tests, not a wrong engine**:
  `५.० + ५.०` really is `१०.०` (decimal arithmetic preserves scale) and
  `सङ्ख्या("४.५") + ०.५` really is `५.०`. Checking against the reference before
  "fixing" the code is why the differential harness exists.
- The old `arithmetic_overflow_errors` test became obsolete by design: with
  bignums there is nothing to overflow, so it was replaced by a test asserting
  the correct large-number answer.

## Quality gates now in place

- `cargo test` — 20 unit tests (lexer, NFC, evaluator, regressions)
- `तुल्यता.py` — differential harness: 24 programs must produce byte-identical
  output from both engines, 7 must fail in both, 3 divergences tracked
- CI runs `cargo test`, `clippy`, `fmt`, and the differential harness on every push
- `परीक्षा.py` — 51-case conformance suite (the completion criterion for वेगः)

## Honest status

Slices 1–3 are **correct for the subset they implement** and now defended by
tests at three levels (unit, differential, conformance). They are *not* a
complete language yet: no functions, decimals, collections, classes, or stdlib.
"Enterprise-ready" applies to the engineering process — tested, reviewed, CI-gated,
divergences tracked — not yet to feature coverage. Slice 4 begins from this base.
