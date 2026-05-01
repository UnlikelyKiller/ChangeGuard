## Plan: Track E0-3 Federated Dependency Matching

### Phase 1: Write Failing Tests for False Positives
- [ ] Task 1.1: Add test `test_no_false_positive_substring_match` in `src/federated/scanner.rs`: create a temp file containing `"let x = map_item;"` and a sibling schema with public symbol `"api"`. Assert that `discover_dependencies_in_current_repo` returns no edges.
- [ ] Task 1.2: Add test `test_word_boundary_match`: create a temp file containing `"pub fn local_fn() { let r = remote_api(); }"` and a sibling schema with symbol `"remote_api"`. Assert the edge `(local_fn, remote_api)` is discovered.
- [ ] Task 1.3: Add test `test_no_false_positive_common_word`: create a temp file containing `"let token = reset_token();"` and a sibling schema with symbol `"set"`. Assert no edges are returned.
- [ ] Task 1.4: Add test `test_word_boundary_match_with_qualified_path`: create a temp file containing `"use crate::RemoteApi;"` and a sibling schema with symbol `"RemoteApi"`. Assert the edge is discovered.
- [ ] Task 1.5: Run `cargo test` and confirm the false-positive tests fail (because `contains` matches substrings).

### Phase 2: Implement Word-Boundary Matching
- [ ] Task 2.1: Create the `symbol_matches(symbol: &str, content: &str) -> bool` function in `src/federated/scanner.rs`. Use `regex::Regex::new(&format!(r"\b{}\b", regex::escape(symbol)))` for word-boundary matching. If regex compilation fails, fall back to `content.contains(symbol)` and log `tracing::warn!("Regex compilation failed for symbol '{}', falling back to substring match", symbol)`.
- [ ] Task 2.2: Create a `SymbolMatcher` struct that caches compiled regexes in a `HashMap<String, Regex>`. The struct method `matches(&mut self, symbol: &str, content: &str) -> bool` checks the cache first, compiles and caches on miss, then calls `is_match`.
- [ ] Task 2.3: Replace `file_content.contains(symbol_to_find)` in `discover_dependencies` (the `local_packet.changes` loop) with `matcher.matches(symbol_to_find, &file_content)`.
- [ ] Task 2.4: Replace `file_content.contains(symbol_to_find)` in `scan_dependency_dir` with `matcher.matches(symbol_to_find, &file_content)`.
- [ ] Task 2.5: Construct the `SymbolMatcher` at the top of `discover_dependencies` and `discover_dependencies_in_current_repo`, passing it through to `scan_dependency_dir`.

### Phase 3: Verify False-Positive Tests Pass
- [ ] Task 3.1: Run the tests from Phase 1. Confirm `test_no_false_positive_substring_match` now passes.
- [ ] Task 3.2: Confirm `test_word_boundary_match` passes.
- [ ] Task 3.3: Confirm `test_no_false_positive_common_word` passes.
- [ ] Task 3.4: Confirm `test_word_boundary_match_with_qualified_path` passes.
- [ ] Task 3.5: Confirm the original `discovers_dependencies_outside_latest_packet` test still passes.

### Phase 4: Import-Based Matching Enhancement (Already Implemented)
- [ ] Task 4.1: The `symbol_imported` function has already been implemented in `src/federated/scanner.rs`. It uses `extract_import_export` from `src/index/references.rs` to check if a symbol appears in the local file's imports. Both `discover_dependencies` and `scan_dependency_dir` use `matches_import || matches_word` logic. Verify this is working correctly and document in code comments if needed.
- [ ] Task 4.2: Verify that `SymbolMatcher` caching (Component 4) is integrated with the import-based matching. If the current implementation does not use `SymbolMatcher`, consider whether caching is still beneficial given that `symbol_matches_content` is called per symbol per file.

### Phase 5: Regression and Integration
- [ ] Task 5.1: Run `cargo test` and confirm all existing tests pass.
- [ ] Task 5.2: Run `cargo clippy` and resolve any new warnings.
- [ ] Task 5.3: Add `regex` to the dependency check: verify `regex` is already in `Cargo.toml` (it should be, as the crate uses it elsewhere).
- [ ] Task 5.4: Manual smoke test: run `changeguard federate scan` on a repo with sibling schemas. Verify that short symbol names (like `"api"`, `"set"`, `"get"`) no longer produce spurious edges.