# Track J5: KG Enrichment Progress Indicator and Configurable Timeout

## Status
Planned

## Milestone
J: Developer Experience Hardening

## Problem
`changeguard scan --impact` and `changeguard impact` hang silently for ~28 seconds while the KG enrichment provider runs its Datalog reachability query. There is:

1. No progress indication — the terminal appears frozen.
2. No timeout — a slow or locked CozoDB query can block indefinitely.
3. No degradation path — if the KG query fails or times out, the entire impact command fails rather than skipping KG and continuing.

## Fix Strategy

### Progress indication
Run the KG enrichment on a background thread and print a spinner (or a simple elapsed-time line) to stderr while waiting. Use `std::thread::spawn` + `mpsc::channel` to receive the result. This keeps the enrichment synchronous from the caller's perspective (blocking on `receiver.recv_timeout`) while giving the user visible feedback.

Use a simple character-based spinner (`|`, `/`, `-`, `\`) printed to stderr at 100ms intervals. Clear the line on completion with `\r`. This works in all terminals including Windows CMD and PowerShell.

### Configurable timeout
Add `kg_timeout_secs: u64` to `KgConfig` (or `ImpactConfig`) with default `60`. If the background thread does not return within that time, cancel the KG enrichment and return a `ProviderResult` with status `Degraded("KG enrichment timed out after {n}s — run with --verbose for details")`.

### Graceful degradation
On timeout or any error from the KG provider, log `warn!` and return a degraded result so the rest of the impact report still renders. The overall impact command exit code is 0 (not an error for the user).

## Scope of Changes

### 1. `src/impact/enrichment/kg_provider.rs`
- Wrap the existing Datalog query in `std::thread::spawn`
- Poll with `receiver.recv_timeout(Duration::from_secs(config.kg_timeout_secs))`
- On timeout: emit `warn!` and return `ProviderResult::degraded("timed out")`
- On error: emit `warn!` and return `ProviderResult::degraded(err.to_string())`

### 2. `src/impact/orchestrator.rs` (or caller)
- Before invoking the KG enrichment provider, start a spinner thread writing to stderr
- After receiving the KG result (success or degraded), stop the spinner and clear the line

### 3. Spinner implementation
- Small utility in `src/ui/spinner.rs` (new file, ~50 lines):
  - `Spinner::start(message: &str) -> SpinnerHandle`
  - `SpinnerHandle::stop()` — clears the line and joins the thread
  - Uses `\r` to overwrite the same line; falls back to newlines if `TERM=dumb` or on Windows CI (`CI` env var set)

### 4. Config addition
- Add `kg_timeout_secs: u64` to the relevant config struct with `serde(default)` = `60`
- Add to `.changeguard/config.toml` template under `[impact]` or `[kg]`

## Success Criteria
- `changeguard scan --impact` shows a spinner or "Running KG analysis…" line during the 28s query.
- On completion (success or timeout), the spinner line is cleared and normal output follows.
- If KG times out (configurable; default 60s), the impact report still renders with a `[DEGRADED]` note for the KG section.
- `kg_timeout_secs = 5` in config causes timeout after 5 seconds with a clear message.
- `--verbose` shows the underlying error/timeout reason.
- On CI (`CI=true`), spinner uses newlines instead of `\r` to avoid garbled output.
- All existing tests pass.

## Files Changed
- `src/impact/enrichment/kg_provider.rs`
- `src/impact/orchestrator.rs`
- `src/ui/spinner.rs` (new)
- `src/ui/mod.rs` (new or extended)
- `src/config/model.rs` (add `kg_timeout_secs`)
- `.changeguard/config.toml`

## Edge Cases
- **KG provider panics**: The `thread::spawn` closure may panic if CozoDB panics. Use `thread::Builder::spawn` and check `JoinHandle::join()` for `Err` (panic), degrade gracefully.
- **No KG index** (CozoDB not initialized): Existing error path already handles this; just ensure it returns `ProviderResult::degraded` rather than propagating an error that kills the command.
- **Windows CI / dumb terminal**: Detect `CI` env var or `TERM=dumb`; emit `info!("Running KG analysis…")` once and omit spinner updates to keep CI logs clean.
- **`kg_timeout_secs = 0`**: Treat as "no timeout" (unlimited wait) to allow power users to disable the timeout.
- **Multiple concurrent KG queries**: Not currently possible (single-process, sequential enrichment). No special handling needed.

## Definition of Done
- [ ] A visible spinner or progress message appears during KG enrichment.
- [ ] KG timeout fires after `kg_timeout_secs` and the impact report still renders.
- [ ] Degraded KG result shows a `[DEGRADED]` note in the impact output.
- [ ] On CI (`CI=true`), no spinner escape codes appear in logs.
- [ ] `kg_timeout_secs = 0` disables the timeout.
- [ ] CI gate passes: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --workspace`.
