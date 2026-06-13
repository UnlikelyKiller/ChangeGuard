## Plan: Federated Dependency Matching Cache (Track E0-3)

### Phase 1: Create Caching Struct
- [ ] Task 1.1: Define `SymbolMatcher` struct with a `HashMap<String, Option<Regex>>` cache in `src/federated/scanner.rs`.
- [ ] Task 1.2: Implement `SymbolMatcher::new()` and `SymbolMatcher::matches(&mut self, symbol: &str, content: &str) -> bool`.
- [ ] Task 1.3: Ensure `matches` prevents allocation on cache hit and properly escapes symbols for word-boundary matching.

### Phase 2: Integrate with Scanner
- [ ] Task 2.1: Modify `scan_dependency_dir` signature in `FederatedScanner` to include `matcher: &mut SymbolMatcher`.
- [ ] Task 2.2: Update `scan_dependency_dir` to pass `matcher` in its recursive calls and use it for evaluating matches instead of `symbol_matches_content`.
- [ ] Task 2.3: Update `discover_dependencies_in_current_repo` to instantiate `SymbolMatcher` and pass it to `scan_dependency_dir`.
- [ ] Task 2.4: Update `discover_dependencies` to instantiate `SymbolMatcher` and use it inside the `local_packet.changes` loop.
- [ ] Task 2.5: Remove the now-unused standalone `symbol_matches_content` function.

### Phase 3: Testing and Verification
- [ ] Task 3.1: Update `symbol_matches_content_unit_tests` in `src/federated/scanner.rs` to instantiate `SymbolMatcher` and assert behaviors.
- [ ] Task 3.2: Verify tests compile and pass via `cargo nextest run --lib --workspace` (focusing on `federated` module tests).
- [ ] Task 3.3: Run `cargo clippy --all-targets --all-features -- -D warnings` to ensure no warnings or borrow-checker issues were introduced.