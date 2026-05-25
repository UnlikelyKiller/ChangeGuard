## Plan: Context-Aware Intelligence Defaults
### Phase 1: Ask Context Logic
- [x] Task 1.1: Update `src/commands/ask.rs` to inspect the latest impact packet.
- [x] Task 1.2: If the payload is empty and no explicit mode override was set, default to `GLOBAL` mode.
- [x] Task 1.3: Reroute the query to use the `GLOBAL` context mode and pull semantic snippets.
### Phase 2: Testing & Verification
- [x] Task 2.1: Run `changeguard ask "what does this repo do"` on a clean working tree. Verify it provides an answer instead of an error.