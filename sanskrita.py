#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
ॐ  संस्कृता (Sanskrita) — v0.2 "वृक्षः" (Tree)
The Sanskrit programming language — Phase 2 interpreter.

Usage:
    python3 sanskrita.py program.सं        run a program
    python3 sanskrita.py                   interactive REPL
    python3 sanskrita.py --roman prog.सं   output digits in ASCII
    python3 sanskrita.py --convert f.sam   rewrite file in canonical Devanagari

Phase 2 adds (per BLUEPRINT.md):
    विधि functions (first-class, closures, recursion) with फलम् return,
    kāraka-labeled arguments (कर्म:, करण:, सम्प्रदान:…),
    सूची lists & कोशः maps (1-based indexing — प्रथमः = १),
    प्रत्येकम् … इति for-each, वर्गः classes with सृज and अयम्,
    प्रयत/दोषे error handling, आनय Python-bridge.
"""

import difflib
import importlib
import os
import sys
import unicodedata
from decimal import Decimal

sys.setrecursionlimit(4000)

# ---------------------------------------------------------------- digits

DEV_DIGITS = "०१२३४५६७८९"

def to_ascii_digits(s: str) -> str:
    return "".join(str(DEV_DIGITS.index(c)) if c in DEV_DIGITS else c for c in s)

def to_dev_digits(s: str) -> str:
    return "".join(DEV_DIGITS[int(c)] if ("0" <= c <= "9") else c for c in s)

# ------------------------------------------------------- keywords/aliases

# roman alias  ->  canonical Devanagari word
ALIASES = {
    "maanaya": "मानय", "manaya": "मानय", "manay": "मानय",
    "dhruva": "ध्रुव", "dhruv": "ध्रुव",
    "yadi": "यदि",
    "atha": "अथ",
    "anyathaa": "अन्यथा", "anyatha": "अन्यथा",
    "yaavat": "यावत्", "yavat": "यावत्",
    "satyam": "सत्यम्",
    "asatyam": "असत्यम्",
    "shuunyam": "शून्यम्", "shunyam": "शून्यम्",
    "cha": "च", "ca": "च",
    "vaa": "वा",
    "na": "न",
    "virama": "विरम", "viram": "विरम",
    "anuvarta": "अनुवर्त", "anuvart": "अनुवर्त",
    # Phase 2 keywords
    "vidhi": "विधि",
    "phalam": "फलम्",
    "pratyekam": "प्रत्येकम्",
    "iti": "इति",
    "vargah": "वर्गः", "varga": "वर्गः",
    "srja": "सृज", "srija": "सृज",
    "ayam": "अयम्",
    "prayata": "प्रयत", "prayat": "प्रयत",
    "doshe": "दोषे",
    "aanaya": "आनय", "anaya": "आनय",
    "aarambha": "आरम्भ", "arambha": "आरम्भ",   # constructor name (identifier)
    # kāraka role labels
    "kartaa": "कर्ता", "karta": "कर्ता",
    "karma": "कर्म",
    "karana": "करण",
    "sampradaana": "सम्प्रदान", "sampradana": "सम्प्रदान",
    "apaadaana": "अपादान", "apadana": "अपादान",
    "adhikarana": "अधिकरण",
    # type names
    "purnankah": "पूर्णाङ्कः", "purnanka": "पूर्णाङ्कः",
    "dashamanshah": "दशमांशः", "dashamansha": "दशमांशः",
    "satyasatyam": "सत्यासत्यम्",
    "suchee": "सूची", "suchi": "सूची",
    "koshah": "कोशः", "kosha": "कोशः",
    # builtins
    "vada": "वद", "vad": "वद",
    "prccha": "पृच्छ", "pruccha": "पृच्छ", "puccha": "पृच्छ",
    "vaakyam": "वाक्यम्", "vakyam": "वाक्यम्",
    "sankhyaa": "सङ्ख्या", "sankhya": "सङ्ख्या",
    "prakarah": "प्रकारः", "prakara": "प्रकारः",
    "dairghyam": "दैर्घ्यम्", "dairghya": "दैर्घ्यम्",
    "paridhih": "परिधिः", "paridhi": "परिधिः",
    # वाक्यकर्म (string) module functions
    "vibhaja": "विभज",
    "sanyojaya": "संयोजय",
    "khoja": "खोज",
    "pratisthapaya": "प्रतिस्थापय",
    "ansha": "अंश",
    "yojaya": "योजय",
    "apanaya": "अपनय",
    "kunjikaah": "कुञ्जिकाः", "kunjikah": "कुञ्जिकाः", "kunjika": "कुञ्जिकाः",
    "kramaya": "क्रमय",
}

KEYWORDS = {"मानय", "ध्रुव", "यदि", "अथ", "अन्यथा", "यावत्",
            "सत्यम्", "असत्यम्", "शून्यम्", "च", "वा", "न",
            "विरम", "अनुवर्त",
            "विधि", "फलम्", "प्रत्येकम्", "इति", "वर्गः", "सृज",
            "प्रयत", "दोषे", "आनय"}

BUILTIN_NAMES = {"वद", "पृच्छ", "वाक्यम्", "सङ्ख्या", "प्रकारः", "दैर्घ्यम्",
                 "योजय", "अपनय", "कुञ्जिकाः", "क्रमय", "परिधिः"}

TYPE_NAMES = {"पूर्णाङ्कः", "दशमांशः", "वाक्यम्", "सत्यासत्यम्", "सूची", "कोशः"}

KARAKAS = {"कर्ता", "कर्म", "करण", "सम्प्रदान", "अपादान", "अधिकरण"}

# ------------------------------------------------- roman → Devanagari converter

def devanagarify(src: str) -> str:
    """Convert roman-mode source to canonical Devanagari form:
    keyword aliases -> केवल-देवनागरी keywords, ASCII digits -> ०-९, '|' -> '।'.
    Strings and comments are left untouched. Identifiers stay as typed."""
    src = unicodedata.normalize("NFC", src)
    out = []
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if c == '"':                                   # copy strings verbatim
            j = i + 1
            while j < n and src[j] != '"':
                if src[j] == "\\":
                    j += 1
                j += 1
            j = min(j + 1, n)
            out.append(src[i:j]); i = j; continue
        if c == "#":                                   # copy comments verbatim
            j = src.find("\n", i)
            j = n if j == -1 else j
            out.append(src[i:j]); i = j; continue
        if c == "|":
            out.append("।"); i += 1; continue
        if c.isascii() and c.isdigit():
            out.append(DEV_DIGITS[int(c)]); i += 1; continue
        cat = unicodedata.category(c)
        if c == "_" or cat[0] == "L":                  # word: alias -> canonical
            j = i
            while j < n:
                cj = src[j]; catj = unicodedata.category(cj)
                if cj == "_" or catj[0] == "L" or catj in ("Mn", "Mc") or cj.isdigit():
                    j += 1
                else:
                    break
            word = src[i:j]
            out.append(ALIASES.get(word, word))
            i = j; continue
        out.append(c); i += 1
    return "".join(out)

# ------------------------------------------------------- did-you-mean hints

def hint_for(word, extra=()):
    """Fuzzy-match a mistyped word against keywords, builtins, aliases and
    known variables. Returns bilingual hint suffixes ('' if no match)."""
    cands = set(ALIASES) | KEYWORDS | BUILTIN_NAMES | set(extra)
    cutoff = 0.4 if len(word) <= 3 else 0.55      # short words need a looser net
    matches = difflib.get_close_matches(word, cands, n=3, cutoff=cutoff)
    shown = [f"{m} ({ALIASES[m]})" if m in ALIASES else m for m in matches]
    if not shown:
        return "", ""
    joined = ", ".join(shown)
    return (f" — किं '{joined}' इति अभिप्रेतम्?",
            f" — did you mean: {joined}?")

# ---------------------------------------------------------------- errors

class SanskritaError(Exception):
    """Bilingual runtime/syntax error."""
    def __init__(self, line, sa, en):
        self.line = line
        self.sa = sa
        self.en = en
        msg = (f"दोषः पङ्क्तौ {to_dev_digits(str(line))} — {sa}\n"
               f"Error at line {line} — {en}")
        super().__init__(msg)


class BreakSignal(Exception):
    def __init__(self, line):
        self.line = line


class ContinueSignal(Exception):
    def __init__(self, line):
        self.line = line


class ReturnSignal(Exception):
    def __init__(self, value, line):
        self.value = value
        self.line = line

# ------------------------------------------------------------ type helpers

def type_name_of(v):
    if isinstance(v, bool):
        return "सत्यासत्यम्"
    if v is None:
        return "शून्यम्"
    if isinstance(v, int):
        return "पूर्णाङ्कः"
    if isinstance(v, Decimal):
        return "दशमांशः"
    if isinstance(v, str):
        return "वाक्यम्"
    if isinstance(v, list):
        return "सूची"
    if isinstance(v, dict):
        return "कोशः"
    if isinstance(v, (SFunction, Builtin, BoundMethod)):
        return "विधिः"
    if isinstance(v, SClass):
        return "वर्गः"
    if isinstance(v, SInstance):
        return v.cls.name
    if isinstance(v, PyVal):
        return "python-वस्तु"
    if isinstance(v, SModule):
        return "कोष्ठकम्"
    return "अज्ञातः"


def type_matches(typename, v):
    if typename == "पूर्णाङ्कः":
        return isinstance(v, int) and not isinstance(v, bool)
    if typename == "दशमांशः":
        return isinstance(v, (int, Decimal)) and not isinstance(v, bool)
    if typename == "वाक्यम्":
        return isinstance(v, str)
    if typename == "सत्यासत्यम्":
        return isinstance(v, bool)
    if typename == "सूची":
        return isinstance(v, list)
    if typename == "कोशः":
        return isinstance(v, dict)
    return True

# ----------------------------------------------------------------- lexer

def lex(src: str):
    src = unicodedata.normalize("NFC", src)    # §10 #8: NFC mandatory —
    toks = []                                  # visually identical text is identical
    i, n, line = 0, len(src), 1
    while i < n:
        c = src[i]
        if c == "\n":
            line += 1; i += 1; continue
        if c in " \t\r":
            i += 1; continue
        if c == "#":                                   # comment
            while i < n and src[i] != "\n":
                i += 1
            continue
        if c in "।॥|":                                 # statement end (danda)
            toks.append(("END", "।", line)); i += 1; continue
        if c == '"':                                   # string
            i += 1; buf = []; start = line
            while i < n and src[i] != '"':
                if src[i] == "\\" and i + 1 < n:
                    esc = src[i + 1]
                    buf.append({"n": "\n", "t": "\t", '"': '"', "\\": "\\"}.get(esc, esc))
                    i += 2
                else:
                    if src[i] == "\n":
                        line += 1
                    buf.append(src[i]); i += 1
            if i >= n:
                raise SanskritaError(start, "अपूर्णं वाक्यम् — समापनचिह्नं नास्ति",
                                     'unterminated string — missing closing "')
            i += 1
            toks.append(("STR", "".join(buf), line)); continue
        if c.isdigit():                                # number (Devanagari or ASCII)
            j, dot = i, False
            while j < n:
                cj = src[j]
                if cj.isdigit():
                    j += 1
                elif cj == "." and not dot and j + 1 < n and src[j + 1].isdigit():
                    dot = True; j += 1
                else:
                    break
            raw = to_ascii_digits(src[i:j])
            toks.append(("NUM", Decimal(raw) if dot else int(raw), line))
            i = j; continue
        if src[i:i + 2] in ("==", "!=", "<=", ">="):   # two-char operators
            toks.append(("OP", src[i:i + 2], line)); i += 2; continue
        if c in "+-*/%<>=(){}[],:.":
            toks.append(("OP", c, line)); i += 1; continue
        cat = unicodedata.category(c)
        if c == "_" or cat[0] == "L":                  # identifier / keyword
            j = i
            while j < n:
                cj = src[j]; catj = unicodedata.category(cj)
                if cj == "_" or catj[0] == "L" or catj in ("Mn", "Mc") or cj.isdigit():
                    j += 1
                else:
                    break
            word = src[i:j]
            has_deva = any("ऀ" <= ch <= "ॿ" for ch in word)
            has_latin = any(ch.isascii() and ch.isalpha() for ch in word)
            if has_deva and has_latin:                 # §10 #7/#8: anti-spoofing
                raise SanskritaError(line,
                                     f"मिश्रलिपि-नाम '{word}' — एकस्यामेव लिप्यां लिखतु",
                                     f"mixed-script name '{word}' — use one script only "
                                     f"(Devanagari or roman, not both in one name)")
            word = ALIASES.get(word, word)             # roman alias -> canonical
            toks.append(("KW" if word in KEYWORDS else "ID", word, line))
            i = j; continue
        raise SanskritaError(line, f"अज्ञातं चिह्नम् '{c}'", f"unknown character '{c}'")
    toks.append(("EOF", None, line))
    return toks

# ---------------------------------------------------------------- parser

class Parser:
    def __init__(self, toks):
        self.t = toks
        self.p = 0

    def peek(self, k=0):
        return self.t[min(self.p + k, len(self.t) - 1)]

    def advance(self):
        tok = self.t[self.p]
        self.p += 1
        return tok

    def at(self, ty, val=None):
        tok = self.peek()
        return tok[0] == ty and (val is None or tok[1] == val)

    def expect_op(self, val, sa, en):
        if not self.at("OP", val):
            raise SanskritaError(self.peek()[2], sa, en)
        return self.advance()

    def expect_id(self, sa, en):
        tok = self.advance()
        if tok[0] != "ID":
            raise SanskritaError(tok[2], sa, en)
        return tok

    def end_stmt(self):
        if self.at("END"):
            self.advance()
        elif self.at("OP", "{"):
            raise SanskritaError(self.peek()[2],
                                 "'{' प्राप्तम् — किं 'यदि' (if) 'यावत्' (while) वा अभिप्रेतम्?",
                                 "found '{' — did you mean यदि/yadi (if) or यावत्/yaavat (while)?")
        else:
            raise SanskritaError(self.peek()[2],
                                 "वाक्यान्ते दण्डः '।' अपेक्षितः",
                                 "expected danda '।' (or '|') at end of statement")

    # ---- statements

    def program(self):
        stmts = []
        while not self.at("EOF"):
            stmts.append(self.statement())
        return stmts

    def statement(self):
        tok = self.peek()
        if tok[0] == "KW":
            if tok[1] in ("मानय", "ध्रुव"):
                kind = "let" if tok[1] == "मानय" else "const"
                self.advance()
                name_tok = self.expect_id("नाम अपेक्षितम्", "expected a name")
                typename = None                        # optional : प्रकारः
                if self.at("OP", ":"):
                    self.advance()
                    t = self.advance()
                    if t[0] != "ID" or t[1] not in TYPE_NAMES:
                        got = t[1] if t[0] in ("ID", "KW") else ""
                        raise SanskritaError(
                            t[2],
                            f"अज्ञातः प्रकारः '{got}' — प्रकाराः: पूर्णाङ्कः, दशमांशः, वाक्यम्, सत्यासत्यम्, सूची, कोशः",
                            f"unknown type '{got}' — types are: पूर्णाङ्कः (int), दशमांशः (decimal), "
                            f"वाक्यम् (text), सत्यासत्यम् (boolean), सूची (list), कोशः (map)")
                    typename = t[1]
                self.expect_op("=", "'=' अपेक्षितम्", "expected '='")
                expr = self.expression()
                self.end_stmt()
                return (kind, name_tok[1], expr, name_tok[2], typename)
            if tok[1] == "विरम":
                self.advance(); self.end_stmt()
                return ("break", tok[2])
            if tok[1] == "अनुवर्त":
                self.advance(); self.end_stmt()
                return ("continue", tok[2])
            if tok[1] == "यदि":
                return self.if_stmt()
            if tok[1] == "यावत्":
                self.advance()
                self.expect_op("(", "'(' अपेक्षितम्", "expected '(' after यावत्")
                cond = self.expression()
                self.expect_op(")", "')' अपेक्षितम्", "expected ')'")
                body = self.block()
                return ("while", cond, body, tok[2])
            if tok[1] == "विधि":
                return self.func_def()
            if tok[1] == "फलम्":
                self.advance()
                expr = None if self.at("END") else self.expression()
                self.end_stmt()
                return ("return", expr, tok[2])
            if tok[1] == "प्रत्येकम्":
                self.advance()
                var_tok = self.expect_id("चरनाम अपेक्षितम्", "expected a loop variable name")
                if not self.at("KW", "इति"):
                    raise SanskritaError(self.peek()[2], "'इति' अपेक्षितम्",
                                         "expected 'इति' (iti) after the loop variable")
                self.advance()
                it = self.expression()
                body = self.block()
                return ("foreach", var_tok[1], it, body, tok[2])
            if tok[1] == "वर्गः":
                return self.class_def()
            if tok[1] == "प्रयत":
                self.advance()
                try_body = self.block()
                if not self.at("KW", "दोषे"):
                    raise SanskritaError(self.peek()[2], "'दोषे' अपेक्षितम्",
                                         "expected 'दोषे' (catch) after प्रयत block")
                self.advance()
                self.expect_op("(", "'(' अपेक्षितम्", "expected '('")
                err_tok = self.expect_id("दोषनाम अपेक्षितम्", "expected an error variable name")
                self.expect_op(")", "')' अपेक्षितम्", "expected ')'")
                catch_body = self.block()
                return ("try", try_body, err_tok[1], catch_body, tok[2])
            if tok[1] == "आनय":
                self.advance()
                mod_tok = self.advance()
                if mod_tok[0] != "STR":
                    raise SanskritaError(mod_tok[2],
                                         'आनय-अनन्तरं "python:नाम" इति वाक्यम् अपेक्षितम्',
                                         'expected a module string after आनय, e.g. आनय "python:math" इति गणितम्।')
                if not self.at("KW", "इति"):
                    raise SanskritaError(self.peek()[2], "'इति' अपेक्षितम्",
                                         "expected 'इति' (iti) and a name after the module string")
                self.advance()
                alias_tok = self.expect_id("नाम अपेक्षितम्", "expected a name")
                self.end_stmt()
                return ("import", mod_tok[1], alias_tok[1], tok[2])
            raise SanskritaError(tok[2], f"अनपेक्षितः शब्दः '{tok[1]}'",
                                 f"unexpected keyword '{tok[1]}' here")
        # mistyped declaration keyword?  e.g.  मनय क = ५।
        if (tok[0] == "ID" and self.peek(1)[0] == "ID"
                and self.peek(2)[0] == "OP" and self.peek(2)[1] == "="):
            sa_h, en_h = hint_for(tok[1])
            raise SanskritaError(tok[2], f"अज्ञातः आदेशः '{tok[1]}'{sa_h}",
                                 f"unknown command '{tok[1]}'{en_h}")
        # expression statement / assignment
        expr = self.expression()
        if self.at("OP", "="):
            eq_line = self.advance()[2]
            if expr[0] not in ("var", "index", "attr"):
                raise SanskritaError(eq_line, "एतस्मै मूल्यं दातुं न शक्यम्",
                                     "cannot assign to this expression")
            value = self.expression()
            self.end_stmt()
            return ("assign", expr, value, eq_line)
        self.end_stmt()
        return ("expr", expr, tok[2])

    def func_def(self):
        tok = self.advance()                           # विधि
        name_tok = self.expect_id("विधिनाम अपेक्षितम्", "expected a function name")
        self.expect_op("(", "'(' अपेक्षितम्", "expected '('")
        params = []
        if not self.at("OP", ")"):
            params.append(self.param())
            while self.at("OP", ","):
                self.advance()
                params.append(self.param())
        self.expect_op(")", "')' अपेक्षितम्", "expected ')'")
        body = self.block()
        return ("func", name_tok[1], params, body, tok[2])

    def param(self):
        first = self.expect_id("मापदण्डनाम अपेक्षितम्", "expected a parameter name")
        if self.at("ID"):                              # kāraka-labeled parameter
            if first[1] not in KARAKAS:
                raise SanskritaError(first[2],
                                     f"'{first[1]}' कारकं न — कारकाणि: कर्ता, कर्म, करण, सम्प्रदान, अपादान, अधिकरण",
                                     f"'{first[1]}' is not a kāraka — roles are: कर्ता (agent), कर्म (object), "
                                     f"करण (instrument), सम्प्रदान (recipient), अपादान (source), अधिकरण (location)")
            name_tok = self.advance()
            return (first[1], name_tok[1])
        return (None, first[1])

    def class_def(self):
        tok = self.advance()                           # वर्गः
        name_tok = self.expect_id("वर्गनाम अपेक्षितम्", "expected a class name")
        self.expect_op("{", "'{' अपेक्षितम्", "expected '{'")
        methods = []
        while not self.at("OP", "}"):
            if not self.at("KW", "विधि"):
                raise SanskritaError(self.peek()[2],
                                     "वर्गे केवलं 'विधि' लेख्याः",
                                     "only 'विधि' (methods) are allowed inside a वर्गः")
            methods.append(self.func_def())
        self.advance()
        return ("class", name_tok[1], methods, tok[2])

    def if_stmt(self):
        tok = self.advance()                           # यदि
        self.expect_op("(", "'(' अपेक्षितम्", "expected '(' after यदि")
        cond = self.expression()
        self.expect_op(")", "')' अपेक्षितम्", "expected ')'")
        branches = [(cond, self.block())]
        else_body = None
        while True:
            if self.at("KW", "अथ"):                    # अथ यदि  (else if)
                self.advance()
                if not self.at("KW", "यदि"):
                    raise SanskritaError(self.peek()[2],
                                         "'अथ' अनन्तरं 'यदि' अपेक्षितम्",
                                         "expected 'यदि' after 'अथ'")
                self.advance()
                self.expect_op("(", "'(' अपेक्षितम्", "expected '('")
                c = self.expression()
                self.expect_op(")", "')' अपेक्षितम्", "expected ')'")
                branches.append((c, self.block()))
            elif self.at("KW", "अन्यथा"):
                self.advance()
                else_body = self.block()
                break
            else:
                break
        return ("if", branches, else_body, tok[2])

    def block(self):
        self.expect_op("{", "'{' अपेक्षितम्", "expected '{'")
        stmts = []
        while not self.at("OP", "}"):
            if self.at("EOF"):
                raise SanskritaError(self.peek()[2], "'}' अपेक्षितम्", "expected '}'")
            stmts.append(self.statement())
        self.advance()
        return stmts

    # ---- expressions (precedence climbing)

    def expression(self):
        return self.or_expr()

    def or_expr(self):
        left = self.and_expr()
        while self.at("KW", "वा"):
            line = self.advance()[2]
            left = ("bin", "वा", left, self.and_expr(), line)
        return left

    def and_expr(self):
        left = self.not_expr()
        while self.at("KW", "च"):
            line = self.advance()[2]
            left = ("bin", "च", left, self.not_expr(), line)
        return left

    def not_expr(self):
        if self.at("KW", "न"):
            line = self.advance()[2]
            return ("un", "न", self.not_expr(), line)
        return self.comparison()

    def comparison(self):
        left = self.additive()
        if self.at("OP") and self.peek()[1] in ("==", "!=", "<", ">", "<=", ">="):
            op, line = self.peek()[1], self.peek()[2]
            self.advance()
            left = ("bin", op, left, self.additive(), line)
        return left

    def additive(self):
        left = self.multiplicative()
        while self.at("OP") and self.peek()[1] in ("+", "-"):
            op, line = self.peek()[1], self.peek()[2]
            self.advance()
            left = ("bin", op, left, self.multiplicative(), line)
        return left

    def multiplicative(self):
        left = self.unary()
        while self.at("OP") and self.peek()[1] in ("*", "/", "%"):
            op, line = self.peek()[1], self.peek()[2]
            self.advance()
            left = ("bin", op, left, self.unary(), line)
        return left

    def unary(self):
        if self.at("OP", "-"):
            line = self.advance()[2]
            return ("un", "-", self.unary(), line)
        return self.postfix(self.primary())

    def postfix(self, e):
        while True:
            if self.at("OP", "("):                     # call
                line = self.advance()[2]
                args = []
                if not self.at("OP", ")"):
                    args.append(self.argument())
                    while self.at("OP", ","):
                        self.advance()
                        args.append(self.argument())
                self.expect_op(")", "')' अपेक्षितम्", "expected ')'")
                e = ("call", e, args, line)
            elif self.at("OP", "["):                   # index
                line = self.advance()[2]
                idx = self.expression()
                self.expect_op("]", "']' अपेक्षितम्", "expected ']'")
                e = ("index", e, idx, line)
            elif self.at("OP", "."):                   # attribute / method
                line = self.advance()[2]
                name_tok = self.expect_id("नाम अपेक्षितम् '.' अनन्तरम्",
                                          "expected a name after '.'")
                e = ("attr", e, name_tok[1], line)
            else:
                break
        return e

    def argument(self):
        if (self.at("ID") and self.peek(1)[0] == "OP" and self.peek(1)[1] == ":"):
            lab_tok = self.advance()
            self.advance()                             # ':'
            if lab_tok[1] not in KARAKAS:
                raise SanskritaError(lab_tok[2],
                                     f"'{lab_tok[1]}' कारकं न — कारकाणि: कर्ता, कर्म, करण, सम्प्रदान, अपादान, अधिकरण",
                                     f"'{lab_tok[1]}' is not a kāraka label — valid: कर्ता, कर्म, करण, "
                                     f"सम्प्रदान, अपादान, अधिकरण")
            return (lab_tok[1], self.expression())
        return (None, self.expression())

    def primary(self):
        tok = self.advance()
        ty, val, line = tok
        if ty == "NUM":
            return ("lit", val, line)
        if ty == "STR":
            return ("lit", val, line)
        if ty == "KW":
            if val == "सत्यम्":
                return ("lit", True, line)
            if val == "असत्यम्":
                return ("lit", False, line)
            if val == "शून्यम्":
                return ("lit", None, line)
            if val == "सृज":                           # सृज वर्गः(...) — object creation
                inner = self.postfix(self.primary())
                return ("new", inner, line)
            raise SanskritaError(line, f"अनपेक्षितः शब्दः '{val}'",
                                 f"unexpected keyword '{val}' in expression")
        if ty == "ID":
            return ("var", val, line)
        if ty == "OP" and val == "(":
            e = self.expression()
            self.expect_op(")", "')' अपेक्षितम्", "expected ')'")
            return e
        if ty == "OP" and val == "[":                  # सूची literal
            items = []
            if not self.at("OP", "]"):
                items.append(self.expression())
                while self.at("OP", ","):
                    self.advance()
                    items.append(self.expression())
            self.expect_op("]", "']' अपेक्षितम्", "expected ']'")
            return ("list", items, line)
        if ty == "OP" and val == "{":                  # कोशः literal
            pairs = []
            if not self.at("OP", "}"):
                pairs.append(self.pair())
                while self.at("OP", ","):
                    self.advance()
                    pairs.append(self.pair())
            self.expect_op("}", "'}' अपेक्षितम्", "expected '}'")
            return ("map", pairs, line)
        raise SanskritaError(line, "अनपेक्षितं चिह्नम्", f"unexpected token '{val}'")

    def pair(self):
        k = self.expression()
        self.expect_op(":", "':' अपेक्षितम् कोशे", "expected ':' between key and value")
        v = self.expression()
        return (k, v)

# ------------------------------------------------------------ environments

class Env:
    def __init__(self, parent=None):
        self.vars = {}
        self.consts = set()
        self.types = {}
        self.parent = parent

    def find(self, name):
        e = self
        while e is not None:
            if name in e.vars:
                return e
            e = e.parent
        return None

    def all_names(self):
        names = set()
        e = self
        while e is not None:
            names |= set(e.vars)
            e = e.parent
        return names

# --------------------------------------------------------- runtime values

class SFunction:
    def __init__(self, name, params, body, closure):
        self.name = name
        self.params = params          # list of (kāraka-label or None, name)
        self.body = body
        self.closure = closure


class BoundMethod:
    def __init__(self, instance, func):
        self.instance = instance
        self.func = func


class Builtin:
    def __init__(self, name, fn):
        self.name = name
        self.fn = fn


class SClass:
    def __init__(self, name, methods):
        self.name = name
        self.methods = methods        # dict name -> SFunction


class SInstance:
    def __init__(self, cls):
        self.cls = cls
        self.fields = {}


class PyVal:
    """Wrapper around a Python object (आनय bridge)."""
    def __init__(self, raw):
        self.raw = raw


class SModule:
    """A user's own .सं file, imported as a namespace (आनय "x.सं" इति नाम।)."""
    def __init__(self, name, env):
        self.name = name
        self.env = env

