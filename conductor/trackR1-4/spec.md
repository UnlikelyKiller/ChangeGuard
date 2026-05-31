# Track R1-4: Project Index Decomposition

## Objective
Decompose the monolithic `src/index/project_index.rs` (77KB) into modular components: `orchestrator.rs`, `git_worker.rs`, `ast_worker.rs`, and `graph_worker.rs`.

## Requirements
- Split functionality into domain-specific workers.
- Maintain the `ProjectIndex` public API used by `src/commands/index.rs`.
- Optimize for parallel execution of AST extraction and Graph insertion where safe.
- Follow project standards: `miette` for errors, zero `unwrap()`, TDD methodology.

## Design Details
- `src/index/orchestrator.rs`: Contains the main `ProjectIndex` struct and orchestrates calls to the workers.
- `src/index/git_worker.rs`: Handles git log parsing, file walking, and ignore resolution.
- `src/index/ast_worker.rs`: Handles AST extraction. Should be optimized for parallel execution (e.g., using `rayon`).
- `src/index/graph_worker.rs`: Handles Knowledge Graph interactions and insertion. Optimized for parallel batch insertion if safe.
- `src/index/project_index.rs` will be removed, or repurposed as a module file (`src/index/mod.rs` update) exporting `ProjectIndex`.