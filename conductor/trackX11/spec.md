# Track X11: `verify` Uses `cargo nextest run` Instead of `cargo test`

**Status:** Planned  
**Milestone:** X — Command Surface Correctness  
**Priority:** Low

## Objective

`changeguard verify` on a clean project tree runs `cargo test -j 1 -- --test-threads=1`, which is slow and inconsistent with the project's own `verify.commands` in CLAUDE.md (`cargo nextest run --lib --bins --workspace`). When nextest is installed, verify should prefer it.

## Problem Statement

The verify engine in `src/verify/engine.rs` (or the default verification plan) defaults to `cargo test`. The CLAUDE.md specifies nextest as the canonical test runner. On this machine, `cargo nextest` is installed. Running the slow `cargo test -j 1` form wastes significant time in the pre-commit / manual verification loop.

## Acceptance Criteria

1. `changeguard verify` checks whether `cargo nextest` is available by running `cargo nextest --version` (exit code 0 = available).
2. If nextest is available, the default verification command is `cargo nextest run --lib --bins --workspace`.
3. If nextest is NOT available, fall back to `cargo test --workspace` (not `-j 1 --test-threads=1`).
4. The selected runner is shown in the "Verification Plan" output: `Using nextest: yes/no`.
5. A `verify.prefer_nextest = false` config option allows opting out.

## Key Files

- `src/verify/engine.rs` — default command selection
- `src/commands/verify.rs` — output of verification plan
- `src/config/model.rs` — `VerifyConfig` struct (add `prefer_nextest` field)

## Definition of Done

- `changeguard verify` on a nextest-equipped machine runs `cargo nextest run --lib --bins --workspace`.
- Falls back to `cargo test --workspace` when nextest is absent.
- The plan output shows which runner is selected.
- `cargo nextest run --lib --bins --workspace` itself still passes (no circular irony).
- `cargo clippy --all-targets --all-features -- -D warnings` passes.