# ------------------------------------------------------ python bridge glue

def to_py(v):
    if isinstance(v, Decimal):
        return float(v)
    if isinstance(v, list):
        return [to_py(x) for x in v]
    if isinstance(v, dict):
        return {to_py(k): to_py(x) for k, x in v.items()}
    if isinstance(v, PyVal):
        return v.raw
    return v


def to_sk(v):
    if isinstance(v, bool) or v is None or isinstance(v, (int, str)):
        return v
    if isinstance(v, float):
        return Decimal(repr(v))
    if isinstance(v, (list, tuple)):
        return [to_sk(x) for x in v]
    if isinstance(v, dict):
        return {to_sk(k): to_sk(x) for k, x in v.items()}
    return PyVal(v)

# ------------------------------------------- संस्कृतम् linguistics library
# The world's first programming language with Sanskrit linguistics built in:
# akṣara (syllable) analysis, laghu/guru weights, meter detection,
# Devanagari ↔ IAST transliteration, and vowel sandhi.

_CONS = {'क': 'k', 'ख': 'kh', 'ग': 'g', 'घ': 'gh', 'ङ': 'ṅ',
         'च': 'c', 'छ': 'ch', 'ज': 'j', 'झ': 'jh', 'ञ': 'ñ',
         'ट': 'ṭ', 'ठ': 'ṭh', 'ड': 'ḍ', 'ढ': 'ḍh', 'ण': 'ṇ',
         'त': 't', 'थ': 'th', 'द': 'd', 'ध': 'dh', 'न': 'n',
         'प': 'p', 'फ': 'ph', 'ब': 'b', 'भ': 'bh', 'म': 'm',
         'य': 'y', 'र': 'r', 'ल': 'l', 'व': 'v',
         'श': 'ś', 'ष': 'ṣ', 'स': 's', 'ह': 'h', 'ळ': 'ḷ'}
