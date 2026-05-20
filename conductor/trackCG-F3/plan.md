## Plan: Fix ledger note Deprecation Warning (Track CG-F3)

### Summary
`changeguard ledger note <ENTITY> <NOTE>` prints "warning: Note as a positional argument is deprecated. Use --message <note> instead." This is confusing for agents following the SKILL.md workflow. The positional note UX matches `git notes` conventions and should be kept; the deprecation warning should be removed.

### Phase 1: Fix
- [ ] Task 1.1: Identify where the deprecation warning is emitted (likely in the ledger note command handler)
- [ ] Task 1.2: Remove the deprecation warning — keep positional note as a supported (non-deprecated) argument
- [ ] Task 1.3: If `--message` flag already exists, keep it as an alias; if not, add it as an optional alternative
- [ ] Task 1.4: Verify: `changeguard ledger note docs/test.md "a test note"` records successfully without warning

### Phase 2: Update Skill Docs
- [ ] Task 2.1: Fix `--entity` flag in skill files: both `ledger atomic` and `ledger note` take `<ENTITY>` as a positional arg, not `--entity`
- [ ] Task 2.2: Update `.agents/skills/changeguard/SKILL.md` — ledger atomic and ledger note examples
- [ ] Task 2.3: Update `.agents/skills/changeguard/references/commands.md` — same fixes

### Phase 3: Gate
- [ ] Task 3.1: `cargo fmt --all -- --check` passes
- [ ] Task 3.2: `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] Task 3.3: `cargo test --workspace` passes
