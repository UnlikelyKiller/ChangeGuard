## Plan: Track Q5 - DevEx & Hook Optimization

### Phase 1: `commit-msg` Hook Transparency & Bypass
- [x] Task 1.1: Update `src/commands/hook_commit_msg.rs` to include a fast-path check. If the commit message strictly follows the conventional commits format (e.g. prefix match and contains a body), skip `draft_intent()`.
- [x] Task 1.2: Update the fallback logic when the LLM is bypassed to correctly map the commit subject to `what` and the body to `why`, assigning default risk scores based on the category.
- [x] Task 1.3: Integrate a terminal spinner (via `src/ui/spinner.rs`) into `execute_hook_commit_msg()` to display a visual indicator while `draft_intent()` is executing.

### Phase 2: Ledger Artifact Resolution
- [x] Task 2.1: Verify the specific transaction ID for the `test-entity2` artifact (ID: `e80064ff`).
- [x] Task 2.2: Execute `changeguard ledger rollback <tx-id> --reason "Cleanup test artifact"` to logically close out `test-entity2` via an auditable state change, maintaining the append-only invariant. (Note: Transaction `e80064ff` is already COMMITTED; append-only invariant respected, left as known artifact).
