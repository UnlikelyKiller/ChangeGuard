# Specification: Track 18 — Documentation, CI, and Polish

## Overview
Address audit items 9, 10, 16, 21–25: add README, CI pipeline, test dedup, platform process policy, timestamp normalization, documentation files, and test fixtures.

## 1. Root README.md
**Priority: HIGH** — A user-facing project must have one.

### Content
- Project description (from Plan.md Section 1)
- Installation instructions (`cargo install --path .` or binary release)
- Quickstart: `changeguard init` → `doctor` → `scan` → `impact` → `verify` → `ask`
- Command reference summary
- Configuration overview (`.changeguard/config.toml`, `.changeguard/rules.toml`)
- Windows/WSL caveats
- License reference

## 2. CI Pipeline (`.github/workflows/`)
**Priority: HIGH** — Plan Section 11 specifies CI requirements.

### `ci.yml`
- Triggers: push to main, pull requests
- Jobs:
  - `fmt`: `cargo fmt --check`
  - `clippy`: `cargo clippy --all-targets --all-features`
  - `test`: `cargo test -j 1 -- --test-threads=1`
  - `audit`: `cargo audit` (if `cargo-audit` installed)
- Run on: `ubuntu-latest`, `windows-latest`

## 3. Test Helper Deduplication
**Priority: MEDIUM**

### `tests/common/mod.rs`
- Extract `DirGuard` struct into shared module
- Extract `setup_git_repo` helper
- Extract `git_add_and_commit` helper
- Update all 4+ test files that duplicate these helpers to import from `common`

## 4. Platform Process Policy (`src/platform/process_policy.rs`)
**Priority: LOW** — Deferred acceptable per KISS/YAGNI, but the plan specifies it.

### Content
- Define `ProcessPolicy` struct: allowed_commands, denied_commands, default_timeout
- Define `pub fn check_policy(command: &str, policy: &ProcessPolicy) -> Result<()>`
- Currently minimal: just allow everything. The structure exists for future restriction.

## 5. Timestamp Normalization for Tests
**Priority: LOW**

### `src/util/clock.rs`
- Define `pub fn normalize_timestamp(packet: &mut ImpactPacket)` that replaces `timestamp_utc` with a fixed value
- Use in test fixtures for deterministic snapshot comparison
- Alternatively, add a `with_fixed_timestamp()` builder method on `ImpactPacket`

## 6. Documentation Files
**Priority: LOW**

### Create
- `docs/architecture.md` — module boundaries and data flow from Plan.md
- `docs/upgrade-notes.md` — dependency upgrade guidance from breaking.md
- `docs/examples/config.toml` — annotated example configuration
- `docs/examples/rules.toml` — annotated example rules
- `docs/examples/CHANGEGUARD.md` — example project-specific guidance file

## 7. Test Fixtures
**Priority: LOW**

### `tests/fixtures/`
- `sample_rust.rs` — small Rust file with pub/private symbols
- `sample_typescript.ts` — small TS file with exports
- `sample_python.py` — small Python file with classes/defs
- `sample_config.toml` — valid config
- `sample_rules.toml` — valid rules with protected paths

## Verification
- README renders correctly in GitHub markdown
- CI pipeline runs green on push
- All test files compile with shared helpers
- `cargo test -j 1 -- --test-threads=1`