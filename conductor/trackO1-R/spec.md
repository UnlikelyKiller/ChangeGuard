# Track O1-R Specification: Milestone O Remediation

## Overview

This specification addresses all Critical, High, Medium, and Low severity findings from the GPT-5.4 Codex cross-model review of Milestone O (Intent & Provenance). The review identified correctness bugs, lifecycle bypass risks, schema migration gaps, and missing test coverage. All findings are addressed before Milestone O can be considered complete.

---

## Findings Addressed & Technical Design

### 1. Phantom Committed Records on Aborted Commits (Critical)

**Finding:** Both `silently_record_ledger` and the TUI accept path in `src/commands/hook_commit_msg.rs` write a `COMMITTED` ledger entry *before* Git has created the actual commit object. If the commit is later aborted (e.g., empty commit, rejected by another hook, post-commit hook failure, user `git reset`), ChangeGuard retains a "committed" provenance record for a commit that never existed.

**Remediation:**

The `commit-msg` hook must use a **deferred two-phase write**:

1. **In the hook** (`commit-msg`): Write a `PENDING` ledger entry only. Store the `tx_id` in a temporary sidecar file at `.changeguard/state/pending_hook_tx` alongside the commit message hash.
2. **In a new `post-commit` hook** (`changeguard internal hook-post-commit`): Read `.changeguard/state/pending_hook_tx`, verify the commit hash matches expectations, then promote the entry to `COMMITTED` via `TransactionManager::commit_change`. On any failure, delete the sidecar file and roll back via `rollback_change`.
3. **In `src/commands/init.rs`**: Generate both `.git/hooks/commit-msg` and `.git/hooks/post-commit` hooks.
4. **Sidecar format**: `{ "tx_id": "...", "commit_msg_hash": "<sha256 of msg file>" }` written as JSON to `.changeguard/state/pending_hook_tx`.
5. **Cleanup**: The `post-commit` hook deletes the sidecar file after either promoting or rolling back.

**Key files:** `src/commands/hook_commit_msg.rs`, `src/commands/hook_post_commit.rs` (new), `src/commands/init.rs`, `src/cli.rs`

---

### 2. Hook Bypasses Commit Lifecycle (High)

**Finding:** Both `silently_record_ledger` and the TUI accept path call `start_change` followed by raw `update_transaction_status` / `insert_ledger_entry` instead of going through `TransactionManager::commit_change`. This skips: the verification gate, validator execution, and the atomic SQLite transaction wrapper. A crash between the two raw writes leaves a `COMMITTED` transaction with no matching `ledger_entries` row.

**Remediation:**

- Refactor `silently_record_ledger` and the TUI accept path to use `TransactionManager::commit_change` exclusively, just as the `ledger commit` CLI command does.
- Remove the direct calls to `db.update_transaction_status` and `db.insert_ledger_entry` from `hook_commit_msg.rs` entirely.
- Pass the signed fields (`signature`, `public_key`, `risk`, `related_tickets`) through a new optional `CommitRequest` extension or by extending the existing `CommitRequest` struct, so `commit_change` can write them atomically.
- Extend `CommitRequest` in `src/ledger/types.rs` with: `signature: Option<String>`, `public_key: Option<String>`, `risk: Option<String>`, `related_tickets: Option<String>`.
- Update `TransactionManager::commit_change` in `src/ledger/transaction.rs` to persist these new fields when present.

**Key files:** `src/commands/hook_commit_msg.rs`, `src/ledger/transaction.rs`, `src/ledger/types.rs`, `src/ledger/db.rs`

---

### 3. Cozo `ledger_entry` Schema Not Migrated for Existing Repos (High)

**Finding:** `src/state/storage_cozo.rs:156` only creates the `ledger_entry` relation when it does not exist. Existing repos have the old relation (12 columns) without the 4 new columns (`signature`, `public_key`, `risk`, `related_tickets`). Cozo does not support `ALTER TABLE`, so upgraded repos will get write failures or silent drift between SQLite and Cozo.

**Remediation:**

