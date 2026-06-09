# Specification: Track G7 Native Semantic Extraction (De-coupling Part 2)

## Goal
Implement native LLM-based semantic extraction and "Cord-Cutting," making ChangeGuard a truly standalone, single-binary intelligence tool.

## Context
This is the final phase of the Knowledge Graph integration. We port the chunking, prompting, and community detection logic from Python to Rust, allowing ChangeGuard to build the full Knowledge Graph natively using its own `ai::complete()` interface.

## Technical Details

### 1. Semantic Engine (`src/ai/semantic_extractor.rs`)
Implement the "Hardened" extraction loop:
- **Chunker**: Native Markdown and Text chunking.
- **Prompting**: Moving the `extraction_prompt` and `community_label_prompt` into Rust string constants.
- **Token Budgeting**: Enforcing the **30,000 token limit** and performing **adaptive recursion** (splitting chunks) if the LLM response is truncated.

### 2. Native Community Detection
Use CozoDB's built-in **Leiden Community Detection** algorithm (via `:run community_leiden`) to partition the graph into domains natively. This replaces the dependency on the Python `graspologic` library.

### 3. The "Cord-Cut"
- Remove the `graphifyy` subprocess orchestration from `src/commands/index.rs`.
- Clean up any legacy Python scripts or environment checks.
- Update the `doctor` command to verify the native Graph engine instead of the Python environment.

## TDD Requirements
1.  **Chunking Test**: Verify that a long document is split into overlapping chunks according to the budget.
2.  **Prompt Assembly**: Test that the LLM prompt correctly includes the context and the requested JSON schema.
3.  **Standalone Verification**: Verify that `changeguard index` runs successfully in a Docker container that lacks a Python interpreter.

## Definition of Done
- [ ] Native semantic extraction loop implemented and tested.
- [ ] CozoDB-based community detection integrated.
- [ ] `graphifyy` dependency removed completely.
- [ ] No more than 4 files modified: `src/ai/semantic_extractor.rs`, `src/ai/mod.rs`, `src/commands/index.rs`, `src/index/mod.rs`.
