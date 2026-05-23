# Track O1-R Plan: Milestone O Remediation

## Phase 1: Critical — Two-Phase Ledger Write (Phantom Records Fix)

- [ ] **Task 1.1: Add `hook-post-commit` CLI command**
  - Register `changeguard internal hook-post-commit` in `src/cli.rs`.
  - Create `src/commands/hook_post_commit.rs` implementing `execute_hook_post_commit()`.
  - Logic: read `.changeguard/state/pending_hook_tx` sidecar; call `TransactionManager::commit_change`; delete sidecar on success or `rollback_change` on failure.

- [ ] **Task 1.2: Write PENDING-only in `commit-msg` hook**
  - In `silently_record_ledger` and the TUI accept path: replace the direct `update_transaction_status` / `insert_ledger_entry` calls with a `start_change` (→ `PENDING`) followed by writing the sidecar file `{ "tx_id": "...", "commit_msg_hash": "<sha256>" }`.
  - Remove all direct raw DB writes from `hook_commit_msg.rs`.

- [ ] **Task 1.3: Install `post-commit` hook in `init`**
  - In `src/commands/init.rs`, generate `.git/hooks/post-commit` alongside the existing `commit-msg` hook.
  - Content: `#!/bin/sh\nchangeguard internal hook-post-commit "$@"`
  - Ensure idempotent append (do not duplicate if already present).

---

## Phase 2: High — Lifecycle Bypass Fix & CommitRequest Extension

- [ ] **Task 2.1: Extend `CommitRequest` with provenance fields**
  - In `src/ledger/types.rs`, add `signature: Option<String>`, `public_key: Option<String>`, `risk: Option<String>`, `related_tickets: Option<String>` to `CommitRequest`.

- [ ] **Task 2.2: Persist new fields in `commit_change`**
  - In `src/ledger/transaction.rs`, update `commit_change` to pass the new fields from `CommitRequest` into the `LedgerEntry` construction.
  - In `src/ledger/db.rs`, update `insert_ledger_entry` to write all 4 new fields (already done for the schema columns; ensure `CommitRequest` path also flows through).

- [ ] **Task 2.3: Update hook to use `commit_change`**
  - In `src/commands/hook_post_commit.rs` (from Task 1.1), build a `CommitRequest` populated with `signature`, `public_key`, `risk`, `related_tickets` from the sidecar's stored intent data.
  - Remove all remaining direct raw DB writes from `hook_commit_msg.rs`.

---

## Phase 3: High — Cozo Schema Migration for Existing Repos

- [ ] **Task 3.1: Add `cozo_meta` version relation**
  - In `src/state/storage_cozo.rs`, create `cozo_meta { key: String => value: String }` on first init.
  - Seed `cozo_schema_version = "1"` for freshly created stores that lack the new columns, and `"2"` for new stores with all 16 columns.

- [ ] **Task 3.2: Implement `migrate_cozo_schema` function**
  - Detect old schema: query `cozo_meta` for `cozo_schema_version`; if missing or `"1"`, migration is needed.
  - Migration steps:
    1. Copy `ledger_entry` data to a temp in-memory structure.
    2. Remove old relation with `:remove ledger_entry`.
    3. Create new relation with 16 columns.
    4. Re-insert all rows, padding new columns with `''`.
    5. Update `cozo_meta` version to `"2"`.
  - Expose `pub fn migrate_cozo_schema(cozo: &CozoStorage) -> Result<()>`.

- [ ] **Task 3.3: Call migration on startup**
  - In `CozoStorage::ensure_schema`, after creating relations, call `migrate_cozo_schema` if schema version is outdated.

---

## Phase 4: High — Signing Error Propagation & `require_signing` Config

- [ ] **Task 4.1: Change `sign_ledger_entry` return type**
  - In `src/ledger/crypto.rs`, change return type from `Option<(Option<String>, Option<String>)>` to `Result<(Option<String>, Option<String>)>`.
  - Return `Err` on IO failure, key decode failure, etc.

