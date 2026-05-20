# Track I5-3 Plan

## Steps

1. [x] Edit `src/commands/viz.rs` — add `create_dir_all` for output parent path before writing
2. [x] Run CI gate: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --workspace`
