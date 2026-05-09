# Track S2: Precise LSP-Based Indexing (SCIP/LSIF)

## Overview
Achieve compiler-perfect navigation and impact analysis by ingesting external SCIP (Symbolic Code Index Protocol) indices from language-specific indexers. Merge SCIP symbols with ChangeGuard's existing symbol table so that navigation and impact queries use the most precise source of truth available.

## Objectives
- Ingest SCIP protobuf indices using the `scip` crate.
- Map SCIP symbols into ChangeGuard's unified `project_symbols` table (SQLite/CozoDB) with stable symbol IDs.
- Support incremental ingestion: re-process only when the SCIP file's BLAKE3 hash changes.
- Gracefully fall back to Tree-sitter symbols for files not covered by SCIP.
- Detect stale indices by comparing SCIP file hashes against the current working tree.

## Success Criteria
- `changeguard index --scip <path>` successfully ingests a SCIP file and populates symbols.
- SCIP-derived symbols are queryable via `changeguard impact` and `changeguard viz` with the same IDs as Tree-sitter symbols.
- Stale or partial SCIP indices trigger warnings but do not block analysis.
- Cross-platform path normalization ensures SCIP relative paths resolve correctly to the repo root.

## Architecture
- `src/scip/mod.rs`: SCIP ingestion module and public API.
- `src/scip/ingest.rs`: Protobuf parsing, symbol extraction, and hash-based incremental checks.
- `src/scip/symbol_table.rs`: Mapping between SCIP symbols and ChangeGuard's unified symbol IDs.
- `src/scip/stale_detect.rs`: File-hash validation and stale-index warnings.
- `src/scip/path_normalize.rs`: Cross-platform lexical path normalization.
- `src/commands/index.rs`: CLI wrapper for `--scip` ingestion.
