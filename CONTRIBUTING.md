# Contributing to संस्कृता

धन्यवादः for your interest! संस्कृता welcomes contributors — programmers, Sanskrit scholars, teachers, and students alike.

**New to GitHub, or only have a phone? Start in [`contributions/`](contributions/)** - pick your folder, no terminal needed.

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

## Before you open a pull request

Four commands. All four run in CI, so it saves a round trip to run them first:

```bash
python3 परीक्षा.py             # conformance: micro-tests, error cases, every example
python3 तुल्यता.py             # differential: both engines, byte-identical output
python3 यादृच्छिकपरीक्षा.py    # property: random programs must agree and not crash
cd rust-engine && cargo test  # वेगः unit tests
```

**If you touch the language itself, you must change both engines.**
`sanskrita.py` is the reference — where they disagree, it is right by definition
and वेगः has a bug (`docs/STABILITY.md`). A change that lands in only one engine
is a divergence, and divergences are how a second implementation quietly stops
being the same language.

Also update, whichever apply: `docs/GRAMMAR.md` (syntax or semantics),
`docs/शब्दकोशः.md` (any new Sanskrit term, **with its derivation**),
`docs/AI-SPEC.md`, the README, and `CHANGELOG.md` under `[Unreleased]`.

## Project layout

| Path | What it is |
|---|---|
| `sanskrita.py` | **the reference engine** — lexer → parser → प्राक्परीक्षा → interpreter. This file defines the language. |
| `rust-engine/` | **वेगः** — the native engine. Must match the reference byte for byte. |
| `द्रुतम्.py` | experimental संस्कृता → C compiled mode (`--druta`), subset only |
| `playground.py` | browser playground |
| `vscode-sanskrita/` | VS Code extension |
| `examples/` | runnable programs — **every new feature needs one** |
| `docs/` | tutorial, AI spec, grammar, stability policy, glossary, Hindi README |
| `परीक्षा.py` | conformance suite |
| `तुल्यता.py` | differential harness (the two-engine contract) |
| `यादृच्छिकपरीक्षा.py` | property-based random testing |
| `मापनम्.py` | benchmarks — regenerates `BENCHMARKS.md` (design §7c) |
| `STATUS.md` | promise-by-promise audit against the design document |

## Adding a standard-library function

1. Name it from a **root**, not a transliteration. Write the derivation down.
2. Implement it in `sanskrita.py`. If it touches numbers, it must be a `RawFn` —
   otherwise exact decimals silently become floats at the module boundary, a bug
   we have already had once.
3. Implement it in `rust-engine/src/stdlib.rs`, and add it to the method tables
   if it reads naturally as `मूल्यम्.नाम()`.
4. Add: a conformance case (`परीक्षा.py`), a differential case (`तुल्यता.py`),
   a Rust unit test, a line in `docs/शब्दकोशः.md`, and an example if it opens up
   something new.

जयतु संस्कृतम् ।
