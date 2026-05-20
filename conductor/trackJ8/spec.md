# Track J8: `index --check` Exit Code Fix

## Status
Planned

## Milestone
J: Developer Experience Hardening

## Problem
`changeguard index --check` exits with code 1 when stale files are detected. This is semantically wrong:

- **Stale** means the index exists but some files have changed since last indexing. The data is usable; it is just not current. This is a routine, expected state in active development and should exit 0.
- **Missing or corrupt** means the index does not exist or is unreadable. Commands that depend on the index will fail. This is a genuine error and should exit 1.

Scripted callers (CI pipelines, pre-commit hooks, Makefile targets) that use `changeguard index --check && do_something` are broken by a stale index even when the something would work fine with slightly-stale data.

Additionally, the output message for stale does not distinguish between the two states:

```
Index check: 5 stale files detected.
```

There is no clear indication of what action the user should take.

## Fix Strategy

### Exit codes
| State | Condition | Exit Code |
|-------|-----------|-----------|
| Current | 0 stale files, index readable | 0 |
| Stale | > 0 stale files, index readable | 0 |
| Missing | Index directory does not exist | 1 |
| Corrupt | Index exists but cannot be opened | 1 |

### Output messages
- **Current**: `Index is current.` (exit 0)
- **Stale**: `Index is stale: {n} file(s) changed since last index. Run 'changeguard index --semantic' to refresh.` (exit 0)
- **Missing**: `Index not found. Run 'changeguard index --semantic' to create.` (exit 1)
- **Corrupt**: `Index is corrupt: {error}. Run 'changeguard index --semantic' to rebuild.` (exit 1)

### Optional `--strict` flag
Add `--strict` to `index --check`. When set, stale → exit 1. This allows CI pipelines that require a current index to opt in to stricter checking without affecting the default behavior.

## Scope of Changes

### 1. `src/commands/index.rs` — check path (lines ~88-90)
- Replace `if status.stale_files > 0 { std::process::exit(1); }` with the new exit code table above.
- Distinguish missing vs. corrupt via the error type returned by the index open call.
- Add `--strict` flag to `IndexCheckArgs`.

### 2. Output messages
- Update `println!` calls to use the descriptive messages above.

## Success Criteria
- `changeguard index --check` with stale files exits 0 and prints the stale message with refresh hint.
- `changeguard index --check` with missing index exits 1.
- `changeguard index --check` with corrupt index exits 1.
- `changeguard index --check --strict` with stale files exits 1.
- `changeguard index --check` with current index exits 0 and prints "Index is current."
- All existing index tests pass.

## Files Changed
- `src/commands/index.rs`

## Edge Cases
- **Partially corrupt index** (some segments readable, some not): Treat as corrupt → exit 1.
- **Index directory exists but is empty**: Treat as missing → exit 1.
- **`--strict` combined with missing index**: Already exit 1 for missing regardless of `--strict`; no change.
- **`changeguard index --check` in a repo with no files**: 0 stale files, index current → exit 0.
- **Permission error reading index**: Surface the permission error in the corrupt message; do not silently return exit 0.

## Definition of Done
- [ ] `changeguard index --check` exits 0 when index is stale (prints stale message with hint).
- [ ] `changeguard index --check` exits 1 when index is missing.
- [ ] `changeguard index --check` exits 1 when index is corrupt.
- [ ] `changeguard index --check --strict` exits 1 when index is stale.
- [ ] Output messages include actionable hints for each state.
- [ ] CI gate passes: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --workspace`.