_VOWELS = {'अ': 'a', 'आ': 'ā', 'इ': 'i', 'ई': 'ī', 'उ': 'u', 'ऊ': 'ū',
           'ऋ': 'ṛ', 'ॠ': 'ṝ', 'ए': 'e', 'ऐ': 'ai', 'ओ': 'o', 'औ': 'au'}
_MATRAS = {'ा': 'ā', 'ि': 'i', 'ी': 'ī', 'ु': 'u', 'ू': 'ū',
           'ृ': 'ṛ', 'ॄ': 'ṝ', 'े': 'e', 'ै': 'ai', 'ो': 'o', 'ौ': 'au'}
_SIGNS = {'ं': 'ṃ', 'ः': 'ḥ', 'ँ': 'm̐', 'ऽ': "'"}
_VIRAMA = '्'
_MATRA_OF = {'अ': '', 'आ': 'ा', 'इ': 'ि', 'ई': 'ी', 'उ': 'ु', 'ऊ': 'ू',
             'ऋ': 'ृ', 'ॠ': 'ॄ', 'ए': 'े', 'ऐ': 'ै', 'ओ': 'ो', 'औ': 'ौ'}
_MATRA_TO_VOWEL = {m: v for v, m in _MATRA_OF.items() if m}
_LONG = set('आईऊॠएऐओऔ') | set('ाीूॄेैोौ')

