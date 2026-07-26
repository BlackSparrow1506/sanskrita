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

## Found by running `cargo test` on the real machine (v0.5.1)

The Rust engine is written here and compiled on Gauri's Mac, so `cargo test` is
where blind mistakes surface. Three did, and one of them was not a mistake at
all but a missing feature nobody had noticed.

| # | Severity | Issue | Fix |
|---|---|---|---|
| 20 | **High** | **वेगः never supported lambdas.** `विधि(क) { … }` as a *value* — passed to `सू.छानय`, stored in a सूची, returned from a विधि — parsed only in statement position. The reference has had them since v0.3.1, and the slice-6 notes claimed them. Every वेगः test that passed happened to use *named* functions, so nothing caught it: a language feature missing from one engine for two releases. | `Expr::Lambda` added through ast → parser → interp → precheck. Five conformance cases, four differential cases, including closures and lambdas inside collections. |
| 21 | Medium | My first traceback implementation wrapped `call_function` in a thin outer function so it could append a frame on the way out. That cost **one native stack frame on every call** — and a tree-walker pays for every frame several times over. A 25-deep factorial went over the recursion guard. | Frames are appended only while *unwinding*, so the normal path costs nothing. |
| 22 | Low | A first attempt at the traceback snapshotted the whole call stack at the innermost frame. That would have given a `प्रयत` sitting half-way up frames from *above* itself, which the reference does not do. Caught by reasoning about the semantics before shipping, not by a test. | Each विधि appends its own frame as the error escapes it — so a catch half-way up sees only what the error actually passed through. Differential case added. |

## Found by the property tester, once it could finish (v0.5.1)

With the hang fixed, `यादृच्छिकपरीक्षा.py` found **five real divergences in
eight seconds** — all of them the same defect.

| # | Severity | Issue | Fix |
|---|---|---|---|
| 30 | **High** | **`Decimal::div` in वेगः was wrong.** It grew the numerator one digit at a time, counting digits with `BigInt::digits()`, and stopped when the quotient "looked" long enough. On small quotients it returned **30 or 34 significant digits instead of 28**; on others it returned a different *value* — `१२७.७७८८९४४७२३६१८०९०४५२२६१३०७` came back as `१२७.७७८८९४४७२३६१८०९००००००००१०००००००`. Division is the one operation in this language that is allowed to be inexact, which makes it exactly the operation that must be inexact *identically* in both engines. | Rewritten: the shift is computed directly from the operand digit counts, corrected by at most one step, and rounding happens once. **The algorithm was validated in Python against the reference on 20,000 random operand pairs — zero mismatches — before a line of Rust was written.** Ten unit tests plus six differential cases, taken from the actual failures. |

This is the clearest argument for the property tester in the whole project.
Division had unit tests, differential tests and a conformance case; all of them
passed. Every one of them used operands a human would think to write. The
generator wrote `११३ / ७५१३०६८१६४५३८९८३८९८५८९९७` and the bug fell out
immediately.

## Found by a CI job that ran for 13 hours (v0.5.1)

The property tester wedged GitHub Actions. Not slow — **stuck**, with no
timeout anywhere to stop it.

**Root cause, in the generator, not the engines.** Loop bodies were built as
`स = स + <any expression>`, and that expression could reference the accumulator
itself: `स = स + (स * स)` squares it every iteration. Twelve iterations starting
from १०²⁴ produces a number with roughly **98,000 digits**. Both engines compute
it *correctly* — that is the uncomfortable part — but वेगः's hand-written
`BigInt` uses schoolbook O(n²) multiplication, so a single such multiply is on
the order of 10⁸ limb operations, compounding each round. Python's `int` switches
to Karatsuba above a threshold and shrugs it off; वेगः does not.

| # | Severity | Issue | Fix |
|---|---|---|---|
| 27 | **High** | A generated program could grow super-exponentially, so CI hung for 13 hours. | The loop accumulator is now excluded from its own update expression, and loop bodies use small literals. Generated programs are provably quick, not merely provably valid. |
| 28 | **High** | Nothing could stop a hang: the reference engine ran **in-process with no timeout**, and the run had no wall-clock limit. | Both engines now run as subprocesses with a per-program timeout (15s default); the whole run has a `--budget` (600s default) and stops cleanly when spent; `shrink()` is capped and skips timeout failures instead of re-running them forty times. Running the reference out-of-process also makes a Python traceback visible as a crash rather than something we catch. |
| 29 | Medium | The workflow had no `timeout-minutes` on any job or step. | Added throughout: 10 minutes for the reference job, 20 for वेगः, and tighter per-step limits. A hung job now fails in minutes with a log. |

