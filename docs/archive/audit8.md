# Audit 8 - Tracks 46-49 Current Work Audit

**Date:** 2026-05-07
**Branch:** `track47-harden-service-detection`
**HEAD:** `ac7bdd1 docs: add tracks 46-49 to address Codex M7-2 findings`
**Scope:** Current Track 46-49 work in `conductor/conductor.md`. Tracks 41-45 are baseline from the previous commit (`852726b`) and were not treated as new current-commit work.

**Resolution Update:** Findings AUD8-1 through AUD8-7 have been remediated in the working tree after this audit was written. Track 46-49 status and plans are now marked completed in `conductor/`, with verification evidence from `cargo fmt`, `cargo clippy`, and `cargo test --workspace`.

---

## Executive Summary

Tracks 46-49 are not implementation-complete in the current working state. HEAD contains only conductor/spec/plan documentation for Tracks 46-49. The worktree has a partial Track 47 implementation in `src/commands/impact.rs` and an untracked `track46.patch`, but the critical Track 46 and Track 48 fixes are not applied to source, Track 47 has only one of its required behavior changes, and Track 49 lacks its specified regression test.

ChangeGuard self-scan reports high impact for `src/commands/impact.rs` and shows the local repository is dirty. The scan also surfaced graceful-degradation warnings from unrelated enrichment providers (`CI Gate`, `Infrastructure`) due to local DB schema mismatch; those did not block the orchestrator and are not counted as Track 46-49 findings.

| Track | Status in current worktree | Audit Result |
|-------|----------------------------|--------------|
| 46 | Planned plus untracked patch only | Fail |
| 47 | Partial implementation only | Fail |
| 48 | Not implemented | Fail |
| 49 | Production line present, required test absent | Partial |

---

## Findings

### AUD8-1 (Critical): Track 46 risk preservation is not applied

**Files:** `src/impact/analysis.rs:650`, `track46.patch`

Track 46 requires `analyze_risk()` to preserve enrichment-elevated risk levels by using the max of the existing packet risk and the rule-derived risk. The checked-out source still unconditionally overwrites `packet.risk_level`:

```rust
packet.risk_level = if total_weight > 50 {
    RiskLevel::High
} else if total_weight > 20 {
    RiskLevel::Medium
} else {
    RiskLevel::Low
};
```

There is an untracked `track46.patch` that contains the intended guard, but it has not been applied. This means service-map, KG, observability, or any other enrichment provider can still elevate the packet and then have that elevation downgraded by final risk analysis.

**Required fix:** Apply the risk-level max/guard in `src/impact/analysis.rs` and add the Track 46 regression test proving a pre-existing `High` packet remains `High` after low rule-based scoring.

---

### AUD8-2 (High): Track 48 service config gates are not wired

**Files:** `src/commands/index.rs:112`, `src/commands/impact.rs:58`

Track 48 requires `coverage.services.enabled` to gate service inference during both indexing and impact enrichment. The implementation still runs both paths unconditionally:

- `src/commands/index.rs:112` always calls `indexer.infer_services()?`.
- `src/commands/impact.rs:58` always builds `ImpactOrchestrator::with_builtins()`, which always registers `ServiceProvider`.

This ignores `ServicesConfig.enabled`, whose default is false. Users who disable service coverage still pay the indexing/enrichment cost and still get service-map-derived packet fields.

**Required fix:** Gate `indexer.infer_services()` on `config.coverage.services.enabled`, and either make the built-in provider registration config-aware or have `ServiceProvider` no-op from config before reading service tables.

---

### AUD8-3 (High): Track 48 threshold config is unused in risk analysis

**File:** `src/impact/analysis.rs:476`

Track 48 requires `coverage.services.cross_service_elevation_threshold` to control when cross-service risk is triggered. The current code still uses hardcoded buckets:

- `count >= 5`
- `count >= 3`
- `count == 2`

