use crate::federated::schema::FederatedSchema;
use camino::{Utf8Path, Utf8PathBuf};
use miette::{IntoDiagnostic, Result};
use std::fs;
use tracing::warn;

pub struct FederatedScanner {
    root: Utf8PathBuf,
}

impl FederatedScanner {
    pub fn new(root: Utf8PathBuf) -> Self {
        Self { root }
    }

    /// Discovers sibling repositories and their schemas.
    /// Strictly depth=1 (direct siblings of the parent directory).
    pub fn scan_siblings(&self) -> Result<Vec<(Utf8PathBuf, FederatedSchema)>> {
        let parent = match self.root.parent() {
            Some(p) => p,
            None => return Ok(Vec::new()),
        };

        let mut discovered = Vec::new();
        let entries = fs::read_dir(parent).into_diagnostic()?;

        for entry in entries {
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
                let schema_path = path.join(".changeguard").join("schema.json");
                if schema_path.exists() {
                    match self.load_schema(&schema_path) {
                        Ok(schema) => discovered.push((path, schema)),
                        Err(e) => {
                            warn!("Failed to load schema from {}: {:?}", schema_path, e);
                        }
                    }
                }
            }
        }

        // Engineering standard: deterministic sorting by repo name
        discovered.sort_by(|a, b| a.1.repo_name.cmp(&b.1.repo_name));

        Ok(discovered)
    }

    fn load_schema(&self, path: &Utf8Path) -> Result<FederatedSchema> {
        let content = fs::read_to_string(path).into_diagnostic()?;
        let schema: FederatedSchema = serde_json::from_str(&content).into_diagnostic()?;
        Ok(schema)
    }
}