- Implement a **Cozo schema migration helper** in `src/state/storage_cozo.rs`:
  - After the `if !existing.contains` guard, additionally check the column count (or a sentinel stored in a `cozo_meta` relation) to detect old-schema stores.
  - If the old schema is detected: rename the old relation to `ledger_entry_backup`, create the new relation with all 16 columns, and back-fill from the backup using a CozoScript query (padding new columns with `''`). Then drop `ledger_entry_backup`.
- Add a `cozo_meta` relation `{ key: String => value: String }` to track the Cozo schema version. Seed it with `cozo_schema_version = "2"` on first creation and increment on migrations.
- Expose a `migrate_cozo_schema` function callable from `update --migrate --force` as well.

**Key files:** `src/state/storage_cozo.rs`, `src/state/migration/cozo_port.rs`

---

### 4. Cryptographic Signing Silently Degrades (High)

**Finding:** Both `transaction.rs:260` and `hook_commit_msg.rs:205,283` use `.unwrap_or((None, None))` after calling `sign_ledger_entry`. This means key generation failures, IO errors, or corrupt key files produce unsigned entries silently — contradicting the O1-5 spec requirement that every transaction be signed.

**Remediation:**

- In `src/ledger/crypto.rs`, change `sign_ledger_entry` to return `Result<(Option<String>, Option<String>)>` (using `miette::Result`) so callers can decide error policy.
- In `src/ledger/transaction.rs`, propagate signing failures as warnings logged via `tracing::warn!` but continue (signing is best-effort by policy for now — see note below). Include the error in the ledger entry's `outcome_notes` field so auditors can see it.
- In `src/commands/hook_commit_msg.rs`, apply the same pattern: log the warning, continue without blocking the commit.
- Add `[intent]` config key `require_signing = false` to `src/config/model.rs` (default `false`). When `require_signing = true`, propagate signing failure as a hard error that blocks the commit. This gives teams an enforcement escape hatch.
- Update `src/config/defaults.rs` with the new key.

**Key files:** `src/ledger/crypto.rs`, `src/ledger/transaction.rs`, `src/commands/hook_commit_msg.rs`, `src/config/model.rs`, `src/config/defaults.rs`

---

### 5. Hook Discards Commit Message Trailers (Medium)

**Finding:** `hook_commit_msg.rs:137` and `:240` replace the entire commit message file with `WHAT\n\nWHY`, discarding any trailers (`Signed-off-by`, `Co-authored-by`, issue footers) and user-authored commit-template content that appeared in the original file.

**Remediation:**

- Read the original commit message file contents before any rewrite.
- Parse it to separate the body from trailers. Trailers are lines matching the git trailer format: `Token: value` appearing after a blank line following the subject.
- When rewriting, preserve trailers by appending them after the new body: `WHAT\n\nWHY\n\n{original_trailers}`.
- Use a simple line-based trailer extractor (no need for `git interpret-trailers` subprocess) since we only need to round-trip existing lines.

**Key files:** `src/commands/hook_commit_msg.rs`

---

### 6. Multi-File Commits Use Only First Staged Path (Medium)

**Finding:** `get_staged_files` returns all staged paths, but the hook takes only the first element (`staged_files.first()`) as the ledger entity. All subsequent staged files are ignored, misrepresenting multi-file commits in the ledger, pending-conflict checks, search, and audit.

**Remediation:**

- When multiple staged files are present, create a **single ledger entry** using a canonical entity representation:
  - If all staged files share a common directory prefix, use that prefix as the entity (e.g., `src/ledger`).
  - Otherwise, use the first file but append a count annotation in `summary`: `(+N more files)`.
- Store the full file list in the `related_tickets` field as a JSON array string, separated by commas, so auditors can see the full scope. (This field is free-text; no schema change needed.)
- Expose a `StagedFiles` struct in `hook_commit_msg.rs` to encapsulate this logic cleanly.

**Key files:** `src/commands/hook_commit_msg.rs`

---

### 7. Category Inference Defaults to FEATURE (Medium)

**Finding:** `parse_category_from_message` at line 369 defaults to `Category::Feature` for any commit message without a recognized conventional commit prefix, leading to misclassification and wrong verification/risk policies.

**Remediation:**

