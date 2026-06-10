# Track GF5: CLI Command Definition and Dispatch Split

## Objective

Split `src/cli.rs` into navigable command-group argument and dispatch modules while preserving `run_with` as the stable entry point. The user-supplied analysis reports 1224 lines and one large dispatch function dominated by clap command definitions and match arms.

## Evidence

- User analysis ranks `src/cli.rs` as refactor need 7/10 due to navigation and dispatch sprawl.
- `changeguard hotspots trend` includes `src/cli.rs` in the current top hotspots.
- `changeguard hotspots explain src/cli.rs` reports 11 temporal couplings, including `src/commands/mod.rs` at 0.99, `src/commands/reset.rs` at 0.94, and `src/commands/bridge.rs` at 0.87.
- CLI behavior has recent Y/X/Z hardening, so command help, JSON stdout/stderr separation, and aliases are compatibility-critical.

## Scope

Required module boundaries:

- `args`: root clap parser and global options.
- `command_groups`: ledger, index, config, ask, verify, graph/surfaces, bridge/federation, maintenance/update/watch, and diagnostics.
- `dispatch`: small dispatch helpers per command group.
- Keep `run_with` and any public test harness entry points stable.
- Preserve command aliases, defaults, env behavior, help text semantics, exit codes, and stdout/stderr contract.

## Non-Goals

- Do not change command behavior or rename flags.
- Do not rewrite command implementations in `src/commands/*`.
- Do not move every command into a new abstraction if a small dispatch helper is enough.

## Implementation Notes

- Start by moving clap type definitions without changing variants.
- Add help snapshot or smoke tests before moving high-risk groups. Note: dev-dependencies are only `tempfile` and `httpmock` — there is no snapshot crate (`insta`/`trycmd`). Use a unit test that calls clap's `Command::debug_assert()` plus help-text capture through the existing `tests/integration/common` binary-invocation harness.
- Alias inventory to protect with tests (verified 2026-06-09, `src/cli.rs`): arg alias `out` (line 306), `update` command `visible_alias = "upgrade"` (line 319), arg alias `output-dir` (line 394). If more exist at implementation time, enumerate them in Phase 0.
- Keep JSON and human output behavior in command implementations, not dispatch glue.
- Avoid trait-heavy command registries unless there is an existing local pattern.

## Verification Strategy

Targeted:

- `cargo test cli`
- `cargo nextest run --test integration cli_`
- CLI smokes for `--help`, representative subcommand `--help`, `config view --json`, `scan --impact`, `ledger status --compact`, `index --help`, and `verify --dry-run`.

Final:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run --lib --bins --workspace`
- `cargo nextest run --test integration`
- `changeguard verify`
- `cargo install --path .`

## Definition of Done

- `src/cli.rs` is a small root parser plus `run_with` orchestration facade.
- Command-group modules own their argument definitions and dispatch helpers.
- Help output and structured output behavior remain stable.
- Every command group has at least one dispatch-level smoke.
- Final verification and reinstall pass.

## Risks

- Clap derive movement can alter help ordering or defaults.
- Match-arm refactors can accidentally drop flags or change exit behavior.
- Broad command fan-out makes integration tests more important than source inspection.
