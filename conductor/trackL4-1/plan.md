## Plan: Ledger ADR Export

### Phase 1: Database & Data Retrieval
- [x] Task 1.1: In `src/ledger/db.rs`, implement `LedgerDb::get_adr_entries(days: Option<u64>)` to fetch `ledger_entries` where `entry_type = 'ARCHITECTURE' OR is_breaking = 1`. Handle the `days` filter logic (e.g., `committed_at >= datetime('now', '-N days')`).
- [x] Task 1.2: In `src/ledger/transaction.rs`, add `TransactionManager::get_adr_entries` which delegates to `LedgerDb::get_adr_entries`.

### Phase 2: MADR Template Generation
- [x] Task 2.1: Create `src/ledger/adr.rs`.
- [x] Task 2.2: Implement `slugify_summary(summary: &str) -> String` in `src/ledger/adr.rs` for converting summaries to kebab-case filenames.
- [x] Task 2.3: Implement `generate_madr_content(entry: &LedgerEntry) -> String` in `src/ledger/adr.rs` following the v3 MADR specification provided in the plan.
- [x] Task 2.4: Write unit tests in `src/ledger/adr.rs` for `slugify_summary` and `generate_madr_content`.

### Phase 3: CLI Command & Routing
- [x] Task 3.1: Create `src/commands/ledger_adr.rs` and implement `pub fn execute_ledger_adr(output_dir: Option<Utf8PathBuf>, days: Option<u64>) -> Result<()>`.
- [x] Task 3.2: Implement the directory creation (`std::fs::create_dir_all`) and file writing loop inside `execute_ledger_adr`. Handle zero-results gracefully.
- [x] Task 3.3: Register the new module in `src/commands/mod.rs` (`pub mod ledger_adr;`).
- [x] Task 3.4: Add the `Adr` command to `LedgerCommands` enum in `src/cli.rs` with `--output-dir` (default: "docs/adr") and `--days` options. Route it to `execute_ledger_adr` in `run()`.

### Phase 4: Integration Testing
- [x] Task 4.1: Create `tests/ledger_adr.rs`.
- [x] Task 4.2: Write a test that sets up an in-memory or temp-dir test environment, commits multiple transactions (some ARCHITECTURE, some IMPLEMENTATION, some breaking), and verifies the output of `execute_ledger_adr`.
- [x] Task 4.3: Ensure the test verifies correct markdown structure and file naming conventions.
- [x] Task 4.4: Run `cargo test` and `cargo clippy` to ensure the new feature is fully tested and compliant.