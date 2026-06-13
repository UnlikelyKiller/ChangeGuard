# Track S4: Automated SCIP Orchestration

## Objective
Automate the generation and ingestion of Semantic Code Intelligence Protocol (SCIP) indices to provide compiler-grade resolution of ambiguous call edges without manual developer intervention.

## Requirements
1. **Auto-Detection:** Detect the presence of native SCIP indexers (e.g., `rust-analyzer` for Rust, `scip-typescript` for TS/JS) on the system `PATH`.
2. **Execution Orchestration:** Add a `--auto-scip` flag to the `changeguard index` command that seamlessly spawns the detected indexer (e.g., `rust-analyzer scip .`), waits for generation, and automatically feeds the output into the existing `--scip` ingestion path.
3. **Graceful Degradation:** If the indexer fails or is not installed, the command should gracefully fall back to native Tree-Sitter parsing without failing the entire indexing process.
4. **Performance Preservation:** SCIP generation is slow. It must be explicitly triggered via the `--auto-scip` flag and should never block the fast path of `changeguard scan --impact`.

## API Contracts
*   `changeguard index --auto-scip`: Triggers automated detection, generation, and ingestion of SCIP indices for the current repository.

## Testing Strategy
*   Unit tests for path detection and language identification.
*   Integration tests mocking the `rust-analyzer` binary to verify subprocess orchestration and fallback behavior.
