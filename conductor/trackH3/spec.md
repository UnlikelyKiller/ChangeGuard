# Track H3: Global Knowledge Retrieval

## Objective
Decouple the `ask` command from the local impact analysis report, allowing general codebase questions at any time.

## Requirements
- **Fallback Logic**: If no `latest-impact.json` is found, fallback to a "Global Retrieval" mode.
- **Context Construction**: In Global mode, use the full Knowledge Graph (Datalog) and Semantic Search to find relevant context for the user's question.
- **LLM Prompting**: Update the Gemini prompt templates to handle context gathered from the entire repo rather than just the current git diff.

## Definition of Done (DoD)
- [ ] `changeguard ask "..."` works in a clean repository with no local changes.
- [ ] The command no longer errors with "No impact report found" when run outside of a change workflow.
- [ ] Context gathered for global questions is relevant to the query.
