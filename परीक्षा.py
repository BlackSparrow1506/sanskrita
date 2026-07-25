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

# 1b — conformance micro-tests: one feature per case, exact expected output.
# This table is the language's contract — any future engine must pass it.
MICRO = [
    # arithmetic & numbers
    ('वद(२ + ३)।', '५'),
    ('वद(१० - ४)।', '६'),
    ('वद(६ * ७)।', '४२'),
    ('वद(७ % ३)।', '१'),
    ('वद(१० / ४)।', '२.५'),
    ('वद(०.१ + ०.२)।', '०.३'),
    ('वद(०.१ + ०.२ == ०.३)।', 'सत्यम्'),
    ('वद(-५ + ३)।', '-२'),
    ('वद(2 + 3)।', '५'),                          # ASCII digits in, dev digits out
    # variables & types
    ('मानय क = ५। क = क + १। वद(क)।', '६'),
    ('ध्रुव क = ५। वद(क)।', '५'),
    ('मानय क : पूर्णाङ्कः = ५। वद(क)।', '५'),
    ('वद(प्रकारः(५), प्रकारः(०.५), प्रकारः("अ"), प्रकारः(सत्यम्))।',
     'पूर्णाङ्कः दशमांशः वाक्यम् सत्यासत्यम्'),
    # strings
    ('वद("अ" + "ब")।', 'अब'),
    ('वद(दैर्घ्यम्("नमस्ते"))।', '६'),
    ('वद(वाक्यम्(५) + "अ")।', '५अ'),
    ('वद(सङ्ख्या("४२") + १)।', '४३'),
    # booleans & logic
    ('वद(सत्यम् च असत्यम्)।', 'असत्यम्'),
    ('वद(सत्यम् वा असत्यम्)।', 'सत्यम्'),
    ('वद(न सत्यम्)।', 'असत्यम्'),
    ('वद(५ > ३ च २ < ४)।', 'सत्यम्'),
    # control flow
    ('यदि (५ > ३) { वद("अ")। } अन्यथा { वद("ब")। }', 'अ'),
    ('यदि (१ > ३) { वद("अ")। } अथ यदि (२ > १) { वद("ब")। } अन्यथा { वद("स")। }', 'ब'),
    ('मानय क = ०। यावत् (क < ३) { क = क + १। } वद(क)।', '३'),
    ('मानय योगः = ०। प्रत्येकम् इ इति परिधिः(१, ४) { योगः = योगः + इ। } वद(योगः)।', '१०'),
    ('मानय क = ०। यावत् (सत्यम्) { क = क + १। यदि (क == ३) { विरम। } } वद(क)।', '३'),
    # lists & maps (1-based!)
    ('मानय स = [१०, २०, ३०]। वद(स[१])।', '१०'),
    ('मानय स = [१०, २०]। स[२] = ९९। वद(स[२])।', '९९'),
    ('मानय स = [३, १, २]। वद(क्रमय(स))।', '[१, २, ३]'),
    ('मानय स = [१]। योजय(स, २)। वद(दैर्घ्यम्(स))।', '२'),
    ('मानय क = {"अ": १}। क["ब"] = २। वद(क["ब"])।', '२'),
    ('मानय क = {"अ": १, "ब": २}। अपनय(क, "अ")। वद(दैर्घ्यम्(क))।', '१'),
    # functions & kāraka
    ('विधि योग(क, ख) { फलम् क + ख। } वद(योग(२, ३))।', '५'),
    ('विधि फ(म) { यदि (म <= १) { फलम् १। } फलम् म * फ(म - १)। } वद(फ(५))।', '१२०'),
    ('विधि प्रे(कर्म क, करण ख) { फलम् क + ख। } वद(प्रे(करण: "ब", कर्म: "अ"))।', 'अब'),
    ('मानय द्वि = विधि(क) { फलम् क * २। }। वद(द्वि(७))।', '१४'),      # lambda
    # classes & inheritance
    ('वर्गः क { विधि आरम्भ() { अयम्.मूल्यम् = ५। } } मानय व = सृज क()। वद(व.मूल्यम्)।', '५'),
    ('वर्गः पि { विधि नम() { फलम् "पि"। } } वर्गः पु : पि { } '
     'मानय व = सृज पु()। वद(व.नम())।', 'पि'),
    # errors caught by प्रयत
    ('प्रयत { मानय क = १ / ०। } दोषे (त्रु) { वद("गृहीतः")। }', 'गृहीतः'),
    # sandhi (संस्कृतम् library)
    ('आनय "संस्कृतम्" इति सं। वद(सं.संधय("देव", "आलयः"))।', 'देवालयः'),
    ('आनय "संस्कृतम्" इति सं। वद(सं.संधय("रामः", "गच्छति"))।', 'रामो गच्छति'),
    ('आनय "संस्कृतम्" इति सं। वद(सं.संधय("रामः", "अस्ति"))।', 'रामोऽस्ति'),
    ('आनय "संस्कृतम्" इति सं। वद(सं.अक्षरगणना("नमस्ते"))।', '३'),
    # --- Phase 3 completion ---
    # क्षिप: a program can raise its own error, and प्रयत catches it
    ('प्रयत { क्षिप "मम दोषः"। } दोषे (त्रु) { वद(त्रु)। }', 'मम दोषः'),
    # शून्यम्-safety: nullable is opt-in with '?'
    ('मानय क? : वाक्यम् = शून्यम्। वद(क)।', 'शून्यम्'),
    # सूचीकर्म
    ('आनय "सूचीकर्म" इति सू। वद(सू.छानय([१, २, ३, ४], विधि(क) { फलम् क > २। }))।',
     '[३, ४]'),
    ('आनय "सूचीकर्म" इति सू। वद(सू.प्रतिचित्रय([१, २, ३], विधि(क) { फलम् क * क। }))।',
     '[१, ४, ९]'),
    ('आनय "सूचीकर्म" इति सू। '
     'वद(सू.न्यूनीकरण([१, २, ३, ४], विधि(अ, ब) { फलम् अ + ब। }, ०))।', '१०'),
    ('आनय "सूचीकर्म" इति सू। वद(सू.अद्वितीयम्([१, २, １, ३]))।'.replace('１', '१'),
     '[१, २, ३]'),
    ('आनय "सूचीकर्म" इति सू। वद(सू.महत्तमम्([३, ९, १]), सू.लघुत्तमम्([३, ९, १]))।',
     '९ १'),
    # वाक्यकर्म additions
    ('आनय "वाक्यकर्म" इति व। वद(व.परिष्कार("  क  "), व.आरभते("नमस्ते", "नम"))।',
     'क सत्यम्'),
    # जेसन — and the exactness that survives a round trip
    ('आनय "जेसन" इति ज। वद(ज.पाठय({"क": १, "ख": [२, ३]}))।',
     '{"क": 1, "ख": [2, 3]}'),
    ('आनय "जेसन" इति ज। मानय क = ज.विश्लेषय("{\\"म\\": 0.1}")। वद(क["म"] + ०.२)।',
     '०.३'),
    # आदेशचराः exists and is a list (empty when nothing was passed)
    ('वद(प्रकारः(आदेशचराः()))।', 'सूची'),
    # --- blueprint gaps closed in v0.4 ---
    # §2b default parameter values
    ('विधि नम(क, ख = ५) { फलम् क + ख। } वद(नम(१), नम(१, १०))।', '६ ११'),
    # …evaluated fresh every call, so Python's mutable-default bug cannot happen
    ('विधि य(म, स = []) { योजय(स, म)। फलम् स। } य(१)। वद(य(२))।', '[२]'),
    # §7d #2 method chaining on built-in types
    ('वद([३, १, २].क्रमय().विपर्यय())।', '[३, २, １]'.replace('１', '१')),
    ('वद([१, २, ३, ४].छानय(विधि(क) { फलम् क > २। }).योगः())।', '७'),
    ('वद("  नमस्ते  ".परिष्कार())।', 'नमस्ते'),
    ('वद({"अ": १, "ब": २}.कुञ्जिकाः())।', '["अ", "ब"]'),
    # §2b द्रुतदशमांशः — exact by default, fast only when asked
    ('वद(प्रकारः(०.१ + ०.२))।', 'दशमांशः'),
    ('वद(द्रुतदशमांशः(०.१) + द्रुतदशमांशः(०.२))।', '०.३०००००००००००००००४'),
    ('वद(प्रकारः(द्रुतदशमांशः(१)))।', 'द्रुतदशमांशः'),
    # exactness must survive the native-module boundary (audit issue #8)
    ('आनय "सूचीकर्म" इति सू। वद(सू.योगः([०.१, ०.२]))।', '०.३'),
    ('आनय "सूचीकर्म" इति सू। वद(सू.अद्वितीयम्([०.१०, ०.१]))।', '[०.१०]'),
    ('आनय "जेसन" इति ज। वद(ज.पाठय({"क": ०.१०}))।', '{"क": 0.10}'),
    ('आनय "जेसन" इति ज। वद(ज.विश्लेषय("1e2") + १)।', '१०१'),
]

