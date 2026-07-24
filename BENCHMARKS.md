# मितव्यय Benchmarks (§7c rule: honest numbers, every release)

Engine: sanskrita.py v0.3 (Python tree-walking interpreter) — measured 2026-07-25

| Workload | संस्कृता time | Python time | ratio | संस्कृता peak KB | Python peak KB |
|---|---|---|---|---|---|
| loop sum 1..50,000 | 351.3 ms | 10.3 ms | 34× | 34 | 14 |
| fibonacci(18) recursive | 217.8 ms | 0.7 ms | 322× | 119 | 24 |
| string build ×2,000 | 10.8 ms | 0.6 ms | 17× | 14 | 14 |

## द्रुत (experimental compiled subset — द्रुतम्.py via gcc -O2)

| Workload | compiled run | vs interpreter |
|---|---|---|
| loop sum 1..50,000 | 0.17 ms | 2110× faster |
| fibonacci(18) recursive | 0.17 ms | 1292× faster |

Compiled-run times are dominated by ~0.15 ms process startup — the
computation itself is smaller still. Subset only (ints, loops, functions);
this is the §7b यन्त्रसङ्कलकः mode proven early, not a release feature.

## वेगः — native Rust engine (Phase 3c, in progress)

Measured on Apple Silicon MacBook, `cargo build --release`, slices 1–3
(lexer + parser + naive tree-walking evaluator, no optimization passes yet):

| Workload | Python engine | वेगः (Rust) | gain |
|---|---|---|---|
| loop sum 1..50,000 | ~106 ms | **~20 ms** | ~5× |

Same program, same answer (१२५००२५०००). This is the first measured evidence
for the blueprint's speed promise — and it is the *slowest* the Rust engine will
ever be, since no optimization work has been done. Subset so far: integers,
variables, arithmetic, comparisons, logic, यदि, यावत्, वद. Decimals, functions,
collections and the stdlib arrive in later slices; the 51-case conformance suite
(`परीक्षा.py`) is the completion criterion.

**Honest reading:** the current engine is a tree-walking interpreter written in
Python, so it pays Python's cost *plus* interpretation overhead — the ratio
column is the price of Phase 2 convenience. The Phase 3 Rust engine exists
precisely to close this gap; per §7c, its release must beat these numbers and
publish the comparison. Exactness note: संस्कृता's decimals are exact
(०.१+०.२=०.३) while Python's binary floats are approximate — correctness is
part of what these milliseconds buy.
