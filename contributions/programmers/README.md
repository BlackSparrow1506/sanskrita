# contributions/programmers/

Engine ideas, standard-library proposals, tooling, and bug write-ups.

## If you have a computer

Real engine and stdlib code lives in `sanskrita.py` and `rust-engine/`, and
every change has to pass the four checks in the main
[CONTRIBUTING.md](../../CONTRIBUTING.md) (`परीक्षा.py`, `तुल्यता.py`,
`यादृच्छिकपरीक्षा.py`, `cargo test`). If you can run those, skip this folder
and go straight to the real files - that's the fastest path for working code.

## If you only have a phone

You can still contribute real engineering value without running code:

1. Copy [`_template.md`](_template.md).
2. Write up one concrete idea: a missing stdlib function, a bug you can
   describe exactly (what you ran, what you expected, what happened), or a
   small improvement with pseudocode.
3. Submit it as a new file (see the [root README](../README.md) for the
   phone-only steps) named after your topic, e.g. `idea-string-reverse.md`.
4. A programmer with a dev setup will pick it up, implement it against both
   engines, and credit you in the commit.

**Want to try real code from your phone anyway?** GitHub Codespaces
(github.com → the repo → **Code → Codespaces → Create codespace**) gives you
a full VS Code and terminal in the browser - no install. It's usable on a
phone but easier on a tablet. This is optional; the write-up path above is
the one built for phones.

## What a good entry looks like

See `_template.md`. Name your file for the topic, not your name
(`idea-...`, `bug-...`, `tooling-...`).
