# Specification: Track 18 — Documentation, CI, and Polish

## Overview
Address audit items 9, 10, 16, 21–25: add README, CI pipeline, test dedup, platform process policy, timestamp normalization, documentation files, and test fixtures.

## 1. Root README.md
**Priority: HIGH** — A user-facing project must have one.

### Content
- Project description (from Plan.md Section 1)
- Installation instructions (`cargo install --path .` or binary release)
- Quickstart: `changeguard init` → `doctor` → `scan` → `impact` → `verify` → `ask`
- Command reference summary (all 8 subcommands with one-line descriptions)
- Configuration overview (`.changeguard/config.toml`, `.changeguard/rules.toml`)
- Gemini integration setup (how to configure `gemini` CLI)
- Windows/WSL caveats
- Architecture section with link to `docs/architecture.md`
- Contributing section (TDD workflow, track system, branch naming)
- License reference

## 2. CI Pipeline (`.github/workflows/`)
**Priority: HIGH** — Plan Section 11 specifies CI requirements.

### `ci.yml`
- Triggers: push to main, pull requests
- Jobs:
  - `fmt`: `cargo fmt --check`
  - `clippy`: `cargo clippy --all-targets --all-features`
  - `test`: `cargo test -j 1 -- --test-threads=1`
  - `audit`: `cargo audit` — install via `cargo install cargo-audit` in CI step
- Run on: `ubuntu-latest`, `windows-latest`
- **Caching**: Use `Swatinem/rust-cache@v2` action for cargo artifact caching (significant speedup on both platforms)
- **Timeout**: Set 30-minute timeout per job to prevent hung CI

## 3. Test Helper Deduplication
**Priority: MEDIUM**

### `tests/common/mod.rs`
- Extract `DirGuard` struct into shared module
- Extract `setup_git_repo` helper (creates temp dir, git init, config user.email/name)
- Extract `git_add_and_commit` helper
- **Also extract** the git config setup pattern (`git config user.email` / `git config user.name`) which is duplicated across test files
- Update all 4+ test files that duplicate these helpers to import from `common`

## 4. Platform Process Policy (`src/platform/process_policy.rs`)
**Priority: LOW** — Deferred acceptable per KISS/YAGNI, but the plan specifies it.

### Content
- Define `ProcessPolicy` struct: `allowed_commands: Vec<String>`, `denied_commands: Vec<String>`, `default_timeout_secs: u64`
- Define `pub fn check_policy(command: &str, policy: &ProcessPolicy) -> Result<(), ProcessPolicyError>`
- Default policy: allow everything, deny nothing, 300s timeout
- **Truly minimal**: no enforcement logic beyond the type definition and the `check_policy` function. The policy exists as a seam for future restriction, not as active enforcement.

## 5. Timestamp Normalization for Tests
**Priority: LOW**

### `src/util/clock.rs`
- Define a `Clock` trait:
  ```rust
  pub trait Clock: Send + Sync {
      fn now(&self) -> chrono::DateTime<chrono::Utc>;
  }
  ```
- Implement `SystemClock` (uses `Utc::now()` — production)
- Implement `FixedClock` (returns a fixed timestamp — testing)
- Add `with_timestamp_override()` method to `ImpactPacket` for tests, or accept a `&dyn Clock` in `execute_impact`
- **This is more idiomatic Rust** than a post-hoc `normalize_timestamp` function — it injects the dependency at the right level and avoids mutating already-constructed data

### `src/util/mod.rs`
- Public module root, declares `clock`

## 6. Documentation Files
**Priority: LOW**

### Create
- `docs/architecture.md` — module boundaries and data flow from Plan.md. Include a diagram showing: CLI → Commands → (Git, Index, Impact, Verify, Gemini) → State → Storage
- `docs/upgrade-notes.md` — dependency upgrade guidance from breaking.md
- `docs/examples/config.toml` — annotated example configuration with all fields explained
- `docs/examples/rules.toml` — annotated example rules with protected paths and required_verifications
- `docs/examples/CHANGEGUARD.md` — example project-specific guidance file showing how to write project-level change guidelines

## 7. Test Fixtures
**Priority: LOW**

### `tests/fixtures/`
- `sample_rust.rs` — small Rust file with `pub fn`, `pub struct`, `use` statements
- `sample_typescript.ts` — small TS file with `export function`, `import` statements
- `sample_python.py` — small Python file with `class`, `def`, `import`, `os.environ` usage
- `sample_config.toml` — valid config with all fields
- `sample_rules.toml` — valid rules with protected paths and required_verifications

### Integration
- Update relevant tests to use fixture files via `include_str!` or runtime file reading instead of inline strings where it improves readability

## Verification
- README renders correctly in GitHub markdown
- CI pipeline runs green on push
- All test files compile with shared helpers
- `cargo test -j 1 -- --test-threads=1`