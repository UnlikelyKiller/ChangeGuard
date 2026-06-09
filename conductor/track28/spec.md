# Track 28: Federated Intelligence (Cross-Repo) Specification

## 1. Objective
Support risk analysis and dependency mapping across multiple local repositories using a shared intelligence model. This phase (Phase 22) enables ChangeGuard to recognize when a change in Repo A impacts Repo B, utilizing a lightweight, deterministic, local-first approach.

## 2. Core Architecture

The federation model relies on explicitly exported schemas (`schema.json` or a lightweight SQLite export) rather than complex runtime RPC or multi-repo atomic commits. 

### Modules
- `src/federated/mod.rs`: The core federation engine. Orchestrates discovery and aggregation.
- `src/federated/schema.rs`: JSON/struct definitions for cross-repo interfaces (exported functions, public types, module boundaries).
- `src/federated/scanner.rs`: Safe filesystem traversal logic for discovering sibling repositories.
- `src/federated/impact.rs`: Cross-repo impact resolution. Merges local impact packets with federated dependencies.
- `src/commands/federate.rs`: Command handler for `changeguard federate`.

### CLI Command (`changeguard federate`)
- `changeguard federate export`: Generates/updates the `.changeguard/schema.json` containing the repository's public interface.
- `changeguard federate scan`: Discovers linked or sibling repositories and outputs a summary of cross-repo dependencies.
- `changeguard federate status`: Displays the current federation footprint (which repos rely on this one, and which this one relies on).

## 3. Engineering Constraints (from docs/Engineering.md & docs/Plan-Phase2.md)

### 3.1 Security & Threat Model
- **No Symlink Escapes**: The scanner MUST NOT follow symlinks when traversing `../`. Use `std::fs::symlink_metadata`.
- **Path Confinement**: After resolving `../`, the scanner MUST canonicalize the result and verify it is exactly one directory level above the repo root. Reject paths that escape further.
- **Privacy (Secret Redaction)**: Exported schemas MUST strip all values from detected environment/config patterns. Only keys/names are exported.
- **Cycle Detection**: Federation depth MUST be strictly capped at 1 (direct siblings only) to prevent unbounded recursion or infinite loops. No transitive federation in v1.

### 3.2 Determinism Contract
- **Stable Sorting**: All cross-repo file paths, symbol lists, and impact dependencies must be sorted deterministically (alphabetically by repo name, then by path).
- **Graceful Degradation**: If a sibling repository has a malformed `schema.json`, log a clear diagnostic warning and skip it. DO NOT abort the local analysis.
- **Explicit Versioning**: The `schema.json` MUST contain a `schema_version` field to ensure backward compatibility as the data model evolves.

### 3.3 Idiomatic Rust (SRP & Error Visibility)
- **SRP Boundaries**: `federated/scanner.rs` ONLY reads schemas. It does not write to siblings. `federated/impact.rs` ONLY computes overlap, it does not perform AST parsing.
- **Error Handling**: Use `miette::Result` for user-facing commands and `anyhow::Result` for internal logic. No `unwrap()` or `expect()` in production code. Failures must explain *what* failed (e.g., "Failed to parse sibling schema"), *where* (e.g., "../repo-b/.changeguard/schema.json"), and *next steps*.

## 4. Data Model

### Local SQLite Impact
Federation data does not overwrite local AST intelligence. Cross-repo edges are stored in local SQLite under a separate table:
- `federated_links`: `(id, sibling_name, sibling_path, last_scanned_at)`
- `federated_dependencies`: `(id, local_symbol, sibling_name, sibling_symbol)`

### The Schema Export (`schema.json`)
```json
{
  "schema_version": "1.0",
  "repo_name": "auth-service",
  "public_interfaces": [
    {
      "symbol": "authenticate_user",
      "file": "src/auth.rs",
      "type": "function"
    }
  ]
}
```

## 5. User Workflow
1. Developer works on `Repo A` and runs `changeguard federate export`.
2. Developer switches to `Repo B` (a sibling directory).
3. Running `changeguard scan` or `changeguard impact` in `Repo B` automatically detects `../Repo A/.changeguard/schema.json`.
4. If `Repo B` consumes an interface from `Repo A` that recently changed, `Repo B`'s impact packet includes a `FederatedImpact` warning, indicating tests in `Repo B` should be verified.
