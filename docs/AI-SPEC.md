# संस्कृता (Sanskrita) — Complete Language Specification for AI Assistants

> Paste this document into any AI (ChatGPT, Claude, Gemini…) and it can write, explain, and debug correct संस्कृता code. Version 0.2 "वृक्षः".

## What संस्कृता is

A real interpreted programming language with Sanskrit (Devanagari) keywords, run as `python3 sanskrita.py file.सं`. Files use extension `.सं` (or `.sam` for roman mode). It is NOT a Python skin: it has its own lexer/parser/interpreter, but can call Python libraries through a bridge.

## Core rules (never violate these)

1. **Every statement ends with danda `।`** (roman mode: `|`). Blocks `{ }` don't need one after `}`.
2. **Comments:** `#` to end of line.
3. **Blocks use `{ }`**, never indentation.
4. **Conditions must be boolean** — `यदि (५)` is an error; write `यदि (क > ०)`.
5. **Lists/strings are 1-based**: `सूची[१]` is the first element.
6. **No string+number mixing**: `"आयुः" + ५` errors; convert with `वाक्यम्(५)`.
7. **Decimals are exact**: ०.१ + ०.२ == ०.३ (true, unlike Python/Java).
8. **Devanagari digits ०-९ and ASCII 0-9 both work.** Output defaults to Devanagari.
9. **Reserved words cannot be identifiers**: notably न (not), फलम् (return), इति, च, वा, सृज, अयम्.
10. Every keyword has a roman alias (see table); both scripts are ONE language.

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
| आनय … इति … | anaya … iti … | import module as name |
| सत्यम् / असत्यम् / शून्यम् | satyam / asatyam / shunyam | true / false / null |
| च / वा / न | cha / vaa / na | and / or / not |

## Types

पूर्णाङ्कः (int) • दशमांशः (exact decimal) • वाक्यम् (string) • सत्यासत्यम् (bool) • सूची (list) • कोशः (map). Optional annotations: `मानय क : पूर्णाङ्कः = ५।` — enforced on all later assignments.

## Builtins

वद(…) print • पृच्छ(prompt) input • वाक्यम्(x) to-string • सङ्ख्या(s) to-number • प्रकारः(x) type-of • दैर्घ्यम्(x) length • योजय(list, v) append • अपनय(list, i) remove-at • कुञ्जिकाः(map) keys • क्रमय(list) sorted copy.

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
```

## Modules

**Native (Sanskrit names):**

```
आनय "संस्कृतम्" इति सं।     # linguistics: सं.अक्षराणि सं.मात्राः सं.छन्दः सं.रोमनय सं.देवनागरय सं.संधय
आनय "गणितम्" इति ग।        # math: ग.वर्गमूलम् ग.घातः ग.ज्या ग.कोज्या ग.पाई ग.तलम् ग.उपरितलम्
आनय "यादृच्छिकम्" इति य।   # random: य.अन्तरे(a,b) य.वरय(सूची) य.भिन्नम्()
आनय "कालः" इति का।         # time: का.अद्य() का.संप्रति() का.वर्षः()
```

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
