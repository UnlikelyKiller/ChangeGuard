# Plan: Language-Aware Symbol Extraction

### Phase 1: Data Model and Interface Setup
- [ ] Task 1.1: Create `src/index/mod.rs`, `src/index/symbols.rs`, and `src/index/languages/mod.rs`. Expose them correctly through `src/lib.rs` or `src/main.rs`.
- [ ] Task 1.2: Define `Symbol` and `SymbolKind` data models in `src/index/symbols.rs` deriving `Serialize` and `Deserialize` (with `camelCase` renaming to match existing schema).
- [ ] Task 1.3: Update `ChangedFile` structure in `src/impact/packet.rs` to include `pub symbols: Option<Vec<Symbol>>`.
- [ ] Task 1.4: Define the generic parsing interface in `src/index/languages/mod.rs` (e.g., `parse_symbols(path: &Path, content: &str) -> miette::Result<Option<Vec<Symbol>>>`).

### Phase 2: Tree-Sitter Language Parsers
- [ ] Task 2.1: Implement Rust symbol extractor in `src/index/languages/rust.rs` using `tree-sitter-rust`. Add basic unit tests for public/private functions and structs.
- [ ] Task 2.2: Implement TypeScript symbol extractor in `src/index/languages/typescript.rs` using `tree-sitter-typescript`. Add basic unit tests for exported functions and classes.
- [ ] Task 2.3: Implement Python symbol extractor in `src/index/languages/python.rs` using `tree-sitter-python`. Add basic unit tests for classes and defs.
- [ ] Task 2.4: Implement file extension-based dispatcher in `src/index/languages/mod.rs` routing the content to the appropriate parser.

### Phase 3: Integration and Error Resilience
- [ ] Task 3.1: Update the scan/impact generation flow to read changed file contents from disk and invoke the language dispatcher to populate the `symbols` field.
- [ ] Task 3.2: Implement graceful degradation for parse/read failures. If reading or parsing fails, log a warning via `tracing::warn!`, default the file's `symbols` field to `None`, and ensure the overall scan does not crash.
- [ ] Task 3.3: Write an integration test validating that the `ImpactPacket` JSON contains correctly extracted symbols from mock Rust, TypeScript, and Python files.
- [ ] Task 3.4: Run full verification using `cargo test -j 1`. Ensure `cargo clippy` and `cargo fmt` pass, resolving any warnings.