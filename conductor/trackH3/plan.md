# Plan: Track H3 (Global Knowledge Retrieval)

- [ ] 1. Refactor `src/commands/ask.rs` to make the impact report optional.
- [ ] 2. Implement a `GlobalContextGatherer` that runs a semantic query and a Datalog neighborhood search based on the user's input.
- [ ] 3. Merge the results of semantic search and graph traversal into a ranked list of "Context Snippets".
- [ ] 4. Update the LLM client to use this broader context when the git diff is empty.
- [ ] 5. Test with a series of general architectural questions (e.g., "How does the storage layer work?").
