# Track K14: Non-Mutating Federation Export

## Status
Planned

## Milestone
K: Service Discovery & Storage Hardening

## Problem
`federate export` behaves like an export command but writes `.changeguard/state/schema.json`. That makes it unsuitable for read-only command audits and surprising for users who expect stdout or an explicit output path.

## Objective
Split federation schema generation into explicit write and preview modes so users can inspect export payloads without mutating repository-local ChangeGuard state.

## Scope
- Add a non-mutating `--dry-run` mode for federation export that writes schema JSON to stdout.
- Add `--out <path>` for explicit non-state destination control.
- Preserve the current state-writing behavior as the default for compatibility, but document it clearly.
- Update bridge/federation docs and smoke checks.

## Non-Goals
- Do not break existing `federate scan` consumers that read `.changeguard/state/schema.json`.
- Do not conflate `federate export` with `bridge export`; federation schema remains a separate payload.
- Do not write state files when `--dry-run` is provided.

## Implementation Notes
- `--dry-run` should be stdout-only unless paired with `--out`.
- `--out` should create parent directories when needed and must not update `.changeguard/state/schema.json`.
- Default write mode should explicitly print the path it wrote and mention `--dry-run` for preview.

## Success Criteria
- [ ] `federate export --dry-run` emits the schema without writing `.changeguard/state/schema.json`.
- [ ] `federate export --out <path>` writes the schema to the requested path without writing `.changeguard/state/schema.json`.
- [ ] The mutating default is documented clearly in help and command output.
- [ ] Tests assert dry-run mode does not touch state files.
- [ ] Existing federation consumers continue to find the schema file when write mode is used.
- [ ] CI gate passes.

## Definition of Done
- [ ] `federate export --dry-run` is safe to include in non-destructive command audits.
- [ ] `federate export --out output/schema.json` writes only that file and creates its parent directory.
- [ ] `federate export` still writes `.changeguard/state/schema.json` for compatibility and says so.
- [ ] Tests cover default write mode, dry-run stdout mode, and explicit output mode.
- [ ] `changeguard verify` passes.
- [ ] `cargo install --path . --force` succeeds and installed-binary federation smoke checks pass.
