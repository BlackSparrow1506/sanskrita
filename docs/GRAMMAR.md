# GRAMMAR — the formal grammar of संस्कृता

*व्याकरणम् · complete EBNF for v0.6.0*

This is the normative grammar. Both engines implement it — `sanskrita.py`
(reference) and `rust-engine/` (वेगः) — and `तुल्यता.py` checks that they agree.
If an engine disagrees with this document, that is a bug in the engine or in
this document, and either way it is worth an issue.

**Notation.** `=` defines a rule, `|` alternation, `[ x ]` optional,
`{ x }` zero or more, `( )` grouping, `"x"` a literal, `..` a character range.
Rules are lowercase; terminals produced by the lexer are `UPPERCASE`.

---

## 1. Lexical structure

### 1.1 Source encoding

Source is UTF-8 and is **normalized to NFC before anything else happens**. Two
files that look identical are identical to the engine — this is mandatory, not
an optimization (design risk #8).

### 1.2 Whitespace and comments

```ebnf
whitespace = " " | "\t" | "\r" | "\n" ;
comment    = "#" , { any-char - "\n" } ;
```

Whitespace and comments separate tokens and are otherwise ignored.
**Indentation has no meaning.** Blocks are `{ }` and always were.

### 1.3 Statement terminator — the danda

```ebnf
END = "।" | "॥" | "|" ;
```

`।` is U+0964, `॥` is U+0965. `|` is the roman-mode spelling. The danda is
**punctuation, never part of an identifier** — a subtlety that matters because
U+0964 sits inside the Devanagari Unicode block.

### 1.4 Identifiers

```ebnf
IDENT      = ident-start , { ident-cont } ;
ident-start = "_" | letter | devanagari ;
ident-cont  = ident-start | digit | dev-digit | devanagari-mark ;

devanagari      = U+0900..U+097F - danda - dev-digit ;
devanagari-mark = U+0900..U+097F - danda ;   (* matras, virama, nukta, anusvara *)
```

Two rules the lexer enforces:

- **One script per identifier.** `नामx` — Devanagari mixed with Latin letters —
  is a lex error (design risk #7). Digits and `_` are script-neutral.
- **No sandhi in names.** What you type is the name. Sandhi lives in the
  standard library as a tool, never in the grammar (design risk #7).

### 1.5 Numbers

```ebnf
NUMBER    = digits , [ "." , digits ] ;
digits    = ( digit | dev-digit ) , { digit | dev-digit } ;
digit     = "0".."9" ;
dev-digit = "०".."९" ;   (* U+0966..U+096F *)
```

Devanagari and ASCII digits are interchangeable everywhere and may be mixed
within one literal. A literal is kept **as typed** and converted to an exact
integer or exact decimal at evaluation — never to a binary float.

A `.` is only a decimal point when a digit follows it; `५।` is the number ५
followed by a danda, not a malformed decimal.

### 1.6 Strings

```ebnf
STRING = '"' , { string-char | escape } , '"' ;
escape = "\\" , any-char ;
```

Recognised escapes: `\n`, `\t`, `\"`, `\\`. Any other `\x` yields `x`. Strings
may span lines.

### 1.7 Operators and punctuation

```ebnf
OP = "==" | "!=" | "<=" | ">="
   | "+" | "-" | "*" | "/" | "%"
   | "<" | ">" | "=" | "?"
   | "(" | ")" | "{" | "}" | "[" | "]"
   | "," | ":" | "." ;
```

Two-character operators are matched before one-character ones.

### 1.8 Keywords

```
मानय  ध्रुव  यदि  अथ  अन्यथा  यावत्  सत्यम्  असत्यम्  शून्यम्
च  वा  न  विरम  अनुवर्त  विधि  फलम्  प्रत्येकम्  इति
वर्गः  सृज  अयम्  प्रयत  दोषे  आनय  क्षिप
```

Keywords are reserved and cannot be identifiers. Every keyword has a permanent
roman alias (`maanaya`, `yadi`, `yaavat`, `kshipa`, …) which the lexer maps to
the Devanagari form before parsing. **Script is presentation; the grammar is one
language** (design risk #5).

### 1.9 Type names

Valid only after `:` in a declaration:

```
पूर्णाङ्कः  दशमांशः  द्रुतदशमांशः  वाक्यम्  सत्यासत्यम्  सूची  कोशः
```

Any other name there is an error, in both engines.

### 1.10 Kāraka roles

Valid only as argument-role labels:

```
कर्ता  कर्म  करण  सम्प्रदान  अपादान  अधिकरण
```

Exactly six, permanently (design risk #9). They are **explicit syntax**, never
inferred from word endings — the engine does zero morphology.

---

## 2. Program and statements

```ebnf
program = { statement } , EOF ;

statement = declaration
          | assignment
          | if-statement
          | while-statement
          | foreach-statement
          | function-def
          | class-def
          | try-statement
          | throw-statement
          | import-statement
          | return-statement
          | break-statement
          | continue-statement
          | expression-statement ;

block = "{" , { statement } , "}" ;
```

Note that `block` needs no danda after `}`.

### 2.1 Declaration

```ebnf
declaration = ( "मानय" | "ध्रुव" ) , IDENT , [ "?" ] ,
              [ ":" , TYPE-NAME ] , "=" , expression , END ;
```

- `ध्रुव` declares a constant; reassigning it is an error.
- `?` marks the variable **nullable**. Without it, a *typed* variable cannot
  hold `शून्यम्` — the billion-dollar mistake, refused (design §2b).
- The type annotation is optional. Where a violation is provable from the
  source, it is reported **before the program runs** (see §5); otherwise it is
  checked at every assignment.

### 2.2 Assignment

```ebnf
assignment = target , "=" , expression , END ;
target     = IDENT
           | postfix-expression , "[" , expression , "]"
           | postfix-expression , "." , IDENT ;
```

Assigning to a name that was never declared is an error — there is no implicit
declaration.

### 2.3 Conditionals

```ebnf
if-statement = "यदि" , "(" , expression , ")" , block ,
               { "अथ" , "यदि" , "(" , expression , ")" , block } ,
               [ "अन्यथा" , block ] ;
```

The condition **must be a boolean**. There is no truthiness: `यदि (५)` is an
error, and so is `यदि ("")`.

### 2.4 Loops

```ebnf
while-statement   = "यावत्" , "(" , expression , ")" , block ;
foreach-statement = "प्रत्येकम्" , IDENT , "इति" , expression , block ;
break-statement    = "विरम" , END ;
continue-statement = "अनुवर्त" , END ;
```

`प्रत्येकम्` iterates a `सूची`, the keys of a `कोशः`, or the characters of a
`वाक्यम्`. `विरम`/`अनुवर्त` outside a loop is an error.

### 2.5 Functions

```ebnf
function-def = "विधि" , IDENT , param-list , block ;
param-list   = "(" , [ param , { "," , param } ] , ")" ;
param        = [ KARAKA ] , IDENT , [ "=" , expression ] ;

return-statement = "फलम्" , [ expression ] , END ;
```

- A parameter may carry a **kāraka role**, which lets callers pass it by name in
  any order (§3.3).
- A default value is stored as an **expression** and re-evaluated in the
  function's own scope on **every call** — so a mutable default can never be
  shared between calls (design §2b).
- `फलम्` with no expression returns `शून्यम्`. `फलम्` outside a function is an
  error.

### 2.6 Classes

```ebnf
class-def = "वर्गः" , IDENT , [ ":" , IDENT ] , "{" , { method } , "}" ;
method    = "विधि" , IDENT , param-list , block ;
```

`: नाम` names the parent class. The constructor is the method named `आरम्भ`.
Inside a method, `अयम्` is the instance. Instances are made with `सृज` (§3.5).

### 2.7 Errors

```ebnf
try-statement   = "प्रयत" , block , "दोषे" , "(" , IDENT , ")" , block ;
throw-statement = "क्षिप" , expression , END ;
```

`क्षिप` raises an error carrying the given value's text, with kind `स्वयंदोषः`.
If the value is itself an error (one caught earlier), it is re-raised
**unchanged** — same kind, same line, same traceback.

The name bound by `दोषे` holds an error **value**, whose type is `दोषः`. It
renders as its Sanskrit message, and exposes exactly five fields:

```ebnf
error-field = "सन्देशः" | "आङ्ग्लसन्देशः" | "पङ्क्तिः" | "प्रकारः" | "अनुरेखा" ;
```

`प्रकारः` is one of eight stable kinds — `दोषः`, `नामदोषः`, `प्रकारदोषः`,
`गणितदोषः`, `सीमादोषः`, `व्याकरणदोषः`, `आयातदोषः`, `स्वयंदोषः`. The set is part
of the compatibility promise (`docs/STABILITY.md`): code that branches on a kind
must keep working. `अनुरेखा` is a `सूची` of `"नाम (पङ्क्तिः)"` text, outermost
call first.

### 2.8 Imports

```ebnf
import-statement = "आनय" , STRING , "इति" , IDENT , END ;
```

The string is one of:

| Form | Meaning |
|---|---|
| `"गणितम्"`, `"सूचीकर्म"`, … | a native standard-library module |
| `"सहायः.सं"` | another संस्कृता file, resolved relative to the importing file, executed once and cached |
| `"python:math"` | any installed Python module, through the bridge — **reference engine only**; वेगः rejects it by design |

### 2.9 Expression statement

```ebnf
expression-statement = expression , END ;
```

---

## 3. Expressions

Lowest precedence first. Every level is **left-associative** except unary.

```ebnf
expression     = or-expression ;
or-expression  = and-expression , { "वा" , and-expression } ;
and-expression = not-expression , { "च" , not-expression } ;
not-expression = "न" , not-expression | comparison ;
comparison     = additive , [ ( "==" | "!=" | "<" | ">" | "<=" | ">=" ) , additive ] ;
additive       = multiplicative , { ( "+" | "-" ) , multiplicative } ;
multiplicative = unary , { ( "*" | "/" | "%" ) , unary } ;
unary          = "-" , unary | postfix ;
postfix        = primary , { call-suffix | index-suffix | attr-suffix } ;
```

### 3.1 Precedence summary

| Level | Operators | Associativity |
|---|---|---|
| 1 (loosest) | `वा` | left, short-circuits |
| 2 | `च` | left, short-circuits |
| 3 | `न` | prefix, right |
| 4 | `==` `!=` `<` `>` `<=` `>=` | **non-associative** — `अ < ब < स` is not allowed |
| 5 | `+` `-` | left |
| 6 | `*` `/` `%` | left |
| 7 | unary `-` | right |
| 8 (tightest) | `(…)` call, `[…]` index, `.` attribute | left |

`च` and `वा` require booleans on both sides and short-circuit.

### 3.2 Postfix

```ebnf
call-suffix  = "(" , [ argument , { "," , argument } ] , ")" ;
index-suffix = "[" , expression , "]" ;
attr-suffix  = "." , IDENT ;
```

Indexing is **1-based**: `सूची[१]` is the first element. A `कोशः` is indexed by
its key. Out-of-range and missing keys are errors, not `शून्यम्`.

`.नाम` reaches: a field or method of an object, a member of a module, or — on a
`सूची`, `वाक्यम्` or `कोशः` — the matching standard-library function with the
receiver as its first argument (design §7d #2 sandhi-style composition). So
`स.छानय(f)` and `सू.छानय(स, f)` are the same call.

### 3.3 Arguments and kārakas

```ebnf
argument = [ KARAKA , ":" ] , expression ;
```

An argument may name the **role** it fills instead of relying on position:

```
प्रेषय(कर्म: "नमस्ते", सम्प्रदान: "रामः")।
प्रेषय(सम्प्रदान: "रामः", कर्म: "नमस्ते")।    # identical
```

Rules: only the six kārakas are valid labels; a role must exist on the function
being called; labelled and positional arguments may be mixed, and labels are
matched first. Module functions and builtins do not take labels.

### 3.4 Primary

```ebnf
primary = NUMBER
        | STRING
        | "सत्यम्" | "असत्यम्" | "शून्यम्"
        | IDENT
        | "अयम्"
        | list-literal
        | map-literal
        | lambda
        | new-expression
        | "(" , expression , ")" ;

list-literal = "[" , [ expression , { "," , expression } , [ "," ] ] , "]" ;
map-literal  = "{" , [ pair , { "," , pair } , [ "," ] ] , "}" ;
pair         = expression , ":" , expression ;
lambda       = "विधि" , param-list , block ;
```

A lambda is a `विधि` with no name — a first-class value like any other.

### 3.5 Object creation

```ebnf
new-expression = "सृज" , postfix ;
```

`सृज` must be followed by a class call: `सृज छात्रः("रमा")`.

---

## 4. Types and values

| Type | Literal | Notes |
|---|---|---|
| `पूर्णाङ्कः` | `५`, `42`, `-७` | arbitrary precision — no overflow, ever |
| `दशमांशः` | `३.१४`, `०.१` | **exact decimal**; `०.१ + ०.२ == ०.३` is `सत्यम्` |
| `द्रुतदशमांशः` | *(no literal)* | IEEE-754 binary float, made only by `द्रुतदशमांशः(x)` |
| `वाक्यम्` | `"नमस्ते"` | UTF-8 text, 1-based indexing |
| `सत्यासत्यम्` | `सत्यम्` / `असत्यम्` | |
| `सूची` | `[१, २]` | shared and mutable; assigning aliases it |
| `कोशः` | `{"क": १}` | insertion-ordered; keys are text or whole numbers |
| `शून्यम्` | `शून्यम्` | only storable in a variable declared `नाम?` |

### 4.1 Arithmetic

- Integer arithmetic is exact and unbounded.
- `/` always produces a `दशमांशः`, so a result's *type* never depends on its
  runtime values. **Division is exact whenever it divides evenly — at any size,
  with no digit ceiling.** `३०६५०९…४३ / २२५` returns all 55 of its digits, and
  `१ / ५१२` returns `०.००१९५३१२५` in full. Only a quotient that would repeat
  forever is cut, to 28 significant digits, rounded half-even — `१ / ३` is
  `०.३३३३३३३३३३३३३३३३३३३३३३३३३३३३`.
- `%` is **floored**, matching the reference: `(०-७) % ३` is `२`, not `-१`.
- Mixing an integer and a `दशमांशः` promotes to `दशमांशः`.
- A `द्रुतदशमांशः` on either side makes the whole operation binary-float — the
  fast path is contagious, and `प्रकारः()` will always tell you what you hold.
- `+` also concatenates two `वाक्यम्`s and joins two `सूची`s. Mixing text and a
  number is an error, not a coercion.
- Division or modulo by zero is an error.

#### How a quotient is written

Two values can be equal and still say different things about precision, so the
*form* of a quotient is specified, not incidental:

- An **exact** division sheds trailing zeros only down to the ideal exponent —
  `exp(dividend) − exp(divisor)`. So `२४४.२० / २` is `१२२.१०`, and `१२२१.० / १०`
  is `१२२.१`. The retained zero is the precision the operands claimed, which is
  what a money column depends on.
- A **whole-number** result is written as a whole number, even when the division
  was inexact. `६.०० / ३` is `२`, and `ब / (ब+१)` — which rounds to exactly
  `१.०००…०` at 28 digits — is `१`.

#### शून्यम् has no sign

There is no negative zero. `(०-७०) * ०` is `०`, not `-०`. The scale is still
kept, so `(०-१) * ०.००` is `०.००`.

The one exception is `द्रुतदशमांशः`, which is IEEE-754 by name and by
definition; `-०.०` is a real value there, and printed as such.

---

## 5. प्राक्परीक्षा — checks before execution

After parsing and before the first statement runs, the engine proves what it can
from the source alone and reports **all** findings at once. If there is even one,
**nothing executes**.

Currently proved:

- a typed declaration initialised with `शून्यम्` and no `?`
- a typed declaration initialised with a literal of a provably wrong type
- arithmetic that provably mixes text and a number

This pass is deliberately conservative — it never guesses. Anything it cannot
prove is still checked at runtime.

---

## 6. Reserved for future versions

These are not in the language today. They are listed so nobody builds a tool
that would break when they arrive:

- `सूत्र` (concurrency), `प्रतीक्ष` (await)
- `सङ्ग्रहः` (set literals)
- `साञ्चम्` (pattern matching / destructuring)
- `अन्तराल` (slice syntax `सूची[१..३]`)
- string interpolation
- `प्रकारः` as a user-definable type declaration

Do not use these as identifiers if you want your code to survive.
