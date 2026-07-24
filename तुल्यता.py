#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""तुल्यता — differential test: the Python reference vs the वेगः Rust engine.

Every program below must produce IDENTICAL output from both engines. This is
how a second implementation is kept honest; when वेगः covers the whole language,
this harness plus परीक्षा.py is the definition of "correct".

Run:  python3 तुल्यता.py          (builds the Rust engine if needed)
      python3 तुल्यता.py --debug  (use debug build — catches overflow panics)
"""

import io
import os
import subprocess
import sys
from contextlib import redirect_stdout

import sanskrita

HERE = os.path.dirname(os.path.abspath(__file__))
ENGINE_DIR = os.path.join(HERE, "rust-engine")

# Programs inside the slice 1–3 subset (integers, vars, arithmetic, logic,
# यदि, यावत्, विरम/अनुवर्त, वद, वाक्यम्, दैर्घ्यम्, प्रकारः, सङ्ख्या).
PROGRAMS = [
    ('वद("नमस्ते जगत्")।', "hello"),
    ('वद(२ + ३ * ४)।', "precedence"),
    ('वद((२ + ३) * ४)।', "parens"),
    ('वद(१० % ३, (०-७) % ३, ७ % (०-३))।', "modulo incl. negatives"),
    ('वद(१० - ४, ६ * ७, १० / ५)।', "arithmetic"),
    ('मानय क = ५। क = क + १। वद(क)।', "assignment"),
    ('ध्रुव प = ३। वद(प)।', "constant"),
    ('वद(५ > ३, ५ < ३, ५ >= ५, "अ" < "ब")।', "comparisons"),
    ('वद(सत्यम् च असत्यम्, सत्यम् वा असत्यम्, न सत्यम्)।', "logic"),
    ('यदि (५ > ३) { वद("अ")। } अन्यथा { वद("ब")। }', "if/else"),
    ('यदि (१ > ३) { वद("अ")। } अथ यदि (२ > १) { वद("ब")। } अन्यथा { वद("स")। }',
     "else-if chain"),
    ('मानय स = ०। मानय इ = १। यावत् (इ <= १००) { स = स + इ। इ = इ + १। } वद(स)।',
     "loop sum"),
    ('मानय इ = ०। यावत् (सत्यम्) { इ = इ + १। यदि (इ >= ५) { विरम। } } वद(इ)।',
     "break"),
    ('मानय स = ०। मानय इ = ०। यावत् (इ < १०) { इ = इ + १। '
     'यदि (इ % २ == ०) { अनुवर्त। } स = स + इ। } वद(स)।', "continue"),
    ('वद("अ" + "ब")।', "string concat"),
    ('वद(दैर्घ्यम्("नमस्ते"))।', "length"),
    ('वद(प्रकारः(५), प्रकारः("अ"), प्रकारः(सत्यम्), प्रकारः(शून्यम्))।', "types"),
    ('वद(वाक्यम्(५) + "अ")।', "to-text"),
    ('वद(सङ्ख्या("४२") + १)।', "to-number"),
    ('वद(०-५ + ३, ०-(५))।', "negatives"),
    ('मानय क़मल = ७। वद(क़मल)।', "NFC identity (क़ two ways)"),
    ('# only a comment\nवद("पश्चात्")।', "comments"),
    ('वद(१ == १, "अ" == "अ", सत्यम् == सत्यम्)।', "equality"),
    ('मानय वर्ष२ = ९। वद(वर्ष२)।', "identifier with digit"),
    # --- slice 5: exact numbers (previously tracked divergences) ---
    ('वद(०.१ + ०.२)।', "0.1 + 0.2 is exactly 0.3"),
    ('वद(०.१ + ०.२ == ०.३)।', "exactness is observable"),
    ('वद(४५०.५० + ३२०.२५ + ५९९.००)।', "money math"),
    ('वद(१ / ४, १० / ५, २ + ०.५)।', "division & promotion"),
    ('वद(५.० + ५.०)।', "decimal scale is preserved"),
    ('वद(प्रकारः(०.५), प्रकारः(५), प्रकारः(१० / ५))।', "numeric types"),
    ('मानय क = ९२२३३७२०३६८५४७७५८०७। वद(क + १)।', "beyond i64 (bignum)"),
    ('विधि फ(म) { यदि (म <= १) { फलम् १। } फलम् म * फ(म - १)। } वद(फ(२५))।',
     "25! exactly (bignum)"),
    ('वद(सङ्ख्या("४.५") + ०.५)।', "to-number with decimals"),
    ('वद(०.३० == ०.३, ०.१ < ०.२)।', "decimal comparison"),
    ('वद(०-२.५, ०.००१ * ०.००१)।', "negatives & small decimals"),
]

# Programs that must FAIL in both engines (error text differs, failure must not)
MUST_FAIL = [
    ('वद(क)।', "unknown name"),
    ('ध्रुव क = १। क = २।', "const reassign"),
    ('वद(१ / ०)।', "divide by zero"),
    ('क = ५।', "undeclared assignment"),
    ('यदि (५) { वद("अ")। }', "non-boolean condition"),
    ('मानय नामx = १।', "mixed script"),
    ('वद("अ" + ५)।', "text + number"),
]

# KNOWN, TRACKED divergences — documented rather than hidden.
# Each must be resolved (or ratified as a language decision) before वेगः is
# declared complete. This list is the honest ledger of the second implementation.
#
# All three original entries (integer range, decimal division, division type)
# were RESOLVED by slice 5 and promoted into PROGRAMS above, where they are now
# checked for byte-identical output on every run.
KNOWN_DIVERGENCES: list = []


def build_engine(debug=False):
    cmd = ["cargo", "build"] + ([] if debug else ["--release"])
    r = subprocess.run(cmd, cwd=ENGINE_DIR, capture_output=True)
    if r.returncode != 0:
        print(r.stderr.decode(), file=sys.stderr)
        return None
    sub = "debug" if debug else "release"
    return os.path.join(ENGINE_DIR, "target", sub, "sanskrita-veg")


def run_python(src):
    buf = io.StringIO()
    try:
        with redirect_stdout(buf):
            sanskrita.run_source(src, sanskrita.Interpreter())
        return True, buf.getvalue()
    except Exception as err:
        return False, str(err)


def run_rust(binary, src):
    path = "/tmp/_tulyata.सं"
    with open(path, "w", encoding="utf-8") as f:
        f.write(src)
    r = subprocess.run([binary, path], capture_output=True)
    return r.returncode == 0, r.stdout.decode()


# Whole example programs that both engines must run identically. Only those
# inside वेगः's current feature set are listed; the rest join as slices land.
EXAMPLE_FILES = [
    "नमस्ते.सं",
    "गणना.सं",
    "गुणनसारणी.सं",
    "श्रेणी.सं",
    "क्रमगुणितम्.सं",
    "अभाज्यता.सं",
    "व्याजगणना.सं",
    "नियन्त्रणम्.सं",
    "विधयः.सं",
    "सूचीकोशौ.सं",
    "वर्गाः.सं",
    "दोषनिवारणम्.सं",
    "द्रुतोदाहरणम्.सं",
    "अङ्कतालिका.सं",
    "व्ययगणकः.सं",
    "प्रतिमानानि.सं",
]


def run_python_file(path):
    src = open(path, encoding="utf-8").read()
    return run_python(src)


def main():
    debug = "--debug" in sys.argv
    print("तुल्यता — differential test (Python reference ⟷ वेगः Rust engine)")
    binary = build_engine(debug)
    if binary is None or not os.path.exists(binary):
        print("वेगः न निर्मितम् / could not build the Rust engine "
              "(is cargo installed?)", file=sys.stderr)
        return 2

    fails = 0
    print("\n— programs that must agree —")
    for src, name in PROGRAMS:
        p_ok, p_out = run_python(src)
        r_ok, r_out = run_rust(binary, src)
        if not p_ok:
            print(f"  ! {name}: python reference errored — {p_out.splitlines()[0]}")
            fails += 1
        elif not r_ok:
            print(f"  ✗ {name}: वेगः errored, python succeeded")
            fails += 1
        elif p_out != r_out:
            print(f"  ✗ {name}: DIVERGENCE\n      python: {p_out!r}\n      veg   : {r_out!r}")
            fails += 1
        else:
            print(f"  ✓ {name}")

    print("\n— programs that must fail in both —")
    for src, name in MUST_FAIL:
        p_ok, _ = run_python(src)
        r_ok, _ = run_rust(binary, src)
        if p_ok or r_ok:
            who = "python" if p_ok else ""
            who += " veg" if r_ok else ""
            print(f"  ✗ {name}: did NOT fail in:{who}")
            fails += 1
        else:
            print(f"  ✓ {name}")

    print("\n— whole example programs (byte-identical output required) —")
    ex_dir = os.path.join(HERE, "examples")
    for fname in EXAMPLE_FILES:
        path = os.path.join(ex_dir, fname)
        if not os.path.exists(path):
            print(f"  – {fname}: not found, skipped")
            continue
        src = open(path, encoding="utf-8").read()
        p_ok, p_out = run_python(src)
        r_ok, r_out = run_rust(binary, src)
        if not p_ok:
            print(f"  ! {fname}: python reference errored")
            fails += 1
        elif not r_ok:
            print(f"  ✗ {fname}: वेगः errored, python succeeded")
            fails += 1
        elif p_out != r_out:
            print(f"  ✗ {fname}: DIVERGENCE")
            for a, b in zip(p_out.splitlines(), r_out.splitlines()):
                if a != b:
                    print(f"      python: {a!r}\n      veg   : {b!r}")
            fails += 1
        else:
            print(f"  ✓ {fname}")

    print("\n— known, tracked divergences (not failures) —")
    for name, src, note in KNOWN_DIVERGENCES:
        p_ok, p_out = run_python(src)
        r_ok, r_out = run_rust(binary, src)
        agree = (p_ok == r_ok) and (p_out == r_out)
        mark = "resolved ✓" if agree else "open"
        print(f"  · {name}: {mark}")
        if not agree:
            print(f"      {note}")

    print()
    if fails:
        print(f"तुल्यता भग्ना ✗ — {fails} divergence(s)")
        return 1
    print("तुल्यता सिद्धा ✓ — both engines agree on every tested case")
    print("(known divergences are listed above and tracked, not hidden)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
