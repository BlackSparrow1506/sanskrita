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
    # --- slice 6: collections, classes, प्रयत/दोषे ---
    ('मानय स = [३, १, २]। वद(क्रमय(स), दैर्घ्यम्(स), स[१])।', "lists"),
    ('मानय क = {"अ": १, "ब": २}। वद(कुञ्जिकाः(क), क["ब"])।', "maps keep order"),
    ('वर्गः प { विधि आरम्भ(न) { अयम्.न = न। } विधि वद्() { फलम् अयम्.न। } } '
     'मानय व = सृज प("क")। वद(व.वद्())।', "class + constructor"),
    ('प्रयत { वद(१ / ०)। } दोषे (त्रु) { वद("गृहीतम्:", त्रु)। }', "try/catch"),
    ('प्रत्येकम् इ इति परिधिः(१, ५) { वद(इ)। }', "for-each over a range"),
    # --- Phase-3 completion: क्षिप, शून्यम्-safety, नव-कोष्ठकानि ---
    ('प्रयत { क्षिप "मम दोषः"। } दोषे (त्रु) { वद(त्रु)। }', "क्षिप is catchable"),
    ('मानय क? : वाक्यम् = शून्यम्। वद(क, प्रकारः(क))।', "nullable declaration"),
    ('मानय क : पूर्णाङ्कः = ५। वद(क + १)।', "typed declaration"),
    ('आनय "सूचीकर्म" इति सू। '
     'वद(सू.छानय([१, २, ३, ४], विधि(क) { फलम् क > २। }))।', "सूचीकर्म.छानय"),
    ('आनय "सूचीकर्म" इति सू। '
     'वद(सू.प्रतिचित्रय([१, २, ३], विधि(क) { फलम् क * क। }))।', "सूचीकर्म.प्रतिचित्रय"),
    ('आनय "सूचीकर्म" इति सू। '
     'वद(सू.न्यूनीकरण([१, २, ३, ४], विधि(अ, ब) { फलम् अ + ब। }, ०))।',
     "सूचीकर्म.न्यूनीकरण"),
    ('आनय "सूचीकर्म" इति सू। वद(सू.योगः([१, २, ३]), सू.महत्तमम्([३, ९, १]), '
     'सू.लघुत्तमम्([३, ९, १]))।', "सूचीकर्म aggregates"),
    ('आनय "सूचीकर्म" इति सू। वद(सू.विपर्यय([१, २, ३]), सू.अद्वितीयम्([१, २, १, ३]), '
     'सू.अनुक्रमः(["अ", "ब"], "ब"), सू.अन्तर्भवति([१, २], २))।', "सूचीकर्म queries"),
    ('आनय "वाक्यकर्म" इति व। वद(व.उच्च("abc"), व.निम्न("ABC"), व.परिष्कार("  क  "))।',
     "वाक्यकर्म case & trim"),
    ('आनय "वाक्यकर्म" इति व। वद(व.आरभते("नमस्ते", "नम"), व.अन्तयति("नमस्ते", "स्ते"), '
     'व.अन्तर्भवति("नमस्ते", "मस्"))।', "वाक्यकर्म predicates"),
    ('आनय "जेसन" इति ज। वद(ज.पाठय({"क": १, "ख": [२, ३], "ग": "घ"}))।', "जेसन.पाठय"),
    ('आनय "जेसन" इति ज। मानय क = ज.विश्लेषय("{\\"म\\": 0.1, \\"स\\": 7}")। '
     'वद(क["म"] + ०.२, प्रकारः(क["स"]))।', "जेसन.विश्लेषय keeps decimals exact"),
    ('आनय "जेसन" इति ज। वद(ज.पाठय(ज.विश्लेषय("[1, 2.5, true, null, \\"क\\"]")))।',
     "जेसन round-trip"),
    # --- blueprint gaps closed in v0.4 ---
    ('विधि नम(क, ख = ५) { फलम् क + ख। } वद(नम(१), नम(१, १०))।', "default parameters"),
    ('विधि अ(कर्म नाम, करण भाषा = "संस्कृतम्") { फलम् नाम + भाषा। } '
     'वद(अ(कर्म: "क"), अ(कर्म: "क", करण: "ख"))।', "defaults with kārakas"),
    ('विधि य(म, स = []) { योजय(स, म)। फलम् स। } य(१)। वद(य(२))।',
     "defaults are fresh every call"),
    ('वद([३, १, २].क्रमय().विपर्यय())।', "method chaining"),
    ('वद([१, २, ३, ४].छानय(विधि(क) { फलम् क > २। }).योगः())।', "chained filter+sum"),
    ('वद("  अ,ब,स  ".परिष्कार().विभज(",").दैर्घ्यम्())।', "text method chain"),
    ('वद({"अ": १, "ब": २}.कुञ्जिकाः(), [१, २, ३].दैर्घ्यम्())।', "builtins as methods"),
    ('वद(द्रुतदशमांशः(०.१) + द्रुतदशमांशः(०.२))।', "द्रुतदशमांशः is honestly inexact"),
    ('वद(प्रकारः(०.१ + ०.२), प्रकारः(द्रुतदशमांशः(०.१) + ०.२))।', "exact vs fast"),
    ('वद(द्रुतदशमांशः(२) / ४, द्रुतदशमांशः(२) > १, द्रुतदशमांशः(१) + ०.५)।',
     "द्रुतदशमांशः arithmetic & comparison"),
    ('वद(द्रुतदशमांशः("१.२५"), द्रुतदशमांशः(३))।', "द्रुतदशमांशः conversions"),
    # audit issue #8 — exactness must survive the native-module boundary
    ('आनय "सूचीकर्म" इति सू। वद(सू.योगः([०.१, ०.२]), सू.योगः([१, २, ३]))।',
     "सूचीकर्म.योगः stays exact"),
    ('आनय "सूचीकर्म" इति सू। वद(सू.अद्वितीयम्([०.१०, ०.१]), सू.महत्तमम्([०.१, ०.२]))।',
     "सूचीकर्म keeps decimal scale"),
    ('आनय "जेसन" इति ज। वद(ज.पाठय({"क": ०.१०, "ख": शून्यम्, "ग": सत्यम्}))।',
     "जेसन.पाठय keeps the scale"),
    ('आनय "जेसन" इति ज। वद(ज.विश्लेषय("1e2") + १, ज.विश्लेषय("1.5e-3"))।',
     "जेसन exponents without a float"),
    ('वद([१, २,], {"क": १,})।', "trailing comma in literals"),
    ('विधि नम(क, ख = ५,) { फलम् क + ख। } वद(नम(१,), नम(१, २,))।',
     "trailing comma in params and args"),
    # --- found by यादृच्छिकपरीक्षा.py: decimals must be exact at ANY size ---
    ('वद(१०५९२७०२७९०६७५४१३१७७२७०००४.३३३३ + ०.०००१)।', "big decimal + tiny addend"),
    ('वद(१२३४५६७८९०१२३४५६७८९०.१२३४५ * १२३४५६७८९०१२३४५६७८९०.१२३४५)।',
     "big decimal multiplication is exact"),
    ('वद(१०५९२७०२७९०६७५४१३१७७२७०००४.३३३३ % ११२)।', "big decimal modulo"),
    ('वद((०-७.५) % ३, ७.५ % (०-३), १०.२५ % ०.५, (०-०.००१) % ०.३)।',
     "decimal modulo is floored, not truncated"),
    ('वद(९९९९९९९९९९९९९९९९९९९९९९९९९९९९.९ - ०.८)।', "no 28-digit rounding"),
    # --- Tier 2 standard library ---
    ('आनय "कालः" इति का। वद(का.वर्षः("२०२६-०७-२५"), का.मासः("२०२६-०७-२५"), '
     'का.दिनम्("२०२६-०७-२५"), का.वासरः("२०२६-०७-२५"))।', "कालः date parts"),
    ('आनय "कालः" इति का। वद(का.दिनयोगः("२०२६-०७-२५", ४०), '
     'का.दिनयोगः("२०२६-०१-०१", ०-१))।', "कालः date arithmetic"),
    ('आनय "कालः" इति का। वद(का.अन्तरम्("२०२४-०२-२८", "२०२४-०३-०१"), '
     'का.पूर्वम्("२०२६-०१-०१", "२०२६-०७-२५"))।', "कालः leap-year span"),
    ('आनय "कालः" इति का। वद(का.रूपय("२०२६-०७-०५", "%d/%m/%Y"), '
     'का.शुद्धः("२०२६-०२-२९"), का.अधिवर्षः(२०००))।', "कालः format & validate"),
    ('आनय "सारणी" इति सा। वद(सा.विश्लेषय("अ,ब\\nक,ख\\n"))।', "सारणी parse"),
    ('आनय "सारणी" इति सा। वद(सा.पाठय([["अ","ब, स"],["१","२"]]))।',
     "सारणी quotes what needs it"),
    ('आनय "सारणी" इति सा। मानय क = सा.कोशाः("नाम,नगरम्\\nआर्या,काशी\\n")। '
     'वद(क[१]["नाम"], दैर्घ्यम्(क))।', "सारणी header rows"),
    ('आनय "गूढ" इति गू। वद(गू.सङ्क्षेपः("नमस्ते"), गू.सङ्क्षेपः(""))।',
     "गूढ sha256"),
    ('आनय "गूढ" इति गू। वद(गू.गूढय("नमस्ते जगत्"), गू.प्रकटय(गू.गूढय("अ")))।',
     "गूढ base64 round trip"),
    ('आनय "परिवेशः" इति प। वद(प.चरः("SANSKRITA_NOT_SET_XYZ", "अनुपस्थितम्"))।',
     "परिवेशः env default"),
    ('आनय "सूचीकर्म" इति सू। वद(सू.क्रमय([३,१,२]), सू.क्रमय(["ग","अ","ब"]))।',
     "सूचीकर्म.क्रमय"),
    ('आनय "सूचीकर्म" इति सू। '
     'वद(सू.क्रमय([[१,"ब"],[२,"अ"],[३,"स"]], विधि(क) { फलम् क[२]। }))।',
     "सूचीकर्म.क्रमय with a key"),
    ('आनय "सूचीकर्म" इति सू। वद(सू.सङ्गमः([१,२],[२,३]), सू.सम्पातः([१,२,३],[२,३]), '
     'सू.भेदः([१,२,३],[२]))।', "सूचीकर्म set operations"),
    ('आनय "वाक्यकर्म" इति वाक। वद(वाक.आकारय("{} = {} ({})", "क", ०.१ + ०.२, सत्यम्))।',
     "वाक्यकर्म.आकारय"),
    ('वद([३,१,२].क्रमय(), "{} व {}".आकारय("अ", "ब"))।', "new methods on values"),
    ('आनय "गणितम्" इति ग। वद(ग.परिवृत्त(११२७२७.२७२७२७, २), ग.परिवृत्त(२.५), '
     'ग.परिवृत्त(०-२.५), ग.परिवृत्त(१००, २))।', "परिवृत्त rounds half away from zero"),
    ('आनय "गणितम्" इति ग। वद(ग.परिवृत्त(१२३४५६७८९०१२३४५६७८९०१२३४५६७८९०.५५५, २))।',
     "परिवृत्त is exact at any size"),
    ('आनय "वाक्यकर्म" इति वाक। '
     'वद("[" + वाक.पूरय("क", ५) + "][" + वाक.पूरय("क", ०-५) + "][" '
     '+ वाक.पूरय("अतिदीर्घम्", ३) + "]")।', "पूरय pads left and right"),
    # --- lambdas as values ---
    ('मानय द्वि = विधि(क) { फलम् क * २। }। वद(द्वि(२१), प्रकारः(द्वि))।',
     "lambda as a value"),
    ('विधि बाह्यम्(न्) { फलम् विधि(क) { फलम् क + न्। }। } '
     'वद(बाह्यम्(१०)(५))।', "closure over the enclosing scope"),
    ('आनय "सूचीकर्म" इति सू। '
     'वद(सू.प्रतिचित्रय([१,२,३], विधि(क) { फलम् क * क। }))।', "lambda as an argument"),
    ('मानय स = [विधि() { फलम् "अ"। }, विधि() { फलम् "ब"। }]। वद(स[१](), स[२]())।',
     "lambdas inside a सूची"),
    # a प्रयत half-way up gets only the frames BELOW it
    ('विधि अ() { प्रयत { फलम् ब()। } दोषे (त्रु) { फलम् त्रु.अनुरेखा। } } '
     'विधि ब() { फलम् स()। } विधि स() { फलम् १/०। } वद(अ())।',
     "traceback stops at the catching विधि"),
    # --- Tier 1: structured errors, identical in both engines ---
    ('प्रयत { वद(१/०)। } दोषे (त्रु) { वद(त्रु, त्रु.प्रकारः, त्रु.पङ्क्तिः)। }',
     "error kind and line"),
    ('प्रयत { वद(अनुपस्थितम्)। } दोषे (त्रु) { वद(त्रु.प्रकारः)। }',
     "unknown name is नामदोषः"),
    ('प्रयत { मानय स = [१]। वद(स[९])। } दोषे (त्रु) { वद(त्रु.प्रकारः)। }',
     "out of range is सीमादोषः"),
    ('प्रयत { क्षिप "मम"। } दोषे (त्रु) { वद(त्रु.प्रकारः, त्रु.सन्देशः, '
     'त्रु.आङ्ग्लसन्देशः)। }', "क्षिप is स्वयंदोषः"),
    ('विधि अ() { फलम् ब()। } विधि ब() { फलम् १/०। } '
     'प्रयत { अ()। } दोषे (त्रु) { वद(त्रु.अनुरेखा)। }', "traceback frames"),
    ('विधि क() { प्रयत { वद(१/०)। } दोषे (त्रु) { क्षिप त्रु। } } '
     'प्रयत { क()। } दोषे (त्रु) { वद(त्रु.प्रकारः, त्रु.अनुरेखा)। }',
     "क्षिप त्रु re-raises unchanged"),
    ('प्रयत { क्षिप "x"। } दोषे (त्रु) { वद(प्रकारः(त्रु))। }',
     "an error's type is दोषः"),
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
    ('क्षिप "विफलम्"।', "uncaught क्षिप"),
    ('मानय क : वाक्यम् = शून्यम्।', "typed variable refuses शून्यम्"),
    ('मानय क : पूर्णाङ्कः = "पञ्च"।', "type mismatch"),
    ('आनय "जेसन" इति ज। वद(ज.विश्लेषय("{अ}"))।', "malformed JSON"),
    ('मानय क : पूर्णाङ्कः = "पञ्च"।', "pre-flight type error"),
    ('मानय क : सङ्ख्या = ५।', "unknown type name"),
    ('मानय स = [१]। वद(स.क्रम())।', "unknown method"),
    ('विधि नम(क) { फलम् क। } वद(नम())।', "missing argument, no default"),
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
    "सूचीकर्मोदाहरणम्.सं",
    "जेसनोदाहरणम्.सं",
    "शृङ्खला.सं",
    "शुद्धिवेगौ.सं",
    "कोशागारम्.सं",
    "दोषविवरणम्.सं",
    "लेखापरीक्षा.सं",
    "वेतनपत्रम्.सं",
    # आदेशसाधनम्.सं is NOT listed: it reads the filesystem and calls
    # परिवेशः.निर्गम, so its output depends on where it is run from. It is
    # exercised by परीक्षा.py instead.
    # स्वपरीक्षा.सं is NOT listed: it imports परीक्षणम्.सं relative to its own
    # folder, and this harness runs sources from a temp path. परीक्षा.py runs it
    # in place instead.
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
