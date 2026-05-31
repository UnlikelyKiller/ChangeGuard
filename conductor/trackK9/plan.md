# Track K9 Plan: Unified Audit Reporting

## Phase 1: Core Abstraction
- [x] Define `ProjectAuditReport` struct in `src/commands/ledger_audit.rs`.
- [x] Implement data gathering logic that populates this struct.

## Phase 2: Consistent Pagination
- [x] Update data gathering to apply `limit` to "Top Churned Files" and other ranked lists where applicable.
- [x] Ensure `offset` behavior is documented or logically handled for summary sections.

## Phase 3: Rendering & Verification
- [x] Implement human-readable and JSON rendering for the `ProjectAuditReport`.
- [x] Verify output consistency with various `--limit` values.
- [x] Run full CI gate.
