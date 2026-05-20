# Track J6 Plan: `bridge export` Stdout Default

## Steps

### Red Phase (failing tests)
1. [ ] Add test: call `execute_export(None, ...)` and assert it returns `Ok(())` and does not require a file path
2. [ ] Add test: call `execute_export(None, ...)` with a mock writer and assert JSON is written to the writer
3. [ ] Add test: `changeguard bridge export` via CLI integration test exits 0 (if integration tests exist)
4. [ ] Run CI gate — new tests expected to fail

### Green Phase (implementation)
5. [ ] Locate `BridgeExportArgs` struct; change `out: String` (or `PathBuf`) to `out: Option<PathBuf>`; remove `required = true` from `#[arg]`
6. [ ] Update `--out` help text to "Output file path (default: stdout)"
7. [ ] In `execute_export()`: match on `out`:
   - `None` → `serde_json::to_writer(std::io::stdout(), &data)?`; handle `BrokenPipe` by returning `Ok(())`
   - `Some(path)` → `fs::create_dir_all(path.parent().unwrap_or(Path::new("."))))?`; write to file
8. [ ] Add `--out path is a directory` guard: if `path.is_dir()`, return `Err(...)`
9. [ ] Run `cargo build` — fix any type/import errors
10. [ ] Run CI gate — all tests expected to pass

### Verification
11. [ ] `cargo install --path .` to rebuild binary
12. [ ] `changeguard bridge export` → JSON output to stdout, exit 0
13. [ ] `changeguard bridge export --out tmp-export.json` → file created
14. [ ] `changeguard bridge export --out nested\new\path\out.json` → parent dirs created
15. [ ] `changeguard bridge export | Select-String "version"` → works in PowerShell pipeline
16. [ ] `changeguard verify` passes
17. [ ] Delete `tmp-export.json` and `nested\` test artifacts

### Finalization
18. [ ] Mark all tasks complete; update `conductor/conductor.md` status to Completed
19. [ ] `changeguard ledger commit` with summary and reason