_IAST_CONS = sorted(((v, k) for k, v in _CONS.items()),
                    key=lambda p: -len(p[0]))
_IAST_VOWELS = sorted(((v, k) for k, v in _VOWELS.items()),
                      key=lambda p: -len(p[0]))


def _aksharani(text):
    """Split Devanagari text into akṣaras (syllables)."""
    out = []
    i, n = 0, len(text)
    while i < n:
        c = text[i]
        if c in _CONS:
            cluster = c
            i += 1
            while i + 1 < n and text[i] == _VIRAMA and text[i + 1] in _CONS:
                cluster += text[i] + text[i + 1]
                i += 2
            if i < n and text[i] == _VIRAMA:          # halanta (final ्)
                cluster += text[i]
                i += 1
            elif i < n and text[i] in _MATRAS:
                cluster += text[i]
                i += 1
            while i < n and text[i] in 'ंःँ':
                cluster += text[i]
                i += 1
            out.append(cluster)
        elif c in _VOWELS:
            cluster = c
            i += 1
            while i < n and text[i] in 'ंःँ':
                cluster += text[i]
                i += 1
            out.append(cluster)
        else:                                          # space, danda, other
            i += 1
    # a trailing halanta cluster (e.g. म् in एवम्) belongs to the previous akṣara
    merged = []
    for a in out:
        if merged and a.endswith(_VIRAMA):
            merged[-1] += a
        else:
            merged.append(a)
    return merged


def _matrah(text):
    """Laghu/guru (ल/ग) weight of each akṣara."""
    aks = _aksharani(text)
    weights = []
    for idx, a in enumerate(aks):
        heavy = any(ch in _LONG for ch in a) or 'ं' in a or 'ः' in a
        if not heavy and a.endswith(_VIRAMA):          # halanta closes the syllable
            heavy = True
        if not heavy and idx + 1 < len(aks):
            nxt = aks[idx + 1]
            if _VIRAMA in nxt:                         # next starts with a conjunct
                heavy = True
        weights.append('ग' if heavy else 'ल')
    return weights


