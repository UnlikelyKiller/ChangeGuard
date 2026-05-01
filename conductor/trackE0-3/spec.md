# Specification: Track E0-3 Federated Dependency Matching

## Overview

The `discover_dependencies_in_current_repo` method in
`src/federated/scanner.rs` uses `file_content.contains(symbol_to_find)` to
check whether a local file references a sibling repository's public symbol.
This is a raw substring match that produces false positives: searching for
`"api"` matches `"map_item"`, searching for `"set"` matches `"reset_token"`,
and so on. This track replaces substring matching with word-boundary-aware
matching, eliminating false positives while preserving true dependencies.

## Components

### 1. Matching Function Refactor (`src/federated/scanner.rs`)

Create a `symbol_matches` function that replaces the raw `contains` call. The
function must determine whether `symbol_to_find` appears in `file_content` as a
whole identifier, not as a substring of a larger identifier.

The matching strategy:

- **Word boundary matching**: Use `Regex::new(&format!(r"\b{}\b", regex::escape(symbol_to_find)))`
  to match the symbol at word boundaries. This ensures `"api"` matches `"use
  api::handler"` and `"api::get_users()"` but not `"map_item"`.
- **Import-path matching (preferred when import data is available)**: If the
  local file's import data (from `src/index/references.rs` or the existing
  `parse_symbols` call) identifies an import like `use sibling_crate::api`, that
  is a definitive match. Import-based matches take priority over content
  matches.
- The `regex` crate is already a dependency of ChangeGuard, so no new
  dependency is needed.

### 2. Update `discover_dependencies` (`src/federated/scanner.rs`)

In the `discover_dependencies` method, the loop over
`local_packet.changes` currently does:

```rust
if file_content.contains(symbol_to_find) {
    for local_symbol in local_symbols {
        edges.push((local_symbol.name.clone(), symbol_to_find.clone()));
    }
}
```

Replace with:

```rust
if symbol_matches(symbol_to_find, &file_content) {
    for local_symbol in local_symbols {
        edges.push((local_symbol.name.clone(), symbol_to_find.clone()));
    }
}
```

### 3. Update `scan_dependency_dir` (`src/federated/scanner.rs`)

In the `scan_dependency_dir` method, the loop over
`sibling_schema.public_interfaces` currently does:

```rust
if file_content.contains(symbol_to_find) {
    for local_symbol in &local_symbol_names {
        edges.push((local_symbol.clone(), symbol_to_find.clone()));
    }
}
```

Replace with the same `symbol_matches` call:

```rust
if symbol_matches(symbol_to_find, &file_content) {
    for local_symbol in &local_symbol_names {
        edges.push((local_symbol.clone(), symbol_to_find.clone()));
    }
}
```

### 4. SymbolMatcher Caching (`src/federated/scanner.rs`)

Compiling a regex for every symbol in every file is wasteful. Introduce a
`SymbolMatcher` struct that caches compiled regexes in a `HashMap<String, Regex>`.
The struct is constructed once per call to `discover_dependencies` /
`discover_dependencies_in_current_repo` and passed through to `scan_dependency_dir`.

```rust
struct SymbolMatcher {
    cache: HashMap<String, Regex>,
}
```

The `matches(&mut self, symbol: &str, content: &str) -> bool` method checks the
cache first, compiles and caches on miss, then calls `is_match`. This avoids
O(files * symbols) regex compilations.

### 5. Import-Based Matching (Implemented Beyond Spec Minimum) (`src/federated/scanner.rs`)

As an enhancement beyond word-boundary matching, import-based matching has
already been implemented in the codebase via the `symbol_imported` function.
This function checks whether a symbol appears in the local file's import list
(using `extract_import_export` from `src/index/references.rs`). When import
data is available, this provides a definitive match without needing regex.

The current implementation in `discover_dependencies` and `scan_dependency_dir`
uses both checks: `matches_import || matches_word`. This is correct behavior
and exceeds the minimum spec requirement. No further changes needed for this
component.

### 6. Update Existing Test (`src/federated/scanner.rs`)

The existing test `discovers_dependencies_outside_latest_packet` uses
`"remote_api"` as the symbol, which would match with both substring and
word-boundary matching. Add new test cases:

- `test_no_false_positive_substring_match`: Sibling symbol `"api"` should NOT
  match `"map_item"` in local file content.
- `test_word_boundary_match`: Sibling symbol `"handler"` should match
  `"let result = handler(request);"` in local file content.
- `test_no_false_positive_common_word`: Sibling symbol `"set"` should NOT
  match `"let token = reset_token();"` in local file content. This tests that
  common short words don't produce false positives from substring matches
  like "reset_token".
- `test_word_boundary_match_with_qualified_path`: Sibling symbol `"RemoteApi"`
  should match `"use crate::RemoteApi;"` and `"RemoteApi::new()"` in local file
  content.
- `test_no_match_in_comment_or_string_context_is_acceptable`: A symbol that
  appears only inside a string literal or comment is still matched by
  word-boundary regex. This is acceptable behavior and matches the current
  approach (which also matches in those contexts). Document this as a known
  limitation rather than trying to parse the code semantically.

## Constraints & Guidelines

- **No new crate dependency**: The `regex` crate is already available. Do not
  add a new dependency.
- **Performance**: Regex compilation is the expensive part. Cache compiled
  regexes per schema to avoid O(files * symbols) compilations. The matching
  itself (a single `is_match` per file-symbol pair) is fast.
- **Backward compatible**: True matches that worked with `contains` must
  continue to work with word-boundary matching. The only behavior change is
  the elimination of false positives (substring matches that should not count).
- **Graceful on regex error**: If `regex::escape` or `Regex::new` fails for an
  unusual symbol name, fall back to the old `contains` behavior and log a
  warning. Do not crash.
- **TDD**: Write the false-positive tests first, confirm they fail (because
  `contains` produces the false positive), then implement word-boundary matching
  and confirm they pass.

## Acceptance Criteria

1. `symbol_matches("api", "let x = map_item;")` returns `false` (no false
   positive for substring match).
2. `symbol_matches("api", "use api::handler;")` returns `true` (real match at
   word boundary).
3. `symbol_matches("handler", "let result = handler(request);")` returns
   `true`.
4. The existing test `discovers_dependencies_outside_latest_packet` continues
   to pass.
5. A symbol `"set"` does not match file content containing `"reset_token"`.
6. Regex compilation errors fall back to `contains` and log a warning.
7. No new crate dependencies are introduced.
8. All existing tests pass (no regressions).

## Definition of Done

- All acceptance criteria pass
- All unit tests pass
- `cargo fmt --all -- --check` passes
- `cargo clippy --all-targets --all-features -- -D warnings` passes
- `cargo test` passes with no regressions
- No deviations from this spec without documented justification