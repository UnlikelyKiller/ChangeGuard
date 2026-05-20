# Track I2-3 Plan: Agent Dotfile Exclusion

## Phase 1 — Red (Failing Tests)

- [ ] Write unit test `agent_dotfiles_excluded_from_scan`:
  - Build a temp directory with `.claude/settings.json`, `.codex/README`, `.opencode/opencode.json`, `src/main.rs`.
  - Invoke the file-walker (or `ProjectIndex`) with default config.
  - Assert `.claude`, `.codex`, `.opencode` paths are absent from results.
  - Assert `src/main.rs` is present.
- [ ] Confirm test fails before fix (patterns not yet in default config).
- [ ] Commit: `test(index): red — agent dotfiles excluded from scan`

## Phase 2 — Green (Implementation)

- [ ] In `src/config/defaults.rs`, add `.claude/**`, `.codex/**`, `.opencode/**`, `.agents/**` to the `ignore_patterns` list in `DEFAULT_CONFIG`.
- [ ] Check `src/index/mod.rs` for any hardcoded exclusion array (e.g., `ALWAYS_IGNORE`). If present, add the same four patterns.
- [ ] Run CI gate: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`.
- [ ] Commit: `fix(index): exclude agent dotfiles from scan and impact analysis (CG-7)`

## Verification

- [ ] `changeguard scan --impact` — confirm no "analysis unsupported" warning for `.claude`, `.codex`, `.opencode`.
- [ ] `changeguard search "anything"` — confirm `.claude` junction is not included in index.