**The honest performance note this exposed:** वेगः's bignum multiplication is
schoolbook. It is exact and dependency-free, which is what §7c asked for, but it
is asymptotically worse than CPython's. For numbers up to a few thousand digits
this is invisible; at ~100,000 digits वेगः is dramatically slower than the
reference engine it is supposed to replace. Karatsuba is the fix and is not yet
written — recorded in `../STATUS.md` rather than discovered later by a user.

## Found by running `तुल्यता.py` on the real machine (v0.5.1)

`cargo test` passed and the differential harness still reported **12
divergences**. Unit tests check what one engine does; only the differential
harness checks that the two engines are the same language.

| # | Severity | Issue | Fix |
|---|---|---|---|
| 23 | **High** | **वेगः had no closures.** Every call's scope was parented to the *globals*, so a विधि defined inside another विधि could not see the enclosing variables — while the reference has had proper lexical closures since v0.2. The scope arena (`Vec<Scope>` indexed by number, popped after each call) made capture impossible by construction. | Scopes are now an `Rc<RefCell<Scope>>` chain, and a `Function` carries the scope it was **defined** in — exactly what the reference does (`Env(parent=fn.closure)`). A captured scope stays alive as long as the function holding it. Four unit tests and four differential cases, including a counter that mutates its captured variable. |
| 24 | **High** | **Five native modules were unimportable in वेगः.** `सारणी`, `गूढ`, `परिवेशः`, `लेखनी` and `नियमितम्` were implemented in `stdlib.rs` and registered in `members()`, but the *import* statement has its own separate list of module names, and that list was never updated. Every program using CSV, hashing, environment access or logging failed on वेगः — including three of the new real-world examples. | Both lists updated, with a comment at each saying the other exists. |
| 25 | Medium | The reference engine refused to **compare a द्रुतदशमांशः with a number**: `compare()` accepted `(int, Decimal)` only, so `द्रुतदशमांशः(२) > १` raised "तुलना समानप्रकारयोः एव". वेगः allowed it. A number is a number — the reference was wrong. | `float` added to the comparison types. |
| 26 | Low | Two of my own differential cases used `न` (the keyword "not") as a parameter name, so the *reference* errored and the case never tested anything. | Renamed. A test that fails to run is worse than no test. |

**The lesson, plainly:** items 20 and 23 are the same mistake — a feature the
reference had for months, missing from वेगः, invisible because no test in the
differential harness happened to use it. Lambdas and closures are not obscure.
The harness is only as good as its cases, which is why every fix above lands
with a case that would have caught it.

**Worth writing down:** in a **debug** build one संस्कृता call costs tens of
kilobytes of native stack — the evaluator is a large recursive function and
rustc gives every match arm its own slots. So the 1 MB default budget, sized for
a 2 MB test thread, allows only about twenty-five levels of recursion *in tests*.
The shipped binary runs on a 256 MB stack with a 192 MB budget, which is why real
programs recurse thousands deep. Tests that genuinely need depth spawn their own
thread, as `moderate_recursion_works` and `money_and_bignums` now both do.

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

---

## 31. The division bug: a missing `trim`, and three wrong diagnoses

**Symptom.** `Decimal::div` returned 29 or 30 significant digits where the rule
is 28, but only on some operands. `११३ / ७५१३०६८१६४५३८९८३८९८५८९९७` was wrong;
`१/३`, `२/३`, `२७५/३` were right. Found by the property tester.

**Cause.** `BigInt::mul_mag` allocates `a.len() + b.len()` limbs. The product
usually needs one fewer, and `mul_mag` returned the buffer with that leading
zero limb still attached.

Every magnitude in `bigint.rs` carries an unwritten invariant: **it must be
trimmed**, because `cmp_mag` compares limb *count* before contents. `mul()`
trimmed `mul_mag`'s output before anyone saw it, so multiplication was correct
and all multiplication tests passed. But `divmod_mag` used `mul_mag` raw, inside
the binary search that picks each quotient limb:

