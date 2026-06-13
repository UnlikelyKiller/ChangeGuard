use crate::federated::schema::FederatedSchema;
use crate::index::languages::{Language, parse_symbols};
use crate::index::references::extract_import_export;
use camino::{Utf8Path, Utf8PathBuf};
use miette::{IntoDiagnostic, Result};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::panic;
use tracing::{debug, warn};

pub const DEFAULT_SIBLING_LIMIT: usize = 20;

/// Stateful matching utility that caches compiled word-boundary regexes.
/// This prevents redundant regex compilation and string allocations when
/// checking the same public interface symbols against many files.
pub struct SymbolMatcher {
    cache: HashMap<String, Option<Regex>>,
}

impl SymbolMatcher {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Check whether a symbol name appears as a whole-word match in the given content.
    /// Uses a cached word-boundary regex to avoid false positives.
    /// Falls back to exact substring match if the regex fails to compile.
    pub fn matches(&mut self, symbol: &str, content: &str) -> bool {
        if symbol.is_empty() {
            return false;
        }

        let re_opt = self.cache.entry(symbol.to_string()).or_insert_with(|| {
            // Escape any regex metacharacters in the symbol name.
            let escaped = regex::escape(symbol);

            // Use word boundary (\b) if the edge character is a word character,
            // otherwise use a non-word boundary (\B) to ensure we don't match
            // when adjacent to a word character.
            let is_word = |c: char| c.is_alphanumeric() || c == '_';
            let start = if symbol.chars().next().is_some_and(is_word) {
                r"\b"
            } else {
                r"\B"
            };
            let end = if symbol.chars().last().is_some_and(is_word) {
                r"\b"
            } else {
                r"\B"
            };

            let pattern = format!("{}{}{}", start, escaped, end);
            match Regex::new(&pattern) {
                Ok(re) => Some(re),
                Err(_) => {
                    warn!(
                        "Failed to compile word-boundary regex for symbol '{}', falling back to substring match",
                        symbol
                    );
                    None
                }
            }
        });

        match re_opt {
            Some(re) => re.is_match(content),
            None => content.contains(symbol),
        }
    }
}

impl Default for SymbolMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Check whether a symbol is imported or referenced via the file's import list.
/// This is a more precise match than word-boundary regex: if the symbol's module/crate
/// appears in the file's imports, it's a definitive dependency.
fn symbol_imported(symbol: &str, path: &Utf8Path, content: &str) -> bool {
    if let Ok(Some(import_export)) = extract_import_export(path.as_std_path(), content) {
        // Check if the symbol name or a module path containing it appears in imports.
        for import in &import_export.imported_from {
            if import.contains(symbol) {
                return true;
            }
        }
        for export in &import_export.exported_symbols {
            if export == symbol {
                return true;
            }
        }
    }
    false
}

pub struct FederatedScanner {
    root: Utf8PathBuf,
    sibling_limit: usize,
}