def _chandah(text):
    """Identify the meter of a verse by its total akṣara count."""
    count = len(_aksharani(text))
    names = {24: 'गायत्री', 32: 'अनुष्टुभ्', 44: 'त्रिष्टुभ्', 48: 'जगती'}
    name = names.get(count, 'अज्ञातम्')
    return f"{name} (अक्षराणि: {to_dev_digits(str(count))})"


def _romanaya(text):
    """Devanagari → IAST transliteration."""
    out = []
    i, n = 0, len(text)
    while i < n:
        c = text[i]
        if c in _CONS:
            out.append(_CONS[c])
            if i + 1 < n and text[i + 1] == _VIRAMA:
                i += 2
                continue
            if i + 1 < n and text[i + 1] in _MATRAS:
                out.append(_MATRAS[text[i + 1]])
                i += 2
                continue
            out.append('a')
            i += 1
            continue
        if c in _VOWELS:
            out.append(_VOWELS[c]); i += 1; continue
        if c in _SIGNS:
            out.append(_SIGNS[c]); i += 1; continue
        if c in DEV_DIGITS:
            out.append(str(DEV_DIGITS.index(c))); i += 1; continue
        if c == '॥':
            out.append('||'); i += 1; continue
        if c == '।':
            out.append('|'); i += 1; continue
        out.append(c); i += 1
    return ''.join(out)


def _devanagaraya(text):
    """IAST (roman) → Devanagari transliteration."""
    out = []
    i, n = 0, len(text)

    def match(table, pos):
        for roman, dev in table:
            if text.startswith(roman, pos):
                return roman, dev
        return None

    while i < n:
        c = text[i]
        if c == 'ṃ':
            out.append('ं'); i += 1; continue
        if c == 'ḥ':
            out.append('ः'); i += 1; continue
        m = match(_IAST_CONS, i)
        if m:
            roman, dev = m
            out.append(dev)
            i += len(roman)
            v = match(_IAST_VOWELS, i)
            if v:
                vroman, vdev = v
                out.append(_MATRA_OF[vdev])            # '' for अ (inherent)
                i += len(vroman)
            else:
                out.append(_VIRAMA)
            continue
        v = match(_IAST_VOWELS, i)
        if v:
            vroman, vdev = v
            out.append(vdev)
            i += len(vroman)
            continue
        if c == '|':
            out.append('।'); i += 1; continue
        if c.isascii() and c.isdigit():
            out.append(DEV_DIGITS[int(c)]); i += 1; continue
        out.append(c); i += 1
    return ''.join(out)


_VCLASS = {'अ': 'a', 'आ': 'a', 'इ': 'i', 'ई': 'i', 'उ': 'u', 'ऊ': 'u',
           'ऋ': 'r', 'ए': 'e', 'ऐ': 'ai', 'ओ': 'o', 'औ': 'au'}


def _sandhaya(a, b):
    """Join two words with (vowel) sandhi: deva + ālayaḥ = devālayaḥ."""
    a, b = a.strip(), b.strip()
    if not a or not b or b[0] not in _VOWELS:
        return a + b                                   # only vowel sandhi handled
    last = a[-1]
    if last in _MATRA_TO_VOWEL:
        v1, base, ends_c = _MATRA_TO_VOWEL[last], a[:-1], True
    elif last in _CONS:
        v1, base, ends_c = 'अ', a, True
    elif last in _VOWELS:
        v1, base, ends_c = last, a[:-1], False
    else:
        return a + b                                   # ends ्/ं/ः — plain join
    v2, rest = b[0], b[1:]
    c1, c2 = _VCLASS.get(v1), _VCLASS.get(v2)

    def attach(vowel):
        return base + (_MATRA_OF[vowel] if ends_c else vowel)

    if c1 == c2 and c1 in ('a', 'i', 'u'):             # dīrgha: सवर्णदीर्घः
        return attach({'a': 'आ', 'i': 'ई', 'u': 'ऊ'}[c1]) + rest
    if c1 == 'a':                                      # guṇa / vṛddhi
        if c2 == 'i':
            return attach('ए') + rest
        if c2 == 'u':
            return attach('ओ') + rest
        if c2 == 'r':
            return attach('अ') + 'र' + _VIRAMA + rest
        if c2 in ('e', 'ai'):
            return attach('ऐ') + rest
        if c2 in ('o', 'au'):
            return attach('औ') + rest
    if c1 == 'i' and c2 != 'i':                        # yaṇ: इ → य्
        return base + (_VIRAMA if ends_c else '') + 'य' + _MATRA_OF[v2] + rest
    if c1 == 'u' and c2 != 'u':                        # yaṇ: उ → व्
        return base + (_VIRAMA if ends_c else '') + 'व' + _MATRA_OF[v2] + rest
    if c1 in ('e', 'o') and c2 == 'a':                 # avagraha: ते + अपि = तेऽपि
        return a + 'ऽ' + rest
    return a + b


def _make_sanskritam():
    import types
    ns = types.SimpleNamespace()
    for dev_name, fn in (
        ('अक्षराणि', _aksharani),
        ('अक्षरगणना', lambda t: len(_aksharani(t))),
        ('मात्राः', _matrah),
        ('छन्दः', _chandah),
        ('रोमनय', _romanaya),
        ('देवनागरय', _devanagaraya),
        ('संधय', _sandhaya),
    ):
        setattr(ns, dev_name, fn)
    return ns


def _make_ganitam():
    """गणितम् — mathematics, with Sanskrit names (wraps Python's math)."""
    import math
    import types
    ns = types.SimpleNamespace()
    for dev_name, obj in (
        ('वर्गमूलम्', math.sqrt),        # square root
        ('घातः', math.pow),              # power
        ('लघुगणकः', math.log),           # logarithm
        ('ज्या', math.sin),              # sine  (Āryabhaṭa's own term!)
        ('कोज्या', math.cos),            # cosine
        ('स्पर्शज्या', math.tan),        # tangent
        ('तलम्', math.floor),            # floor
        ('उपरितलम्', math.ceil),         # ceiling
        ('निरपेक्षम्', abs),             # absolute value
        ('पाई', math.pi),                # π
        ('ई', math.e),                   # e
    ):
        setattr(ns, dev_name, obj)
    return ns


def _make_yadrcchikam():
    """यादृच्छिकम् — randomness, with Sanskrit names (wraps Python's random)."""
    import random
    import types
    ns = types.SimpleNamespace()
    setattr(ns, 'अन्तरे', random.randint)      # random integer in [a, b]
    setattr(ns, 'वरय', random.choice)          # pick one from a list
    setattr(ns, 'भिन्नम्', random.random)      # random decimal 0–1
    return ns


def _make_kalah():
    """कालः — time and dates, with Sanskrit names (wraps Python's datetime)."""
    import datetime
    import time
    import types
    ns = types.SimpleNamespace()
    setattr(ns, 'अद्य', lambda: to_dev_digits(str(datetime.date.today())))       # today
    setattr(ns, 'संप्रति', lambda: to_dev_digits(datetime.datetime.now().strftime("%H:%M:%S")))  # now
    setattr(ns, 'वर्षः', lambda: datetime.date.today().year)           # year
    setattr(ns, 'क्षणविरामः', time.sleep)                              # pause (seconds)
    return ns


def _make_vakyakarma():
    """वाक्यकर्म — text operations, with Sanskrit names."""
    import types
    ns = types.SimpleNamespace()
    setattr(ns, 'विभज', lambda t, sep: t.split(sep))            # split
    setattr(ns, 'संयोजय', lambda lst, sep: sep.join(lst))       # join
    setattr(ns, 'खोज', lambda t, sub: t.find(sub) + 1)          # find (1-based; ० = absent)
    setattr(ns, 'प्रतिस्थापय', lambda t, a, b: t.replace(a, b))  # replace
    setattr(ns, 'अंश', lambda t, i, j: t[i - 1:j])              # substring (1-based, incl.)
    return ns


NATIVE_MODULES = {
    'संस्कृतम्': _make_sanskritam, 'sanskritam': _make_sanskritam,
    'गणितम्': _make_ganitam, 'ganitam': _make_ganitam,
    'यादृच्छिकम्': _make_yadrcchikam, 'yadrcchikam': _make_yadrcchikam,
    'कालः': _make_kalah, 'kalah': _make_kalah,
    'वाक्यकर्म': _make_vakyakarma, 'vakyakarma': _make_vakyakarma,
}

# ----------------------------------------------------------- interpreter

