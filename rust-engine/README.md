# वेगः — the संस्कृता native Rust engine

A compiled, standalone engine for संस्कृता that needs no Python at all.

`../sanskrita.py` remains the **reference implementation** — it defines the
language. वेगः is correct when it produces *byte-identical* output for every
program in `../तुल्यता.py` and passes `../परीक्षा.py`.

## Status: **complete language** (Phase 3, v0.4.0)

Built in slices so every step stayed compilable and testable:

- [x] **Slice 1 — lexer**: tokens, Devanagari + ASCII digits, danda, strings,
  comments, roman aliases, keyword/identifier split, mixed-script guard, NFC.
- [x] **Slice 2 — parser**: full AST + recursive descent.
- [x] **Slice 3 — evaluator**: declarations, assignment, arithmetic,
  comparisons, च/वा/न, यदि/अथ/अन्यथा, यावत्, विरम/अनुवर्त, core builtins.
- [x] **Slice 4 — functions**: `विधि`, `फलम्`, recursion (stack-measured guard),
  lexical scoping, and **kāraka-labelled arguments** in any order.
- [x] **Slice 5 — exact numbers**: arbitrary-precision integers (`bigint.rs`)
  and exact decimals (`decimal.rs`), both hand-written, **zero dependencies**.
  `०.१ + ०.२ == ०.३`, `२५!` exact, int↔decimal promotion, floored `%`.
- [x] **Slice 6 — collections & objects**: सूची, कोशः, वर्गः with inheritance,
  सृज, अयम्, प्रयत/दोषे, lambdas.
- [x] **Slice 7 — modules & stdlib**: आनय for native modules and the user's own
  `.सं` files; संस्कृतम्, गणितम्, वाक्यकर्म, यादृच्छिकम्, कालः.
- [x] **Phase-3 completion**: क्षिप, शून्यम्-safety with `?`, **प्राक्परीक्षा**
  (pre-flight check), default parameter values, method chaining on built-in
  types, **द्रुतदशमांशः** (opt-in binary float), सूचीकर्म, सञ्चिका, जेसन,
  आदेशचराः, and a REPL.

**Intentionally absent:** `आनय "python:…"`. The Python bridge is a bootstrap,
not a foundation (design risk #21), so वेगः rejects it with a clear bilingual
message naming the engine to use instead. This is the only behavioural
difference between the two engines, and it is deliberate.

## Files

| File | What it holds |
|---|---|
| `lexer.rs` | tokens, danda handling, roman aliases, mixed-script guard |
| `nfc.rs` | dependency-free NFC for the Devanagari block (8 nukta exclusions) |
| `parser.rs` | recursive-descent parser → `ast.rs` |
| `precheck.rs` | प्राक्परीक्षा — everything provable before the program runs |
| `interp.rs` | the evaluator: scope arena, stack-measured recursion guard, builtins |
| `bigint.rs` | arbitrary-precision integers, base-10⁹ limbs |
| `decimal.rs` | exact decimals, 28 significant digits on inexact division |
| `value.rs` | the runtime `Value` enum; `Rc<RefCell<…>>` for shared collections |
| `stdlib.rs` | native modules + the built-in method tables (§7d chaining) |
| `sanskritam.rs` | akṣara, mātrā, chandas, sandhi, IAST ↔ Devanagari |

## Build, test, run

Requires Rust (https://rustup.rs):

```bash
cargo build --release              # → target/release/sanskrita-veg
cargo test                         # unit tests
cargo run --release -- ../examples/नमस्ते.सं
cargo run --release --             # no file → REPL
```

Or from the main CLI, which builds it for you on first use:

```bash
sanskrita --veg ../examples/नमस्ते.सं
```

## The contract

```bash
cd .. && python3 तुल्यता.py
```

Every program in that harness must produce identical output from both engines,
and every program in its `MUST_FAIL` list must fail in both. Divergences are
recorded in `AUDIT.md`, never hidden. **Current ledger: empty.**

## Measured (MacBook, 2026-07-12, slices 1–3)

| Workload | reference engine | वेगः |
|---|---|---|
| loop sum 1..50,000 | ~106 ms | **~20 ms** |
| `examples/द्रुतोदाहरणम्.सं` (loops + recursion + primes) | 120 ms | **8 ms** |

Identical output, verified byte-for-byte. Current numbers live in
`../BENCHMARKS.md`; re-measure with `python3 ../मापनम्.py`.

## Design notes

वेगः deliberately mirrors `sanskrita.py` — same token kinds, same keyword set,
same bilingual error shape. Translating a well-tested reference one slice at a
time is far safer than a clean-room rewrite, and the differential harness
catches any drift the moment it appears.

Two decisions worth knowing:

- **No external crates.** Bignums, decimals and NFC are hand-written, so the
  binary has no supply chain and the memory story (§7c) stays ours to control.
- **The recursion guard measures the stack** rather than counting frames: it
  compares the current stack address against an anchor taken at start-up. Frame
  size changes with build profile and language features, so counting frames was
  wrong twice before this replaced it.

जयतु संस्कृतम् ।
