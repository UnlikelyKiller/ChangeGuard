# Implementation Plan - Track S2: Precise LSP-Based Indexing (SCIP/LSIF)

## Goal
Ingest SCIP indices for compiler-grade precision in navigation and impact analysis, merging external LSP-quality symbols into ChangeGuard's unified symbol table.

## Proposed Changes

### 1. SCIP Protobuf Ingestion [src/scip/ingest.rs] [NEW]
- Add `scip` crate dependency for Protobuf parsing.
- Implement `ScipIndex::load(path: &Path) -> Result<Self>` to read and validate SCIP files.
- Compute and store BLAKE3 hash of the SCIP file to enable incremental re-ingestion.

### 2. Unified Symbol Table [src/scip/symbol_table.rs] [NEW]
- Implement `ScipSymbolMapper` that converts SCIP symbols to ChangeGuard's `project_symbols` schema.
- Ensure symbol IDs are deterministic and compatible with Tree-sitter-generated IDs.
- Upsert symbols into CozoDB/SQLite, preserving existing Tree-sitter entries where SCIP does not overlap.

### 3. Incremental Ingestion & Stale Detection [src/scip/stale_detect.rs] [NEW]
- Store last-ingested SCIP hash in the local state DB; skip re-processing if unchanged.
- Validate referenced source file hashes against the working tree; emit warnings for mismatches.
- Flag partial indices (files in SCIP missing from disk, or vice versa) as non-fatal diagnostics.

### 4. Path Normalization [src/scip/path_normalize.rs] [NEW]
- Lexicalize SCIP relative paths to the repo root, handling Windows/Unix separator differences.
- Reject or warn on paths that escape the repo root.

### 5. Fallback Integration [src/index/analysis.rs]
- Update impact and viz queries to prefer SCIP symbols when available, falling back to Tree-sitter for un-indexed modules.
- Ensure `changeguard impact` uses the same symbol IDs regardless of origin.

### 6. CLI Integration [src/commands/index.rs]
- Add `--scip <path>` flag to the `index` command.
- Wire ingestion flow through `ScipIndex` -> `ScipSymbolMapper` -> storage.

## Verification Plan

### Automated Tests
- `cargo test`: All existing tests pass.
- Unit tests for `ScipSymbolMapper` with synthetic SCIP fixtures.
- Unit tests for path normalization across Windows and Unix paths.
- Integration test in `tests/scip_integration.rs` verifying end-to-end ingestion and query.

### Manual Verification
- Run `changeguard index --scip` on a real language-indexer output (e.g., Rust Analyzer SCIP) and verify symbol accuracy in `viz`.

## Definition of Done (DoD)

- [ ] **Ingestion**: `changeguard index --scip <path>` successfully parses and stores SCIP symbols.
- [ ] **Unification**: SCIP symbols are merged with Tree-sitter symbols in CozoDB without collisions.
- [ ] **Precision**: `viz` and `impact` use SCIP-derived symbols for Go-to-Definition and Find-References.
- [ ] **Resilience**: Partial or stale SCIP indices emit warnings and fall back to Tree-sitter.
- [ ] **Test Coverage**: Unit tests for mapper, normalization, and stale detection; integration test for end-to-end flow.
- [ ] **Zero Regression**: All existing `cargo test` suites pass unchanged.
- [ ] **Clean CI**: `cargo fmt`, `cargo clippy`, and `cargo test` pass with zero warnings.