impl FederatedScanner {
    pub fn new(root: Utf8PathBuf) -> Self {
        Self {
            root,
            sibling_limit: DEFAULT_SIBLING_LIMIT,
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.sibling_limit = limit;
        self
    }

    /// Discovers sibling repositories and their schemas.
    /// Returns discovered schemas and a list of deterministic warnings.
    #[allow(clippy::type_complexity)]
    pub fn scan_siblings(&self) -> Result<(Vec<(Utf8PathBuf, FederatedSchema)>, Vec<String>)> {
        let parent = match self.root.parent() {
            Some(p) => p,
            None => return Ok((Vec::new(), Vec::new())),
        };

        // Canonicalize parent for secure path comparison
        let canonical_parent = parent.canonicalize_utf8().into_diagnostic()?;

        let mut discovered = Vec::new();
        let mut warnings = Vec::new();
        let entries = fs::read_dir(parent).into_diagnostic()?;

        for entry in entries {
            if discovered.len() >= self.sibling_limit {
                warnings.push(format!(
                    "Reached sibling limit ({}). Some siblings may have been skipped.",
                    self.sibling_limit
                ));
                break;
            }

            let entry = entry.into_diagnostic()?;
            let path = Utf8PathBuf::from_path_buf(entry.path())
                .map_err(|_| miette::miette!("Invalid UTF-8 path: {:?}", entry.path()))?;

            // Security: Skip symlinks to prevent escapes
            let metadata = fs::symlink_metadata(&path).into_diagnostic()?;
            if metadata.is_symlink() {
                continue;
            }

            // Skip current repo
            let is_root = if let (Ok(p1), Ok(p2)) =
                (path.canonicalize_utf8(), self.root.canonicalize_utf8())
            {
                p1.as_str().to_lowercase() == p2.as_str().to_lowercase()
            } else {
                path.as_str().to_lowercase() == self.root.as_str().to_lowercase()
            };
            if is_root {
                continue;
            }

            if metadata.is_dir() {
                // Path Confinement Check
                let canonical_path = match path.canonicalize_utf8() {
                    Ok(p) => p,
                    Err(_) => {
                        warnings.push(format!("Failed to canonicalize path: {}", path));
                        continue;
                    }
                };

                // Verify the resolved path is exactly parent.join(sibling_name)
                // and resides exactly one level above the local repository root.
                if canonical_path.parent() != Some(&canonical_parent) {
                    warnings.push(format!(
                        "Security violation: Sibling path escapes discovery root: {}",
                        path
                    ));
                    continue;
                }

                // Check for schema in .changeguard/state/schema.json (current)
                // or .changeguard/schema.json (legacy fallback)
                let schema_path = path.join(".changeguard").join("state").join("schema.json");
                let legacy_path = path.join(".changeguard").join("schema.json");

                let final_path = if schema_path.exists() {
                    Some(schema_path)
                } else if legacy_path.exists() {
                    Some(legacy_path)
                } else {
                    None
                };

                if let Some(sp) = final_path {
                    match self.load_schema(&sp) {
                        Ok(schema) => {
                            if let Err(e) = schema.validate() {
                                debug!("Invalid schema at {}: {}", path, e);
                            } else {
                                discovered.push((path, schema));
                            }
                        }
                        Err(e) => {
                            warnings.push(format!("Failed to load schema from {}: {}", path, e));
                            warn!("Failed to load schema from {}: {:?}", sp, e);
                        }
                    }
                }
            }
        }

        // Engineering standard: deterministic sorting by repo name
        discovered.sort_by(|a, b| a.1.repo_name.cmp(&b.1.repo_name));
        warnings.sort();

        Ok((discovered, warnings))
    }

    fn load_schema(&self, path: &Utf8Path) -> Result<FederatedSchema> {
        let content = fs::read_to_string(path).into_diagnostic()?;

        // JSON Safety: Wrap in catch_unwind to prevent panics from malformed JSON
        let result = panic::catch_unwind(|| serde_json::from_str::<FederatedSchema>(&content));

        match result {
            Ok(serde_result) => serde_result.into_diagnostic(),
            Err(_) => Err(miette::miette!("Panic occurred while parsing JSON schema")),
        }
    }

    pub fn discover_dependencies(
        &self,
        local_packet: &crate::impact::packet::ImpactPacket,
        _sibling_name: &str,
        sibling_schema: &FederatedSchema,
    ) -> Result<Vec<(String, String)>> {
        let mut edges = self.discover_dependencies_in_current_repo(sibling_schema)?;
        let mut matcher = SymbolMatcher::new();

        for interface in &sibling_schema.public_interfaces {
            let symbol_to_find = &interface.symbol;

            for change in &local_packet.changes {
                if let Some(local_symbols) = &change.symbols {
                    let Some(utf8_path) = Utf8Path::from_path(&change.path) else {
                        continue;
                    };
                    let full_path = self.root.join(utf8_path);
                    let file_content = match fs::read_to_string(&full_path) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    // Use import-based matching first (definitive), then word-boundary
                    // regex (heuristic). This avoids false positives like "api"
                    // matching "map_item".
                    let matches_import = symbol_imported(symbol_to_find, utf8_path, &file_content);
                    let matches_word = matcher.matches(symbol_to_find, &file_content);
                    if matches_import || matches_word {
                        for local_symbol in local_symbols {
                            edges.push((local_symbol.name.clone(), symbol_to_find.clone()));
                        }
                    }
                }
            }
        }

        edges.sort();
        edges.dedup();
        Ok(edges)
    }

    pub fn discover_dependencies_in_current_repo(
        &self,
        sibling_schema: &FederatedSchema,
    ) -> Result<Vec<(String, String)>> {
        let mut edges = Vec::new();
        let mut matcher = SymbolMatcher::new();
        self.scan_dependency_dir(&self.root, sibling_schema, &mut edges, &mut matcher)?;
        edges.sort();
        edges.dedup();
        Ok(edges)
    }

    fn scan_dependency_dir(
        &self,
        dir: &Utf8Path,
        sibling_schema: &FederatedSchema,
        edges: &mut Vec<(String, String)>,
        matcher: &mut SymbolMatcher,
    ) -> Result<()> {
        for entry in fs::read_dir(dir).into_diagnostic()? {
            let entry = entry.into_diagnostic()?;
            let path = Utf8PathBuf::from_path_buf(entry.path())
                .map_err(|_| miette::miette!("Invalid UTF-8 path: {:?}", entry.path()))?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            if path.is_dir() {
                if matches!(file_name.as_ref(), ".git" | ".changeguard" | "target") {
                    continue;
                }
                self.scan_dependency_dir(&path, sibling_schema, edges, matcher)?;
                continue;
            }

            let Some(extension) = path.extension() else {
                continue;
            };
            if Language::from_extension(extension).is_none() {
                continue;
            }

            let Ok(file_content) = fs::read_to_string(&path) else {
                continue;
            };
            let relative_path = path.strip_prefix(&self.root).unwrap_or(&path);
            let local_symbols =
                parse_symbols(relative_path.as_std_path(), &file_content).unwrap_or_default();

            let local_symbol_names = local_symbols
                .unwrap_or_default()
                .into_iter()
                .map(|symbol| symbol.name)
                .collect::<Vec<_>>();

            if local_symbol_names.is_empty() {
                continue;
            }

            for interface in &sibling_schema.public_interfaces {
                let symbol_to_find = &interface.symbol;
                let matches_import = symbol_imported(symbol_to_find, relative_path, &file_content);
                let matches_word = matcher.matches(symbol_to_find, &file_content);
                if matches_import || matches_word {
                    for local_symbol in &local_symbol_names {
                        edges.push((local_symbol.clone(), symbol_to_find.clone()));
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod dependency_tests {
    use super::*;
    use crate::federated::schema::PublicInterface;
    use crate::index::symbols::SymbolKind;
    use tempfile::tempdir;

    #[test]
    fn discovers_dependencies_outside_latest_packet() {
        let tmp = tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(tmp.path().to_path_buf()).unwrap();
        fs::write(
            root.join("main.rs"),
            "pub fn local_handler() { let _ = remote_api(); }",
        )
        .unwrap();

        let schema = FederatedSchema::new(
            "sibling".to_string(),
            vec![PublicInterface {
                symbol: "remote_api".to_string(),
                file: "src/lib.rs".to_string(),
                kind: SymbolKind::Function,
            }],
        );

        let scanner = FederatedScanner::new(root);
        let dependencies = scanner
            .discover_dependencies_in_current_repo(&schema)
            .unwrap();

        assert_eq!(
            dependencies,
            vec![("local_handler".to_string(), "remote_api".to_string())]
        );
    }

    #[test]
    fn no_false_positive_substring_match() {
        // "api" should NOT match "map_item" — the word boundary prevents it.
        let tmp = tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(tmp.path().to_path_buf()).unwrap();
        fs::write(root.join("main.rs"), "pub fn map_item() { }").unwrap();

        let schema = FederatedSchema::new(
            "sibling".to_string(),
            vec![PublicInterface {
                symbol: "api".to_string(),
                file: "src/lib.rs".to_string(),
                kind: SymbolKind::Function,
            }],
        );

        let scanner = FederatedScanner::new(root);
        let dependencies = scanner
            .discover_dependencies_in_current_repo(&schema)
            .unwrap();

        assert!(
            dependencies.is_empty(),
            "Expected no dependencies, got {:?}",
            dependencies
        );
    }

    #[test]
    fn word_boundary_match_still_works() {
        // "handler" should match "let result = handler(request);" as a whole word.
        let tmp = tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(tmp.path().to_path_buf()).unwrap();
        fs::write(
            root.join("main.rs"),
            "pub fn local_fn() { let result = handler(request); }",
        )
        .unwrap();

        let schema = FederatedSchema::new(
            "sibling".to_string(),
            vec![PublicInterface {
                symbol: "handler".to_string(),
                file: "src/lib.rs".to_string(),
                kind: SymbolKind::Function,
            }],
        );

        let scanner = FederatedScanner::new(root);
        let dependencies = scanner
            .discover_dependencies_in_current_repo(&schema)
            .unwrap();

        assert!(
            !dependencies.is_empty(),
            "Expected to find 'handler' as a whole-word match"
        );
    }

    #[test]
    fn symbol_matches_content_unit_tests() {
        let mut matcher = SymbolMatcher::new();
        // Exact word match
        assert!(matcher.matches("handler", "let result = handler(request);"));
        assert!(matcher.matches("api", "use crate::api;"));

        // False positives prevented: substring should NOT match
        assert!(!matcher.matches("api", "map_item"));
        assert!(!matcher.matches("api", "the_capabilities"));
        assert!(!matcher.matches("set", "upsetting"));

        // Should match identifiers at word boundaries
        assert!(matcher.matches("remote_api", "let x = remote_api();"));
        assert!(matcher.matches("RemoteApi", "use crate::RemoteApi;"));

        // Metacharacters should be escaped and matched correctly
        assert!(matcher.matches("api.v1", "let x = api.v1();"));
        assert!(!matcher.matches("api.v1", "api_v1"));
        assert!(matcher.matches("search(fn)", "call search(fn) now"));

        // Fallback behavior: manual insertion of None to simulate regex failure
        matcher.cache.insert("fallback_sym".to_string(), None);
        assert!(matcher.matches("fallback_sym", "this contains fallback_sym"));
        assert!(!matcher.matches("fallback_sym", "other content"));

        // Edge cases: empty content or symbols
        assert!(!matcher.matches("symbol", ""));
        assert!(!matcher.matches("", "content"));
    }
}
