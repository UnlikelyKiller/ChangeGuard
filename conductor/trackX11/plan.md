# Track X11 Plan: `verify` → nextest

## Phase 1 — Red (Failing Tests)
- [x] 1. Write unit test `test_nextest_probe_returns_true_when_available`: mock `cargo nextest --version` succeeding, assert `probe_nextest() == true`.
- [x] 2. Write unit test `test_verify_plan_uses_nextest_when_available`: given `probe_nextest() == true`, assert the plan's command string contains `nextest`.

## Phase 2 — Implementation
- [x] 3. Add `fn probe_nextest() -> bool` in `src/verify/engine.rs`:
  ```rust
  fn probe_nextest() -> bool {
      std::process::Command::new("cargo")
          .args(["nextest", "--version"])
          .stdout(std::process::Stdio::null())
          .stderr(std::process::Stdio::null())
          .status()
          .map(|s| s.success())
          .unwrap_or(false)
  }
  ```
- [x] 4. In the verify plan builder, replace the default test command:
  ```rust
  let test_cmd = if config.verify.prefer_nextest.unwrap_or(true) && probe_nextest() {
      "cargo nextest run --lib --bins --workspace"
  } else {
      "cargo test --workspace"
  };
  ```
- [x] 5. Add `prefer_nextest: Option<bool>` to `VerifyConfig` in `src/config/model.rs` with `#[serde(default)]` (absent = `true`).
- [x] 6. In the verification plan human output, add: `Using nextest: yes` / `Using nextest: no (not found)`.

## Phase 3 — Green + Cleanup
- [x] 7. Run `changeguard verify`, confirm `cargo nextest run --lib --bins --workspace` appears in plan.
- [x] 8. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [x] 9. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [x] 10. Run `cargo fmt --all -- --check` — clean.
- [x] 11. Update `conductor/conductor.md` status to Completed.
