# Track I5-2 Plan

## Steps

1. [x] Edit `src/commands/scan.rs` — load config, build GlobSet from ignore_patterns, filter changes
2. [x] Run CI gate: `cargo fmt --all -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --workspace`
