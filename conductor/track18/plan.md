## Plan: Track 18 — Documentation, CI, and Polish

### Phase 1: README and CI
- [ ] Task 18.1: Write `README.md` with project description, installation, quickstart, command reference, config overview, Windows/WSL caveats.
- [ ] Task 18.2: Create `.github/workflows/ci.yml` with fmt, clippy, test, audit jobs on ubuntu-latest and windows-latest.
- [ ] Task 18.3: Push and verify CI runs green.

### Phase 2: Test Helper Deduplication
- [ ] Task 18.4: Create `tests/common/mod.rs`. Move `DirGuard`, `setup_git_repo`, `git_add_and_commit` into shared module.
- [ ] Task 18.5: Update `tests/cli_init.rs`, `tests/cli_reset.rs`, `tests/cli_scan.rs`, `tests/e2e_flow.rs` to import from `common`.
- [ ] Task 18.6: Verify all tests pass with `cargo test -j 1 -- --test-threads=1`.

### Phase 3: Platform Process Policy
- [ ] Task 18.7: Create `src/platform/process_policy.rs`. Define `ProcessPolicy` with minimal allow-all default. Define `check_policy`.
- [ ] Task 18.8: Register in `src/platform/mod.rs`. Write unit tests.

### Phase 4: Timestamp Normalization
- [ ] Task 18.9: Create `src/util/mod.rs` and `src/util/clock.rs`. Implement `normalize_timestamp` or `with_fixed_timestamp` on `ImpactPacket`.
- [ ] Task 18.10: Register `util` in `src/lib.rs`. Use in test fixtures where timestamp comparison matters.

### Phase 5: Documentation Files
- [ ] Task 18.11: Write `docs/architecture.md` describing module boundaries and data flow.
- [ ] Task 18.12: Write `docs/upgrade-notes.md` from breaking.md content.
- [ ] Task 18.13: Create `docs/examples/config.toml`, `docs/examples/rules.toml`, `docs/examples/CHANGEGUARD.md`.

### Phase 6: Test Fixtures
- [ ] Task 18.14: Create `tests/fixtures/` with sample source files and config fixtures.
- [ ] Task 18.15: Update relevant tests to use fixture files instead of inline strings where appropriate.

### Phase 7: Final Verification
- [ ] Task 18.16: `cargo clippy --all-targets --all-features` and `cargo fmt --check`.
- [ ] Task 18.17: Full suite `cargo test -j 1 -- --test-threads=1`.
- [ ] Task 18.18: Verify CI passes on both ubuntu-latest and windows-latest.