```rust
let t = Self::mul_mag(b, &[mid as u32]);
if Self::cmp_mag(&t, &rem) != Ordering::Greater { … }
```

With the extra limb, `b × mid` compared as larger than it was. When `rem` had
the same limb count, `t`'s zero top limb made it compare *smaller* than any
`rem`, so the search ran to `BASE-1` — the 27 nines that finally appeared in a
test failure. When `rem` was shorter, `t` compared larger for every `mid`
including `0`, and the quotient limb collapsed to `0`.

Divisors under 10⁹ are one limb and take the `divmod_small` path, which never
calls `mul_mag`. That is the whole reason this survived: every division test
anyone thinks to write by hand uses a small divisor.

**Blast radius.** `/` and `%` on any divisor ≥ 10⁹, and `Decimal::div`, which
shifts the numerator by ~33 digits and therefore always has a multi-limb
divisor in play. A fuzz of 30,000 multi-limb divisions: **11,304 wrong → 0**.

**Fix.** Trim `mul_mag`'s output. One line.

### What this cost, and why

`Decimal::div` was rewritten **three times** — first the digit-counting loop,
then a shift estimate with correction, then a bounds comparison. Each rewrite
was validated in Python against the reference on ~20,000 operand pairs and
passed. Each one still failed in Rust.

The reason the validations passed is the important part: the Python
transcription used Python's own `*` where the Rust used `mul_mag`. The harness
silently replaced the broken component with a correct one. **A differential test
is only as good as its least faithful transcription** — and the least faithful
part was the part nobody suspected.

`Decimal::div` was never the bug. Three rewrites of a correct function.

### The rule this changes

Feature-level tests could not localise this: they only ever said "the quotient
is wrong". What found it was a test that asserts a **primitive**:

```rust
let (q, _) = (113 × 10^50).divmod_trunc(&751306816453898389858997);
assert_eq!(q.digits(), "15040459839476765103808918240");
```

It failed with `999999999999999999999999999` and named the layer immediately.

Two tests now pin the invariant directly, so this class cannot return quietly:

- `mul_mag_never_returns_a_leading_zero_limb` — the invariant itself
- `division_is_exact_for_multi_limb_divisors` — the path `divmod_small` skips

**Standing rule for वेगः:** when a computed result is wrong, test the primitives
underneath it *before* rewriting the algorithm on top. And when transcribing an
algorithm to validate it, transcribe the helpers too — or the harness will
quietly prove the wrong thing.

---

## 32. How a number is *written* is part of the answer

Two divergences survived the `mul_mag` fix (property-tester seeds 151 and 297).
Both engines had the right *value* and disagreed on its *form*.

### The rule the reference actually implements

```python
result = Decimal(a) / Decimal(b)
if result == result.to_integral_value():
    result = result.quantize(Decimal(1))
```

on top of Python's decimal division, which gives:

1. **Exact division sheds trailing zeros only down to the ideal exponent**,
   `exp(dividend) − exp(divisor)`. So `२४४.२० / २` is `१२२.१०`, not `१२२.१`.
   That zero is the precision the operands claimed, and a money column keeps it.
2. **A whole-number result is written as a whole number** — including when the
   division was *inexact*. `ब / (ब+१)` rounds to exactly `१.०००…०` at 28 digits
   and prints as `१`.

The two rules pull in opposite directions, which is why neither a blanket trim
nor a blanket keep can imitate them. वेगः had a blanket trim guarded by
`if r.is_zero() && !round_up`, so it printed `१२२.१` for the first and
`१.०००००००००००००००००००००००००००` for the second.

In `Decimal::div` the ideal exponent was already sitting in the code as `e`
(`other.scale − self.scale`); it simply was not being used as a floor.

`trim_zeros` has been **deleted**, not fixed. Trailing zeros are not noise here:
`०.३०` and `०.३` are the same number but different statements about precision.
Only `div` may drop them, under the two rules above.

### शून्यम् has no sign

`(०-७०) * ०` printed `-०` in the reference and `०` in वेगः. Python's `Decimal`
carries IEEE-754's signed zero; वेगः's `BigInt` uses `sign = 0` for zero and so
cannot represent `-०` at all.