class Interpreter:
    def __init__(self, roman=False):
        self.roman = roman
        base = Env()                       # builtins live one scope above globals,
        for bname in BUILTIN_NAMES:        # so मानय सङ्ख्या = … may shadow them,
            base.vars[bname] = Builtin(bname, None)   # but सङ्ख्या = … (no मानय)
            base.consts.add(bname)                    # still hits the const guard
        self.base = base
        self.globals = Env(parent=base)
        self.source_dir = None             # folder of the running .सं file
        self.modules = {}                  # cache: path -> SModule

    # ---- display

    def display(self, v, inner=False):
        if isinstance(v, bool):
            return "सत्यम्" if v else "असत्यम्"
        if v is None:
            return "शून्यम्"
        if isinstance(v, (int, Decimal)):
            s = format(v, "f") if isinstance(v, Decimal) else str(v)  # no E-notation
            return s if self.roman else to_dev_digits(s)
        if isinstance(v, str):
            return f'"{v}"' if inner else v
        if isinstance(v, list):
            return "[" + ", ".join(self.display(x, inner=True) for x in v) + "]"
        if isinstance(v, dict):
            return "{" + ", ".join(f"{self.display(k, inner=True)}: {self.display(x, inner=True)}"
                                   for k, x in v.items()) + "}"
        if isinstance(v, SFunction):
            return f"<विधिः {v.name}>"
        if isinstance(v, BoundMethod):
            return f"<विधिः {v.func.name}>"
        if isinstance(v, Builtin):
            return f"<विधिः {v.name}>"
        if isinstance(v, SClass):
            return f"<वर्गः {v.name}>"
        if isinstance(v, SInstance):
            return f"<{v.cls.name} वस्तु>"
        if isinstance(v, PyVal):
            return str(v.raw)
        return str(v)

    # ---- statements

    def run(self, stmts, env=None, repl=False):
        if env is None:
            env = self.globals
        for st in stmts:
            self.exec_stmt(st, env, repl)

    def exec_stmt(self, st, env, repl=False):
        kind = st[0]
        if kind in ("let", "const"):
            _, name, expr, line, typename = st
            if name in env.vars and name in env.consts:
                raise SanskritaError(line, f"'{name}' ध्रुवः — परिवर्तनं न शक्यम्",
                                     f"'{name}' is a constant — cannot redeclare")
            val = self.eval(expr, env)
            if typename:
                self.check_type(name, typename, val, line)
                env.types[name] = typename
            else:
                env.types.pop(name, None)
            env.vars[name] = val
            if kind == "const":
                env.consts.add(name)
        elif kind == "assign":
            _, target, vexpr, line = st
            val = self.eval(vexpr, env)
            self.assign(target, val, env, line)
        elif kind == "break":
            raise BreakSignal(st[1])
        elif kind == "continue":
            raise ContinueSignal(st[1])
        elif kind == "if":
            _, branches, else_body, line = st
            for cond, body in branches:
                if self.truth(cond, env):
                    self.run(body, env)
                    return
            if else_body is not None:
                self.run(else_body, env)
        elif kind == "while":
            _, cond, body, line = st
            while self.truth(cond, env):
                try:
                    self.run(body, env)
                except BreakSignal:
                    break
                except ContinueSignal:
                    continue
        elif kind == "foreach":
            _, var, itexpr, body, line = st
            seq = self.eval(itexpr, env)
            if isinstance(seq, str):
                items = list(seq)
            elif isinstance(seq, list):
                items = list(seq)
            elif isinstance(seq, dict):
                items = list(seq.keys())
            else:
                raise SanskritaError(line,
                                     f"प्रत्येकम् सूचीं कोशं वाक्यं वा अपेक्षते — {type_name_of(seq)} प्राप्तम्",
                                     f"प्रत्येकम् needs a list, map, or text — got {type_name_of(seq)}")
            target = env.find(var) or env
            if var in target.consts:
                raise SanskritaError(line, f"'{var}' ध्रुवः", f"'{var}' is a constant")
            for item in items:
                target.vars[var] = item
                try:
                    self.run(body, env)
                except BreakSignal:
                    break
                except ContinueSignal:
                    continue
        elif kind == "func":
            _, name, params, body, line = st
            env.vars[name] = SFunction(name, params, body, env)
        elif kind == "return":
            _, expr, line = st
            val = None if expr is None else self.eval(expr, env)
            raise ReturnSignal(val, line)
        elif kind == "class":
            _, name, method_defs, line = st
            methods = {}
            for md in method_defs:
                _, mname, mparams, mbody, mline = md
                methods[mname] = SFunction(mname, mparams, mbody, env)
            env.vars[name] = SClass(name, methods)
        elif kind == "try":
            _, try_body, err_name, catch_body, line = st
            try:
                self.run(try_body, env)
            except SanskritaError as err:
                env.vars[err_name] = err.sa
                self.run(catch_body, env)
        elif kind == "import":
            _, mod, alias, line = st
            modname = mod[7:] if mod.startswith("python:") else mod
            if modname in NATIVE_MODULES:              # native संस्कृता modules
                env.vars[alias] = PyVal(NATIVE_MODULES[modname]())
                return
            if modname.endswith((".सं", ".sam")):      # the user's own .सं files!
                base_dir = self.source_dir or os.getcwd()
                path = os.path.abspath(os.path.join(base_dir, modname))
                if path in self.modules:
                    env.vars[alias] = self.modules[path]
                    return
                try:
                    with open(path, encoding="utf-8") as f:
                        msrc = f.read()
                except OSError:
                    raise SanskritaError(line,
                                         f"सञ्चिका '{modname}' न प्राप्ता",
                                         f"file '{modname}' not found (looked in {base_dir})")
                menv = Env(parent=self.base)
                module = SModule(alias, menv)
                self.modules[path] = module
                old_dir = self.source_dir
                self.source_dir = os.path.dirname(path)
                try:
                    self.run(Parser(lex(msrc)).program(), menv)
                finally:
                    self.source_dir = old_dir
                env.vars[alias] = module
                return
            try:
                pymod = importlib.import_module(modname)
            except ImportError:
                raise SanskritaError(line,
                                     f"'{modname}' इति python-कोष्ठकं न प्राप्तम्",
                                     f"python module '{modname}' not found — is it installed?")
            env.vars[alias] = PyVal(pymod)
        elif kind == "expr":
            val = self.eval(st[1], env)
            if repl and val is not None:
                print(self.display(val))

    def assign(self, target, val, env, line):
        kind = target[0]
        if kind == "var":
            name = target[1]
            holder = env.find(name)
            if holder is None:
                raise SanskritaError(line,
                                     f"'{name}' अघोषितम् — प्रथमं 'मानय' प्रयुज्यताम्",
                                     f"'{name}' not declared — declare it first with मानय/maanaya")
            if name in holder.consts:
                raise SanskritaError(line, f"'{name}' ध्रुवः — परिवर्तनं न शक्यम्",
                                     f"'{name}' is a constant — cannot change it")
            if name in holder.types:
                self.check_type(name, holder.types[name], val, line)
            holder.vars[name] = val
        elif kind == "index":
            obj = self.eval(target[1], env)
            idx = self.eval(target[2], env)
            if isinstance(obj, list):
                self.list_index(obj, idx, line)        # validates
                obj[idx - 1] = val
            elif isinstance(obj, dict):
                obj[idx] = val
            else:
                raise SanskritaError(line, f"{type_name_of(obj)} स्थानाङ्कं न गृह्णाति",
                                     f"cannot index-assign into {type_name_of(obj)}")
        elif kind == "attr":
            obj = self.eval(target[1], env)
            name = target[2]
            if isinstance(obj, SInstance):
                obj.fields[name] = val
            elif isinstance(obj, PyVal):
                setattr(obj.raw, name, to_py(val))
            else:
                raise SanskritaError(line, f"{type_name_of(obj)} गुणं न गृह्णाति",
                                     f"cannot set attribute on {type_name_of(obj)}")

    def check_type(self, name, typename, val, line):
        if not type_matches(typename, val):
            raise SanskritaError(
                line,
                f"प्रकारदोषः — '{name}' {typename} इति घोषितम्, {type_name_of(val)} प्राप्तम्",
                f"type error — '{name}' is declared {typename}, got {type_name_of(val)}")

    def truth(self, cond_expr, env):
        v = self.eval(cond_expr, env)
        if not isinstance(v, bool):
            raise SanskritaError(cond_expr[-1],
                                 "अत्र सत्यासत्यम् अपेक्षितम् (सत्यम्/असत्यम्)",
                                 "condition must be सत्यम्/असत्यम् (a boolean)")
        return v

    # ---- expressions

    def eval(self, e, env):
        kind = e[0]
        if kind == "lit":
            return e[1]
        if kind == "var":
            _, name, line = e
            holder = env.find(name)
            if holder is None:
                sa_h, en_h = hint_for(name, env.all_names())
                raise SanskritaError(line, f"अज्ञातं नाम '{name}'{sa_h}",
                                     f"unknown name '{name}'{en_h}")
            return holder.vars[name]
        if kind == "list":
            return [self.eval(x, env) for x in e[1]]
        if kind == "map":
            out = {}
            for k, v in e[1]:
                key = self.eval(k, env)
                if not isinstance(key, (str, int)) or isinstance(key, bool):
                    raise SanskritaError(e[2], "कुञ्जिका वाक्यं पूर्णाङ्कः वा भवेत्",
                                         "map keys must be text or integers")
                out[key] = self.eval(v, env)
            return out
        if kind == "un":
            _, op, sub, line = e
            v = self.eval(sub, env)
            if op == "-":
                if isinstance(v, bool) or not isinstance(v, (int, Decimal)):
                    raise SanskritaError(line, "सङ्ख्या अपेक्षिता", "expected a number")
                return -v
            if op == "न":
                if not isinstance(v, bool):
                    raise SanskritaError(line, "'न' सत्यासत्यम् एव अपेक्षते",
                                         "'न' (not) needs a boolean")
                return not v
        if kind == "bin":
            _, op, le, re_, line = e
            if op in ("च", "वा"):                       # short-circuit logic
                lv = self.eval(le, env)
                if not isinstance(lv, bool):
                    raise SanskritaError(line, "'च'/'वा' सत्यासत्यम् अपेक्षेते",
                                         "'च' (and) / 'वा' (or) need booleans")
                if op == "च" and not lv:
                    return False
                if op == "वा" and lv:
                    return True
                rv = self.eval(re_, env)
                if not isinstance(rv, bool):
                    raise SanskritaError(line, "'च'/'वा' सत्यासत्यम् अपेक्षेते",
                                         "'च' (and) / 'वा' (or) need booleans")
                return rv
            lv, rv = self.eval(le, env), self.eval(re_, env)
            if op == "==":
                return lv == rv
            if op == "!=":
                return lv != rv
            if op in ("<", ">", "<=", ">="):
                return self.compare(op, lv, rv, line)
            return self.arith(op, lv, rv, line)
        if kind == "call":
            _, callee, args, line = e
            fval = self.eval(callee, env)
            vals = [(lab, self.eval(a, env)) for lab, a in args]
            return self.call_value(fval, vals, line)
        if kind == "new":
            _, inner, line = e
            v = self.eval(inner, env)
            if not isinstance(v, SInstance):
                raise SanskritaError(line, "सृज-अनन्तरं वर्गाह्वानम् अपेक्षितम् — सृज वर्गः(...)",
                                     "सृज must be followed by a class call — सृज वर्गः(...)")
            return v
        if kind == "index":
            _, oexpr, iexpr, line = e
            obj = self.eval(oexpr, env)
            idx = self.eval(iexpr, env)
            if isinstance(obj, list):
                return obj[self.list_index(obj, idx, line) - 1]
            if isinstance(obj, str):
                i = self.list_index(obj, idx, line)
                return obj[i - 1]
            if isinstance(obj, dict):
                if idx not in obj:
                    raise SanskritaError(line, f"कुञ्जिका '{idx}' कोशे नास्ति",
                                         f"key '{idx}' not found in the कोशः")
                return obj[idx]
            raise SanskritaError(line, f"{type_name_of(obj)} स्थानाङ्कं न गृह्णाति",
                                 f"cannot index into {type_name_of(obj)}")
        if kind == "attr":
            _, oexpr, name, line = e
            obj = self.eval(oexpr, env)
            if isinstance(obj, SInstance):
                if name in obj.fields:
                    return obj.fields[name]
                if name in obj.cls.methods:
                    return BoundMethod(obj, obj.cls.methods[name])
                raise SanskritaError(line,
                                     f"'{obj.cls.name}' वस्तुनि '{name}' नास्ति",
                                     f"'{obj.cls.name}' object has no '{name}'")
            if isinstance(obj, SModule):
                if name in obj.env.vars:
                    return obj.env.vars[name]
                sa_h, en_h = hint_for(name, obj.env.vars)
                raise SanskritaError(line,
                                     f"'{obj.name}' कोष्ठके '{name}' नास्ति{sa_h}",
                                     f"module '{obj.name}' has no '{name}'{en_h}")
            if isinstance(obj, PyVal):
                try:
                    return to_sk(getattr(obj.raw, name))
                except AttributeError:
                    raise SanskritaError(line, f"python-वस्तुनि '{name}' नास्ति",
                                         f"python object has no attribute '{name}'")
            raise SanskritaError(line, f"{type_name_of(obj)} '.{name}' न जानाति",
                                 f"{type_name_of(obj)} has no attribute '.{name}'")
        raise SanskritaError(e[-1], "आन्तरिकदोषः", "internal error")

    def list_index(self, seq, idx, line):
        if isinstance(idx, bool) or not isinstance(idx, int):
            raise SanskritaError(line, "स्थानाङ्कः पूर्णाङ्कः भवेत्",
                                 "index must be a whole number (पूर्णाङ्कः)")
        if idx < 1 or idx > len(seq):
            raise SanskritaError(line,
                                 f"स्थानाङ्कः {to_dev_digits(str(idx))} सीमाबहिः (१..{to_dev_digits(str(len(seq)))})",
                                 f"index {idx} out of range (1..{len(seq)}) — संस्कृता counts from १ (प्रथमः)")
        return idx

    # ---- calling

    def call_value(self, fval, args, line):
        if isinstance(fval, Builtin):
            return self.call_builtin(fval.name, [v for _, v in args], line)
        if isinstance(fval, BoundMethod):
            return self.call_function(fval.func, args, line, self_val=fval.instance)
        if isinstance(fval, SFunction):
            return self.call_function(fval, args, line)
        if isinstance(fval, SClass):
            inst = SInstance(fval)
            ctor = fval.methods.get("आरम्भ")
            if ctor:
                self.call_function(ctor, args, line, self_val=inst)
            elif args:
                raise SanskritaError(line, f"'{fval.name}' वर्गे 'आरम्भ' विधिः नास्ति",
                                     f"class '{fval.name}' has no 'आरम्भ' (constructor) but got arguments")
            return inst
        if isinstance(fval, PyVal):
            if not callable(fval.raw):
                raise SanskritaError(line, "एतत् python-वस्तु आह्वातुं न शक्यम्",
                                     "this python object is not callable")
            pos = [to_py(v) for lab, v in args if lab is None]
            kw = {lab: to_py(v) for lab, v in args if lab is not None}
            try:
                return to_sk(fval.raw(*pos, **kw))
            except Exception as err:
                raise SanskritaError(line, f"python-दोषः: {err}", f"python error: {err}")
        raise SanskritaError(line, f"{type_name_of(fval)} आह्वातुं न शक्यम्",
                             f"cannot call a {type_name_of(fval)}")

    def call_function(self, fn, args, line, self_val=None):
        local = Env(parent=fn.closure)
        if self_val is not None:
            local.vars["अयम्"] = self_val
        positional = [v for lab, v in args if lab is None]
        labeled = {lab: v for lab, v in args if lab is not None}
        for plab, pname in fn.params:
            if plab is not None and plab in labeled:
                local.vars[pname] = labeled.pop(plab)
            elif positional:
                local.vars[pname] = positional.pop(0)
            else:
                role = f" ({plab})" if plab else ""
                raise SanskritaError(line,
                                     f"'{fn.name}' विधौ '{pname}'{role} इत्यस्य मूल्यं न दत्तम्",
                                     f"function '{fn.name}' missing argument '{pname}'{role}")
        if labeled:
            valid = ", ".join(f"{l}" for l, _ in fn.params if l) or "—"
            raise SanskritaError(line,
                                 f"'{fn.name}' विधौ अज्ञातं कारकम् '{next(iter(labeled))}' — विधेः कारकाणि: {valid}",
                                 f"function '{fn.name}' has no role '{next(iter(labeled))}' — its roles are: {valid}")
        if positional:
            raise SanskritaError(line,
                                 f"'{fn.name}' विधौ अधिकानि मूल्यानि दत्तानि",
                                 f"too many arguments for function '{fn.name}'")
        try:
            self.run(fn.body, local)
        except ReturnSignal as r:
            return r.value
        return None

    # ---- builtins

    def call_builtin(self, name, vals, line):
        if name == "वद":
            print(" ".join(self.display(v) for v in vals))
            return None
        if name == "पृच्छ":
            prompt = self.display(vals[0]) if vals else ""
            try:
                return input(prompt)
            except EOFError:
                return ""
        if name == "वाक्यम्":
            if len(vals) != 1:
                raise SanskritaError(line, "वाक्यम्() एकम् एव गृह्णाति",
                                     "वाक्यम्() takes exactly one value")
            return self.display(vals[0])
        if name == "सङ्ख्या":
            if len(vals) != 1 or not isinstance(vals[0], str):
                raise SanskritaError(line, "सङ्ख्या() वाक्यम् एकं गृह्णाति",
                                     "सङ्ख्या() takes one text value")
            raw = to_ascii_digits(vals[0].strip())
            try:
                return Decimal(raw) if "." in raw else int(raw)
            except Exception:
                raise SanskritaError(line, f"'{vals[0]}' सङ्ख्या न",
                                     f"'{vals[0]}' is not a number")
        if name == "प्रकारः":
            if len(vals) != 1:
                raise SanskritaError(line, "प्रकारः() एकम् एव गृह्णाति",
                                     "प्रकारः() takes exactly one value")
            return type_name_of(vals[0])
        if name == "दैर्घ्यम्":
            if len(vals) != 1 or not isinstance(vals[0], (str, list, dict)):
                raise SanskritaError(line, "दैर्घ्यम्() वाक्यं सूचीं कोशं वा गृह्णाति",
                                     "दैर्घ्यम्() takes one text, list, or map")
            return len(vals[0])
        if name == "योजय":
            if len(vals) != 2 or not isinstance(vals[0], list):
                raise SanskritaError(line, "योजय(सूची, मूल्यम्) — द्वे अपेक्षिते",
                                     "योजय(list, value) — needs a list and a value")
            vals[0].append(vals[1])
            return None
        if name == "अपनय":
            if len(vals) == 2 and isinstance(vals[0], dict):
                if vals[1] not in vals[0]:
                    raise SanskritaError(line, f"कुञ्जिका '{vals[1]}' कोशे नास्ति",
                                         f"key '{vals[1]}' not found in the कोशः")
                return vals[0].pop(vals[1])
            if len(vals) != 2 or not isinstance(vals[0], list):
                raise SanskritaError(line, "अपनय(सूची, स्थानाङ्कः) / अपनय(कोशः, कुञ्जिका)",
                                     "अपनय(list, index) or अपनय(map, key)")
            i = self.list_index(vals[0], vals[1], line)
            return vals[0].pop(i - 1)
        if name == "परिधिः":
            ok = (len(vals) == 2 and all(isinstance(v, int)
                  and not isinstance(v, bool) for v in vals))
            if not ok:
                raise SanskritaError(line, "परिधिः(आदिः, अन्तः) — द्वौ पूर्णाङ्कौ अपेक्षितौ",
                                     "परिधिः(start, end) — needs two whole numbers")
            return list(range(vals[0], vals[1] + 1))   # inclusive, 1-based spirit
        if name == "कुञ्जिकाः":
            if len(vals) != 1 or not isinstance(vals[0], dict):
                raise SanskritaError(line, "कुञ्जिकाः(कोशः) — कोशः अपेक्षितः",
                                     "कुञ्जिकाः(map) — needs a map")
            return list(vals[0].keys())
        if name == "क्रमय":
            if len(vals) != 1 or not isinstance(vals[0], list):
                raise SanskritaError(line, "क्रमय(सूची) — सूची अपेक्षिता",
                                     "क्रमय(list) — needs a list")
            try:
                return sorted(vals[0])
            except TypeError:
                raise SanskritaError(line, "मिश्रप्रकाराः क्रमयितुं न शक्याः",
                                     "cannot sort a list of mixed types")
        raise SanskritaError(line, f"अज्ञातो विधिः '{name}'", f"unknown builtin '{name}'")

    def compare(self, op, a, b, line):
        num = (int, Decimal)
        ok = (isinstance(a, num) and not isinstance(a, bool)
              and isinstance(b, num) and not isinstance(b, bool)) or \
             (isinstance(a, str) and isinstance(b, str))
        if not ok:
            raise SanskritaError(line, "तुलना समानप्रकारयोः एव",
                                 "can only compare two numbers or two texts")
        return {"<": a < b, ">": a > b, "<=": a <= b, ">=": a >= b}[op]

    def arith(self, op, a, b, line):
        if isinstance(a, bool) or isinstance(b, bool):
            raise SanskritaError(line, "सत्यासत्येन गणितं न शक्यम्",
                                 "cannot do arithmetic with सत्यम्/असत्यम्")
        if op == "+" and isinstance(a, str) and isinstance(b, str):
            return a + b
        if op == "+" and isinstance(a, list) and isinstance(b, list):
            return a + b
        if isinstance(a, str) or isinstance(b, str):
            raise SanskritaError(line,
                                 "वाक्यं सङ्ख्या च न मिश्रणीये — 'वाक्यम्()' प्रयुज्यताम्",
                                 "cannot mix text and number — convert with वाक्यम्()/vaakyam()")
        if not isinstance(a, (int, Decimal)) or not isinstance(b, (int, Decimal)):
            raise SanskritaError(line, "सङ्ख्ये अपेक्षिते", "expected numbers")
        if op == "+":
            return a + b
        if op == "-":
            return a - b
        if op == "*":
            return a * b
        if op in ("/", "%") and b == 0:
            raise SanskritaError(line, "शून्येन भागो न शक्यः", "division by zero")
        if op == "/":
            result = Decimal(a) / Decimal(b)
            if result == result.to_integral_value():
                try:
                    result = result.quantize(Decimal(1))
                except Exception:
                    pass
            return result
        if op == "%":
            return a % b

