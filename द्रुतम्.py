#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
द्रुतम् — EXPERIMENTAL संस्कृता→C transpiler ("druta" = fast).

Proof-of-concept for the compiled execution mode (blueprint §7b यन्त्रसङ्कलकः):
reuses the SAME lexer/parser as the interpreter, translates a SUBSET to C,
compiles with tcc/gcc/cc, runs natively.

Subset (v0): integers, + - * %, comparisons, च/वा/न, मानय/ध्रुव, यदि/अथ/अन्यथा,
यावत्, विरम/अनुवर्त, विधि/फलम् (int args/returns, recursion), वद.
NOT yet: decimals, strings-as-values, lists, maps, classes, imports, '/' (the
interpreter returns exact decimals for '/'; C would truncate — excluded until
decimals land here).

Usage:  python3 द्रुतम्.py program.सं            # transpile + compile + run
        python3 द्रुतम्.py program.सं --keep-c   # also keep the generated .c
"""

import hashlib
import os
import subprocess
import sys
import tempfile

import sanskrita

# ------------------------------------------------------------ name mangling

def mangle(name):
    if name.isascii() and name.isidentifier():
        return "u_" + name
    return "d_" + "_".join(f"{ord(c):04x}" for c in name)

# ------------------------------------------------------------ C scaffolding

PRELUDE = r"""
#include <stdio.h>
typedef long long ll;
static const char *DEV[10] = {"०","१","२","३","४",
                              "५","६","७","८","९"};
