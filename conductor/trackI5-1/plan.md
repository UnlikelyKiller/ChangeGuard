# Track I5-1 Plan

## Steps

1. [x] Edit `src/search/tantivy_engine.rs` — lowercase trigrams in `search_trigrams()` before creating `TermQuery`
2. [x] Run CI gate: `cargo fmt --all -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --workspace`
