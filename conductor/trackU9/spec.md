# Track U9 Spec: Interactive Category Auto-Correction

## Background
Currently, the category flag (`--category`) in `ledger start` rejects any value that doesn't strictly match the enum keys (e.g. `dev` is rejected). This causes unnecessary friction for developers.

## Objective
Implement fuzzy matching and category auto-correction on `ledger start` to suggest closest matching categories and prompt developers interactively.

## Proposed Design
* Use `fuzzy-matcher` or a simple Levenshtein distance check to match inputs (e.g. `dev` matches `TOOLING` or `INFRA`, `bug` matches `BUGFIX`, `doc` matches `DOCS`).
* If an input does not match, display a list of possible categories sorted by similarity, or run a fast interactive select prompt using the `inquire` crate if stdin is a TTY.
