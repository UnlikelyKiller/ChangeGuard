---
name: graphify
description: "Turn any folder of code, docs, papers, or images into a queryable knowledge graph. Use this skill whenever the user asks about repo architecture, cross-module dependencies, design rationales, or wants a 'map' of the project. Always trigger /graphify if a graphify-out/ directory exists to ensure context is loaded from the graph."
---

# Graphify

A multimodal knowledge graph pipeline for codebases and document collections.

## Orchestration Logic

When the user invokes `/graphify` or asks a structural question, follow this streamlined pipeline.

### 1. Initialization
- If no path is given, use `.`.
- Identify the correct Python interpreter and ensure `graphifyy` is installed.
- Save the interpreter path to `graphify-out/.graphify_python`.

### 2. Detection
- Run `scripts/detect.py <path>`.
- Print a summary of the corpus (counts for code, docs, papers, images, etc.).
- If the corpus is too large (>2M words), ask the user for a specific subfolder.

### 3. Extraction
#### Part A: Structural (Free)
- Run `scripts/extract_ast.py`. This extracts imports and class/function hierarchies.

#### Part B: Semantic (LLM)
- Use the **Agent tool** with `subagent_type="general-purpose"` to process files in parallel.
- Chunk files (20-25 per agent). Images get 1 agent each.
- **Prompt**: Extract entities, relationships (EXTRACTED/INFERRED/AMBIGUOUS), and rationales. Ensure Node IDs match the format `filename_entity` (lowercase, alphanumeric).
- Save each chunk to `graphify-out/.graphify_chunk_N.json`.
- After all agents finish, merge them into `graphify-out/.graphify_semantic.json`.

#### Part C: Merge
- Run `scripts/merge.py` to combine AST and Semantic fragments into `graphify-out/.graphify_extract.json`.

### 4. Build & Analyze
- Run `scripts/build.py <path>`. 
- This constructs the NetworkX graph, performs Leiden community detection, scores cohesion, and generates the initial `GRAPH_REPORT.md`.

### 5. Review & Refine
- **Labeling**: Read the community nodes in `graphify-out/.graphify_analysis.json`. Assign a 2-5 word name to each community.
- **Finalize**: Write labels to `graphify-out/.graphify_labels.json` and run `scripts/label.py <path>` to update the report.

### 6. Export & Cleanup
- Export to desired formats (HTML is default, Obsidian/Wiki/Neo4j on request).
- Run `scripts/cleanup.py` to save the manifest for incremental updates and clear temp files.

## Guidelines
- **Progressive Disclosure**: Refer to `references/schema.md` for JSON formats and `references/cli.md` for advanced flags.
- **Honesty**: Mark uncertain connections as AMBIGUOUS in the graph.
- **Guidance**: After building, pick the most interesting "Suggested Question" from the report and offer to trace it using `/graphify query`.

---

## Skill Resources
- `scripts/`: Python logic for each pipeline stage.
- `evals/`: Test cases for performance benchmarking.
- `references/`: Detailed documentation for internal modules.
