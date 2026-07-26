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
import os
import random
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ENGINE_DIR = os.path.join(HERE, "rust-engine")
VEG = os.path.join(ENGINE_DIR, "target", "release", "sanskrita-veg")


def build_veg():
    """Rebuild वेगः before testing it.

    This harness used to run whatever binary happened to be sitting in
    target/release. `cargo test` only builds the DEBUG profile, so a source
    change followed by `cargo test` left this file testing the previous engine
    and reporting its old divergences — byte for byte, which reads exactly like
    a fix that did not work. Silently testing a stale binary is worse than
    failing loudly, so we build first, like तुल्यता.py already did.
    """
    print("वेगः निर्मीयते… (cargo build --release)", file=sys.stderr)
    r = subprocess.run(["cargo", "build", "--release"],
                       cwd=ENGINE_DIR, capture_output=True)
    if r.returncode != 0:
        print(r.stderr.decode(), file=sys.stderr)
        return False
    return os.path.exists(VEG)

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

    def number(self, small=False):
        r = self.rng.random()
        if r < 0.15 and not small:        # beyond i64 — bignum territory
            return dev(self.rng.randint(10**19, 10**24))
        if r < 0.35:                      # decimals, including trailing zeros
            whole = self.rng.randint(0, 999)
            frac = self.rng.choice(["1", "25", "10", "005", "3333"])
            return f"{dev(whole)}.{''.join(DEV[int(c)] for c in frac)}"
        if r < 0.42:                      # ASCII digits are equally legal
            return str(self.rng.randint(0, 999))
        return dev(self.rng.randint(0, 200))

    def atom(self, exclude=(), small=False):
        usable = [n for n in self.scope if n not in exclude]
        r = self.rng.random()
        if usable and r < 0.35:
            return self.rng.choice(usable)
        return self.number(small)

    def arith(self, depth=0, exclude=(), small=False):
        """An arithmetic expression.

        `exclude` keeps named variables out — used for a loop accumulator, and
        it matters more than it looks. `स = स + (स * स)` squares the
        accumulator every iteration: twelve rounds starting from १०²⁴ yields a
        number with ~98,000 digits. Both engines compute it *correctly*; वेगः's
        schoolbook bignum multiply then takes hours, and a CI job that never
        finishes is worse than one that fails. Generated programs must be
        provably quick, not merely provably valid.
        """
        if depth >= 2 or self.rng.random() < 0.35:
            return self.atom(exclude, small)
        op = self.rng.choice(["+", "-", "*", "%", "/"])
        a = self.arith(depth + 1, exclude, small)
        b = self.arith(depth + 1, exclude, small)
        if op in ("%", "/"):
            # keep it defined: division by zero is an error in both engines,
            # and we are hunting for *disagreement*, not for known errors
            b = f"({b} + १)"
        if op == "-" and self.rng.random() < 0.4:
            # negative operands are where floored vs truncated % diverges
            return f"(०-{a}) {self.rng.choice(['%', '+', '*'])} {b}"
        return f"({a} {op} {b})"

    def small_arith(self, exclude=()):
        """A loop-body expression: no bignum literals, and the accumulator
        cannot feed back into itself."""
        return self.arith(0, exclude, True)

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
            out.append(f"{indent}    _स = _स + {self.small_arith(exclude=('_स', i))}।")
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

# A property tester that can hang is worse than no property tester: it turns a
# red build into a build that never finishes. BOTH engines therefore run as
# subprocesses with a timeout, and the whole run has a wall-clock budget.
#
# Running the reference out-of-process costs a fork per program. That is the
# price of never wedging CI again, and it buys something else too: a Python
# traceback becomes visible as a crash instead of being caught as an exception.

TIMEOUT = 15          # seconds per program, per engine
_TMP = "/tmp/_yadrcchika.सं"


def _write(src):
    with open(_TMP, "w", encoding="utf-8") as f:
        f.write(src)
    return _TMP


