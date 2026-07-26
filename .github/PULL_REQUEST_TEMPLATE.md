## What this changes

<!-- One or two sentences. If it fixes an issue, write "Fixes #123". -->

## Why

<!-- The problem this solves. A program that didn't work and now does is the
     most convincing form of "why". -->

## Checklist

- [ ] `python3 परीक्षा.py` ends with `सर्वं शुद्धम् ✓`
- [ ] `cd rust-engine && cargo test` passes
- [ ] `python3 तुल्यता.py` passes — **both engines produce identical output**
- [ ] I added a test that fails without this change
- [ ] Documentation updated (README, `docs/AI-SPEC.md`, `docs/GRAMMAR.md`,
      `docs/शब्दकोशः.md` — whichever apply)
- [ ] `CHANGELOG.md` updated under `[Unreleased]`

## If this touches the language itself

- [ ] Implemented in **both** engines — `sanskrita.py` *and* `rust-engine/`
- [ ] `docs/GRAMMAR.md` updated
- [ ] New Sanskrit terms added to `docs/शब्दकोशः.md` **with their derivation**
- [ ] I checked `docs/STABILITY.md` — if this breaks existing programs, it is
      listed under **Breaking** in the changelog with a migration line

## If this touches only one engine

Say why. The two engines are one language; a divergence is either a deliberate,
documented decision (like वेगः refusing `python:` imports) or a bug.
