# Track K2: Intelligence Precision (Ask Context)

## Status
Planned

## Milestone
K: Service Discovery & Storage Hardening

## Problem
`changeguard ask --semantic` hallucinations occur when no transaction is active. The system defaults to explaining the (empty) impact report instead of querying the indexed codebase. This leads to responses like "Based on the provided impact summary, there is no information about X."

## Solution
1. **Context Pivot**: If no transaction is active (or no changed files exist), `execute_ask` should pivot the system prompt and context assembly to focus exclusively on codebase chunks retrieved from the semantic index.
2. **Chunk Prioritization**: Ensure codebase snippets are not truncated by the (empty) impact packet in the context budget.
3. **Prompt Hardening**: Refine the system prompt to distinguish between "Changes Analysis" (active tx) and "Codebase Inquiry" (clean state).

## Definition of Done (DoD)
- [ ] `changeguard ask --semantic "How does the search engine work?"` returns an architecture explanation derived from code chunks even when the git state is clean.
- [ ] No mention of "Based on the provided impact summary" when git state is clean.
- [ ] Verification test: mock semantic retrieval and confirm context assembly priorities.
- [ ] CI gate passes.
