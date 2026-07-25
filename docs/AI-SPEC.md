# संस्कृता (Sanskrita) — Complete Language Specification for AI Assistants

> Paste this document into any AI (ChatGPT, Claude, Gemini…) and it can write, explain, and debug correct संस्कृता code. Version 0.4 "फलम्" (Phase 3 complete).

## What संस्कृता is

A real programming language with Sanskrit (Devanagari) keywords, run as `python3 sanskrita.py file.सं` (or `sanskrita file.सं` after install). Files use extension `.सं` (or `.sam` for roman mode). It is NOT a Python skin: it has its own lexer/parser/interpreter.

Two engines run the same language: the **reference engine** (`sanskrita.py`, which defines the language and hosts the Python bridge) and **वेगः** (`sanskrita --veg`, a native Rust binary). Both must produce byte-identical output; the only intentional difference is that वेगः refuses `python:` imports.

## Core rules (never violate these)

1. **Every statement ends with danda `।`** (roman mode: `|`). Blocks `{ }` don't need one after `}`.
2. **Comments:** `#` to end of line.
3. **Blocks use `{ }`**, never indentation.
4. **Conditions must be boolean** — `यदि (५)` is an error; write `यदि (क > ०)`.
5. **Lists/strings are 1-based**: `सूची[१]` is the first element.
6. **No string+number mixing**: `"आयुः" + ५` errors; convert with `वाक्यम्(५)`.
7. **Decimals are exact**: ०.१ + ०.२ == ०.३ (true, unlike Python/Java). A binary float exists but is opt-in: `द्रुतदशमांशः(०.१)`.
8. **Devanagari digits ०-९ and ASCII 0-9 both work.** Output defaults to Devanagari.
9. **Reserved words cannot be identifiers**: notably न (not), फलम् (return), इति, च, वा, सृज, अयम्.
10. Every keyword has a roman alias (see table); both scripts are ONE language.
11. **One script per identifier**: `नामx` (Devanagari + Latin mixed) is a lex error. Digits and `_` are neutral.
12. Source is NFC-normalized automatically — visually identical Devanagari is identical.
13. **शून्यम्-safety:** a typed variable cannot hold शून्यम् unless declared nullable — `मानय नाम? : वाक्यम् = शून्यम्।`
14. **प्राक्परीक्षा:** provable type errors and text/number mixing are reported *before the program runs*; if any are found, nothing executes.

## Keywords

| Devanagari | Roman | Meaning |
|---|---|---|
| मानय | manay(a) | declare variable |
| ध्रुव | dhruva | declare constant |
| यदि / अथ यदि / अन्यथा | yadi / atha yadi / anyatha | if / else if / else |
| यावत् | yavat | while loop |
| विरम / अनुवर्त | viram / anuvart | break / continue |
| प्रत्येकम् … इति | pratyekam … iti | for-each |
| विधि | vidhi | function definition |
| फलम् | phalam | return |
| वर्गः | vargah | class |
| सृज | srja | create instance |
| अयम् | ayam | this/self |
| प्रयत / दोषे | prayat / doshe | try / catch |
| क्षिप | kship(a) | throw / raise an error |
| आनय … इति … | anaya … iti … | import module as name |
| सत्यम् / असत्यम् / शून्यम् | satyam / asatyam / shunyam | true / false / null |
| च / वा / न | cha / vaa / na | and / or / not |

## Types

पूर्णाङ्कः (arbitrary-precision int) • दशमांशः (exact decimal) • **द्रुतदशमांशः** (IEEE-754 binary float, opt-in) • वाक्यम् (string) • सत्यासत्यम् (bool) • सूची (list) • कोशः (map) • शून्यम् (null).

Optional annotations: `मानय क : पूर्णाङ्कः = ५।` — checked before the run where provable, and on every later assignment. Only those seven names are valid type names. Add `?` for nullable: `मानय क? : वाक्यम् = शून्यम्।`

## Builtins

वद(…) print • पृच्छ(prompt) input • वाक्यम्(x) to-string • सङ्ख्या(s) to-number • **द्रुतदशमांशः(x) to-float** • प्रकारः(x) type-of • दैर्घ्यम्(x) length • योजय(list, v) append • अपनय(list, i) remove-at / अपनय(map, key) remove-key • कुञ्जिकाः(map) keys • क्रमय(list) sorted copy • परिधिः(a, b) inclusive integer range as a list • **आदेशचराः()** command-line arguments as a सूची.

## Syntax examples (canonical)

