# Track Q5: DevEx & Hook Optimization

## Objective
Optimize the developer experience (DevEx) by reducing the latency and improving the transparency of the `commit-msg` hook. Additionally, evaluate options for cleaning up test artifacts from the append-only ledger.

## Requirements

### 1. `commit-msg` Hook Optimization
- **Transparency**: Implement a visual spinner (using the existing `src/ui/spinner.rs` or similar) when the `commit-msg` hook invokes the local LLM (`draft_intent`). This prevents the user from wondering if the terminal has hung during the 1-2s delay.
- **Fast-Path Bypass**: Implement logic to skip the LLM intent drafting step if the raw commit message already conforms to a well-formed conventional commit (e.g., starts with `feat:`, `fix:`, `chore:`, etc.) AND contains sufficient context (e.g., a message body). If bypassed, derive `what` (subject), `why` (body), and `category` directly from the raw message using existing heuristics, and default the risk based on the category.

### 2. Ledger Artifact Cleanup (`test-entity2`)
- **Ledger Analysis**: A review of `src/ledger/db.rs` confirms that the ledger is strictly append-only. There are no SQL `DELETE` methods for `ledger_entries` and no support for hard deletions.
- **Action**: Do not attempt to execute a hard SQL delete. The append-only design must be respected.
- **Resolution**: Use the `ledger rollback` feature (introduced in Track Q4) to logically "revert" the state of `test-entity2`. This satisfies the cleanup requirement via an auditable state change without violating ledger integrity.

## Testing Strategy
- **Bypass Test**: Create a test commit with a properly formatted conventional commit message (with a subject and body). Verify the LLM is bypassed and the hook completes rapidly.
- **LLM / Spinner Test**: Create a test commit with a brief, ambiguous message. Verify the spinner appears, the LLM is invoked, and it correctly infers the intent.
- **Artifact Test**: Verify that executing a rollback on `test-entity2` registers a valid rollback entry in the ledger and removes it from pending/active queries as expected.
