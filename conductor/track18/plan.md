## Plan: Track 18 — Documentation, CI, and Polish

### Phase 1: README and CI
- [ ] Task 18.1: Write `README.md` with project description, installation, quickstart, command reference, config overview, Gemini setup, Windows/WSL caveats, architecture link, contributing, license.
- [ ] Task 18.2: Create `.github/workflows/ci.yml` with fmt, clippy, test, audit jobs on ubuntu-latest and windows-latest. Use `Swatinem/rust-cache@v2` for caching. Install `cargo-audit` in CI step. Set 30-minute job timeout.
- [ ] Task 18.3: Push and verify CI runs green.

### Phase 2: Test Helper Deduplication
- [ ] Task 18.4: Create `tests/common/mod.rs`. Move `DirGuard`, `setup_git_repo`, `git_add_and_commit` into shared module. Also extract git config pattern (`git config user.email` / `git config user.name`).
- [ ] Task 18.5: Update `tests/cli_init.rs`, `tests/cli_reset.rs`, `tests/cli_scan.rs`, `tests/e2e_flow.rs`, `tests/cli_verify.rs` to import from `common`.
- [ ] Task 18.6: Verify all tests pass with `cargo test -j 1 -- --test-threads=1`.

### Phase 3: Platform Process Policy
- [ ] Task 18.7: Create `src/platform/process_policy.rs`. Define `ProcessPolicy` with minimal allow-all default. Define `ProcessPolicyError`. Define `check_policy`.
- [ ] Task 18.8: Register in `src/platform/mod.rs`. Write unit tests (default policy allows everything, deny list blocks matching).

### Phase 4: Timestamp Normalization (Clock Trait)
- [ ] Task 18.9: Create `src/util/mod.rs` and `src/util/clock.rs`. Define `Clock` trait, `SystemClock`, `FixedClock` implementations.
- [ ] Task 18.10: Register `util` in `src/lib.rs`. Wire `SystemClock` as default in `execute_impact`. Use `FixedClock` in test fixtures where timestamp comparison matters.
- [ ] Task 18.11: Write unit tests for `FixedClock` returning deterministic timestamps. Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 5: Documentation Files
- [ ] Task 18.12: Write `docs/architecture.md` describing module boundaries and data flow with ASCII diagram.
- [ ] Task 18.13: Write `docs/upgrade-notes.md` from breaking.md content.
- [ ] Task 18.14: Create `docs/examples/config.toml`, `docs/examples/rules.toml`, `docs/examples/CHANGEGUARD.md`.

### Phase 6: Test Fixtures
- [ ] Task 18.15: Create `tests/fixtures/` with sample source files and config fixtures.
- [ ] Task 18.16: Update relevant tests to use fixture files instead of inline strings where appropriate.

### Phase 7: Final Verification
- [ ] Task 18.17: `cargo clippy --all-targets --all-features` and `cargo fmt --check`.
- [ ] Task 18.18: Full suite `cargo test -j 1 -- --test-threads=1`.
- [ ] Task 18.19: Verify CI passes on both ubuntu-latest and windows-latest.