# Track K14 Plan: Non-Mutating Federation Export

## Phase 1: Contract Decision
- [ ] Keep default state-writing behavior for compatibility.
- [ ] Specify `--dry-run` as stdout-only preview mode.
- [ ] Specify `--out <path>` as explicit non-state file output mode.
- [ ] Document compatibility implications for existing federation scans.
- [ ] Add tests around schema file modification time or absence.

## Phase 2: CLI Implementation
- [ ] Add explicit non-mutating `--dry-run` export mode.
- [ ] Add `--out <path>` output path support.
- [ ] Create parent directories for `--out`.
- [ ] Update help text to distinguish write mode from preview mode.

## Phase 3: Verification
- [ ] Run non-mutating federation export and confirm no state file changes.
- [ ] Run `federate export --out output/schema.json` and confirm only that path changes.
- [ ] Run write-mode federation export and confirm sibling scan compatibility.
- [ ] Run `cargo install --path . --force` and repeat installed-binary federation checks.
- [ ] Run full CI gate.