The config field exists at `src/config/model.rs:574`, but it does not influence `analyze_risk()`. A user setting `cross_service_elevation_threshold = 5` would still get risk for two affected services.

**Required fix:** Apply the threshold before assigning any cross-service weight, then test that a threshold of 5 suppresses the two-service and four-service risk reasons.

---

### AUD8-4 (High): Track 47 cross-service edge collision fix is not implemented

**File:** `src/coverage/services.rs:228`

Track 47 requires cross-service edge attribution to avoid collisions when multiple services have the same handler/model names. The current implementation still builds a single `HashMap<String, String>` keyed only by bare symbol name:

```rust
symbol_to_service.insert(route.clone(), service.name.clone());
symbol_to_service.insert(model.clone(), service.name.clone());
```

If two services both expose `index`, `health`, `handler`, or `User`, the later insert wins and call edges can be assigned to the wrong service or dropped. The spec explicitly calls this out as a required fix.

**Required fix:** Resolve services by file path/directory containment, qualified name, or a composite key that can be matched consistently from `CallEdge` data. Add the duplicate-name regression test required by Track 47.

---

### AUD8-5 (High): Track 48 topology-only service creation is still blocked

**File:** `src/coverage/services.rs:24`

Track 48 requires explicit `ServiceRoot` topology classifications to create services even when routes and call graph edges are absent. The current early return runs before topology pre-population:

```rust
if routes.is_empty() && call_graph.edges.is_empty() {
    return Vec::new();
}
```

As a result, topology-only service roots are discarded and never reach the service map.

**Required fix:** Include topology in the early-return condition, e.g. only return when routes, call graph edges, and topology classifications are all empty. Add the topology-only service test from the plan.

---

### AUD8-6 (Medium): Track 47 deleted-file handling is only partially done

**File:** `src/commands/impact.rs:152`

The worktree changes `ChangeType::Deleted` to set `old_path = Some(c.path.clone())`, and includes a unit test for packet mapping. That addresses only one of Track 47's three stated requirements. The root-service containment and duplicate-symbol cross-service edge fixes are still absent.

This should not be marked complete as Track 47. At most it is the first subtask.

**Required fix:** Keep the deleted-file change, then complete the remaining service containment and edge-disambiguation work with integration coverage.

---

### AUD8-7 (Medium): Track 49 behavior exists but the required regression test is missing

**File:** `src/impact/packet.rs:807`

`truncate_for_context()` already clears `self.service_map_delta = None`, which satisfies the production behavior requested by Track 49. However, `rg` found no dedicated test for clearing `service_map_delta`; existing truncation tests cover other fields.

Without the confirming test, this can regress silently when truncation phases are rearranged.

**Required fix:** Add `test_truncate_clears_service_map_delta` or equivalent with a populated `ServiceMapDelta` and a small target budget.

---

## Verification Performed

- `git status --short`: dirty worktree with `src/commands/impact.rs`, verify tests, and untracked `track46.patch`.
- `git log --oneline -12`: confirmed Tracks 41-45 landed in previous commit `852726b`; current HEAD only adds Track 46-49 plans/specs.
- `changeguard doctor`: passed.
- `changeguard scan --impact`: completed; reported high impact for `src/commands/impact.rs` and graceful-degradation warnings from unrelated DB-schema provider mismatches.
- Targeted cargo test command was attempted with two test filters and failed because `cargo test` accepts one filter. No full Rust test suite was run for this audit because the source work is incomplete and the requested deliverable is this audit document.

---

## Recommended Remediation Order

1. Apply Track 46 risk preservation and its regression test first. This is the highest behavioral risk because it can suppress enrichment risk signals globally.
2. Complete Track 48 config wiring in index, impact, and risk analysis. Add tests for disabled services, threshold behavior, and topology-only services.
3. Finish Track 47 by disambiguating cross-service edges and adding the duplicate-symbol/root-service tests.
4. Add the Track 49 truncation regression test.
5. Run the CI gate: `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --workspace`.
