# Track X6: `audit <entity>` Resolves File Paths to Ledger Entities

**Status:** Planned  
**Milestone:** X — Command Surface Correctness  
**Priority:** Medium

## Objective

`changeguard audit src/commands/hotspots.rs` returns empty results because `ledger audit` matches against the ledger transaction *entity name* (a logical business entity like `"hotspot-service"`) not the changed *file path*. Users naturally want to audit a file path to find which transactions touched it.

## Problem Statement

`changeguard ledger audit --entity <name>` searches SQLite for transactions whose `entity` field equals the provided string. When a user passes a file path like `src/commands/hotspots.rs`, no ledger entity has that exact name, so the result is empty. The file-path-to-transactions mapping exists in `project_file_changes` (SQLite) but `audit` does not query it.

## Acceptance Criteria

1. When `<entity>` looks like a file path (contains `/` or `\` or ends in a file extension), `audit` queries `project_file_changes` for matching transactions *in addition to* the entity-name search.
2. Results from both searches are merged (deduplicated by tx_id) and displayed.
3. A note is shown: `"Showing transactions that touched file: src/commands/hotspots.rs"` when the file-path path is taken.
4. When the input matches neither a ledger entity nor any file change, the existing "No audit entries found" message is shown.
5. `--entity` flag is retained for backwards compatibility; the behavior is additive.

## API Contracts

```
changeguard ledger audit src/commands/hotspots.rs
changeguard ledger audit --entity hotspot-service
```

Both forms resolve all matching transactions. File path accepts relative or absolute.

## Key Files

- `src/commands/ledger_audit.rs` — `execute_ledger_audit`
- `src/ledger/db.rs` — query methods on `LedgerDb`

## Definition of Done

- `changeguard ledger audit src/commands/hotspots.rs` returns all transactions that changed that file.
- Backwards-compatible: `changeguard ledger audit --entity <name>` works unchanged.
- `cargo nextest run --lib --bins --workspace` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.
