# Track I1-2 Plan: Self-Federation False Positive Exclusion

## Phase 1 — Red (Failing Tests)

- [ ] In `src/federated/impact.rs` `mod tests` (or `tests/federated_discovery.rs`):
  - Add `self_federation_excluded`: set up an in-memory SQLite with `federated_links` containing the current tmpdir path; call `check_cross_repo_impact`; assert no "Sibling" risk reason referencing the current dir is present.
  - Add `schema_path_legacy_fallback`: sibling has schema only at `.changeguard/schema.json`; assert it is not reported as invalid.
  - Add `schema_path_current_recognized`: sibling has schema only at `.changeguard/state/schema.json`; assert it is not reported as invalid.
- [ ] Commit: `test(federated): red — self-exclusion and dual schema path lookup`

## Phase 2 — Green (Implementation)

- [ ] **Fix B (schema path):** In `check_cross_repo_impact`, replace the single `schema_path` lookup with a helper that tries current path first, falls back to legacy:
  ```rust
  fn resolve_sibling_schema_path(sibling_path: &str) -> Option<std::path::PathBuf> {
      let base = std::path::Path::new(sibling_path).join(".changeguard");
      let current = base.join("state").join("schema.json");
      let legacy  = base.join("schema.json");
      if current.exists() { Some(current) }
      else if legacy.exists() { Some(legacy) }
      else { None }
  }
  ```
- [ ] **Fix A (self-exclusion):** At the top of the `for (name, path, _) in links` loop, derive the canonical current root (from `storage` or a passed `root` param) and `continue` if `path` resolves to the same directory (case-insensitive on Windows).
- [ ] Run CI gate: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`.
- [ ] Commit: `fix(federated): exclude self from impact check and align schema path lookup (CG-3)`

## Verification

- [ ] Run `changeguard scan --impact` — confirm `riskLevel` is no longer `"medium"` due to self-sibling.
- [ ] Read `.changeguard/reports/latest-impact.json` — confirm no "Sibling 'ChangeGuard' schema is unavailable" entry in `riskReasons`.
