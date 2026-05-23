# Plan: Track O1-2 (Intent Capture LLM Pipeline)

- [ ] 1. Create `src/ai/intent_drafter.rs`.
- [ ] 2. Define the JSON schema/prompt template for Intent extraction.
- [ ] 3. Implement context assembly logic (read `git diff --staged` and `.git/COMMIT_EDITMSG`).
- [ ] 4. Wire the context to `LocalModelClient` with a strict 2000ms timeout.
- [ ] 5. Parse the LLM output into an `IntentPayload` struct.
- [ ] 6. Write a test with a mock git diff to verify successful extraction.