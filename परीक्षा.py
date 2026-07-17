#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""परीक्षा — sync & regression test for संस्कृता.
Run:  python3 परीक्षा.py
Checks: (1) all examples still run, (2) engine and VS Code converter agree."""

import glob
import io
import os
import subprocess
import sys
from contextlib import redirect_stdout

import sanskrita

HERE = os.path.dirname(os.path.abspath(__file__))
ok = True

# 1 — every example must run without error
print("— examples —")
for path in sorted(glob.glob(os.path.join(HERE, "examples", "*.सं"))
                   + glob.glob(os.path.join(HERE, "examples", "*.sam"))):
    name = os.path.basename(path)
    src = open(path, encoding="utf-8").read()
    old_stdin, sys.stdin = sys.stdin, io.StringIO("गौरी\n२५\n")
    try:
        interp = sanskrita.Interpreter()
        interp.source_dir = os.path.dirname(path)
        with redirect_stdout(io.StringIO()):
            sanskrita.run_source(src, interp)
        print(f"  ✓ {name}")
    except Exception as err:
        print(f"  ✗ {name}: {err}")
        ok = False
    finally:
        sys.stdin = old_stdin

# 2 — Python converter and VS Code JS converter must agree
print("— converter sync (engine vs VS Code extension) —")
SAMPLE = ('# comment | stays\nmanay k = 105|\nyavat (k >= 5) { vad("hi 5", k)| '
          'k = k - 50| }\nyadi (satyam cha na asatyam) { vada("ok")| }\n'
          'dhruv pai = 3.14| vad(vaakyam(pai))|\n'
          'vidhi preshaya(karma m, sampradana p) { phalam m + p| }\n'
          'vada(preshaya(karma: "a", sampradana: "b"))|\n'
          'manay s = [1, 2]| yojaya(s, 3)| pratyekam f iti s { vada(f)| }\n'
          'vargah X { vidhi aarambha() { ayam.n = 1| } }\n'
          'manay o = srja X()| prayata { vada(o.n)| } doshe (t) { vada(t)| }\n'
          'aanaya "python:math" iti ganitam| vada(kramaya(s), kunjikah({"a": 1}))|\n')
py_out = sanskrita.devanagarify(SAMPLE)
try:
    js_out = subprocess.run(
        ["node", "-e",
         "const{devanagarify}=require(process.argv[1]);"
         "process.stdout.write(devanagarify(require('fs').readFileSync(0,'utf8')))",
         os.path.join(HERE, "vscode-sanskrita", "converter.js")],
        input=SAMPLE.encode(), capture_output=True, timeout=30).stdout.decode()
    if py_out == js_out:
        print("  ✓ converters identical")
    else:
        print("  ✗ CONVERTERS DIFFER — update vscode-sanskrita/converter.js!")
        ok = False
except FileNotFoundError:
    print("  – node not installed; skipped JS check")

print("\nसर्वं शुद्धम् ✓ (all good)" if ok else "\nदोषाः सन्ति ✗ (failures above)")
sys.exit(0 if ok else 1)
