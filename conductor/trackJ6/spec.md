# Track J6: `bridge export` Stdout Default (--out Optional)

## Status
Planned

## Milestone
J: Developer Experience Hardening

## Problem
`changeguard bridge export` fails immediately with "required arguments not provided: --out <OUT>" when run without a path argument. This is inconsistent with every other ChangeGuard command that produces output (they all default to stdout), and breaks the common Unix pattern of `changeguard bridge export | jq .` or piping into another tool.

`bridge verify` already defaults to stdout. The asymmetry is a usability defect.

## Fix Strategy
Make `--out` optional in the `BridgeExportArgs` struct. When omitted, write to stdout. When provided, write to the given file path (existing behavior). The file-write path should create parent directories if they do not exist, matching the behavior of `viz --output`.

## Scope of Changes

### 1. `src/commands/bridge.rs` (or wherever `BridgeExportArgs` is defined)
- Change `out: String` (required) to `out: Option<PathBuf>` with `#[arg(long)]` (no `required = true`)
- Update `execute_export()` to dispatch on `out`:
  - `None` → serialize to stdout via `println!` or `serde_json::to_writer(std::io::stdout(), ...)?`
  - `Some(path)` → create parent dirs, write to file (existing path)

### 2. Help text
- Update `--out` help string: `"Output file path (default: stdout)"`

### 3. `src/bridge/export.rs` (if separate from commands)
- If `execute_export` signature currently takes `out_path: String`, change to `out_path: Option<PathBuf>` and apply the same dispatch logic.

## Success Criteria
- `changeguard bridge export` (no args) prints JSON to stdout and exits 0.
- `changeguard bridge export | jq .` works in PowerShell and bash.
- `changeguard bridge export --out report.json` still writes to the file.
- `changeguard bridge export --out nested/dir/report.json` creates parent dirs.
- All existing bridge tests pass.

## Files Changed
- `src/commands/bridge.rs`
- `src/bridge/export.rs` (if applicable)

## Edge Cases
- **Stdout is a TTY**: JSON written to stdout regardless. Users piping want the data; users at a terminal get the data (they can redirect themselves). This matches `jq` behavior.
- **--out path parent does not exist**: Create it with `fs::create_dir_all`. Return a descriptive `Err` if creation fails (e.g., permission denied).
- **--out path is a directory**: Return `Err("--out path is a directory, expected a file path")`.
- **stdout closed** (e.g., `| head -1` exits early, breaking the pipe): On `BrokenPipe` (errno 32), exit 0 silently. This matches standard Unix tool behavior. Use `std::io::ErrorKind::BrokenPipe` check.
- **Large export**: No buffering concerns — `serde_json::to_writer` streams to stdout without loading the full JSON into a `String`.

## Definition of Done
- [ ] `changeguard bridge export` prints JSON to stdout and exits 0.
- [ ] `changeguard bridge export --out out.json` writes to file (unchanged behavior).
- [ ] `changeguard bridge export --out nested/new/dir/out.json` creates missing parent dirs.
- [ ] `BrokenPipe` on stdout does not produce an error message.
- [ ] CI gate passes: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --workspace`.
