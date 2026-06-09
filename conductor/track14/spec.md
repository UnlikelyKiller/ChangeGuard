# Specification: Track 14 — Critical Safety Fixes

## Overview
Address the 4 critical and high-severity correctness/security issues identified in the audit: secret redaction, verification planning, silent error suppression, and production `unwrap()`/`expect()`. Also adds a Gemini subprocess timeout — a critical safety gap where `run_query` can hang indefinitely.

## 1. Secret Redaction (`src/impact/redact.rs` and `src/gemini/sanitize.rs`)
**Priority: CRITICAL** — Plan Section 7.2 requires redaction of likely secrets.

### `src/impact/redact.rs`
- Define `pub fn redact_secrets(packet: &mut ImpactPacket) -> Vec<Redaction>` that scans:
  - Changed file paths matching `.env*`, `credentials*`, `*.pem`, `*.key`, `*.p12`, `*.jks`
  - File contents for patterns matching common secret formats:
    - AWS access keys: `AKIA[0-9A-Z]{16}`
    - AWS secret keys: high-entropy strings after `aws_secret_access_key`
    - GitHub tokens: `ghp_[A-Za-z0-9_]{36,}`, `gho_[A-Za-z0-9_]{36,}`, `ghu_[A-Za-z0-9_]{36,}`, `ghs_[A-Za-z0-9_]{36,}`
    - Google/Gemini API keys: `AIza[0-9A-Za-z_\-]{35}`
    - OpenAI keys: `sk-[A-Za-z0-9]{20,}T3BlbkFJ`, `sk-proj-[A-Za-z0-9]{48,}`
    - Private key blocks: `-----BEGIN .*PRIVATE KEY-----`
    - Generic env-var secrets: patterns after `=`, `:`, or `:=` containing 20+ consecutive base64/alpha chars in `.env*` files
  - Replace matched content with `[REDACTED:<reason>]` markers (include reason for auditability)
  - Return `Vec<Redaction>` where `Redaction { path, pattern_name, line_range }` for audit log
  - Use the `regex` crate (already in Cargo.toml)
  - **Entropy check**: For generic high-entropy string detection (env files), compute Shannon entropy on candidate strings. Only flag strings with entropy >= 4.5 bits/char and length >= 20 chars. This reduces false positives from regular config values.

### `src/gemini/sanitize.rs`
- Define `pub fn sanitize_prompt(prompt: &str, max_bytes: usize) -> SanitizeResult` that strips or masks likely secrets from the combined prompt text before piping to Gemini
- Reuse the same regex patterns from `redact.rs` (extract into a shared `SECRET_PATTERNS` lazy static or function)
- Add a size limit: truncate prompts exceeding `max_bytes` (default 256KB)
  - **Truncate at paragraph boundary** (double newline `\n\n`) to avoid cutting mid-sentence which confuses AI models
  - If no paragraph boundary found within the last 10% of the allowed size, truncate at the last newline
  - Append `[TRUNCATED: original was N bytes, showing first M bytes]` annotation
- Return `SanitizeResult { sanitized: String, redactions: Vec<Redaction>, truncated: bool, original_bytes: usize }`

### Integration
- `commands/impact.rs`: call `redact_secrets(&mut packet)` before `write_impact_report` and `save_packet`
- `commands/ask.rs`: call `sanitize_prompt` on the combined prompt before `run_query`
- Add `redact` module to `src/impact/mod.rs`
- Add `sanitize` module to `src/gemini/mod.rs`

## 2. Verification Planning (`src/verify/plan.rs`)
**Priority: CRITICAL** — Phase 11 core requirement; `required_verifications` is defined but never consumed.

### `src/verify/mod.rs`
- Public module root

### `src/verify/plan.rs`
- Define `VerificationStep` struct: `command: String`, `timeout_secs: u64`, `description: String`
- Define `VerificationPlan` struct: `steps: Vec<VerificationStep>`
- Define `pub fn build_plan(packet: &ImpactPacket, rules: &Rules) -> VerificationPlan`
  - Merge global `required_verifications` from rules
  - Merge path-specific `required_verifications` from matching `PathRule` entries (rules whose `glob` matches a changed file in the packet)
  - Deduplicate commands by exact string match
  - Sort deterministically by command string (alphabetical)
- If no rules file exists OR rules have no `required_verifications`, fall back to `VerificationPlan { steps: vec![VerificationStep { command: "cargo test -j 1 -- --test-threads=1".into(), timeout_secs: 300, description: "Default: run project tests".into() }] }`
- Deterministic: same packet + same rules = same plan, always

