# Track K8 Plan: CLI Consistency

## Phase 1: CLI Updates
- [x] Add `json` and `out` flags to `ScanArgs` in `src/cli.rs`.
- [x] Update `execute_scan` to pass these flags down.

## Phase 2: Implementation
- [x] Refactor `execute_scan` in `src/commands/scan.rs` to handle conditional JSON output.
- [x] Ensure the full `ImpactPacket` is serialized when `--json` is active.

## Phase 3: Verification
- [x] Manual test: `changeguard scan --impact --json | jq .`
- [x] Add integration test for JSON output.
- [x] Run full CI gate.
