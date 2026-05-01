# Specification: Track E1-4 Entry Point Identification

## Overview

This track adds entry point detection to ChangeGuard. It identifies "first breath" functions -- `main()`, HTTP handlers, CLI entry points, test entry points, and public API functions -- so that downstream features can reason about call chains and blast radius. Entry points are labeled via an `entrypoint_kind` column on `project_symbols` and integrated with risk scoring (entry points contribute up to 35 points within the API Surface category, per expansion plan Section 4.2) and LSP CodeLens (showing "Entry Point" / "Handler" / "Public API" / "Test" labels).

This track depends on E1-1 for the `project_symbols` table (the `entrypoint_kind` column is on that table) and indexing infrastructure. **E1-1 owns M15**; E1-4's schema additions are included in E1-1's migration.

## Components

### 1. Database Schema: `entrypoint_kind` Column on `project_symbols` (`src/state/migrations.rs`)

The `project_symbols` table (created in M15 by E1-1) gains an `entrypoint_kind` column. Since E1-1 and E1-4 share the same migration M15, the column is already included in the initial `project_symbols` table creation as defined in the E1-1 spec. The column uses these values:

- `ENTRYPOINT`: The primary entry point of a binary or service (e.g., `fn main()`, `if __name__ == "__main__"`)
- `HANDLER`: A request/event handler (e.g., `#[actix_web::get("/path")]`, Express route handler)
- `PUBLIC_API`: A public library function that is not a route handler. Public library functions that are not entry points or handlers should be labeled `PUBLIC_API`.
- `TEST`: A test entry point (e.g., `#[test]`, `#[tokio::test]`, `describe()/it()` blocks)
- `INTERNAL`: All other symbols (default)

The `entrypoint_kind` column is defined in the E1-1 spec's `project_symbols` CREATE TABLE as `entrypoint_kind TEXT NOT NULL DEFAULT 'INTERNAL'`. An index exists on it:
```sql
CREATE INDEX IF NOT EXISTS idx_project_symbols_entrypoint
    ON project_symbols(entrypoint_kind);
```

**Note to E1-1 implementer:** The `project_symbols` table definition in M15 must include the `entrypoint_kind` column from the start. If E1-1 has already been implemented without this column, use `ALTER TABLE project_symbols ADD COLUMN entrypoint_kind TEXT NOT NULL DEFAULT 'INTERNAL'` in a subsequent migration. Coordinate with E1-1.

### 2. Entrypoint Enum (`src/index/entrypoint.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntrypointKind {
    Entrypoint,
    Handler,
    PublicApi,
    Test,
    Internal,
}

impl Default for EntrypointKind {
    fn default() -> Self {
        Self::Internal
    }
}
```

**PUBLIC_API** is for public library functions that are not route handlers. Per the expansion plan: "Public library functions that are not route handlers should be labeled PUBLIC_API." When no identifiable entry points are found (e.g., a library crate), all `pub fn` symbols should be labeled `PUBLIC_API` with `confidence = 0.7` and `evidence = 'no entry point found; all public symbols labeled as public API'`.

### 3. Entry Point Detection (`src/index/entrypoint.rs`)

The detector examines each symbol in `project_symbols` (after the initial extraction by E1-1) and classifies it as `ENTRYPOINT`, `HANDLER`, `PUBLIC_API`, `TEST`, or `INTERNAL`.

**Detection strategies per language:**

**Rust:**
- `ENTRYPOINT`: Any function named `main` that is `pub` or in the crate root. Specifically: `fn main()` in a binary crate.
- `HANDLER`: Functions annotated with `#[actix_web::get(...)]`, `#[actix_web::post(...)]`, `#[actix_web::put(...)]`, `#[actix_web::delete(...)]`, `#[actix_web::patch(...)]`, `#[axum::routing::get(...)]`, `#[rocket::get(...)]`, `#[rocket::post(...)]`, etc. Also: `#[tokio::main]` on an async `main` function (this is `ENTRYPOINT`, not `HANDLER`).
- `TEST`: Functions annotated with `#[test]` or `#[tokio::test]`.

**TypeScript:**
- `ENTRYPOINT`: Functions in files named `main.ts`, `index.ts`, `server.ts`, `app.ts`. Also: exported default functions.
- `HANDLER`: Functions passed as arguments to `app.get()`, `app.post()`, `app.put()`, `app.delete()`, `router.get()`, `router.post()`, etc. Also: functions decorated with `@Get(...)`, `@Post(...)`, `@Put(...)`, `@Delete(...)` (NestJS-style decorators).
- `TEST`: Functions inside `describe()`, `it()`, `test()` blocks. Also: functions named `test*` or `*Test` in `__tests__/` directories.

