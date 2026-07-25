#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""यादृच्छिकपरीक्षा — property-based differential testing.

`तुल्यता.py` checks programs *we* thought of. This checks programs nobody
thought of: it generates random but valid संस्कृता, runs it on both engines, and
demands identical output. Hand-written test suites test the author's
imagination; a generator tests the language.

Two properties are asserted, and only two — both are things that must hold for
every program in the language, not just for the ones we happened to write:

  1. **AGREEMENT.**  The reference engine and वेगः produce byte-identical
     output, or both fail. Where they differ, the reference is right by
     definition (docs/STABILITY.md) and वेगः has a bug.

  2. **NO CRASH.**  Neither engine may crash — segfault, panic, traceback, or
     hang. A clean bilingual error is a fine outcome; an unhandled Rust panic
     or a Python traceback is not. This is what finds the sharp edges that
     hand-written tests never reach.

Run:  python3 यादृच्छिकपरीक्षा.py                 (200 programs, default seed)
      python3 यादृच्छिकपरीक्षा.py --count 5000
      python3 यादृच्छिकपरीक्षा.py --seed 42       (reproduce a failure exactly)
      python3 यादृच्छिकपरीक्षा.py --only-reference (skip वेगः; no Rust needed)

Every failure prints the **shrunken** program — the generator reduces a failing
case to the smallest one that still fails, because a 4-line repro is worth
fifty.
"""

import argparse
import io
import os
import random
import subprocess
import sys
from contextlib import redirect_stdout

import sanskrita

HERE = os.path.dirname(os.path.abspath(__file__))
VEG = os.path.join(HERE, "rust-engine", "target", "release", "sanskrita-veg")

DEV = "०१२३४५६७८९"


def dev(n):
    """A number written the way संस्कृता writes it."""
    s = str(abs(n))
    return "".join(DEV[int(c)] for c in s)


class Gen:
    """A generator of *valid* संस्कृता.

    Deliberately biased towards the places where two implementations of the
    same language usually drift apart: negative operands to `%`, integers that
    outgrow 64 bits, decimal scale, 1-based indexing at the boundaries, deep
    call chains, and mixed Devanagari/ASCII digits.
    """

    NAMES = ["क", "ख", "ग", "घ", "च", "ज", "प", "फ", "ब", "म", "य", "र", "ल", "व"]

    def __init__(self, rng, depth=0):
        self.rng = rng
        self.depth = depth
        self.scope = []

    # ---- expressions

    def number(self):
        r = self.rng.random()
        if r < 0.15:                      # beyond i64 — bignum territory
            return dev(self.rng.randint(10**19, 10**24))
        if r < 0.35:                      # decimals, including trailing zeros
            whole = self.rng.randint(0, 999)
            frac = self.rng.choice(["1", "25", "10", "005", "3333"])
            return f"{dev(whole)}.{''.join(DEV[int(c)] for c in frac)}"
        if r < 0.42:                      # ASCII digits are equally legal
            return str(self.rng.randint(0, 999))
        return dev(self.rng.randint(0, 200))

    def atom(self):
        r = self.rng.random()
        if self.scope and r < 0.35:
            return self.rng.choice(self.scope)
        return self.number()

    def arith(self, depth=0):
        if depth >= 2 or self.rng.random() < 0.35:
            return self.atom()
        op = self.rng.choice(["+", "-", "*", "%", "/"])
        a, b = self.arith(depth + 1), self.arith(depth + 1)
        if op in ("%", "/"):
            # keep it defined: division by zero is an error in both engines,
            # and we are hunting for *disagreement*, not for known errors
            b = f"({b} + १)"
        if op == "-" and self.rng.random() < 0.4:
            # negative operands are where floored vs truncated % diverges
            return f"(०-{a}) {self.rng.choice(['%', '+', '*'])} {b}"
        return f"({a} {op} {b})"

    def condition(self):
        op = self.rng.choice(["<", ">", "<=", ">=", "==", "!="])
        return f"({self.arith()} {op} {self.arith()})"

    # ---- statements

    def declare(self, out, indent):
        name = self.rng.choice([n for n in self.NAMES if n not in self.scope]
                               or self.NAMES)
        out.append(f"{indent}मानय {name} = {self.arith()}।")
        self.scope.append(name)
        return name

    def stmt(self, out, indent, depth):
        r = self.rng.random()
        if r < 0.30 or not self.scope:
            self.declare(out, indent)
        elif r < 0.45:
            out.append(f"{indent}{self.rng.choice(self.scope)} = {self.arith()}।")
        elif r < 0.60:
            out.append(f"{indent}वद({self.arith()})।")
        elif r < 0.72 and depth < 2:
            out.append(f"{indent}यदि {self.condition()} {{")
            self.stmt(out, indent + "    ", depth + 1)
            if self.rng.random() < 0.5:
                out.append(f"{indent}}} अन्यथा {{")
                self.stmt(out, indent + "    ", depth + 1)
            out.append(f"{indent}}}")
        elif r < 0.82 and depth < 2:
            # a bounded loop — an unbounded one would hang the harness
            i = self.declare(out, indent)
            out.append(f"{indent}{i} = ०।")
            out.append(f"{indent}मानय _स = ०।")
            self.scope.append("_स")
            out.append(f"{indent}यावत् ({i} < {dev(self.rng.randint(1, 12))}) {{")
            out.append(f"{indent}    _स = _स + {self.arith()}।")
            out.append(f"{indent}    {i} = {i} + १।")
            out.append(f"{indent}}}")
            out.append(f"{indent}वद(_स)।")
        elif r < 0.92:
            items = ", ".join(self.arith() for _ in range(self.rng.randint(1, 4)))
            name = self.rng.choice([n for n in self.NAMES if n not in self.scope]
                                   or self.NAMES)
            out.append(f"{indent}मानय {name} = [{items}]।")
            self.scope.append(name)
            out.append(f"{indent}वद(दैर्घ्यम्({name}), {name}[१])।")   # 1-based
        else:
            out.append(f"{indent}वद(प्रकारः({self.arith()}))।")

    def program(self, n_stmts):
        out = []
        for _ in range(n_stmts):
            self.stmt(out, "", 0)
        if self.scope:
            shown = self.rng.sample(self.scope, min(3, len(self.scope)))
            out.append("वद(" + ", ".join(shown) + ")।")
        return "\n".join(out) + "\n"


# ---------------------------------------------------------------- execution

def run_reference(src):
    buf = io.StringIO()
    try:
        with redirect_stdout(buf):
            sanskrita.run_source(src, sanskrita.Interpreter())
        return True, buf.getvalue(), None
    except sanskrita.SanskritaError as err:
        return False, buf.getvalue(), str(err)
    except RecursionError:
        return False, buf.getvalue(), "RecursionError"
    except Exception as err:                       # a bug in the engine itself
        return False, buf.getvalue(), f"CRASH {type(err).__name__}: {err}"


def run_veg(src):
    path = "/tmp/_yadrcchika.सं"
    with open(path, "w", encoding="utf-8") as f:
        f.write(src)
    try:
        r = subprocess.run([VEG, path], capture_output=True, timeout=20)
    except subprocess.TimeoutExpired:
        return False, "", "TIMEOUT"
    if r.returncode < 0:                           # killed by a signal
        return False, "", f"CRASH signal {-r.returncode}"
    err = r.stderr.decode(errors="replace")
    if "panicked at" in err:
        return False, r.stdout.decode(errors="replace"), f"PANIC {err.strip()[:200]}"
    return r.returncode == 0, r.stdout.decode(errors="replace"), err.strip() or None


def check(src, use_veg):
    """Return None if the program is fine, or a description of the problem."""
    p_ok, p_out, p_err = run_reference(src)
    if p_err and p_err.startswith("CRASH"):
        return f"reference engine crashed: {p_err}"
    if not use_veg:
        return None
    v_ok, v_out, v_err = run_veg(src)
    if v_err and (v_err.startswith("PANIC") or v_err.startswith("CRASH")
                  or v_err == "TIMEOUT"):
        return f"वेगः {v_err}"
    if p_ok != v_ok:
        return (f"one engine succeeded and the other failed\n"
                f"  reference ok={p_ok} err={p_err}\n"
                f"  वेगः      ok={v_ok} err={v_err}")
    if p_ok and p_out != v_out:
        return (f"output differs\n  reference: {p_out!r}\n  वेगः      : {v_out!r}")
    return None


def shrink(src, use_veg, problem):
    """Drop lines while the failure survives. A 4-line repro beats a 40-line one."""
    lines = src.splitlines()
    changed = True
    while changed and len(lines) > 1:
        changed = False
        for i in range(len(lines)):
            if lines[i].strip() in ("}", "") or lines[i].rstrip().endswith("{"):
                continue                            # never break block structure
            trial = "\n".join(lines[:i] + lines[i + 1:]) + "\n"
            if check(trial, use_veg) is not None:
                lines = lines[:i] + lines[i + 1:]
                changed = True
                break
    return "\n".join(lines) + "\n"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--count", type=int, default=200)
    ap.add_argument("--seed", type=int, default=None)
    ap.add_argument("--only-reference", action="store_true")
    ap.add_argument("--verbose", action="store_true")
    args = ap.parse_args()

    use_veg = not args.only_reference and os.path.exists(VEG)
    if not use_veg and not args.only_reference:
        print("वेगः not built — checking the reference engine only.\n"
              "  cd rust-engine && cargo build --release\n", file=sys.stderr)

    base_seed = args.seed if args.seed is not None else random.randrange(1 << 30)
    print(f"यादृच्छिकपरीक्षा — {args.count} programs, base seed {base_seed}"
          f"{'' if use_veg else ' (reference only)'}")

    failures = 0
    for n in range(args.count):
        seed = base_seed + n
        rng = random.Random(seed)
        src = Gen(rng).program(rng.randint(2, 8))
        problem = check(src, use_veg)
        if problem:
            failures += 1
            small = shrink(src, use_veg, problem)
            print(f"\n  ✗ seed {seed}: {problem}")
            print("  ---- smallest failing program ----")
            for line in small.splitlines():
                print(f"  {line}")
            print("  ----------------------------------")
            print(f"  reproduce: python3 यादृच्छिकपरीक्षा.py --seed {seed} --count 1")
            if failures >= 5:
                print("\n  (stopping after 5 failures)")
                break
        elif args.verbose:
            print(f"  ✓ seed {seed}")

    if failures:
        print(f"\n{failures} failure(s) ✗")
        return 1
    print(f"\nसर्वं तुल्यम् ✓  ({args.count} random programs agreed)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
