## Plan: Track B6 - Unified Retrieval in Ask
### Phase 1: Ask Command Augmentation
- [ ] Task 1.1: Modify `src/commands/ask.rs` to dynamically invoke `src/bridge/client.rs::query(...)` during context preparation.
- [ ] Task 1.2: Inject formatted `Insight` outputs into the payload context block.
- [ ] Task 1.3: Ensure the total context size respects truncation boundaries within `src/gemini/mod.rs`.
- [ ] Task 1.4: Write a test verifying the `ask` prompt cleanly forms even if the AI-Brains retrieval function returns an `Err`.