static void vd_ll(ll x) {                 /* print number in Devanagari digits */
    char buf[32]; int i = 0;
    if (x < 0) { fputs("-", stdout); x = -x; }
    if (x == 0) { fputs(DEV[0], stdout); return; }
    while (x > 0) { buf[i++] = (char)(x % 10); x /= 10; }
    while (i > 0) fputs(DEV[(int)buf[--i]], stdout);
}
"""


class Unsupported(Exception):
    pass


class CEmitter:
    def __init__(self):
        self.funcs = []
        self.declared_stack = [set()]

    # ---- expressions -> C strings

    def expr(self, e):
        k = e[0]
        if k == "lit":
            v = e[1]
            if isinstance(v, bool):
                return "1" if v else "0"
            if isinstance(v, int):
                return str(v)
            raise Unsupported(f"साहित्यम् '{v}' उपसमुच्चये नास्ति / literal type not in subset")
        if k == "var":
            return mangle(e[1])
        if k == "un":
            _, op, sub, _ = e
            c = {"-": "-", "न": "!"}[op]
            return f"({c}{self.expr(sub)})"
        if k == "bin":
            _, op, l, r, _ = e
            cop = {"+": "+", "-": "-", "*": "*", "%": "%",
                   "==": "==", "!=": "!=", "<": "<", ">": ">",
                   "<=": "<=", ">=": ">=", "च": "&&", "वा": "||"}.get(op)
            if cop is None:
                raise Unsupported(f"'{op}' उपसमुच्चये नास्ति / operator not in subset (yet)")
            return f"({self.expr(l)} {cop} {self.expr(r)})"
        if k == "call":
            _, callee, args, _ = e
            if callee[0] != "var":
                raise Unsupported("complex call not in subset")
            vals = [self.expr(a) for lab, a in args]
            return f"{mangle(callee[1])}({', '.join(vals)})"
        raise Unsupported(f"'{k}' उपसमुच्चये नास्ति / expression kind not in subset")

    # ---- statements -> C lines

    def stmt(self, st, out, indent):
        pad = "    " * indent
        k = st[0]
        if k in ("let", "const"):
            _, name, expr, line, _ = st
            cn = mangle(name)
            if name in self.declared_stack[-1]:
                out.append(f"{pad}{cn} = {self.expr(expr)};")
            else:
                self.declared_stack[-1].add(name)
                out.append(f"{pad}ll {cn} = {self.expr(expr)};")
        elif k == "assign":
            _, target, vexpr, line = st
            if target[0] != "var":
                raise Unsupported("index/attr assignment not in subset")
            out.append(f"{pad}{mangle(target[1])} = {self.expr(vexpr)};")
        elif k == "if":
            _, branches, else_body, _ = st
            for i, (cond, body) in enumerate(branches):
                kw = "if" if i == 0 else "} else if"
                out.append(f"{pad}{kw} ({self.expr(cond)}) {{")
                for s in body:
                    self.stmt(s, out, indent + 1)
            if else_body is not None:
                out.append(f"{pad}}} else {{")
                for s in else_body:
                    self.stmt(s, out, indent + 1)
            out.append(f"{pad}}}")
        elif k == "while":
            _, cond, body, _ = st
            out.append(f"{pad}while ({self.expr(cond)}) {{")
            for s in body:
                self.stmt(s, out, indent + 1)
            out.append(f"{pad}}}")
        elif k == "break":
            out.append(f"{pad}break;")
        elif k == "continue":
            out.append(f"{pad}continue;")
        elif k == "return":
            _, expr, _ = st
            out.append(f"{pad}return {self.expr(expr) if expr else '0'};")
        elif k == "func":
            _, name, params, body, _ = st
            ps = ", ".join(f"ll {mangle(p)}" for _, p in params)
            self.declared_stack.append({p for _, p in params})
            fb = [f"ll {mangle(name)}({ps}) {{"]
            for s in body:
                self.stmt(s, fb, 1)
            fb.append("    return 0;")
            fb.append("}")
            self.declared_stack.pop()
            self.funcs.append("\n".join(fb))
        elif k == "expr":
            e = st[1]
            if e[0] == "call" and e[1][0] == "var" and e[1][1] == "वद":
                parts = []
                for j, (lab, a) in enumerate(e[2]):
                    if j:
                        parts.append(f'{pad}fputs(" ", stdout);')
                    if a[0] == "lit" and isinstance(a[1], str):
                        s = a[1].replace("\\", "\\\\").replace('"', '\\"')
                        parts.append(f'{pad}fputs("{s}", stdout);')
                    else:
                        parts.append(f"{pad}vd_ll({self.expr(a)});")
                parts.append(f'{pad}fputs("\\n", stdout);')
                out.extend(parts)
            else:
                out.append(f"{pad}(void)({self.expr(e)});")
        else:
            raise Unsupported(f"'{k}' उपसमुच्चये नास्ति / statement kind not in subset")


def transpile(src):
    stmts = sanskrita.Parser(sanskrita.lex(src)).program()
    em = CEmitter()
    main_body = []
    for st in stmts:
        em.stmt(st, main_body, 1)
    funcs = "\n\n".join(em.funcs)
    # forward declarations so functions may call each other in any order
    fwd = []
    for f in em.funcs:
        fwd.append(f.split("{")[0].strip() + ";")
    return (PRELUDE + "\n" + "\n".join(fwd) + "\n\n" + funcs +
            "\n\nint main(void) {\n" + "\n".join(main_body) +
            "\n    return 0;\n}\n")


def find_cc():
    for cc in ("tcc", "gcc", "cc"):
        try:
            subprocess.run([cc, "--version"], capture_output=True, timeout=10)
            return cc
        except (OSError, subprocess.TimeoutExpired):
            continue
    return None


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    keep = "--keep-c" in sys.argv
    if not args:
        print(__doc__)
        return 1
    path = args[0]
    with open(path, encoding="utf-8") as f:
        src = f.read()
    try:
        c_code = transpile(src)
    except Unsupported as err:
        print(f"द्रुतम्: {err}", file=sys.stderr)
        print("(यह प्रयोगः उपसमुच्चयः एव — full language: use `sanskrita` interpreter)",
              file=sys.stderr)
        return 2
    cc = find_cc()
    if cc is None:
        print("No C compiler found (tcc/gcc/cc)", file=sys.stderr)
        return 3
    if keep:
        with open(path + ".c", "w", encoding="utf-8") as f:
            f.write(c_code)
        print(f"(C saved: {path}.c)", file=sys.stderr)
    # cache compiled binary by content hash — instant re-run when unchanged
    cache_dir = os.path.join(tempfile.gettempdir(), "sanskrita-druta-cache")
    os.makedirs(cache_dir, exist_ok=True)
    tag = hashlib.sha256((cc + "\n" + c_code).encode("utf-8")).hexdigest()[:16]
    binfile = os.path.join(cache_dir, tag)
    if not os.path.exists(binfile):
        with tempfile.TemporaryDirectory() as td:
            cfile = os.path.join(td, "out.c")
            with open(cfile, "w", encoding="utf-8") as f:
                f.write(c_code)
            flags = ["-O2"] if cc != "tcc" else []
            tmpbin = binfile + ".tmp"
            r = subprocess.run([cc, *flags, cfile, "-o", tmpbin], capture_output=True)
            if r.returncode != 0:
                print(r.stderr.decode(), file=sys.stderr)
                return 4
            os.replace(tmpbin, binfile)
    return subprocess.run([binfile]).returncode


if __name__ == "__main__":
    sys.exit(main())
