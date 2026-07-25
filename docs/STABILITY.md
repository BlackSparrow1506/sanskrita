# STABILITY — versioning and compatibility policy

*स्थिरता-नीतिः*

This document says exactly what we promise not to break, and when. A language
nobody can rely on is a toy; a language that can never change is a fossil. This
is where the line is drawn.

---

## Versioning

संस्कृता uses **MAJOR.MINOR.PATCH**.

**Before v1.0 (where we are now):**

| Bump | What it may contain |
|---|---|
| **PATCH** (0.4.0 → 0.4.1) | bug fixes only. No syntax, semantic, or library changes. Safe to take blind. |
| **MINOR** (0.4.0 → 0.5.0) | new features, **and possibly breaking changes**. Every break is listed under a **Breaking** heading in `CHANGELOG.md` with a migration line. |
| **MAJOR** (0.x → 1.0) | the moment the guarantees below become permanent. |

**After v1.0:**

| Bump | What it may contain |
|---|---|
| **PATCH** | bug fixes |
| **MINOR** | new features, **backwards compatible** — a program that ran on 1.4 runs on 1.5 |
| **MAJOR** | breaking changes, announced at least one MINOR release ahead with a deprecation warning |

**Do not build production software on 0.x** and expect it to keep running
untouched. That is not false modesty; it is what 0.x means.

---

## What "the language" means

These are the things versioned by this policy:

1. **Syntax** — keywords, the danda, blocks, operators, literal forms
2. **Semantics** — evaluation order, exact-decimal arithmetic, 1-based indexing,
   kāraka argument binding, scoping, error behaviour
3. **The standard library** — every module and function documented in
   `README.md` and `docs/AI-SPEC.md`
4. **The CLI** — `sanskrita`, its flags, and its exit codes
5. **The file extension** `.सं` and the roman alias set

These are **not** covered, and may change in any release:

- Anything marked **experimental** — today: `--druta`
- Internal engine structure (`sanskrita.py` internals, `rust-engine/src/*`).
  Do not import `sanskrita.py` as a Python module and rely on its classes.
- Exact error *wording*. Error **behaviour** (that something fails, and roughly
  why) is stable; the sentence is not. Do not parse error text — that is what
  the structured error object exists for.
- Benchmark numbers
- The playground and VS Code extension internals

---

## Frozen already — the decisions we will not revisit

Some things are load-bearing. Changing them would break every program ever
written in संस्कृता, and we are committing now that they will not change:

| Decision | Why it is frozen |
|---|---|
| **Danda `।` ends every statement** | design decision #3; the whole visual identity of the language |
| **1-based indexing** (प्रथमः = १) | design log; matches Sanskrit ordinals. A silent switch to 0-based would corrupt data, not just break builds |
| **Exact `दशमांशः` by default** | the headline promise. Binary floats stay opt-in via `द्रुतदशमांशः` forever |
| **Roman aliases are permanent** | design risk #5 — "never deprecated" was the promise made to every Python programmer trying this |
| **The six kārakas are the only argument roles** | design risk #9; adding a seventh would make Pāṇini's system decorative |
| **NFC normalization is mandatory** | design risk #8; this is a correctness and security property |
| **Devanagari digits ०–९ work everywhere ASCII digits do** | Phase 1 promise |
| **`{ }` blocks; whitespace is never syntax** | design §2b |

If a future change appears to require breaking one of these, the change is
wrong, not the rule.

---

## The two engines are one language

`sanskrita.py` is the **reference implementation**: where the two engines
disagree, the reference is correct by definition and वेगः has a bug.

Both engines must produce **byte-identical output** for every program in
`तुल्यता.py`. This is checked in CI on every push. The current divergence ledger
is in `rust-engine/AUDIT.md` and is **empty**, apart from one deliberate
difference: वेगः rejects `आनय "python:…"` by design (the bridge is a bootstrap,
not a foundation).

A release is not made unless `परीक्षा.py`, `cargo test`, and `तुल्यता.py` are all
green.

---

## Deprecation, when we get there

After v1.0, removing anything follows this sequence:

1. **Announce** in `CHANGELOG.md` under **Deprecated**, with the replacement and
   the release it will be removed in — never sooner than the next MAJOR.
2. **Warn** — the engine prints a bilingual deprecation notice to stderr, which
   `--शान्तम्`/`--quiet` suppresses. The program still runs.
3. **Remove** in the announced MAJOR release, listed under **Removed**.

Nothing is removed without going through all three. Before v1.0, step 1 alone
(the CHANGELOG entry) is what you get.

---

## What has to be true before v1.0

v1.0 is a promise of stability, so it is not a date — it is a checklist. From
`STATUS.md` and the design document's risk register:

- [ ] Errors carry a stack trace and are structured objects, not text
- [ ] Standard library covers ordinary work: dates, regex, CSV, hashing,
      environment, logging
- [ ] Paṇḍit review of the entire keyword set (design risk #12), with the
      grammatical rationale published
- [ ] `docs/GRAMMAR.md` complete and matching both engines
- [ ] `BENCHMARKS.md` fully honouring design §7c — memory and speed against
      Python, Java and Rust, re-measured every release
- [ ] वेगः verified green on Linux, macOS and Windows in CI
- [ ] A second person has shipped something real in संस्कृता and reported back
- [ ] A written maintenance commitment — who answers issues, and how fast

Until every box is ticked, the version number stays below 1.

---

## Reporting a break

If a PATCH or (post-1.0) MINOR release breaks a program that used to work, that
is a bug, not a policy question. Open an issue with the smallest program that
shows it and the two version numbers. It will be treated as a regression.
