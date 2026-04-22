# Core Mandates - ChangeGuard

1. **Security & Safety**: Never commit `.env`, secrets, or API keys. No `.unwrap()` or `expect()` in production code.
2. **TDD (Two-Commit Minimum)**: Commit 1 = failing tests (Red). Commit 2+ = implementation (Green). A task is not "Verified" until behavioral correctness is proven via tests.
3. **CI Pipeline**: Before every commit, all must pass:
   `cargo fmt --all -- --check` ; `cargo clippy --all-targets --all-features -- -D warnings` ; `cargo test`
4. **Output Hygiene**: All temporary output goes to `output/`. Run `rm -rf output/` before completing a work session.