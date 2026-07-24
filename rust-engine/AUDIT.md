# वेगः — audit of slices 1–3

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
