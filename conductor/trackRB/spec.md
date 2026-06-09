# Specification: Milestone B Remediation (Track R-B)

## Overview
Address critical and high-severity findings from the Milestone B Codex review to ensure fail-open integrity, sync safety, and architectural cleanliness.

## Requirements
1. **Hang Protection**: Implement a 5-second timeout for `ai-brains recall` shell execution using a thread-based waiter.
2. **Thread Leak Prevention**: Use a more robust IPC connection pattern or ensure orphaned threads don't accumulate (e.g., use a global connection pool or just log the risk if unfixable in sync std).
3. **B6 Completion**: Correctly wire `bridge::client::query_unified` into `src/commands/ask.rs`.
4. **Contract Enforcement**: Add a `version` check to `deserialize_record` and warn on mismatch.
5. **Decoupling**: Define `BridgeVerifyOutcome` in `bridge::model` instead of importing `VerificationResult` in `notify.rs`.
6. **Data Integrity**: Update export to pull from CozoDB where appropriate for high-fidelity ledger deltas.
