# Track Z-R2 Plan: Ledger Adopt Path Deduplication & Defense-in-Depth

## Phase 1 — Red (Verify Current Behavior)
- [ ] 1. Run the existing `test_adopt_writes_kg_edges_with_real_files` and confirm it passes (baseline).
- [ ] 2. Add a temporary `assert!` in `write_ledger_graph_edges` to prove it is still being called (redundant write exists).

## Phase 2 — Centralize File Override in CommitRequest
- [ ] 3. In `src/ledger/transaction.rs`, add `pub changed_files: Option<Vec<String>>` to `CommitRequest` with `#[serde(skip_serializing_if = "Option::is_none")]`.
- [ ] 4. In `TransactionManager::commit_change`, locate the KG edge-building loop (around lines 353–371). Replace the `get_transaction_files` call with:
  ```rust
  let changed_files = match req.changed_files {
      Some(files) => files,
      None => match self.get_transaction_files(&tx_id) { ... },
  };
  ```
  Apply the existing synthetic filter (`drift_adoption:` / UUID) to `changed_files`.
- [ ] 5. In `src/commands/ledger/maintenance.rs`, update `execute_ledger_adopt`:
  - In the `CommitRequest` passed to `commit_change`, set `changed_files: Some(changed_files)`.
  - Remove the `write_ledger_graph_edges` call after `drop(tx_mgr)`.
  - Keep `drop(tx_mgr)` and `drop(storage)` as-is.

## Phase 3 — Defense-in-Depth & Hardening
- [ ] 6. In `write_ledger_graph_edges`, add synthetic filtering before mapping any file to a `File` URN:
  ```rust
  if file.contains("drift_adoption:") || uuid::Uuid::parse_str(&file).is_ok() {
      continue;
  }
  ```
- [ ] 7. In `get_transaction_files`, before `files.insert(tx.entity_normalized)`:
  ```rust
  if !tx.entity_normalized.contains('/') && !tx.entity_normalized.contains('.') {
      // Skip synthetic/non-path entities
  } else if tx.entity_normalized.contains("drift_adoption:")
      || uuid::Uuid::parse_str(&tx.entity_normalized).is_ok()
  {
      // Skip synthetic entities
  } else {
      files.insert(tx.entity_normalized);
  }
  ```

## Phase 4 — Green + Verification
- [ ] 8. Run `cargo nextest run --lib --bins --workspace`.
- [ ] 9. Run `cargo nextest run --test integration`.
- [ ] 10. Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] 11. Run `cargo fmt --all -- --check`.
- [ ] 12. Install binary with `cargo install --path .`.
