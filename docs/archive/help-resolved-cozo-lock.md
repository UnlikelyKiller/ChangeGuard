# Troubleshooting Note: CozoDB Initialization Lock

## Status

Resolved for the current workspace as of 2026-05-09.

`changeguard doctor` now initializes the native graph successfully:

```text
Native Graph: Ready (CozoDB active, 0 nodes, 0 edges)
```

The original CozoDB lock error is no longer reproducible from the repo root.

## Original Symptom

The `changeguard index --semantic` command previously failed while initializing CozoDB:

```text
Failed to initialize CozoDB: IO error: could not acquire lock on ".changeguard/state/ledger.cozo/db"
```

The failure persisted after stopping visible `changeguard.exe` and `llama-server.exe`
processes and recreating `.changeguard/`.

## Current Finding

The remaining local-state problem is separate from the CozoDB lock. Commands that
open the SQLite ledger database, such as `changeguard ledger status`, currently
fail with:

```text
rusqlite_migration error in migrations definition: Attempt to migrate a database with a migration number that is too high
```

This means the local `.changeguard` SQLite database was created or migrated by a
newer schema set than the `changeguard` binary currently on PATH understands.

## Resolution

For the CozoDB lock:

1. Verify no long-running ChangeGuard process is active.
2. Run `changeguard doctor` from the repository root.
3. Confirm `Native Graph: Ready`.

For the SQLite migration mismatch:

1. Rebuild and reinstall the current repository binary:

   ```powershell
   cargo install --path .
   ```

2. Re-run:

   ```powershell
   changeguard doctor
   changeguard ledger status
   ```

3. If the migration mismatch persists, preserve the current state directory before
   resetting:

   ```powershell
   Rename-Item .changeguard .changeguard.backup
   changeguard init
   ```

Do not delete `.changeguard/` unless the ledger provenance in that workspace is
known to be disposable.
