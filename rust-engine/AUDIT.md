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
| 5 | Low | Error format didn't match the reference (single line, ASCII digits). | `err.rs` produces the exact two-line bilingual shape with Devanagari line numbers. |
| 6 | Low | No `--version`/`--help`; dead-code warnings. | Added; warnings silenced with intent documented. |

## Known, tracked divergences (not bugs — decisions pending)

Listed and checked by `तुल्यता.py` on every run, so they can never be forgotten:

1. **Integer range** — reference is arbitrary-precision, वेगः is i64 with a
   checked error. Bignum planned alongside decimals (slice 5).
2. **Decimal division** — `१ / ४` is exact `०.२५` in the reference; वेगः errors
   until slice 5.
3. **Type of exact division** — `प्रकारः(१० / ५)` is दशमांशः vs पूर्णाङ्कः; resolves with slice 5.

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