# ------------------------------------------------------------------ main

BANNER = """ॐ  संस्कृता ०.२ — वृक्षः   (Sanskrita v0.2 'Tree')
लिखतु आदेशम्; निर्गमाय Ctrl-D    (type code; Ctrl-D to exit)"""

def run_source(src, interp, repl=False):
    stmts = Parser(lex(src)).program()
    try:
        interp.run(stmts, repl=repl)
    except (BreakSignal, ContinueSignal) as sig:
        raise SanskritaError(sig.line,
                             "'विरम'/'अनुवर्त' चक्रात् बहिः न शक्यम्",
                             "'विरम' (break) / 'अनुवर्त' (continue) only work inside a loop")
    except ReturnSignal as sig:
        raise SanskritaError(sig.line,
                             "'फलम्' विधेः बहिः न शक्यम्",
                             "'फलम्' (return) only works inside a विधि (function)")
    except RecursionError:
        raise SanskritaError(0, "अतिगभीरा पुनरावृत्तिः",
                             "recursion too deep — does your विधि ever stop calling itself?")

def repl(interp):
    print(BANNER)
    buf = ""
    prompt = "॥ "
    while True:
        try:
            line = input(prompt)
        except (EOFError, KeyboardInterrupt):
            print("\nपुनर्मिलामः ।")
            break
        buf += line + "\n"
        if buf.count("{") > buf.count("}"):
            prompt = "… "
            continue
        prompt = "॥ "
        code = buf.strip()
        buf = ""
        if not code:
            continue
        if not code.endswith(("।", "॥", "|", "}")):
            code += "।"
        try:
            run_source(code, interp, repl=True)
        except SanskritaError as err:
            print(err)

def main(argv):
    roman = "--roman" in argv
    convert = "--convert" in argv
    args = [a for a in argv if not a.startswith("--")]
    interp = Interpreter(roman=roman)
    if not args:
        repl(interp)
        return 0
    path = args[0]
    try:
        with open(path, encoding="utf-8") as f:
            src = f.read()
    except OSError as err:
        print(f"सञ्चिका न प्राप्ता / cannot open file: {path}\n{err}", file=sys.stderr)
        return 1
    if convert:                      # rewrite the file in canonical Devanagari
        converted = devanagarify(src)
        if converted != src:
            with open(path, "w", encoding="utf-8") as f:
                f.write(converted)
            print(f"देवनागरीकृतम् ✓ {path}")
        else:
            print(f"पूर्वमेव देवनागरी ✓ {path}")
        return 0
    interp.source_dir = os.path.dirname(os.path.abspath(path))
    try:
        run_source(src, interp)
    except SanskritaError as err:
        print(err, file=sys.stderr)
        return 1
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
