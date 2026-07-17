# मितव्यय Benchmarks (§7c rule: honest numbers, every release)

Engine: sanskrita.py v0.3 (Python tree-walking interpreter) — measured 2026-07-18

| Workload | संस्कृता time | Python time | ratio | संस्कृता peak KB | Python peak KB |
|---|---|---|---|---|---|
| loop sum 1..50,000 | 314.3 ms | 8.3 ms | 38× | 36 | 14 |
| fibonacci(18) recursive | 188.5 ms | 0.5 ms | 402× | 118 | 24 |
| string build ×2,000 | 10.9 ms | 0.4 ms | 29× | 14 | 14 |

**Honest reading:** the current engine is a tree-walking interpreter written in
Python, so it pays Python's cost *plus* interpretation overhead — the ratio
column is the price of Phase 2 convenience. The Phase 3 Rust engine exists
precisely to close this gap; per §7c, its release must beat these numbers and
publish the comparison. Exactness note: संस्कृता's decimals are exact
(०.१+०.२=०.३) while Python's binary floats are approximate — correctness is
part of what these milliseconds buy.
