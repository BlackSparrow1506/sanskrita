# Contributing to संस्कृता

धन्यवादः for your interest! संस्कृता welcomes contributors — programmers, Sanskrit scholars, teachers, and students alike.

## Ground rules (from our blueprint)

1. **One language, one spec.** Roman aliases are input convenience; canonical source is Devanagari. Never fork the syntax. (We studied how Perl died.)
2. **No big-bang rewrites.** Breaking changes only in small, migratable steps.
3. **Every change passes the suite:** `python3 परीक्षा.py` must print सर्वं शुद्धम् ✓.
4. **The VS Code converter must stay identical** to the engine's — the suite checks this.
5. **Simple spoken-register Sanskrit** for all keywords and messages; scholarly review welcome for new vocabulary.
6. **Errors are bilingual** (Sanskrit + English) with line numbers, and helpful — add did-you-mean hints where possible.

## How to help

- **Programmers:** engine features, stdlib modules, tooling, tests.
- **Sanskrit scholars:** keyword grammar review, sandhi rules, the technical glossary, meter patterns.
- **Teachers:** example programs, tutorials, classroom feedback.
- **Everyone:** try it, break it, report what confused you — beginner confusion reports are gold.

## Project layout

- `sanskrita.py` — the engine (lexer → parser → interpreter)
- `playground.py` — browser playground
- `vscode-sanskrita/` — VS Code extension
- `examples/` — runnable programs (each new feature needs one)
- `docs/` — guides and the AI spec
- `परीक्षा.py` — test suite

जयतु संस्कृतम् ।
