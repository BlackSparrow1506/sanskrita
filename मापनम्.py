#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""मापनम् — the §7c benchmark rule: every release publishes honest numbers.
Run:  python3 मापनम्.py   (rewrites BENCHMARKS.md)"""

import io
import time
import tracemalloc
from contextlib import redirect_stdout

import sanskrita

WORKLOADS = [
    ("loop sum 1..50,000",
     "मानय योगः = ०। मानय इ = १।\nयावत् (इ <= ५००००) { योगः = योगः + इ। इ = इ + १। }\n",
     "s = 0\nfor i in range(1, 50001):\n    s = s + i\n"),
    ("fibonacci(18) recursive",
     "विधि फिब(म) { यदि (म <= १) { फलम् म। } फलम् फिब(म-१) + फिब(म-२)। }\nमानय फ = फिब(१८)।\n",
     "def fib(n):\n    return n if n <= 1 else fib(n-1) + fib(n-2)\nf = fib(18)\n"),
    ("string build ×2,000",
     "मानय पाठः = \"\"। मानय इ = ०।\nयावत् (इ < २०००) { पाठः = पाठः + \"अ\"। इ = इ + १। }\n",
     "t = ''\nfor i in range(2000):\n    t = t + 'a'\n"),
]


def measure(fn):
    tracemalloc.start()
    t0 = time.perf_counter()
    fn()
    dt = time.perf_counter() - t0
    _, peak = tracemalloc.get_traced_memory()
    tracemalloc.stop()
    return dt, peak / 1024


def run_sanskrita(src):
    interp = sanskrita.Interpreter()
    with redirect_stdout(io.StringIO()):
        sanskrita.run_source(src, interp)


rows = []
for name, sk_src, py_src in WORKLOADS:
    sk_t, sk_m = measure(lambda: run_sanskrita(sk_src))
    py_t, py_m = measure(lambda: exec(py_src, {}))
    rows.append((name, sk_t, py_t, sk_t / py_t if py_t else 0, sk_m, py_m))

# --- experimental द्रुत (compiled) backend, subset workloads only ---
druta_rows = []
try:
    import importlib.util
    import subprocess
    import tempfile
    spec = importlib.util.spec_from_file_location("druta", "द्रुतम्.py")
    druta = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(druta)
    for name, sk_src, _ in WORKLOADS[:2]:          # string workload: not in subset
        c_code = druta.transpile(sk_src)
        with tempfile.TemporaryDirectory() as td:
            cf = f"{td}/o.c"
            bf = f"{td}/o"
            open(cf, "w").write(c_code)
            subprocess.run(["gcc", "-O2", cf, "-o", bf], check=True,
                           capture_output=True)
            times = []
            for _ in range(5):
                t0 = time.perf_counter()
                subprocess.run([bf], check=True, capture_output=True)
                times.append(time.perf_counter() - t0)
            druta_rows.append((name, min(times)))
except Exception:
    druta_rows = []

lines = [
    "# मितव्यय Benchmarks (§7c rule: honest numbers, every release)",
    "",
    f"Engine: sanskrita.py v0.3 (Python tree-walking interpreter) — "
    f"measured {time.strftime('%Y-%m-%d')}",
    "",
    "| Workload | संस्कृता time | Python time | ratio | संस्कृता peak KB | Python peak KB |",
    "|---|---|---|---|---|---|",
]
for name, st, pt, r, sm, pm in rows:
    lines.append(f"| {name} | {st*1000:.1f} ms | {pt*1000:.1f} ms | "
                 f"{r:.0f}× | {sm:.0f} | {pm:.0f} |")
if druta_rows:
    lines += [
        "",
        "## द्रुत (experimental compiled subset — द्रुतम्.py via gcc -O2)",
        "",
        "| Workload | compiled run | vs interpreter |",
        "|---|---|---|",
    ]
    for (name, rt), (n2, st, *_ ) in zip(druta_rows, rows):
        lines.append(f"| {name} | {rt*1000:.2f} ms | {st/rt:.0f}× faster |")
    lines += [
        "",
        "Compiled-run times are dominated by ~0.15 ms process startup — the",
        "computation itself is smaller still. Subset only (ints, loops, functions);",
        "this is the §7b यन्त्रसङ्कलकः mode proven early, not a release feature.",
    ]

lines += [
    "",
    "**Honest reading:** the current engine is a tree-walking interpreter written in",
    "Python, so it pays Python's cost *plus* interpretation overhead — the ratio",
    "column is the price of Phase 2 convenience. The Phase 3 Rust engine exists",
    "precisely to close this gap; per §7c, its release must beat these numbers and",
    "publish the comparison. Exactness note: संस्कृता's decimals are exact",
    "(०.१+०.२=०.३) while Python's binary floats are approximate — correctness is",
    "part of what these milliseconds buy.",
]

with open("BENCHMARKS.md", "w", encoding="utf-8") as f:
    f.write("\n".join(lines) + "\n")
print("\n".join(lines))