**Python:**
- `ENTRYPOINT`: Code inside `if __name__ == "__main__":` blocks. Also: `app = FastAPI(...)`, `app = Flask(...)` at module level.
- `HANDLER`: Functions decorated with `@app.get(...)`, `@app.post(...)`, `@app.route(...)` (Flask), or `@router.get(...)` (FastAPI APIRouter).
- `TEST`: Functions named `test_*` or `*_test` in files named `test_*.py` or `*_test.py`.

**Detection approach:**
- Entry point detection operates on the tree-sitter AST, not on raw text matching.
- For Rust: Examine the attribute list preceding each function item. Use tree-sitter queries to find `attribute_item` nodes with specific attribute paths.
- For TypeScript: Examine the call expression context and decorator list. Use tree-sitter queries to find call expressions with `app.*` or `router.*` receivers.
- For Python: Examine `if_statement` nodes with `name == "__main__"` comparisons. Examine decorator lists on function definitions.

**Implementation detail:** Since the initial symbol extraction in E1-1 already runs tree-sitter parsing, the entry point detection should be a post-processing step that re-reads the file's AST (or caches it) to classify symbols. This avoids parsing each file twice.

### 4. Entrypoint Classification Pipeline

The classification pipeline runs after the initial symbol extraction (E1-1's `ProjectIndexer::full_index`):

```rust
pub fn classify_entrypoints(&self, files: &[ProjectFile]) -> Result<EntrypointStats>
```

**`EntrypointStats`:**
```rust
pub struct EntrypointStats {
    pub entrypoints: usize,
    pub handlers: usize,
    pub public_apis: usize,
    pub tests: usize,
    pub internal: usize,
}
```

The pipeline:
1. For each `ProjectFile`, re-read the source file and run language-specific entry point detection.
2. For each symbol in the module, determine its `EntrypointKind`.
3. Update the `entrypoint_kind` column in `project_symbols` for each classified symbol.
4. Update the `is_entrypoint` column in `project_files` to 1 if any symbol in the file is `ENTRYPOINT` or `HANDLER`.
5. Return `EntrypointStats`.

### 5. `index` Command Integration (`src/commands/index.rs`)

Extend `execute_index` to call `ProjectIndexer::classify_entrypoints()` after source file indexing. Print entrypoint stats (e.g., "Entrypoints: 3 entrypoints, 12 handlers, 45 tests, 800 internal").

### 6. Risk Scoring Integration (`src/impact/analysis.rs`)

Modify `analyze_risk()` to query `project_symbols` for the entrypoint classification of changed symbols:

- If a changed symbol has `entrypoint_kind = 'ENTRYPOINT'`, its risk contribution falls within the API Surface category (max 35 points, per expansion plan Section 4.2). Add `"Entry point changed: {symbol_name}"` to `risk_reasons`.
- If a changed symbol has `entrypoint_kind = 'HANDLER'`, its risk contribution falls within the API Surface category (max 35 points). Add `"Handler changed: {symbol_name}"` to `risk_reasons`.
- If a changed symbol has `entrypoint_kind = 'PUBLIC_API'`, its risk contribution falls within the API Surface category (max 35 points). Add `"Public API changed: {symbol_name}"` to `risk_reasons`.
- If a changed symbol has `entrypoint_kind = 'TEST'`, no additional risk weight (test changes are not elevated in risk). The entrypoint classification is informational for test mapping (E4-1).
- If a changed file has no `project_symbols` data, skip entrypoint-based risk adjustment (graceful degradation).

**Note on risk weight hierarchy (category-capped per Section 4.2):**
- `ENTRYPOINT` and `HANDLER`: contribute up to 35 points in the API Surface category
- `PUBLIC_API`: contributes up to 35 points in the API Surface category
- Infrastructure/Config directories (E1-3): contribute up to 25 points in the Runtime/Config category

### 7. LSP Integration (`src/commands/daemon.rs`) — Deferred to E3

**Note:** LSP integration for entry point labels (CodeLens, hover) and symbol-level diagnostics is deferred to the E3 milestone per the expansion plan. E1-4 provides the data foundation (`entrypoint_kind` column on `project_symbols`) but does not implement LSP features.

When implemented in E3, the LSP daemon will:

- **CodeLens:** Show "Entry Point" above symbols with `entrypoint_kind = 'ENTRYPOINT'`, "Handler" above symbols with `entrypoint_kind = 'HANDLER'`, "Public API" above symbols with `entrypoint_kind = 'PUBLIC_API'`, and "Test" above symbols with `entrypoint_kind = 'TEST'`.
- **Hover:** When hovering over an entry point, show its classification in the hover information.
- **Diagnostics:** When `project_symbols` has `line_start` and `line_end` data, use actual symbol positions instead of (0,0) for diagnostics (fixing Gap 5.5).

## Constraints

- **Tree-sitter based detection:** All entry point detection must use tree-sitter AST queries, not regex or raw text matching. This ensures accuracy and handles formatting variations.
- **No false positives as fact:** If an entry point classification is uncertain (e.g., a function named `main` that might not be a true entry point), mark it with a `confidence` flag (future enhancement) or use the most conservative classification (`INTERNAL`). The current implementation classifies by pattern match with high confidence; uncertain matches should be `INTERNAL`.
- **Graceful degradation:** If `project_symbols` has no `entrypoint_kind` data, risk scoring works without entrypoint adjustments. If the LSP daemon has no symbol data, diagnostics fall back to (0,0).
- **Performance:** Entry point classification must add less than 1 second to the total `index` time for a 2,000-file repo.

## Edge Cases

- **No identifiable entry points (library crate):** If no `fn main()`, no `if __name__ == "__main__"`, and no handler attributes are found, the crate is a library. All `pub fn` symbols should be labeled `PUBLIC_API` with `confidence = 0.7` and `evidence = 'no entry point found; all public symbols labeled as public API'`. Do not mark them as `INTERNAL` (they are public API surface) and do not mark them as `HANDLER` (they are not route handlers).
- **Macro-generated entry points:** `#[derive(...)]`, `#[serde(...)]`, `#[async_trait]` and similar attribute macros are not entry points. Skip them. Only `#[test]`, `#[tokio::test]`, and HTTP framework attributes are entry-point-relevant.
- **Multiple entry points in one file:** Mark all that match. A file can have multiple `#[test]` functions (all marked `TEST`) and a `main` function (marked `ENTRYPOINT`).
- **`#[tokio::main]` on async `main`:** This is an `ENTRYPOINT`, not a `HANDLER`. The `#[tokio::main]` attribute is a convenience wrapper around `main`.
- **TypeScript files with no default export:** Files named `index.ts` are `ENTRYPOINT` candidates even without a default export, because they serve as package entry points.
- **Python `if __name__ == "__main__":` with no function calls:** The block exists but only calls other functions. Mark the module as having an `ENTRYPOINT` but don't elevate any specific function.
- **Express route handlers passed inline:** `app.get("/users", (req, res) => { ... })` -- the handler is an anonymous arrow function. Mark it as `HANDLER` with the symbol name as the route path (e.g., `GET /users`).

## Acceptance Criteria

1. `changeguard index` labels entry points in Rust, TypeScript, and Python files with `entrypoint_kind` values of `ENTRYPOINT`, `HANDLER`, `PUBLIC_API`, `TEST`, or `INTERNAL`.
2. `changeguard impact` gives elevated risk within the API Surface category (max 35 points, per expansion plan Section 4.2) to changed entry points (`ENTRYPOINT`, `HANDLER`, `PUBLIC_API`) and includes a reason string.
3. `changeguard impact` does not elevate risk for `TEST` entry points.
4. Library crates (with no `main` function) label all `pub fn` symbols as `PUBLIC_API` with `confidence = 0.7` and appropriate evidence. They are not labeled `INTERNAL` or `HANDLER`.
5. Multiple entry points in one file are all correctly labeled.
6. `#[tokio::main]` on async `main` is classified as `ENTRYPOINT`, not `HANDLER`.
7. The `entrypoint_kind` column (not `entrypoint`) is used on `project_symbols`.

## Verification Gate

- **Unit tests:** Rust entry point detection: `fn main()`, `#[tokio::main]`, `#[actix_web::get("/path")]`, `#[test]`, `#[tokio::test]`.
- **Unit tests:** TypeScript entry point detection: `main.ts`, `index.ts`, `app.get()`, `describe()/it()` blocks.
- **Unit tests:** Python entry point detection: `if __name__ == "__main__":`, `@app.route(...)`, `def test_*():`.
- **Unit tests:** `EntrypointKind` serialization/deserialization (including `PUBLIC_API` variant).
- **Integration test:** `changeguard index` on a Rust binary crate labels `main()` as `ENTRYPOINT`.
- **Integration test:** `changeguard index` on an Express project labels route handlers as `HANDLER`.
- **Integration test:** `changeguard index` on a Rust library crate labels `pub fn` symbols as `PUBLIC_API` with `confidence = 0.7`.
- **Integration test:** `changeguard impact` on a changed entry point produces "Entry point changed" in risk reasons within the API Surface category.
- **Regression test:** Existing `impact` tests pass without `project_symbols` data (graceful degradation).

## Definition of Done

- [ ] All acceptance criteria pass
- [ ] All unit tests pass
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test` passes with no regressions
- [ ] No deviations from this spec without documented justification
- [ ] Migration M15 applied cleanly to existing ledger.db
- [ ] `changeguard index` on a fixture repo produces non-empty project_symbols