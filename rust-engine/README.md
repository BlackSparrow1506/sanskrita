# वेगः — the संस्कृता native Rust engine

The Phase 3c rewrite: a compiled, standalone engine for संस्कृता that needs no
Python. The Python interpreter (`../sanskrita.py`) stays as the **reference
implementation**; this engine is correct when it produces identical results and
passes the conformance suite (`../परीक्षा.py`, 51 cases).

## Status: **slices 1–3 — it runs programs** (incremental)

Built incrementally so every step is compilable and testable:

- [x] **Slice 1 — lexer**: tokens, Devanagari + ASCII digits, danda, strings,
  comments, roman aliases, keyword/identifier split, mixed-script guard.
- [x] **Slice 2 — parser**: full AST + recursive-descent for the core subset.
- [x] **Slice 3 — evaluator**: मानय/ध्रुव, assignment, integer arithmetic,
  comparisons, च/वा/न, यदि/अथ/अन्यथा, यावत्, विरम/अनुवर्त, वद, वाक्यम्,
  दैर्घ्यम्, प्रकारः, सङ्ख्या. **Runs real programs natively.**
- [ ] Slice 4 — functions, फलम्, recursion
- [ ] Slice 5 — exact decimals (the correctness promise)
- [ ] Slice 5 — exact decimals (the correctness promise), strings-as-values
- [ ] Slice 6 — lists, maps, classes, प्रयत/दोषे
- [ ] Slice 7 — kāraka arguments, संस्कृतम् stdlib, conformance parity

## Build & test

Requires Rust (install from https://rustup.rs):

```bash
cargo build --release      # produces target/release/sanskrita-veg
cargo test                 # runs the lexer unit tests
cargo run -- ../examples/नमस्ते.सं   # slice 1: prints the token stream
```

## Design

Mirrors `sanskrita.py` deliberately — same token kinds, same keyword set, same
error style (bilingual). Translating a well-tested Python reference into Rust
one slice at a time is far safer than a clean-room rewrite, and the conformance
suite catches any drift.

Target: single binary, ~30–50× faster than the Python interpreter, no runtime
dependency. See `../BENCHMARKS.md` for the numbers this must beat.

जयतु संस्कृतम् ।
