# Implementation Plan: Track 51-1 — Probabilistic Reachability & Dead Code Detection

## Phase 1: Data Model & Config (Red Commit)
- [ ] Task 1.1: Add `DeadCodeConfig` struct to `src/config/model.rs` with all fields, serde defaults, and standalone default functions.
- [ ] Task 1.2: Wire `dead_code: DeadCodeConfig` into the top-level `Config` struct.
- [ ] Task 1.3: Add `ConfidenceFactor` and `DeadCodeFinding` structs to `src/impact/packet.rs` with full `Eq + Ord` derivation.
- [ ] Task 1.4: Add `dead_code_findings: Vec<DeadCodeFinding>` to `ImpactPacket` with `#[serde(default, skip_serializing_if = "Vec::is_empty")]`.
- [ ] Task 1.5: Write test: `DeadCodeConfig::default()` returns `enabled = false` and expected numeric defaults.
- [ ] Task 1.6: Write test: M7-era config (no `[dead_code]` section) deserializes with defaults.
- [ ] Task 1.7: Write test: `DeadCodeFinding` serialization roundtrip with all `ConfidenceFactor` variants populated.
- [ ] Task 1.8: Write test: empty `dead_code_findings` is absent from JSON output.

## Phase 2: Module Restructure & Core Scorer (Red Commit)
- [ ] Task 2.1: Convert `src/impact/analysis.rs` → `src/impact/analysis/mod.rs`, preserving the existing `analyze_risk` function and all unit tests.
- [ ] Task 2.2: Create `src/impact/analysis/dead_code.rs` with the `ConfidenceScorer` struct and constructor.
- [ ] Task 2.3: Implement `reachability_score()` using CozoDB fixed-point reverse reachability from entrypoint nodes.
- [ ] Task 2.4: Implement `git_activity_score()` using `gix` traversal of `HEAD` history, counting file touches within `git_inactivity_days`.
- [ ] Task 2.5: Implement `test_coverage_score()` using `test_mapping` table with fallback to `test_outcome_history`.
- [ ] Task 2.6: Implement `blend()` that normalizes weights and computes final confidence.
- [ ] Task 2.7: Write test: unreachable symbol returns `reachability_score == 1.0` (mock CozoDB in memory).
- [ ] Task 2.8: Write test: reachable symbol returns `reachability_score == 0.0`.
- [ ] Task 2.9: Write test: inactive file returns `git_activity_score > 0.0` (mock git history).
- [ ] Task 2.10: Write test: active file returns `git_activity_score == 0.0`.
- [ ] Task 2.11: Write test: no test mapping returns `test_coverage_score == 1.0` (in-memory SQLite).
- [ ] Task 2.12: Write test: existing test mapping returns `test_coverage_score == 0.0`.
- [ ] Task 2.13: Write test: blend produces expected confidence within `1e-6`.
- [ ] Task 2.14: Write test: entrypoint symbols are skipped (`score_symbol` returns `None`).

## Phase 3: ImpactPacket Determinism (Green Commit)
- [ ] Task 3.1: Add `dead_code_findings.sort_unstable_by(...)` to `ImpactPacket::finalize()` (confidence descending, then path, then symbol name).
- [ ] Task 3.2: Add `self.dead_code_findings.clear()` to `ImpactPacket::truncate_for_context()` Phase 3.
- [ ] Task 3.3: Write test: `finalize()` sorts findings in correct order.
- [ ] Task 3.4: Write test: `truncate_for_context()` clears findings when Phase 3 is reached.
- [ ] Task 3.5: Write test: serialization roundtrip preserves `ConfidenceFactor` enum variants.

## Phase 4: Enrichment & Risk Provider (Green Commit)
- [ ] Task 4.1: Create `src/impact/enrichment/dead_code.rs` implementing `EnrichmentProvider`.
- [ ] Task 4.2: Create `src/impact/providers/dead_code_provider.rs` implementing `RiskProvider` (advisory-only, zero weight).
- [ ] Task 4.3: Register `DeadCodeEnrichment` in `src/impact/enrichment/mod.rs` and wire into `ImpactOrchestrator` execution after `analyze_risk()`.
- [ ] Task 4.4: Register `DeadCodeProvider` in `src/impact/providers/mod.rs` `RiskRegistry::default()`.
- [ ] Task 4.5: Write test: enrichment populates `dead_code_findings` when `dead_code.enabled = true`.
- [ ] Task 4.6: Write test: enrichment is skipped when `dead_code.enabled = false`.
- [ ] Task 4.7: Write test: provider emits advisory risk reason containing "likely dead code".
- [ ] Task 4.8: Write test: provider does not change risk level (weight = 0).
- [ ] Task 4.9: Write test: CozoDB unavailable → enrichment logs warning and continues with git + test signals.

## Phase 5: CLI Integration (Green Commit)
- [ ] Task 5.1: Add `--dead-code` boolean flag to `Impact` variant in `src/cli.rs`.
- [ ] Task 5.2: Wire `--dead-code` flag into `execute_impact()` in `src/commands/impact.rs` (sets `config.dead_code.enabled = true` for this run).
- [ ] Task 5.3: Add `DeadCode` subcommand to `Commands` in `src/cli.rs` with `--threshold <f64>` and `--limit <usize>`.
- [ ] Task 5.4: Create `src/commands/dead_code.rs` and implement `execute_dead_code()` performing a full-repo scan.
- [ ] Task 5.5: Wire `DeadCode` command in `src/commands/mod.rs` and match arm in main CLI dispatch.
- [ ] Task 5.6: Add dead code findings table to `src/output/human.rs` (`print_impact_summary` and new `print_dead_code_summary`).
- [ ] Task 5.7: Write test: `Impact` with `--dead-code` parses and populates packet.
- [ ] Task 5.8: Write test: `DeadCode` command respects `--limit` and `--threshold`.
- [ ] Task 5.9: Write test: human output renders correctly when findings are present and absent.

## Phase 6: Testing & Hardening (Green Commit)
- [ ] Task 6.1: Run `grep -n 'unwrap()\|expect(' src/impact/analysis/dead_code.rs src/impact/enrichment/dead_code.rs src/impact/providers/dead_code_provider.rs src/commands/dead_code.rs` and confirm zero matches.
- [ ] Task 6.2: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Task 6.3: Run full `cargo test`; fix any regressions in `analysis/mod.rs` from the restructure.
- [ ] Task 6.4: Verify CozoDB reachability query performance on a graph with ≥1,000 nodes and ≥5,000 edges.
- [ ] Task 6.5: Verify graceful degradation: rename CozoDB file temporarily, run `impact --dead-code`, ensure warning is emitted and process completes.
- [ ] Task 6.6: Verify config backward compatibility: load an M7-era `config.toml` with the new binary, confirm identical behavior.
- [ ] Task 6.7: Run `changeguard verify` (full verification suite) and confirm no failures.