def run_reference(src, timeout=TIMEOUT):
    path = _write(src)
    try:
        r = subprocess.run([sys.executable, os.path.join(HERE, "sanskrita.py"), path],
                           capture_output=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        return False, "", "TIMEOUT"
    out = r.stdout.decode(errors="replace")
    err = r.stderr.decode(errors="replace")
    if r.returncode < 0:
        return False, out, f"CRASH signal {-r.returncode}"
    if "Traceback (most recent call last)" in err:
        return False, out, f"CRASH {err.strip().splitlines()[-1][:160]}"
    return r.returncode == 0, out, err.strip() or None


def run_veg(src, timeout=TIMEOUT):
    path = _write(src)
    try:
        r = subprocess.run([VEG, path], capture_output=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        return False, "", "TIMEOUT"
    if r.returncode < 0:                           # killed by a signal
        return False, "", f"CRASH signal {-r.returncode}"
    err = r.stderr.decode(errors="replace")
    if "panicked at" in err:
        return False, r.stdout.decode(errors="replace"), f"PANIC {err.strip()[:200]}"
    return r.returncode == 0, r.stdout.decode(errors="replace"), err.strip() or None


def check(src, use_veg, timeout=TIMEOUT):
    """Return None if the program is fine, or a description of the problem."""
    p_ok, p_out, p_err = run_reference(src, timeout)
    if p_err == "TIMEOUT":
        return (f"the reference engine did not finish in {timeout}s — either the "
                f"generator produced a non-terminating program (fix the "
                f"generator) or the engine hangs on it (fix the engine)")
    if p_err and p_err.startswith("CRASH"):
        return f"reference engine crashed: {p_err}"
    if not use_veg:
        return None
    v_ok, v_out, v_err = run_veg(src, timeout)
    if v_err == "TIMEOUT":
        return (f"वेगः did not finish in {timeout}s while the reference did — "
                f"a hang in the native engine")
    if v_err and (v_err.startswith("PANIC") or v_err.startswith("CRASH")):
        return f"वेगः {v_err}"
    if p_ok != v_ok:
        return (f"one engine succeeded and the other failed\n"
                f"  reference ok={p_ok} err={p_err}\n"
                f"  वेगः      ok={v_ok} err={v_err}")
    if p_ok and p_out != v_out:
        return (f"output differs\n  reference: {p_out!r}\n  वेगः      : {v_out!r}")
    return None


def shrink(src, use_veg, problem, budget=40):
    """Drop lines while the failure survives. A 4-line repro beats a 40-line one.

    `budget` caps how many candidate programs we are willing to run: shrinking a
    TIMEOUT failure by re-running it forty times would take ten minutes, and the
    smaller program is not worth that. Timeouts are reported unshrunk.
    """
    if "did not finish" in problem:
        return src
    lines = src.splitlines()
    spent = 0
    changed = True
    while changed and len(lines) > 1 and spent < budget:
        changed = False
        for i in range(len(lines)):
            if spent >= budget:
                break
            if lines[i].strip() in ("}", "") or lines[i].rstrip().endswith("{"):
                continue                            # never break block structure
            trial = "\n".join(lines[:i] + lines[i + 1:]) + "\n"
            spent += 1
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
    ap.add_argument("--timeout", type=int, default=TIMEOUT,
                    help="seconds allowed per program, per engine")
    ap.add_argument("--budget", type=int, default=600,
                    help="wall-clock seconds for the whole run; stops cleanly "
                         "when spent, so CI can never hang")
    args = ap.parse_args()

    use_veg = not args.only_reference and build_veg()
    if not use_veg and not args.only_reference:
        print("वेगः could not be built — checking the reference engine only.\n"
              "  cd rust-engine && cargo build --release\n", file=sys.stderr)

    base_seed = args.seed if args.seed is not None else random.randrange(1 << 30)
    print(f"यादृच्छिकपरीक्षा — {args.count} programs, base seed {base_seed}"
          f"{'' if use_veg else ' (reference only)'}")

    failures = 0
    started = time.time()
    ran = 0
    for n in range(args.count):
        if time.time() - started > args.budget:
            print(f"\n  (budget of {args.budget}s spent after {ran} programs — "
                  f"stopping cleanly)")
            break
        seed = base_seed + n
        rng = random.Random(seed)
        src = Gen(rng).program(rng.randint(2, 8))
        ran += 1
        problem = check(src, use_veg, args.timeout)
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

    elapsed = time.time() - started
    if failures:
        print(f"\n{failures} failure(s) ✗   ({ran} programs, {elapsed:.0f}s)")
        return 1
    print(f"\nसर्वं तुल्यम् ✓  ({ran} random programs agreed in {elapsed:.0f}s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
