# Technical Specification: Federated Dependency Matching Cache (Track E0-3)

## Objective
Optimize federated dependency discovery in ChangeGuard by replacing the repeated regex compilation in `symbol_matches_content` with a cached `SymbolMatcher` approach. This will significantly improve performance on large repositories by reducing redundant allocations and compilation overhead.

## Background
In `src/federated/scanner.rs`, the function `symbol_matches_content(symbol: &str, content: &str)` compiles a new word-boundary `Regex` each time it is called. Because this is invoked inside nested loops iterating over multiple files and multiple public interfaces, the compilation overhead grows multiplicatively ($O(\text{files} \times \text{interfaces})$), causing severe latency during federated dependency discovery.

## Architecture & Implementation Details

### 1. `SymbolMatcher` Struct
Introduce a stateful matching utility:
```rust
pub struct SymbolMatcher {
    cache: std::collections::HashMap<String, Option<regex::Regex>>,
}
```
- Uses a `HashMap` to cache compiled `Regex` objects, keyed by `symbol`.
- The value is `Option<Regex>` to cache compilation *failures* as well. This ensures we only warn and fallback to substring matching once per symbol, instead of repeatedly spamming logs and retrying compilation.

### 2. Zero-Allocation Cache Hits
The `matches` function will be implemented to prevent string allocation on cache hits:
```rust
impl SymbolMatcher {
    pub fn new() -> Self {
        Self { cache: std::collections::HashMap::new() }
    }

    pub fn matches(&mut self, symbol: &str, content: &str) -> bool {
        if !self.cache.contains_key(symbol) {
            let pattern = format!(r"\b{}\b", regex::escape(symbol));
            let re_opt = match regex::Regex::new(&pattern) {
                Ok(r) => Some(r),
                Err(_) => {
                    tracing::warn!(
                        "Failed to compile word-boundary regex for symbol '{}', falling back to substring match",
                        symbol
                    );
                    None
                }
            };
            self.cache.insert(symbol.to_string(), re_opt);
        }

        match self.cache.get(symbol).unwrap() {
            Some(re) => re.is_match(content),
            None => content.contains(symbol),
        }
    }
}
```

### 3. Integration in `FederatedScanner`
Rather than making `FederatedScanner` globally stateful with interior mutability (`RefCell`, which would make it `!Sync`), we instantiate `SymbolMatcher` locally at the entry points of scanning to maintain thread-safety and simplicity:
- **`scan_dependency_dir`**: Update signature to accept `matcher: &mut SymbolMatcher`. Pass it recursively. Replace `symbol_matches_content` calls with `matcher.matches(...)`.
- **`discover_dependencies_in_current_repo`**: Instantiate `let mut matcher = SymbolMatcher::new();` and pass it to `scan_dependency_dir`.
- **`discover_dependencies`**: Instantiate a separate local `let mut matcher = SymbolMatcher::new();` outside the public interfaces loop, and use it inside the `local_packet.changes` loop.

### 4. Testing
- Update the existing `symbol_matches_content_unit_tests` to instantiate `SymbolMatcher` and call `matches`.
- Maintain existing behavioral guarantees (whole-word matching, substring fallback, escape of regex characters).