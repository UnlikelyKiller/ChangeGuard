## Plan: Track E1-4 Entry Point Identification

### Phase 1: Database Schema
- [ ] Task 1.1: Ensure the `project_symbols` table in migration M15 includes the `entrypoint_kind TEXT NOT NULL DEFAULT 'INTERNAL'` column. Coordinate with E1-1 to include this column in the initial CREATE TABLE statement. If E1-1 has already shipped without it, add `ALTER TABLE project_symbols ADD COLUMN entrypoint_kind TEXT NOT NULL DEFAULT 'INTERNAL'` as a separate migration (M15.1 or M16). E1-1 owns M15.
- [ ] Task 1.2: Add `CREATE INDEX IF NOT EXISTS idx_project_symbols_entrypoint ON project_symbols(entrypoint_kind)` to the migration.
- [ ] Task 1.3: Update `test_all_tables_exist` and schema tests to verify the `entrypoint_kind` column exists with default 'INTERNAL'.
- [ ] Task 1.4: Write integration test verifying insert and query of `project_symbols` rows with various `entrypoint_kind` values (ENTRYPOINT, HANDLER, PUBLIC_API, TEST, INTERNAL).

### Phase 2: Domain Types
- [ ] Task 2.1: Create `src/index/entrypoint.rs` with `EntrypointKind` enum: `Entrypoint`, `Handler`, `PublicApi`, `Test`, `Internal`. Derive `Serialize`, `Deserialize`, `Clone`, `Debug`, `PartialEq`, `Eq`. Use `serde(rename_all = "SCREAMING_SNAKE_CASE")`. Implement `Default` returning `Internal`.
- [ ] Task 2.2: Define `EntrypointStats` struct with fields: `entrypoints` (usize), `handlers` (usize), `public_apis` (usize), `tests` (usize), `internal` (usize). Derive `Serialize`.
- [ ] Task 2.3: Add `pub mod entrypoint;` to `src/index/mod.rs`.

### Phase 3: Rust Entrypoint Detection
- [ ] Task 3.1: Implement `detect_rust_entrypoints(content: &str, symbols: &[Symbol]) -> Vec<(String, EntrypointKind)>` that parses Rust source using tree-sitter and classifies each symbol.
- [ ] Task 3.2: Detect `ENTRYPOINT`: function named `main` at the module root. Use tree-sitter query to find `function_item` nodes with name `main` and no containing `impl` block.
- [ ] Task 3.3: Detect `HANDLER`: functions annotated with HTTP framework attributes. Use tree-sitter to find `attribute_item` nodes matching `actix_web::get/post/put/delete/patch`, `axum::routing::get/post/put/delete`, `rocket::get/post/put/delete`. Return the function name and the route path from the attribute.
- [ ] Task 3.4: Detect `TEST`: functions annotated with `#[test]` or `#[tokio::test]`. Use tree-sitter to find `attribute_item` nodes with exact path `test` or `tokio::test`.
- [ ] Task 3.5: Detect `ENTRYPOINT` for `#[tokio::main]`: functions annotated with `#[tokio::main]` or `#[actix_web::main]` are `ENTRYPOINT`, not `HANDLER`.
- [ ] Task 3.6: Write unit tests for Rust entry point detection with fixture Rust files containing: `fn main()`, `#[tokio::main] async fn main()`, `#[actix_web::get("/users")]`, `#[test] fn test_foo()`, a library crate with `pub fn`, multiple `#[test]` functions in one file.

### Phase 4: TypeScript Entrypoint Detection
- [ ] Task 4.1: Implement `detect_typescript_entrypoints(content: &str, symbols: &[Symbol], file_path: &str) -> Vec<(String, EntrypointKind)>` that parses TypeScript source using tree-sitter and classifies each symbol.
- [ ] Task 4.2: Detect `ENTRYPOINT`: files named `main.ts`, `index.ts`, `server.ts`, `app.ts`. Mark their exported functions as `ENTRYPOINT`. Also detect `export default function` as `ENTRYPOINT`.
- [ ] Task 4.3: Detect `HANDLER`: functions passed as arguments to `app.get()`, `app.post()`, `router.get()`, etc. Use tree-sitter to find call expressions with `app.*` or `router.*` receivers. Also detect NestJS-style decorators `@Get(...)`, `@Post(...)`.
- [ ] Task 4.4: Detect `TEST`: functions inside `describe()`, `it()`, `test()` blocks. Use tree-sitter to find call expressions with these names. Also detect functions in `__tests__/` directories or named `test_*`/`*Test`.
- [ ] Task 4.5: Write unit tests for TypeScript entry point detection with fixture files containing: Express route handlers, NestJS decorators, `describe()/it()/test()` blocks, `index.ts` file entry point, inline anonymous handlers.

