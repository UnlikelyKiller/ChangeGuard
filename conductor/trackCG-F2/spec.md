## Spec: Fix bridge_export_tests.rs Formatting (Track CG-F2)

### Acceptance Criteria

1. **fmt passes**: `cargo fmt --all -- --check` exits 0
2. **Test still passes**: `cargo test test_bridge_export_file_creation` passes
3. **No behavior change**: Only formatting changes; test assertions unchanged
4. **CI gate passes**: `cargo fmt --check`, `cargo clippy`, `cargo test` all pass