- [ ] **Task 4.2: Add `require_signing` config key**
  - In `src/config/model.rs`, add `require_signing: bool` (default `false`) to `IntentConfig`.
  - In `src/config/defaults.rs`, add default `false`.

- [ ] **Task 4.3: Update callers**
  - In `src/ledger/transaction.rs`: match on `sign_ledger_entry` result; if `Err` and `require_signing = true`, propagate; if `Err` and `require_signing = false`, log `tracing::warn!` and record error in `outcome_notes`.
  - In `src/commands/hook_commit_msg.rs` / `hook_post_commit.rs`: same pattern.

---

## Phase 5: Medium — Trailer Preservation

- [ ] **Task 5.1: Extract trailers from original commit message**
  - In `hook_commit_msg.rs`, before rewriting the message file, read the original content.
  - Implement `fn extract_trailers(msg: &str) -> &str` that returns the trailer block (lines after the last blank line matching `Token: value` patterns).

- [ ] **Task 5.2: Rewrite message preserving trailers**
  - Format the output as: `{WHAT}\n\n{WHY}\n\n{trailers}` (omit the trailing block if empty).
  - Apply to both the silent path (line 136) and TUI accept path (line 239).

---

## Phase 6: Medium — Multi-File Entity & Category Inference

- [ ] **Task 6.1: Canonical entity from staged files**
  - In `hook_commit_msg.rs`, implement `fn canonical_entity(files: &[String]) -> String`:
    - If only one file, return it.
    - If multiple files share a common parent directory prefix, return that directory.
    - Otherwise return `format!("{} (+{} more)", first_file, files.len() - 1)`.
  - Store the full file list in `related_tickets` (append after ticket IDs, separated by `|`).

- [ ] **Task 6.2: Expand and fix category inference**
  - In `parse_category_from_message`, change fallback from `Category::Feature` to `Category::Chore`.
  - Add mappings: `perf:` → `Refactor`, `build:` → `Infra`, `revert:` → `Bugfix`, `security:` → `Feature` (until a Security category exists), `breaking:` → `Architecture`.
  - Add `tracing::debug!` log on fallback fire.

---

## Phase 7: Low — Test Coverage

- [ ] **Task 7.1: `tests/hook_commit_msg.rs`**
  - `test_trivial_bypass_skips_tui`
  - `test_non_interactive_bypasses_tui`
  - `test_phantom_record_cleanup_on_abort`
  - `test_category_inference_covers_all_prefixes`
  - `test_multi_file_entity_canonical_path`
  - `test_trailer_preservation`

- [ ] **Task 7.2: `tests/ledger_crypto.rs`**
  - `test_sign_and_verify_roundtrip`
  - `test_verify_fails_on_tampered_payload`
  - `test_sign_returns_error_on_missing_key_dir`
  - `test_require_signing_config_blocks_commit`

- [ ] **Task 7.3: `tests/cozo_schema_migration.rs`**
  - `test_new_repo_gets_full_schema`
  - `test_old_schema_is_migrated`
  - `test_migration_is_idempotent`

- [ ] **Task 7.4: `tests/m33_migration.rs`**
  - `test_m33_adds_columns_to_sqlite`
  - `test_m33_backfills_existing_rows`

---

## Phase 8: Verification

- [ ] **Task 8.1**: `cargo fmt --all -- --check` — passes clean.
- [ ] **Task 8.2**: `cargo clippy --all-targets --all-features -- -D warnings` — zero warnings.
- [ ] **Task 8.3**: `cargo test --workspace` — all tests pass including new O1-R tests.
- [ ] **Task 8.4**: Manual smoke test — run `git commit` in a test repo, `git reset HEAD~1`, confirm no phantom `COMMITTED` entries in `changeguard ledger audit`.
- [ ] **Task 8.5**: `changeguard verify` — all steps pass.
- [ ] **Task 8.6**: Commit the remediation via `changeguard ledger commit`.
