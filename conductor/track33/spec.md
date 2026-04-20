# Track 33: Federated Intelligence Completion

## 1. Context and Objective
Track 28 laid the groundwork for Phase 2 Federated Intelligence, but left the implementation in a partial state with generic placeholders and missing safety boundaries. The goal of Track 33 is to complete the federated intelligence engine by implementing true dependency edge resolution, strict schema validation, path confinement, and user-visible diagnostics, addressing all Track 28 findings from `docs/audit3.md`.

## 2. Requirements

### 2.1 Schema Validation and Safety (`src/federated/scanner.rs` & `schema.rs`)
- **Version Validation**: Reject `schema.json` files where `schema_version` does not match the supported version (e.g., `"1.0"`).
- **JSON Safety**: Wrap the `serde_json::from_str` call in a `std::panic::catch_unwind` block. Malformed JSON must not panic the CLI.
- **Path Confinement**: Discovered sibling paths must be canonicalized and explicitly verified to reside exactly one level above the local repository root. Paths that escape (e.g., via `..` inside the sibling name) must be rejected.
- **Sibling Cap**: Implement a configurable sibling scan limit (default: 20 siblings).
- **Diagnostic Visibility**: Parsing failures and path violations must be accumulated and returned as explicit user-visible warnings (via `miette::Diagnostic` or a returned list of warnings), not just silently logged with `tracing::warn!`.

### 2.2 Dependency Edge Discovery (`src/commands/federate.rs` & `src/federated/scanner.rs`)
- **Dependency Population**: During `changeguard federate scan`, implement a discovery phase that maps local code to sibling interfaces. 
  - For each valid sibling schema, perform a lightweight scan (e.g., text search or utilizing the local index) of local source files for the sibling's `public_interfaces`.
  - Persist these edges using the existing `save_federated_dependencies(local_symbol, sibling_name, sibling_symbol)` function in `storage.rs`.
- **Export Redaction**: In `execute_federate_export()`, explicitly apply secret redaction to the exported schema before writing it to `.changeguard/schema.json`. 
- **Error Handling**: Remove the `unwrap_or("unknown")` when determining the repo name during export. Surface an actionable error if the repo name cannot be determined.

### 2.3 Cross-Repo Impact Resolution (`src/federated/impact.rs`)
- **Remove Placeholder**: Remove the generic `"Cross-repo monitoring active..."` warning.
- **Impact Logic**:
  - Load the stored `federated_dependencies` for known siblings.
  - Load the current `schema.json` from the sibling path on disk.
  - If a sibling's schema is missing or malformed, generate an impact reason for the user.
  - For each stored dependency `(local_symbol, sibling_symbol)`, verify that `sibling_symbol` still exists in the sibling's current schema.
  - If the symbol was removed or its signature changed (if tracked), flag the `local_symbol` as impacted with a specific reason: `"Cross-repo impact: Local symbol '{local_symbol}' depends on sibling '{sibling_name}' interface '{sibling_symbol}' which was removed."`

## 3. Engineering Standards
- **SRP**: `scanner.rs` strictly handles discovery and parsing. `impact.rs` strictly computes resolution.
- **Idiomatic Rust**: Ban `unwrap()` in production paths. Use `?` and return structured errors.
- **Determinism**: Sort sibling iteration, dependency resolution, and impact warnings alphabetically.

## 4. Testing Requirements
- Unit tests for schema validation (version mismatch, malformed JSON panics).
- Unit tests for path canonicalization and confinement (symlink escapes, `..` escapes).
- Integration test simulating sibling API breakage and verifying precise impact packet output.
- Verification of secret redaction during schema export.