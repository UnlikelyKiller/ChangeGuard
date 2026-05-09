# ChangeGuard Engineering Standards

## CI & Hygiene

- **Formatting**: ALWAYS run `cargo fmt --all` before committing.
- **Linting**: ALWAYS run `cargo clippy --all-targets --all-features -- -D warnings`.
- **Testing**: ALWAYS ensure `cargo test --workspace` passes.
- **Node.js**: GitHub Actions are configured to use Node.js 24 via `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true`.

## Git Hooks

A pre-commit hook is available in `scripts/pre-commit.sh` (Bash) and `scripts/pre-commit.ps1` (PowerShell).

To install it on Windows (Git Bash):
```bash
cp scripts/pre-commit.sh .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

To install it on Windows (PowerShell):
```powershell
Copy-Item scripts/pre-commit.sh .git/hooks/pre-commit
```
*(Note: Git for Windows will execute the .sh file even if called from PowerShell).*

## Architectural Invariants

- **No Unwrap**: Use `miette` and `Result` for error handling in production code.
- **SRP**: Keep module boundaries sharp (e.g., `src/search/` for search logic, `src/state/` for persistence).
- **Local-First**: All features must work offline with a local model.
- **Windows Support**: Ensure paths are handled correctly across OSes (use `camino` for UTF-8 paths).
