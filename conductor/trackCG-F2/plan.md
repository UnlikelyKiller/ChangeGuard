## Plan: Fix bridge_export_tests.rs Formatting (Track CG-F2)

### Summary
`cargo fmt --check` fails on `tests/bridge_export_tests.rs` due to long lines from the git init fix in the previous session. This causes `changeguard verify` step 1 to fail. Trivial fix.

### Phase 1: Format
- [ ] Task 1.1: Run `cargo fmt tests/bridge_export_tests.rs`
- [ ] Task 1.2: Verify `cargo fmt --all -- --check` exits 0
- [ ] Task 1.3: Verify `changeguard verify` step 1 passes (or at minimum the fmt step passes)

### Phase 2: Gate
- [ ] Task 2.1: `cargo fmt --all -- --check` passes
- [ ] Task 2.2: `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] Task 2.3: `cargo test --workspace` passes (specifically `test_bridge_export_file_creation`)