# error cases: (code, substring that must appear in the error)
MICRO_ERR = [
    ('वद(क)।', 'अज्ञातं नाम'),
    ('ध्रुव क = १। क = २।', 'ध्रुवः'),
    ('मानय क : पूर्णाङ्कः = १। क = "अ"।', 'प्रकारदोषः'),
    ('वद("अ" + ५)।', 'मिश्रणीये'),
    ('वद(१ / ०)।', 'शून्येन'),
    ('मानय स = [१]। वद(स[०])।', 'सीमाबहिः'),
    ('मानय नामx = १।', 'मिश्रलिपि'),
    ('विरम।', 'चक्रात्'),
    ('क्षिप "विफलम्"।', 'विफलम्'),
    ('मानय क : वाक्यम् = शून्यम्।', 'शून्यं न स्वीकरोति'),
    ('आनय "जेसन" इति ज। वद(ज.विश्लेषय("{अ}"))।', 'दोषः'),
    # §2b — proved BEFORE the program runs (प्राक्परीक्षा)
    ('मानय क : पूर्णाङ्कः = "पञ्च"।', 'प्रकारदोषः'),
    ('वद("आरम्भः")। मानय क : वाक्यम् = शून्यम्।', 'शून्यं न स्वीकरोति'),
    ('मानय स = [१]। वद(स.क्रम())।', 'न जानाति'),
]

print("— conformance micro-tests —")
mfail = 0
for code, expected in MICRO:
    buf = io.StringIO()
    try:
        with redirect_stdout(buf):
            sanskrita.run_source(code, sanskrita.Interpreter())
        got = buf.getvalue().strip()
        if got != expected:
            print(f"  ✗ {code!r}: expected {expected!r}, got {got!r}")
            mfail += 1
    except Exception as err:
        print(f"  ✗ {code!r}: raised {err}")
        mfail += 1
for code, needle in MICRO_ERR:
    try:
        with redirect_stdout(io.StringIO()):
            sanskrita.run_source(code, sanskrita.Interpreter())
        print(f"  ✗ {code!r}: expected error containing {needle!r}, none raised")
        mfail += 1
    except sanskrita.SanskritaError as err:
        if needle not in str(err):
            print(f"  ✗ {code!r}: error lacks {needle!r}")
            mfail += 1
if mfail:
    ok = False
    print(f"  {mfail} micro-test failure(s)")
else:
    print(f"  ✓ all {len(MICRO)} + {len(MICRO_ERR)} error-cases pass")

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
