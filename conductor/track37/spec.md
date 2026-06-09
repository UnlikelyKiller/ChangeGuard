# Track 37: LSP Daemon Functional Completion

## Objective
Address the critical daemon gaps identified in `docs/audit4.md`, making the ChangeGuard LSP fully functional. This includes implementing Hover and CodeLens, resolving read-only SQLite errors, handling stale data, performing real-time structural/complexity analysis, detecting broken stdin/parent processes, and adding comprehensive lifecycle tests.

## Requirements

### 1. Hover & CodeLens Implementation
- **Hover**: Implement `on_hover` in `src/daemon/handlers.rs`. When hovering over a file, return a markdown summary of its temporal and structural impact based on the latest packet data. Remove all stub comments.
- **CodeLens**: Implement `on_code_lens` in `src/daemon/handlers.rs`. Provide a file-level lens (e.g., at line 0) displaying the file's normalized complexity score and risk category. Remove all stub comments.

### 2. Real-Time Analysis (`trigger_analysis`)
- Update `trigger_analysis` to perform active analysis rather than merely replaying the cached packet.
- Upon `DidOpen`, `DidChange`, and `DidSave`, run a lightweight local analysis (e.g., evaluating `NativeComplexityScorer` on the current document text or path) and merge it with the historical temporal/impact data from the latest stored packet.
- Publish the resulting combined diagnostics via `publish_diagnostics`.

### 3. Broken Stdin / Parent Process Self-Termination
- Implement a functional `check_stdin_alive()` (or equivalent termination monitor) in `src/daemon/lifecycle.rs`.
- Since standard `tower-lsp` consumes stdin, implement a parent-process ID (PPID) liveliness check using `sysinfo`, or handle standard input stream EOF gracefully to trigger server shutdown.
- Ensure the daemon terminates to avoid orphaned processes if the parent editor dies or stdin closes unexpectedly. Remove stub comments.

### 4. Read-Only SQLite & Stale Data Handling
- Modify `ReadOnlyStorage::get_connection()` in `src/daemon/state.rs` to **not** execute `PRAGMA journal_mode=WAL;` on the read-only connection, avoiding potential SQLite capability errors.
- If `ReadOnlyStorage::query` returns `data_stale: true`, surface this in the LSP outputs. Inject a synthetic Diagnostic warning (e.g., "ChangeGuard data is stale (database locked)") or append a warning to the Hover/CodeLens text.

### 5. Daemon Lifecycle Tests
- Create `tests/daemon_lifecycle.rs` to verify daemon behavior without requiring a full real editor host.
- **Coverage Required**:
  - PID file creation, stale PID cleanup, and removal on shutdown.
  - SQLite contention retries and `data_stale` handling.
  - URI normalization and path mapping.
  - Hover and CodeLens output structure (asserting they do not return `None`).
  - Broken-stdin/parent-termination mock behavior.

## Out of Scope
- Full temporal git traversal on every keystroke (use cached temporal data + real-time complexity/structural scans).
- Supporting multiple simultaneous workspace folders in a single daemon instance.
