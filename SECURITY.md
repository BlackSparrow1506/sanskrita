# Security Policy — सुरक्षानीतिः

## Reporting a vulnerability

**Do not open a public issue for a security problem.**

Email **langotegouri@gmail.com** with the subject line `संस्कृता security:` and
include:

- what you found, and which file or feature it affects
- the smallest `.सं` program (or input) that demonstrates it
- which engine — reference (`sanskrita.py`), वेगः (`rust-engine/`), the
  playground, or the VS Code extension
- the version (`sanskrita --version`) and your OS

You will get an acknowledgement within **7 days** and a status update at least
every **14 days** until the issue is closed. If a fix is warranted we will agree
a disclosure date with you, and credit you in `CHANGELOG.md` unless you ask us
not to.

This is a small volunteer project. That is not an excuse for slow security
handling, but it is the honest reason a 90-day disclosure window is the shape of
what we can promise, rather than a same-week patch guarantee.

## Supported versions

| Version | Supported |
|---|---|
| 0.4.x | ✅ current |
| 0.3.x | security fixes only, until 0.5.0 |
| < 0.3 | ❌ |

Before v1.0 the language is explicitly unstable — see
[docs/STABILITY.md](docs/STABILITY.md).

## What is and is not a vulnerability here

संस्कृता is a programming language. A program written in it can do whatever the
user running it can do: read and write files (`सञ्चिका`), start Python code
(`आनय "python:…"`), sleep, and consume CPU. **That is the intended design, not a
sandbox escape.** Running an untrusted `.सं` file is exactly as dangerous as
running an untrusted Python or shell script, and neither engine claims otherwise.

**In scope** — please report these:

- A crash, hang, or memory-safety failure in the engine on *valid or invalid
  source input* (in वेगः especially: any panic, out-of-bounds, or unsoundness).
- Anything that makes the two engines disagree in a way an attacker could rely
  on — for example source that the reference accepts and वेगः mis-parses.
- Unicode confusables or normalization handling that lets two visually identical
  identifiers mean different things (design risk #7, #8). We reject mixed-script
  identifiers and normalize to NFC precisely to prevent this; a hole in that is a
  real bug.
- Path traversal in `आनय "…सं"` module resolution, or `सञ्चिका` writing outside
  where the caller asked.
- Anything in the browser playground that lets one visitor's code affect another,
  or that escapes the page.
- Supply-chain issues: a tampered release artefact, or an unexpected dependency.
  वेगः deliberately has **zero external crates** — bignums, decimals and NFC are
  hand-written — so a new dependency appearing is itself worth reporting.

**Out of scope** — these are documented behaviour, not bugs:

- A `.सं` program deleting a file, using the network through the Python bridge,
  or looping forever. There is no sandbox and none is claimed.
- `आनय "python:…"` executing arbitrary Python. That is the feature.
- Resource exhaustion by a program the user chose to run.

## Hardening notes for people embedding संस्कृता

If you run *untrusted* संस्कृता source, do not rely on the engine for isolation.
Use the operating system: a container, a separate user, a read-only filesystem,
`ulimit`. Two engine-level limits exist and can be lowered, but they are guards
against accidents, not adversaries:

- recursion is bounded by a **measured** stack budget (`RUN_STACK_BUDGET` in
  `rust-engine/src/main.rs`, `sys.setrecursionlimit` in `sanskrita.py`)
- वेगः rejects `python:` imports entirely, so the native engine has a much
  smaller blast radius than the reference engine

## Our own supply chain

- वेगः: **no external crates**. `cargo tree` should show only `sanskrita`.
- Reference engine: Python standard library only.
- CI pins actions by major version and runs on every push and pull request.
- `main` is protected; changes land through pull requests.

If you ever see a release that contradicts any of the above, treat it as a
compromise and report it.
