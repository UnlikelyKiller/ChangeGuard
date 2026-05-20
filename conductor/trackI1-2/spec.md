# Track I1-2: Self-Federation False Positive Exclusion

**Milestone:** I — Issue Remediation  
**Phase:** 1 — Hotfixes  
**Issue:** CG-3  
**Status:** In Planning

## Objective

`check_cross_repo_impact` in `src/federated/impact.rs` emits a spurious "medium" risk reason (`"Cross-repo impact: Sibling 'ChangeGuard' schema is unavailable or invalid."`) even when no meaningful changes are present. Two independent bugs cause this:

**Bug A — Self-inclusion:** `get_federated_links` returns all rows in `federated_links`, including the current repository if it was ever registered as a sibling. `impact.rs` has no guard against this.

**Bug B — Wrong schema path:** `impact.rs` checks `.changeguard/schema.json` (legacy), but the current location is `.changeguard/state/schema.json`. The scanner (`scanner.rs`) already checks both paths; `impact.rs` must be made consistent.

## Requirements

### Fix A: Self-exclusion in `check_cross_repo_impact`
- Accept the current repository root path (available via `StorageManager` or a new parameter) and skip any sibling whose `sibling_path` canonicalizes to the same directory.
- Comparison must be case-insensitive on Windows (match the existing pattern in `scanner.rs` lines 108–116).

### Fix B: Dual schema path lookup in `check_cross_repo_impact`
- Before declaring a sibling's schema unavailable, check both paths:
  1. `<sibling_path>/.changeguard/state/schema.json` (current)
  2. `<sibling_path>/.changeguard/schema.json` (legacy fallback)
- Only emit the "unavailable" reason if neither path exists or neither parses.

## API Contract

`check_cross_repo_impact(packet, storage)` signature may grow an optional `root: &Utf8Path` parameter, or `root` can be derived from `storage`. No public interface changes outside `federated::impact`.

## Testing Strategy

- Unit test `self_federation_excluded`: register the current temp directory as a sibling in `federated_links`; call `check_cross_repo_impact`; assert `packet.risk_reasons` does not contain any string matching the current dir name.
- Unit test `schema_path_legacy_fallback`: create a sibling dir with schema only at `.changeguard/schema.json` (legacy); assert the sibling is recognized, not flagged as invalid.
- Unit test `schema_path_current_preferred`: create a sibling dir with schema only at `.changeguard/state/schema.json`; assert the sibling is recognized.
- Add to `tests/federated_discovery.rs` if that file exists, otherwise add to `src/federated/impact.rs` `mod tests`.

## Out of Scope

- Do not modify `scanner.rs` — self-exclusion already works there.
- Do not remove the sibling from the database; only skip it during impact analysis.
