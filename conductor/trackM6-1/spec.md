# Specification: Track M6-1 — OpenAPI Spec Parser & Index Storage

## Objective
Build the OpenAPI/Swagger spec parsing and indexing pipeline: parse spec files into a flat list of endpoints, embed each endpoint's description text, store in `api_endpoints` + `embeddings` tables, and provide a `changeguard index --contracts` CLI flag.

## Components

### 1. `ApiEndpoint` Type (`src/contracts/parser.rs`)

```rust
pub struct ApiEndpoint {
    pub spec_path: String,
    pub method: String,
    pub path: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub operation_id: Option<String>,
    pub embed_text: String,
}
```

### 2. Spec Parser (`src/contracts/parser.rs`)

```rust
pub fn parse_spec(file_path: &Utf8Path) -> Result<Vec<ApiEndpoint>>
```

- Support OpenAPI 3.x (YAML and JSON) and Swagger 2.x (JSON and YAML)
- For each path + method combination:
  - Extract: `method`, `path`, `summary`, `description`, `tags`, `operationId`
  - Construct `embed_text`: `summary + " " + description + " " + tags.join(" ")`
  - If both `summary` and `description` are absent: use `{method} {path}`
  - Skip endpoints where embed text < 10 characters
- YAML parsing via `serde_yaml` (add to `Cargo.toml` as new dependency)
- Handle `$ref` references with a depth limit of 20 (cycle detection)
- Parse failure for a single file: log `WARN` and continue; never abort

### 3. Contract Index Storage (`src/contracts/index.rs`)

```rust
pub fn index_contracts(
    config: &Config,
    conn: &Connection,
) -> Result<ContractsIndexSummary>

pub struct ContractsIndexSummary {
    pub specs_parsed: usize,
    pub endpoints_new: usize,
    pub endpoints_skipped: usize,
    pub endpoints_deleted: usize,
}
```

For each spec file matching `config.contracts.spec_paths`:
1. Parse into `Vec<ApiEndpoint>`
2. For each endpoint, compute `blake3(embed_text)` as `content_hash`
3. Check `api_endpoints` for existing row at `(spec_path, method, path)`
4. If hash matches: skip → increment `endpoints_skipped`
5. If no row or hash differs: INSERT/UPDATE → increment `endpoints_new`. Call `embed_and_store` with `entity_type = "api_endpoint"`, `entity_id = "{spec_path}::{method}::{path}"`
6. After all specs processed: DELETE rows from `api_endpoints` for spec paths that no longer exist in config → increment `endpoints_deleted`. Also delete corresponding `embeddings` rows.

When `config.local_model.base_url` is empty: store endpoints in `api_endpoints` but skip embedding.

### 4. CLI Integration

Add `--contracts` flag to `IndexArgs` in `src/cli.rs`. When set, `execute_index()` calls `index_contracts()` and prints:

```
Contracts indexed: 2 specs, 45 new endpoints, 0 skipped, 3 deleted.
```

When `config.contracts.spec_paths` is empty:
```
No spec paths configured in [contracts].spec_paths — skipping contract index.
```

### 5. Module Declaration

Create `src/contracts/mod.rs` exporting `parser`, `index`, `matcher` submodules. Add `pub mod contracts;` to `src/lib.rs`.

## Test Specifications

| Test | Assertion |
|---|---|
| Parse OpenAPI 3.x YAML fixture | Correct endpoint count, embed_text populated |
| Parse Swagger 2.x JSON fixture | Correct method/path extraction |
| Parse spec with `$ref` | Resolved correctly; cycle at depth 21 returns `Err` |
| Parse malformed YAML | Returns `Err`, does not panic |
| Endpoint < 10 chars | Skipped |
| `index_contracts` fresh index | `endpoints_new > 0`, `endpoints_skipped = 0` |
| `index_contracts` re-index unchanged | `endpoints_skipped = N`, `endpoints_new = 0` |
| `index_contracts` spec removed from config | `endpoints_deleted > 0` |
| `index_contracts` `base_url = ""` | Completes, endpoints stored, no HTTP calls |

## Constraints & Guidelines

- **TDD**: Tests written before implementation.
- **New dependency**: `serde_yaml = "0.9"` added to `Cargo.toml`.
- **Sandboxed parsing**: Malformed specs log `WARN` and skip; never abort.
- **No write to spec files**: Spec files are read-only inputs.
- **Test fixtures**: Embed small OpenAPI 3.x and Swagger 2.x YAML/JSON snippets as test fixtures.

## Hardening Additions (not in original plan)

| Addition | Reason |
|---|---|
| Swagger 2.0 **YAML** support (plan §9 only says JSON) | Many real-world Swagger specs are YAML. Parser should accept both formats for 2.x specs. |