### Phase 5: Python Entrypoint Detection
- [ ] Task 5.1: Implement `detect_python_entrypoints(content: &str, symbols: &[Symbol], file_path: &str) -> Vec<(String, EntrypointKind)>` that parses Python source using tree-sitter and classifies each symbol.
- [ ] Task 5.2: Detect `ENTRYPOINT`: `if __name__ == "__main__":` blocks. Use tree-sitter to find `if_statement` nodes with `name == "__main__"` comparison. Also detect `app = FastAPI(...)` or `app = Flask(...)` at module level.
- [ ] Task 5.3: Detect `HANDLER`: functions decorated with `@app.get(...)`, `@app.post(...)`, `@app.route(...)` (Flask), `@router.get(...)` (FastAPI). Use tree-sitter to find `decorator` nodes matching these patterns.
- [ ] Task 5.4: Detect `TEST`: functions named `test_*` or `*_test` in files named `test_*.py` or `*_test.py`.
- [ ] Task 5.5: Write unit tests for Python entry point detection with fixture files containing: `if __name__ == "__main__":`, `@app.route(...)`, `def test_foo():`, `app = FastAPI(...)`.

### Phase 6: Entrypoint Classification Pipeline
- [ ] Task 6.1: Implement `ProjectIndexer::classify_entrypoints(&self, files: &[ProjectFile]) -> Result<EntrypointStats>` that iterates over all files, calls the language-specific detection function, and updates the `entrypoint_kind` column in `project_symbols`.
- [ ] Task 6.2: For each module, dispatch to the appropriate language detector: Rust, TypeScript, or Python. Skip unsupported languages.
- [ ] Task 6.3: After classifying, update `project_files.is_entrypoint` (or equivalent flag) to 1 for any file containing an `ENTRYPOINT` or `HANDLER` symbol.
- [ ] Task 6.4: Accumulate and return `EntrypointStats` (counts of each kind).
- [ ] Task 6.5: Write integration tests for `classify_entrypoints` using a temp directory with Rust, TypeScript, and Python fixture files.

### Phase 7: CLI Integration
- [ ] Task 7.1: Modify `execute_index` in `src/commands/index.rs` to call `ProjectIndexer::classify_entrypoints()` after source file indexing. Include `EntrypointStats` in the output.
- [ ] Task 7.2: Print entrypoint stats in human-readable output (e.g., "Entrypoints: 3 entrypoints, 12 handlers, 45 tests, 800 internal").
- [ ] Task 7.3: Write CLI integration test for `changeguard index` verifying entrypoint data is populated.

### Phase 8: Risk Scoring Integration
- [ ] Task 8.1: In `src/impact/analysis.rs`, add a helper function `get_entrypoint_kind(storage: &StorageManager, file_id: i64, symbol_name: &str) -> Option<EntrypointKind>` that queries `project_symbols` for the symbol's `entrypoint_kind` value. Use `file_id` (integer FK) instead of `file_path` per expansion plan constraint #7.
- [ ] Task 8.2: Modify `analyze_risk()` to call `get_entrypoint_kind()` for each changed symbol. If `ENTRYPOINT`, add risk within the API Surface category (max 35 points per expansion plan Section 4.2) and add `"Entry point changed: {symbol_name}"` to `risk_reasons`. If `HANDLER`, same category with `"Handler changed: {symbol_name}"`. If `PUBLIC_API`, same category with `"Public API changed: {symbol_name}"`. If `TEST`, no additional weight.
- [ ] Task 8.3: If `project_symbols` has no `entrypoint_kind` data, skip entrypoint-based risk adjustment (graceful degradation).
- [ ] Task 8.4: Write unit tests verifying: ENTRYPOINT gets API Surface category risk, HANDLER gets API Surface category risk, PUBLIC_API gets API Surface category risk, TEST gets no extra weight, INTERNAL gets no extra weight, missing data skips adjustment.
- [ ] Task 8.5: Write integration test: `changeguard impact` on a changed entry point produces "Entry point changed" in risk reasons within the API Surface category.

### Phase 9: LSP Integration — Deferred to E3

**Note:** LSP integration (CodeLens, hover, diagnostics position fix) is deferred to the E3 milestone per the expansion plan. E1-4 provides the `entrypoint_kind` data on `project_symbols`; E3 will consume it in the LSP daemon.

- [ ] Task 9.1 (E3): In `src/commands/daemon.rs` (LSP daemon), add CodeLens provider that queries `project_symbols` for the file being viewed and returns CodeLens entries for symbols with `entrypoint_kind != 'INTERNAL'`. Show "Entry Point" for `ENTRYPOINT`, "Handler" for `HANDLER`, "Public API" for `PUBLIC_API`, "Test" for `TEST`.
- [ ] Task 9.2 (E3): In the LSP hover provider, include entrypoint classification in hover information for symbols.
- [ ] Task 9.3 (E3): Fix the position (0,0) gap (5.5): When `project_symbols` has `line_start` and `line_end` data for a file, use those positions for diagnostics and CodeLens instead of (0,0). Fall back to (0,0) when no symbol data is available.
- [ ] Task 9.4 (E3): Write LSP tests (behind `#[cfg(feature = "daemon")]`) verifying CodeLens shows "Entry Point" for an entrypoint symbol and "Handler" for a handler symbol.
- [ ] Task 9.5 (E3): Write LSP test verifying diagnostics use actual symbol positions from `project_symbols` when available, and fall back to (0,0) when not.
- [ ] Task 9.6 (E3): Write regression test verifying LSP daemon still starts and serves requests when `project_symbols` is empty (graceful degradation).