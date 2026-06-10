# Track GF11: CI Gates Platform Split

## Objective

Split `src/index/ci_gates.rs` (1,045 lines, ~984 production) into focused per-platform parser modules. The file contains four completely independent CI platform parsers — GitHub Actions, GitLab CI, CircleCI, and Makefiles — that share only the `ParsedCIGate` output type and a batch-insert loop. Each parser is a natural unit of extension and future maintenance.

## Evidence

- 1,045 lines total; `#[cfg(test)]` begins at line 985, so ~984 production lines with only ~60 lines of existing tests
- Function inventory identifies four independent platform parsers:
  - `parse_github_actions` (lines 288–538, ~251 lines): YAML workflow parsing, matrix expansion, job/step enumeration
  - `parse_gitlab_ci` (lines 539–683, ~145 lines): GitLab YAML job/stage parsing
  - `parse_circleci` (lines 684–784, ~101 lines): CircleCI config.yml parsing
  - `parse_makefile` (lines 785–829, ~44 lines) + `extract_makefile_steps` (lines 830–862, ~33 lines): Makefile target parsing
- Shared infrastructure (~227 lines): `CIGateStats`, `CIGateRow`, `CIGateExtractor` (whose `extract()` method spans lines 43–275 and includes discovery, clearing, dispatch, and batch insert), `ParsedCIGate`, `is_ci_config_changed`, `detect_pre_commit_changes`, `is_generated_ci_file`, `makefile_has_ci_targets`, and path-classification helpers (`is_known_ci_path`, `is_root_makefile`, `is_unknown_ci_path`, `is_pre_commit_path`, `is_generated_ci_path`)
- The `CIGateExtractor::extract()` method dispatches to each platform parser based on file path — this dispatch logic stays in the facade
- Test coverage is nearly absent (~60 lines). Moving four parsers without characterization tests violates the GF1 precedent (golden test BEFORE moves), so this track front-loads per-platform characterization tests in Phase 0.

## Scope

Facade pattern: keep `src/index/ci_gates.rs` as the facade file and add a sibling `src/index/ci_gates/` directory (GF8 `dead_code.rs` pattern). `mod github_actions;` declared inside `ci_gates.rs` resolves to `ci_gates/github_actions.rs`. No rename to `ci_gates/mod.rs` at any point.

| Module | Assigned items |
|---|---|
| `ci_gates.rs` (facade) | `CIGateStats`, `CIGateRow`, `CI_GATE_BATCH_SIZE`, `ParsedCIGate`, `CIGateExtractor` struct + `extract()` dispatch method, `is_ci_config_changed`, `detect_pre_commit_changes`, `is_generated_ci_file`, `makefile_has_ci_targets`, path-classification helpers, `mod` declarations |
| `ci_gates/github_actions.rs` | `parse_github_actions` + all private GitHub Actions helpers |
| `ci_gates/gitlab_ci.rs` | `parse_gitlab_ci` + all private GitLab helpers |
| `ci_gates/circleci.rs` | `parse_circleci` + all private CircleCI helpers |
| `ci_gates/makefile.rs` | `parse_makefile`, `extract_makefile_steps` + all private Makefile helpers |

`ParsedCIGate` stays in the facade. Child modules reference it as `super::ParsedCIGate` — Rust gives child modules access to the parent's private items, so no visibility change is strictly required (annotating it `pub(super)` for documentation is acceptable but optional). Each moved parser becomes `pub(super)` so the facade's dispatch can call it.

## Non-Goals

- No logic changes to any parser.
- No new CI platform support.
- No schema changes to `CIGateStats` or `CiConfigChange`.
- No call site migration — public symbols stay at the same import path (`crate::index::ci_gates::*`).
- No decomposition of `CIGateExtractor::extract()` itself in this track — the 232-line method's discovery/persist phases are a candidate for a follow-on, but splitting it concurrently with the parser moves multiplies risk.
- No touching `.changeguard` state files.

## Implementation Notes

- **Characterization first**: before any move, write one golden test per platform — feed a representative config fixture (a real-ish GitHub Actions workflow, GitLab CI file, CircleCI config, and Makefile) through each `parse_*` function and assert the full `Vec<ParsedCIGate>` output. These tests pin behavior across the move and stay as permanent per-module tests afterward.
- Move parsers smallest-first (`makefile` → `circleci` → `gitlab_ci` → `github_actions`) so process problems surface on the cheapest module.
- The `parse_github_actions` function is the largest at ~251 lines and may contain nested helpers; move every private helper it calls in the same step.
- Each platform module duplicates its own `use` imports — no shared import re-exports.
- Do not pre-create empty module files with unused imports; create each file in the same step that moves its code (avoids unused-import churn under `-D warnings`).

## Verification Strategy

Targeted (run after each module move):
- `cargo check --all-targets --all-features`
- `cargo test index::ci_gates`

Final:
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run --lib --bins --workspace`
- `cargo nextest run --test integration`
- `changeguard verify`
- `cargo install --path .`

## Definition of Done

- `src/index/ci_gates.rs` contains shared infrastructure and dispatch only; platform parsing logic is absent.
- Each CI platform has its own module with its parser, private helpers, and its characterization golden test.
- All existing public symbols (`CIGateStats`, `CIGateExtractor`, `is_ci_config_changed`, `detect_pre_commit_changes`, `is_generated_ci_file`, `makefile_has_ci_targets`) remain reachable at their existing import paths.
- Per-platform golden tests written in Phase 0 pass unchanged after the moves.
- Full verification and reinstall pass.
- Ledger transaction committed; `changeguard ledger status --compact` shows `0 pending, 0 unaudited drift`.

## Risks

- Near-zero existing test coverage means a silent behavior change during the move would go unnoticed — this is why the Phase 0 characterization tests are mandatory, not optional.
- The GitHub Actions parser may have interdependent private helpers; map them all before the move to avoid unreachable-helper compile errors.
