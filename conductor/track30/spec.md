# Track 30: Foundation & Safety Remediation

## 1. Goal
Restore the repository's foundational engineering standards by fixing all broken CI gates (formatting, tests, clippy), addressing critical safety vulnerabilities related to unredacted packet storage, and eliminating production `unwrap()`/`expect()` calls introduced in Phase 2.

## 2. Context
The Phase 2 audit (`docs/audit3.md`) revealed that the repository violates several core principles defined in `docs/Engineering.md`. Before any further feature work is done, the repository must return to a clean, deterministic, and safe state. 

Key violations include:
- `cargo fmt --check` fails.
- `cargo test` fails due to a stale signature in `tests/cli_watch.rs`.
- `cargo clippy` fails with feature flags enabled.
- **Safety Risk**: `src/commands/impact.rs` persists the impact packet to SQLite *before* applying secret redaction. This means raw unredacted data is stored locally, defeating the purpose of the redactor.
- **Panic Risk**: Production code in `src/commands/impact.rs` and `src/commands/hotspots.rs` uses `unwrap()`.

## 3. Specifications

### 3.1. Verification Gates
- Running `cargo fmt --check` must pass.
- Running `cargo test --all-features` must pass.
- Running `cargo clippy --all-targets --all-features -- -D warnings` must pass. 

### 3.2. Secret Safety
- The impact packet must be finalized and redacted *before* it is saved to SQLite. 
- Ensure `redact_secrets()` is called before `persist_packet()` in `src/commands/impact.rs`.

### 3.3. Idiomatic Rust (No Unwraps)
- Remove all instances of `.unwrap()` and `.expect()` in `src/commands/impact.rs` and `src/commands/hotspots.rs`.
- Use idiomatic `miette::Result` or `anyhow::Result` error propagation.
- Handle `partial_cmp` failures gracefully without panicking (e.g., when encountering `NaN` in floating point sorting, fallback to a deterministic stable ordering, such as `Ordering::Equal` or sorting by path).

## 4. Acceptance Criteria
- [ ] `cargo check --all-features` passes.
- [ ] `cargo fmt --check` passes.
- [ ] `cargo test --all-features -j 1 -- --test-threads=1` passes.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] Unredacted packets are no longer stored in the local SQLite database.
- [ ] `rg "\.unwrap\(\)" src/commands/impact.rs src/commands/hotspots.rs src/impact/hotspots.rs` returns 0 results.
