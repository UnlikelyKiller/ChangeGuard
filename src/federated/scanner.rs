use crate::federated::schema::FederatedSchema;
use crate::index::languages::{Language, parse_symbols};
use crate::index::references::extract_import_export;
use camino::{Utf8Path, Utf8PathBuf};
use miette::{IntoDiagnostic, Result};
use regex::Regex;
use std::fs;
use std::panic;
use tracing::warn;

pub const DEFAULT_SIBLING_LIMIT: usize = 20;

/// Check whether a symbol name appears as a whole-word match in the given content.
/// Uses word-boundary regex to avoid false positives like "api" matching "map_item".
/// Falls back to exact substring match if the regex fails to compile (e.g., symbol
/// contains regex metacharacters that break the pattern).
fn symbol_matches_content(symbol: &str, content: &str) -> bool {
    // Escape any regex metacharacters in the symbol name, then wrap in word boundaries.
    let pattern = format!(r"\b{}\b", regex::escape(symbol));
    match Regex::new(&pattern) {
        Ok(re) => re.is_match(content),
        Err(_) => {
            // Regex compilation failed (unlikely with escaped input); fall back to
            // substring match as a degraded mode rather than crashing.
            warn!(
                "Failed to compile word-boundary regex for symbol '{}', falling back to substring match",
                symbol
            );
            content.contains(symbol)
        }
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
            if path == self.root {
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
                                warnings.push(format!("Invalid schema at {}: {}", path, e));
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
                    let matches_word = symbol_matches_content(symbol_to_find, &file_content);
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
        self.scan_dependency_dir(&self.root, sibling_schema, &mut edges)?;
        edges.sort();
        edges.dedup();
        Ok(edges)
    }

    fn scan_dependency_dir(
        &self,
        dir: &Utf8Path,
        sibling_schema: &FederatedSchema,
        edges: &mut Vec<(String, String)>,
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
                self.scan_dependency_dir(&path, sibling_schema, edges)?;
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
                let matches_word = symbol_matches_content(symbol_to_find, &file_content);
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
        // Exact word match
        assert!(symbol_matches_content(
            "handler",
            "let result = handler(request);"
        ));
        assert!(symbol_matches_content("api", "use crate::api;"));

        // False positives prevented: substring should NOT match
        assert!(!symbol_matches_content("api", "map_item"));
        assert!(!symbol_matches_content("api", "the_capabilities"));
        assert!(!symbol_matches_content("set", "upsetting"));

        // Should match identifiers at word boundaries
        assert!(symbol_matches_content(
            "remote_api",
            "let x = remote_api();"
        ));
        assert!(symbol_matches_content("RemoteApi", "use crate::RemoteApi;"));
    }
}
