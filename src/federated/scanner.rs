use crate::federated::schema::FederatedSchema;
use camino::{Utf8Path, Utf8PathBuf};
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::panic;
use tracing::warn;

pub const DEFAULT_SIBLING_LIMIT: usize = 20;

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

                let schema_path = path.join(".changeguard").join("schema.json");
                if schema_path.exists() {
                    match self.load_schema(&schema_path) {
                        Ok(schema) => {
                            if let Err(e) = schema.validate() {
                                warnings.push(format!("Invalid schema at {}: {}", path, e));
                            } else {
                                discovered.push((path, schema));
                            }
                        }
                        Err(e) => {
                            warnings.push(format!("Failed to load schema from {}: {}", path, e));
                            warn!("Failed to load schema from {}: {:?}", schema_path, e);
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
        let mut edges = Vec::new();

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

                    if file_content.contains(symbol_to_find) {
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
}
