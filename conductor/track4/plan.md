## Plan: Git Scan Foundation
### Phase 5: One-shot Repository Scan and Change Classification
- [ ] Task 5.1: Scaffold `src/git/` module structure (`mod.rs`, `repo.rs`, `status.rs`, `diff.rs`, `classify.rs`) and define data models (`RepoSnapshot`, `FileChange`, `ChangeType`).
- [ ] Task 5.2: Implement `src/git/repo.rs` for repository discovery and opening using `gix::discover`. Handle "Not a Git Repo" errors cleanly with `miette`.
- [ ] Task 5.3: Implement branch/HEAD metadata extraction (hash, branch name, detached state) in `src/git/repo.rs`. Add tests for detached and unborn states.
- [ ] Task 5.4: Implement `src/git/status.rs` and `src/git/classify.rs` to gather clean vs dirty state using `gix`, collecting staged and unstaged file paths, and mapping them to `ChangeType`.
- [ ] Task 5.5: Implement `src/commands/scan.rs` to wire up the scanning logic and output a deterministic, sorted summary.
- [ ] Task 5.6: Add comprehensive integration tests in `tests/cli_scan.rs` using a temporary git repository fixture (testing clean, dirty, and detached HEAD states).
- [ ] Task 5.7: Execute explicit verification using `cargo test -j 1` to ensure tests pass deterministically without race conditions.