Rather than grow signed zero in वेगः to match, the **reference was changed**:
`_no_signed_zero()` is applied where values are produced (`+ - * / %`,
`सङ्ख्या()`, `गणितम्.परिवृत्त`) and again in `display()`. Two reasons — a
payroll line reading `-०` is indistinguishable from a defect, and normalising
removes the whole class of divergence instead of adding a representation whose
only purpose is to reproduce one.

The scale survives: `-०.००` → `०.००`, never `०`. `द्रुतदशमांशः` is untouched —
it is IEEE-754 by name, and `-०.०` is a real value there.

Applying it in `display()` as well is deliberate belt-and-braces: two leaks
(`गणितम्.परिवृत्त` and `सङ्ख्या()`) were found only by probing the stdlib after
the arithmetic was already fixed, so the printing path is guarded too.

### Method

The new `div` tail was validated in Python against the reference on **25,000**
operand pairs (scales 0–10, magnitudes to 10⁴⁰) before any Rust was written —
this time transcribing `mul_mag`, `divmod_mag` and `cmp_mag` literally rather
than substituting Python's own operators, which is what let §31 hide. Every
expectation in `division_writes_the_result_the_way_the_reference_does` was then
produced by *running* the reference engine, not typed from memory. Two of the
values in that test were wrong when first written and were corrected before the
test was saved.

---

## 33. The reference was wrong, and वेगः was right

Two divergences at seeds 1171 and 1381. In both, the **reference** was the one
that had to change — worth recording, because the reference is the
specification and it is easy to assume the specification is never at fault.

### 1171 — division stopped being exact at 28 digits

`ग*ग / २२५` divides evenly. वेगः printed all 55 digits; the reference printed 28
significant digits and then zeros.

The rule stated in `sanskrita.py` itself, since slice 5, is:

> `/` is exact when it divides evenly, otherwise 28 SIGNIFICANT digits

`Decimal(a) / Decimal(b)` does not implement that rule. The context caps every
division at `prec`, exact or not. So the reference had been quietly breaking its
own written promise for any quotient longer than 28 digits, and nothing noticed
because hand-written tests use small numbers.

वेगः got it right by accident of structure: `num_arith` short-circuits
integer÷integer when the remainder is zero and returns the exact quotient,
never reaching `Decimal::div`. Decimal operands, however, *did* reach it and
were capped — so वेगः was internally inconsistent too.

Both engines now share one rule. A quotient terminates exactly when, in lowest
terms, its denominator has no prime factor besides 2 and 5. Writing
`b = 2^i · 5^j · t` with `t` coprime to ten, that is the same as **t divides a**
— a power of ten can supply twos and fives but never a factor of `t`. So no gcd
is needed: strip the twos and fives, test one division, take `k = max(i, j)`.

### 1381 — a positive exponent escaped and changed a later product

`_स / ५१२` is inexact. Python produced `1.3134…E+46` — a **positive** exponent.
The `quantize(Decimal(1))` step that writes whole numbers plainly then raised
`InvalidOperation` (47 digits exceeds `prec`), and the bare `except: pass`
swallowed it, so the positive exponent survived.

That mattered two operations later. Multiplying by `१५५.००` gave exponent
`+19 + (−2) = +17`, so the product printed with **no** decimal places. वेगः
stores a scale ≥ 0 and materialises the zeros, so its product carried scale 2
and printed `.००`.

The reference's own `_unscaled()` already normalises `exp > 0` away, which says
plainly that positive exponents were never meant to circulate. Division now
returns a scale, never a positive exponent.

### Why this pair was worth the trouble

Both bugs are invisible to any test written against small, tidy numbers. Both
were found by the property tester within 2,000 programs. And both were in the
engine that everything else is checked against — a differential harness proves
two engines *agree*, never that either is *right*, so the spec still has to be
read against its own words from time to time.

### Method

The new `div` was validated by importing the real `sanskrita.py` and comparing
`_div` against a faithful transcription of the Rust over **25,033** operand
pairs — random magnitudes to 10⁴⁵ with scales 0–12, plus 13,000 deliberately
terminating cases (numerators over 2^i·5^j), plus every case from the tests and
both failing seeds. Zero mismatches. Every expectation in
`division_that_divides_evenly_is_exact_at_any_size` was produced by *running*
the reference; the first draft of that test used a dividend ten times too large
and was corrected before it was saved.

Both paths through `div` now end in one shared `write_quotient`, so the exact
and rounded cases cannot drift apart in how they write the answer.
