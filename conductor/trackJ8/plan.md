# Track J8 Plan: `index --check` Exit Code Fix

## Steps

### Red Phase (failing tests)
1. [ ] Add test: with a freshly-initialized index and one file modified (mtime > index mtime), `check_index()` returns `IndexStatus::Stale` — assert it maps to exit code 0
2. [ ] Add test: with no index directory present, `check_index()` returns `IndexStatus::Missing` — assert exit code 1
3. [ ] Add test: `--strict` flag with stale index → exit code 1
4. [ ] Add test: `--strict` flag with current index → exit code 0
5. [ ] Run CI gate — new tests expected to fail

### Green Phase (implementation)
6. [ ] Add `--strict` flag to `IndexArgs` or `IndexCheckArgs` (`#[arg(long)]` bool)
7. [ ] Define `IndexStatus` enum: `Current`, `Stale { count: usize }`, `Missing`, `Corrupt { reason: String }`
8. [ ] Refactor `check_index()` (or inline logic in the check branch) to return `IndexStatus` instead of calling `std::process::exit` directly
9. [ ] In the check dispatch:
   - `IndexStatus::Current` → print "Index is current." → exit 0
   - `IndexStatus::Stale { count }` → print stale message → exit `if strict { 1 } else { 0 }`
   - `IndexStatus::Missing` → print missing message → exit 1
   - `IndexStatus::Corrupt { reason }` → print corrupt message → exit 1
10. [ ] Distinguish missing vs. corrupt: check `index_dir.exists()` first; on open error, classify as corrupt
11. [ ] Run `cargo build` — fix any type errors
12. [ ] Run CI gate — all tests expected to pass

### Verification
13. [ ] `cargo install --path .` to rebuild binary
14. [ ] `changeguard index --check` with stale files → exit 0 + stale message
15. [ ] `changeguard index --check --strict` with stale files → exit 1
16. [ ] Delete index dir → `changeguard index --check` → exit 1 + missing message
17. [ ] Rebuild index → `changeguard index --check` → exit 0 + "Index is current."
18. [ ] `changeguard verify` passes

### Finalization
19. [ ] Mark all tasks complete; update `conductor/conductor.md` status to Completed
20. [ ] `changeguard ledger commit` with summary and reason