- Change the fallback to `Category::Chore` instead of `Category::Feature` — this is a safer default as it carries lower risk policy and is correct for unclassified changes.
- Expand prefix detection to cover more patterns: `perf:`, `build:`, `revert:`, `security:`, `breaking:`.
- Map `security:` → `Category::Security` (or nearest available), `revert:` → `Category::Bugfix`, `perf:` → `Category::Refactor`, `build:` → `Category::Infra`, `breaking:` → `Category::Architecture`.
- Log a debug-level message when the fallback fires so it's discoverable during troubleshooting.

**Key files:** `src/commands/hook_commit_msg.rs`

---

### 8. Missing Test Coverage (Low)

**Finding:** No tests exist for `hook_commit_msg`, TUI behavior, signature round-trips, `m33_intent_provenance` migration, or Cozo schema upgrade compatibility.

**Remediation:**

Add the following test modules and cases:

#### `tests/hook_commit_msg.rs` (new)
- `test_trivial_bypass_skips_tui` — commit starting with `chore:` in a CI env bypasses TUI and writes PENDING entry.
- `test_non_interactive_bypasses_tui` — `CHANGEGUARD_NO_TUI=1` set → TUI not launched.
- `test_phantom_record_cleanup_on_abort` — simulate post-commit hook never running; verify sidecar cleanup.
- `test_category_inference_covers_all_prefixes` — unit test `parse_category_from_message` for all known prefixes and fallback.
- `test_multi_file_entity_canonical_path` — verify entity is common prefix when multiple staged files share one.
- `test_trailer_preservation` — verify rewritten commit msg retains `Signed-off-by:` trailer.

#### `tests/ledger_crypto.rs` (new)
- `test_sign_and_verify_roundtrip` — generate key, sign payload, verify passes.
- `test_verify_fails_on_tampered_payload` — change one byte in payload, verify returns `false`.
- `test_sign_returns_error_on_missing_key_dir` — if key dir is not writable, signing returns `Err`.
- `test_require_signing_config_blocks_commit` — when `require_signing = true`, a signing failure returns an error rather than silently degrading.

#### `tests/cozo_schema_migration.rs` (new)
- `test_new_repo_gets_full_schema` — fresh init creates `ledger_entry` with 16 columns.
- `test_old_schema_is_migrated` — create old 12-column relation, call `migrate_cozo_schema`, verify 16 columns and data preserved.
- `test_migration_is_idempotent` — run migration twice; no error, no data loss.

#### `tests/m33_migration.rs` (new)
- `test_m33_adds_columns_to_sqlite` — apply all migrations up to M33, verify `signature` and `public_key` columns exist in `ledger_entries`.
- `test_m33_backfills_existing_rows` — insert row pre-M33, apply M33, verify row survives with `NULL` for new columns.

**Key files:** `tests/hook_commit_msg.rs` (new), `tests/ledger_crypto.rs` (new), `tests/cozo_schema_migration.rs` (new), `tests/m33_migration.rs` (new)

---

## Acceptance Criteria

- [ ] Running `git commit` and then `git reset HEAD~1` leaves no `COMMITTED` entries in the ledger for the reset commit.
- [ ] Running `git commit` successfully results in a `COMMITTED` entry written through `TransactionManager::commit_change`.
- [ ] A repo initialized on schema v1 (pre-O1-5) automatically migrates Cozo `ledger_entry` to v2 without error.
- [ ] A commit message with `Signed-off-by: Alice <alice@example.com>` retains that trailer after hook processing.
- [ ] A commit touching 5 files produces a single ledger entry whose entity is the common prefix or first file with count annotation.
- [ ] `parse_category_from_message("style: fix indentation")` returns `Category::Tooling`, not `Category::Feature`.
- [ ] `sign_ledger_entry` returning `Err` when `require_signing = false` logs a warning and continues. When `require_signing = true`, it propagates the error and blocks.
- [ ] All tests in `tests/hook_commit_msg.rs`, `tests/ledger_crypto.rs`, `tests/cozo_schema_migration.rs`, and `tests/m33_migration.rs` pass.
- [ ] `changeguard verify` passes cleanly (fmt + clippy + all workspace tests).
