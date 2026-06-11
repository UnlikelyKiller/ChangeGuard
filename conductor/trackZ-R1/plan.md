# Track Z-R1 Plan: Cargo.lock Disambiguation & Schema Hardening

## Phase 1 — Red (Failing Tests)
- [ ] 1. Add `test_cargo_lock_version_disambiguation` in `tests/integration/track_z4_repro.rs`:
  - Create a mock `Cargo.lock` with `regex` 1.0.0 and 2.0.0 (both registry-sourced) plus `consumer` depending on `regex`.
  - Run `index --analyze-graph`.
  - Query CozoDB for `DependsOn` edges where source contains `consumer`.
  - Assert exactly one edge exists and its target contains `regex` with a registry source.
- [ ] 2. Add `test_cargo_lock_git_and_path_deps` in `tests/integration/track_z4_repro.rs`:
  - Create a mock `Cargo.lock` with a git-sourced package and a path-sourced package.
  - Assert nodes exist with `source` metadata present.
- [ ] 3. Confirm both tests fail or are inconclusive under the current code (the first because the scenario is untested; the second because git/path sources are not exercised).

## Phase 2 — Parser Hardening
- [ ] 4. In `src/index/graph_loader.rs`, define:
  ```rust
  #[derive(serde::Deserialize)]
  struct CargoLockFile { package: Vec<CargoLockPackage> }

  #[derive(serde::Deserialize)]
  struct CargoLockPackage {
      name: String,
      version: String,
      source: Option<String>,
      dependencies: Option<Vec<String>>,
  }
  ```
- [ ] 5. Update `phase_cargo_dependencies` to attempt `toml::from_str::<CargoLockFile>` first. On success, iterate typed packages. On failure, fall back to `serde_json::Value` with a `warn!` log.
- [ ] 6. Ensure the `source` field is propagated into node metadata for all packages (including git/path), not just registry packages.

## Phase 3 — Green + Verification
- [ ] 7. Run `cargo nextest run --test integration`.
- [ ] 8. Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] 9. Run `cargo fmt --all -- --check`.
- [ ] 10. Install binary with `cargo install --path .`.