```
मानय नाम = "गौरी"।
मानय वयः : पूर्णाङ्कः = २५।
यदि (वयः >= १८ च वयः < ६०) { वद("प्रौढः")। } अन्यथा { वद("अन्यः")। }

मानय क = १।
यावत् (क <= ५) { वद(क)। क = क + १। }

विधि योग(क, ख) { फलम् क + ख। }

# kāraka-labeled parameters (UNIQUE feature): declare role before name,
# call with role: value in ANY order. Valid roles ONLY:
# कर्ता कर्म करण सम्प्रदान अपादान अधिकरण
विधि प्रेषय(कर्म सन्देशः, सम्प्रदान प्राप्ता) { वद(सन्देशः, "→", प्राप्ता)। }
प्रेषय(सम्प्रदान: "रामः", कर्म: "नमस्ते")।

मानय स = [१, २, ३]।            # स[१] is १ (1-based!)
मानय को = {"नाम": "गौरी"}।
प्रत्येकम् वस्तु इति स { वद(वस्तु)। }

वर्गः छात्रः {
    विधि आरम्भ(नाम) { अयम्.नाम = नाम। }      # आरम्भ = constructor
    विधि परिचय() { वद("अहं", अयम्.नाम)। }
}
मानय रमा = सृज छात्रः("रमा")।
रमा.परिचय()।

प्रयत { मानय क = १ / ०। } दोषे (त्रुटिः) { वद(त्रुटिः)। }

# raise your own error
विधि भागः(क, ख) {
    यदि (ख == ०) { क्षिप "शून्येन भागः न शक्यः"। }
    फलम् क / ख।
}

# default parameter values — the expression is re-evaluated on EVERY call,
# so a mutable default can never be shared between calls
विधि अभिवादय(कर्म नाम, करण भाषा = "संस्कृतम्") { वद(नाम, भाषा)। }
अभिवादय(कर्म: "गौरी")।

# शून्यम्-safety: '?' is the opt-in
मानय उपनाम? : वाक्यम् = शून्यम्।

# exact by default, fast when you ask
वद(०.१ + ०.२)।                                # ०.३
वद(द्रुतदशमांशः(०.१) + द्रुतदशमांशः(०.२))।     # ०.३०००००००००००००००४

# method chaining — `.नाम` on a सूची/वाक्यम्/कोशः is the stdlib function
# with the receiver as its first argument
वद([३, १, २].क्रमय().विपर्यय())।
वद([१, २, ३, ४].छानय(विधि(क) { फलम् क > २। }).योगः())।
वद("  अ,ब  ".परिष्कार().विभज(","))।
```

## Modules

**Native (Sanskrit names):**

```
आनय "संस्कृतम्" इति सं।     # linguistics: सं.अक्षराणि सं.मात्राः सं.छन्दः सं.रोमनय सं.देवनागरय सं.संधय
आनय "गणितम्" इति ग।        # math: ग.वर्गमूलम् ग.घातः ग.ज्या ग.कोज्या ग.पाई ग.तलम् ग.उपरितलम्
आनय "यादृच्छिकम्" इति य।   # random: य.अन्तरे(a,b) य.वरय(सूची) य.भिन्नम्()
आनय "कालः" इति का।         # time: का.अद्य() का.संप्रति() का.वर्षः()
आनय "वाक्यकर्म" इति वा।    # strings: वा.विभज(t,sep) वा.संयोजय(list,sep) वा.खोज(t,sub)→1-based(०=absent)
                            #          वा.प्रतिस्थापय(t,old,new) वा.अंश(t,i,j) substring 1-based inclusive
                            #          वा.उच्च वा.निम्न वा.परिष्कार वा.आरभते वा.अन्तयति वा.अन्तर्भवति
आनय "सूचीकर्म" इति सू।     # lists: सू.छानय(l,f) सू.प्रतिचित्रय(l,f) सू.न्यूनीकरण(l,f,init)
                            #        सू.विपर्यय सू.अन्तर्भवति सू.अनुक्रमः(→1-based, ०=absent)
                            #        सू.योगः सू.महत्तमम् सू.लघुत्तमम् सू.अद्वितीयम्
आनय "सञ्चिका" इति स।       # files (UTF-8): स.पठ(p) स.लिख(p,t) स.योजय(p,t) स.अस्ति(p)
                            #                स.निष्कासय(p) स.पङ्क्तयः(p) स.सूचिका(dir)
आनय "जेसन" इति ज।          # JSON: ज.विश्लेषय(text)→value  ज.पाठय(value)→text
                            #       numbers with a fraction come back as दशमांशः, never a float
```

Every सूचीकर्म and वाक्यकर्म function is also reachable as a method on the value
itself: `सू.छानय(l, f)` and `l.छानय(f)` are the same call. So are the builtins
क्रमय, दैर्घ्यम्, योजय, अपनय (on सूची), कुञ्जिकाः, दैर्घ्यम्, अपनय (on कोशः),
and दैर्घ्यम् (on वाक्यम्).

**The user's own .सं files (v0.3+):**

```
आनय "सहायः.सं" इति सहायः।   # runs the file once, exposes its top-level
वद(सहायः.द्विगुणः(२१))।      # विधिs/variables as सहायः.name
```

Paths resolve relative to the importing file; modules are cached (imported once).

**Python bridge (any installed Python module):**

```
आनय "python:statistics" इति सां।
वद(सां.mean([९५, ८८, ९२]))।
```

Values convert automatically (Decimal↔float, lists, dicts, strings).

## Error format

Bilingual with line numbers and did-you-mean hints:

```
दोषः पङ्क्तौ २ — अज्ञातं नाम 'वड' — किं 'वद' इति अभिप्रेतम्?
Error at line 2 — unknown name 'वड' — did you mean: वद?
```

## Common mistakes to avoid when generating code

- Forgetting the danda `।` at statement end (most common).
- Using न, फलम्, or इति as variable names.
- 0-based indexing — it's 1-based.
- `यदि क > ५ {` — parentheses required: `यदि (क > ५) {`.
- Non-kāraka argument labels — only the six kārakas are valid labels.
- Truthiness — conditions must be actual booleans.
- Inventing type names: only पूर्णाङ्कः, दशमांशः, द्रुतदशमांशः, वाक्यम्, सत्यासत्यम्, सूची, कोशः are valid after `:`.
- Assigning शून्यम् to a typed variable without `?` — that is a compile-time error by design.
- Expecting `०.१ + ०.२` to be inexact — it is exactly `०.३` here; use `द्रुतदशमांशः()` if you *want* float behaviour.
- Using `python:` imports in a program meant for the वेगः engine — वेगः rejects them by design.
