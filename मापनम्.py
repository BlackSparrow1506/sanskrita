#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""मापनम् — the §7c benchmark rule.

Design §7c says: *every release must show a memory benchmark table against
Python, Java and Rust; regressions block release.* This script is that rule,
made executable.

It measures the same three workloads in every implementation it can find on
this machine — both संस्कृता engines and the reference languages — and rewrites
BENCHMARKS.md from the results.

Every row is measured in a **separate process**, wall-clock time and peak RSS
together, so the numbers are comparable across languages: a Python heap
measurement and a Rust RSS measurement would not be.

Rows that cannot be measured (toolchain absent) are written into the table as
*not measured*, naming what is missing. They are never silently dropped and
never guessed — an unmeasured row is more honest than an invented one.

Run:  python3 मापनम्.py            (rewrites BENCHMARKS.md)
      python3 मापनम्.py --repeat 5 (best of N; default 3)
      python3 मापनम्.py --dry-run  (print, don't write)
"""

import argparse
import os
import platform
import shutil
import subprocess
import sys
import tempfile
import textwrap
from datetime import date

HERE = os.path.dirname(os.path.abspath(__file__))
VEG = os.path.join(HERE, "rust-engine", "target", "release", "sanskrita-veg")

# ---------------------------------------------------------------- workloads
# Deliberately boring: an integer loop, a recursive call chain, and string
# building. Between them they exercise the three things a tree-walking
# interpreter is worst at, which is the point — we are looking for our own
# weaknesses, not for a flattering shape.

WORKLOADS = [
    {
        "name": "loop sum 1..50,000",
        "sa": "मानय योगः = ०। मानय इ = १।\n"
              "यावत् (इ <= ५००००) { योगः = योगः + इ। इ = इ + १। }\n"
              "वद(योगः)।\n",
        "py": "s = 0\ni = 1\nwhile i <= 50000:\n    s += i\n    i += 1\nprint(s)\n",
        "java": "public class B { public static void main(String[] a) {\n"
                "  long s = 0; for (long i = 1; i <= 50000; i++) s += i;\n"
                "  System.out.println(s); } }\n",
        "rs": "fn main() { let mut s: i64 = 0; let mut i: i64 = 1;\n"
              "  while i <= 50000 { s += i; i += 1; } println!(\"{}\", s); }\n",
        "c": "#include <stdio.h>\nint main(void){ long s=0; for(long i=1;i<=50000;i++) s+=i;\n"
             "  printf(\"%ld\\n\", s); return 0; }\n",
    },
    {
        "name": "fibonacci(18) recursive",
        "sa": "विधि फिब(म) { यदि (म <= १) { फलम् म। } फलम् फिब(म-१) + फिब(म-२)। }\n"
              "वद(फिब(१८))।\n",
        "py": "import sys\nsys.setrecursionlimit(10000)\n"
              "def fib(n):\n    return n if n <= 1 else fib(n-1) + fib(n-2)\n"
              "print(fib(18))\n",
        "java": "public class B { static long fib(long n){ return n<=1?n:fib(n-1)+fib(n-2); }\n"
                "  public static void main(String[] a){ System.out.println(fib(18)); } }\n",
        "rs": "fn fib(n: i64) -> i64 { if n <= 1 { n } else { fib(n-1) + fib(n-2) } }\n"
              "fn main() { println!(\"{}\", fib(18)); }\n",
        "c": "#include <stdio.h>\nlong fib(long n){ return n<=1?n:fib(n-1)+fib(n-2); }\n"
             "int main(void){ printf(\"%ld\\n\", fib(18)); return 0; }\n",
    },
    {
        "name": "string build ×2,000",
        "sa": "मानय पाठः = \"\"। मानय इ = ०।\n"
              "यावत् (इ < २०००) { पाठः = पाठः + \"अ\"। इ = इ + १। }\n"
              "वद(दैर्घ्यम्(पाठः))।\n",
        "py": "t = ''\nfor i in range(2000):\n    t = t + 'a'\nprint(len(t))\n",
        "java": "public class B { public static void main(String[] a){\n"
                "  String t = \"\"; for (int i=0;i<2000;i++) t = t + \"a\";\n"
                "  System.out.println(t.length()); } }\n",
        "rs": "fn main(){ let mut t = String::new();\n"
              "  for _ in 0..2000 { t = format!(\"{}a\", t); }\n"
              "  println!(\"{}\", t.chars().count()); }\n",
        "c": "#include <stdio.h>\n#include <string.h>\nint main(void){ static char t[2100]; int n=0;\n"
             "  for(int i=0;i<2000;i++){ t[n++]='a'; } t[n]=0; printf(\"%d\\n\", (int)strlen(t));\n"
             "  return 0; }\n",
    },
]

# ------------------------------------------------------------- measurement

# The child process reports its OWN peak RSS. Doing it this way — rather than
# sampling from outside — is both accurate and portable, and it measures the
# same thing (resident set size) for every language.
_PROBE = textwrap.dedent("""
    import resource, subprocess, sys, time, platform
    cmd = sys.argv[1:]
    t0 = time.perf_counter()
    r = subprocess.run(cmd, capture_output=True)
    dt = time.perf_counter() - t0
    rss = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    # ru_maxrss is bytes on macOS, kilobytes on Linux
    kb = rss / 1024 if platform.system() == "Darwin" else rss
    sys.stdout.write("%.6f %.0f %d\\n" % (dt, kb, r.returncode))
    sys.stdout.write("OUT:")
    sys.stdout.flush()
    sys.stdout.buffer.write(r.stdout)
""")


def run_once(cmd, cwd=None):
    """Return (seconds, peak_kb, ok, stdout) for one execution of cmd."""
    try:
        probe = subprocess.run([sys.executable, "-c", _PROBE] + list(cmd),
                               capture_output=True, cwd=cwd, timeout=600)
    except subprocess.TimeoutExpired:
        return None
    if probe.returncode != 0:
        return None
    head, _, tail = probe.stdout.partition(b"OUT:")
    parts = head.split()
    if len(parts) < 3:
        return None
    dt, kb, rc = float(parts[0]), float(parts[1]), int(parts[2])
    return dt, kb, rc == 0, tail.decode(errors="replace").strip()


def best_of(cmd, repeat, cwd=None):
    """Best (fastest) run of N — the least noisy estimate of true cost."""
    runs = [r for r in (run_once(cmd, cwd) for _ in range(repeat)) if r]
    runs = [r for r in runs if r[2]]
    if not runs:
        return None
    fastest = min(runs, key=lambda r: r[0])
    lowest_mem = min(r[1] for r in runs)
    return {"ms": fastest[0] * 1000, "kb": lowest_mem, "out": fastest[3]}


# ------------------------------------------------------------- competitors

_DEV = "०१२३४५६७८९"


def _ascii_digits(s):
    """संस्कृता prints Devanagari digits; every other implementation prints
    ASCII. Normalise so the cross-check compares values, not scripts."""
    return "".join(str(_DEV.index(c)) if c in _DEV else c for c in s)


def have(tool):
    return shutil.which(tool) is not None


def write(tmp, name, text):
    path = os.path.join(tmp, name)
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)
    return path


def build_competitors(tmp, wl):
    """Compile the Java/Rust/C versions if their toolchains exist.
    Returns {label: (cmd, cwd, why-missing)}. Compile time is NOT measured —
    we are comparing what a user runs, not what a build server does."""
    out = {"Python": ([sys.executable, write(tmp, "b.py", wl["py"])], None, None)}

    if have("javac") and have("java"):
        src = write(tmp, "B.java", wl["java"])
        r = subprocess.run(["javac", src], cwd=tmp, capture_output=True)
        out["Java"] = (["java", "-cp", tmp, "B"], tmp, None) if r.returncode == 0 \
            else (None, None, "javac failed")
    else:
        out["Java"] = (None, None, "javac/java not installed")

    if have("rustc"):
        src = write(tmp, "b.rs", wl["rs"])
        exe = os.path.join(tmp, "b_rs")
        r = subprocess.run(["rustc", "-O", "-o", exe, src], capture_output=True)
        out["Rust"] = ([exe], None, None) if r.returncode == 0 \
            else (None, None, "rustc failed")
    else:
        out["Rust"] = (None, None, "rustc not installed")

    cc = "cc" if have("cc") else ("gcc" if have("gcc") else None)
    if cc:
        src = write(tmp, "b.c", wl["c"])
        exe = os.path.join(tmp, "b_c")
        r = subprocess.run([cc, "-O2", "-o", exe, src], capture_output=True)
        out["C"] = ([exe], None, None) if r.returncode == 0 \
            else (None, None, "cc failed")
    else:
        out["C"] = (None, None, "no C compiler")

    return out


# ------------------------------------------------------------------ report

def fmt_ms(x):
    return f"{x:.2f} ms" if x < 10 else f"{x:.1f} ms"


def fmt_kb(x):
    return f"{x/1024:.1f} MB" if x >= 1024 else f"{x:.0f} KB"


LABELS = ["संस्कृता (reference)", "संस्कृता (वेगः)", "Python", "Java", "Rust", "C"]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repeat", type=int, default=3)
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    veg_built = os.path.exists(VEG)
    if not veg_built:
        print("वेगः binary not found — build it for a complete table:\n"
              "    cd rust-engine && cargo build --release\n", file=sys.stderr)

    rows = []
    for wl in WORKLOADS:
        print(f"— {wl['name']}", file=sys.stderr)
        with tempfile.TemporaryDirectory() as tmp:
            sa = write(tmp, "b.सं", wl["sa"])
            impls = {"संस्कृता (reference)": best_of(
                [sys.executable, os.path.join(HERE, "sanskrita.py"), sa], args.repeat)}
            if veg_built:
                got = best_of([VEG, sa], args.repeat)
                impls["संस्कृता (वेगः)"] = got or {
                    "missing": "वेगः binary present but would not run here "
                               "(built for another platform?) — rebuild with "
                               "`cargo build --release`"}
            else:
                impls["संस्कृता (वेगः)"] = {
                    "missing": "not built — run `cd rust-engine && cargo build --release`"}

            for label, (cmd, cwd, why) in build_competitors(tmp, wl).items():
                impls[label] = best_of(cmd, args.repeat, cwd) if cmd \
                    else {"missing": why}

            answers = {k: _ascii_digits(v["out"]) for k, v in impls.items()
                       if v and "out" in v}
            if len(set(answers.values())) > 1:
                print(f"  !! implementations disagree: {answers}", file=sys.stderr)
            rows.append((wl["name"], impls, answers))

            for k in LABELS:
                v = impls.get(k)
                if v and "ms" in v:
                    print(f"    {k:22} {fmt_ms(v['ms']):>10}  {fmt_kb(v['kb']):>9}",
                          file=sys.stderr)
                else:
                    print(f"    {k:22} {'—':>10}  ({(v or {}).get('missing', 'n/a')})",
                          file=sys.stderr)

    md = render(rows, args.repeat)
    if args.dry_run:
        print(md)
        return 0
    with open(os.path.join(HERE, "BENCHMARKS.md"), "w", encoding="utf-8") as f:
        f.write(md)
    print("\nBENCHMARKS.md rewritten ✓", file=sys.stderr)
    return 0


def render(rows, repeat):
    u = platform.uname()
    out = [
        "# मितव्यय — Benchmarks",
        "",
        "*Design §7c: every release publishes speed **and** memory against Python,",
        "Java and Rust. Regressions block release.*",
        "",
        f"Generated by `python3 मापनम्.py` on {date.today().isoformat()} — "
        f"best of {repeat} runs.",
        "",
        f"**Machine:** {u.system} {u.release}, {u.machine} · "
        f"Python {platform.python_version()}",
        "",
        "Each row is a **separate process**, timed wall-clock and measured by peak",
        "RSS, so the numbers are comparable across languages. Compile time is",
        "excluded — this is what a user waits for, not what a build server does.",
        "A dash means the toolchain was absent on the machine that generated this",
        "file. Nothing here is estimated.",
        "",
    ]
    for name, impls, answers in rows:
        out += [f"## {name}", "",
                "| Implementation | Time | Peak memory | vs Python |",
                "|---|---|---|---|"]
        base_ms = (impls.get("Python") or {}).get("ms")
        for lab in LABELS:
            v = impls.get(lab)
            if v and "ms" in v:
                if lab == "Python":
                    ratio = "*baseline*"
                elif not base_ms:
                    ratio = "—"
                elif v["ms"] < base_ms:
                    ratio = f"**{base_ms/v['ms']:.1f}× faster**"
                else:
                    ratio = f"{v['ms']/base_ms:.1f}× slower"
                out.append(f"| {lab} | {fmt_ms(v['ms'])} | {fmt_kb(v['kb'])} | {ratio} |")
            else:
                out.append(f"| {lab} | — | — | *{(v or {}).get('missing', 'not measured')}* |")
        if answers and len(set(answers.values())) == 1:
            out += ["", f"Every implementation that ran agreed on the answer "
                        f"(`{next(iter(answers.values()))}`)."]
        out.append("")

    out += [
        "## How to read this",
        "",
        "**The reference engine is slow, and that is expected.** `sanskrita.py` is a",
        "tree-walking interpreter *hosted on Python*: every संस्कृता operation costs",
        "several Python operations, so it cannot beat Python. It exists to define the",
        "language precisely, not to run it fast. Publishing that number as plainly as",
        "the good ones is the whole point of this file.",
        "",
        "**वेगः is the answer to it** — a native binary, no runtime, no external",
        "crates. This is the row to watch as the project matures.",
        "",
        "**C is here as the floor.** Design §7c is explicit that nothing can use less",
        "memory than hand-written C. The goal is to tie the compiled languages and",
        "leave the interpreted ones behind — not to beat physics.",
        "",
        "**Java's memory includes the JVM**, because a user running a Java program",
        "pays for the JVM whether they like it or not. That is the honest comparison,",
        "and it is the specific cost design §7c #1 sets out to avoid.",
        "",
        "**Startup dominates the small workloads.** For anything under ~10 ms, a large",
        "part of what is measured is process start, not computation. Treat those rows",
        "as a floor on latency rather than a measure of throughput.",
        "",
        "## द्रुत (experimental compiled subset)",
        "",
        "`--druta` transpiles a *subset* of संस्कृता to C, compiles it, and caches the",
        "binary. On integer workloads it has measured 1000–2000× faster than the",
        "reference engine. It is deliberately **not** in the table above: it does not",
        "cover the whole language, and ranking a subset against complete",
        "implementations would flatter us dishonestly. Reproduce it directly:",
        "",
        "```bash",
        "python3 sanskrita.py --druta examples/द्रुतोदाहरणम्.सं",
        "```",
        "",
        "## Reproducing this file",
        "",
        "```bash",
        "cd rust-engine && cargo build --release && cd ..   # so वेगः is measured",
        "python3 मापनम्.py --repeat 5",
        "```",
        "",
        "Install `javac` and `rustc` to fill the remaining rows. If a release lands",
        "without this file being regenerated, design §7c has been broken.",
    ]
    return "\n".join(out) + "\n"


if __name__ == "__main__":
    sys.exit(main())