### Integration
- `commands/verify.rs`: when no `--command` is provided, build a plan from the latest packet + rules instead of defaulting to `cargo test`
- Register `verify` module in `src/lib.rs`

## 3. Silent Error Suppression Fix
**Priority: CRITICAL** — Engineering.md: "invalid config never causes silent fallback without warning."

### `commands/impact.rs` lines 37-39
- Replace `if let Ok(rules)` with explicit match:
  ```rust
  match crate::policy::load::load_rules(&layout) {
      Ok(rules) => {
          if let Err(e) = crate::impact::analysis::analyze_risk(&mut packet, &rules) {
              tracing::warn!("Risk analysis failed: {e}");
              println!("{} Risk analysis failed. Impact report written without risk scoring.", warning_marker());
          }
      }
      Err(e) => {
          tracing::warn!("Failed to load rules: {e}");
          println!("{} Could not load rules. Impact report written without risk scoring.", warning_marker());
      }
  }
  ```

### `commands/impact.rs` lines 50-53
- Replace `if let Ok(storage)` with explicit match:
  ```rust
  let db_path = layout.state_subdir().join("ledger.db");
  match crate::state::storage::StorageManager::init(db_path.as_std_path()) {
      Ok(storage) => {
          if let Err(e) = storage.save_packet(&packet) {
              tracing::warn!("SQLite save failed: {e}");
              println!("{} Impact report saved to disk but SQLite ledger was not updated. The 'ask' command may not find this report.", warning_marker());
          }
      }
      Err(e) => {
          tracing::warn!("SQLite init failed: {e}");
          println!("{} Could not initialize SQLite. Impact report saved to disk but not persisted to database.", warning_marker());
      }
  }
  ```

### `src/gemini/prompt.rs` line 12
- Replace `serde_json::to_string_pretty(packet).unwrap_or_else(|_| "{}".to_string())` with proper error handling:
  ```rust
  let packet_json = serde_json::to_string_pretty(packet).unwrap_or_else(|e| {
      tracing::warn!("Packet serialization failed: {e}");
      format!("{{\"error\": \"serialization failed: {e}\"}}")
  });
  ```

## 4. Production `unwrap()`/`expect()` Fix
**Priority: HIGH** — Engineering.md: "no `unwrap`/`expect` in production paths."

### `src/index/languages/python.rs:38`
- Replace `capture.node.parent().unwrap()` with a conditional check:
  ```rust
  if let Some(parent) = capture.node.parent() {
      // check parent kind
  }
  ```
- If `parent()` returns `None`, treat the symbol as non-underscore (default to public convention).

### `src/commands/impact.rs:88`
- Replace `.expect("Failed to set progress bar style")` with `unwrap_or_else`:
  ```rust
  pb.set_style(ProgressStyle::default_bar()
      .template("...").unwrap_or_else(|_| ProgressStyle::default_bar()));
  ```

### `src/gemini/mod.rs:12-13`
- Replace `.expect("Failed to set spinner style")` with `unwrap_or_else`:
  ```rust
  pb.set_style(ProgressStyle::default_spinner()
      .template("{spinner:.green} {msg}").unwrap_or_else(|_| ProgressStyle::default_spinner()));
  ```

## 5. Gemini Subprocess Timeout
**Priority: CRITICAL** — `run_query` in `gemini/mod.rs` calls `child.wait_with_output()` with no timeout. If the Gemini CLI hangs, ChangeGuard hangs forever. The `wait-timeout` crate is already a dependency.

### `src/gemini/mod.rs`
- Replace `child.wait_with_output()` with a bounded wait using `wait_timeout::ChildExt::wait_timeout`:
  ```rust
  let timeout = Duration::from_secs(gemini_timeout_secs);
  let status = child.wait_timeout(timeout)
      .into_diagnostic()?
      .ok_or_else(|| {
          // Timeout: kill the child
          let _ = child.kill();
          miette::miette!("Gemini command timed out after {}s", timeout.as_secs())
      })?;
  let output = child.wait_with_output().into_diagnostic()?;
  ```
- Read `gemini.timeout_secs` from config (default: 120s if not configured)
- Pass config value through to `run_query` or read it in the ask command and pass as parameter

## Verification
- Unit tests for `redact_secrets` with fixtures containing fake API keys, tokens, `.env` content
- Unit tests for `sanitize_prompt` verifying secrets are stripped and truncation works at paragraph boundaries
- Unit tests for `build_plan` verifying determinism, deduplication, path-rule merging, and fallback behavior
- Integration test `tests/cli_impact.rs` verifying warning output when rules/DB fail
- Unit test for Gemini timeout (mock a long-running process)
- `cargo test -j 1 -- --test-threads=1`