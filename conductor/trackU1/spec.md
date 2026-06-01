# Track U1 Spec: Single Integration Test Harness

## Background
Currently, the integration tests in ChangeGuard are written in multiple files under `tests/` (such as `tests/cli_doctor.rs`, `tests/cli_impact.rs`, etc.). In Rust, each of these is compiled as a separate executable. On Windows systems running strict Application Control (WDAC / AppLocker) policies, execution of these dynamically compiled integration binaries is blocked, leading to `os error 4551`.

## Objective
Consolidate the standalone integration test files into a single, unified integration test harness compiled as a single binary (`tests/integration/main.rs`). This dramatically reduces the number of binaries that need to be signed or whitelisted and makes execution under WDAC much cleaner.

## Proposed Design
* Move integration test files from `tests/` into submodules under `tests/integration/`.
* Declare `tests/integration/main.rs` as the entrypoint.
* Declare the target in `Cargo.toml`:
  ```toml
  [[test]]
  name = "integration"
  path = "tests/integration/main.rs"
  ```
* Re-enable full verification commands like `cargo nextest run --workspace` safely.
