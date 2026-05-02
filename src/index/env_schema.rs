use crate::state::storage::StorageManager;
use miette::{IntoDiagnostic, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::LazyLock;
use tracing::{info, warn};

// --- Redacted value markers ---
pub const HAS_DEFAULT: &str = "HAS_DEFAULT";
pub const EMPTY_DEFAULT: &str = "EMPTY_DEFAULT";
pub const PLACEHOLDER_DEFAULT: &str = "PLACEHOLDER_DEFAULT";
pub const POSSIBLE_SECRET_REDACTED: &str = "POSSIBLE_SECRET_REDACTED";

/// Secret name patterns that indicate sensitive values.
const SECRET_PATTERNS: &[&str] = &[
    "SECRET",
    "KEY",
    "PASSWORD",
    "TOKEN",
    "API_KEY",
    "PRIVATE",
    "CREDENTIAL",
    "AUTH",
];

/// Placeholder-like values that indicate a non-real default.
const PLACEHOLDER_VALUES: &[&str] = &[
    "your-",
    "xxx",
    "xxx",
    "change_me",
    "changeme",
    "replace",
    "placeholder",
    "example",
    "todo",
    "fixme",
    "<",
    "{{",
    "{your",
    "insert",
    "fill",
    "put_your",
    "default",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum EnvSourceKind {
    DotenvExample,
    Config,
    Docs,
}

impl std::fmt::Display for EnvSourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvSourceKind::DotenvExample => write!(f, "DOTENV_EXAMPLE"),
            EnvSourceKind::Config => write!(f, "CONFIG"),
            EnvSourceKind::Docs => write!(f, "DOCS"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "camelCase")]
pub enum EnvReferenceKind {
    Read,
    Write,
    Defaulted,
    Dynamic,
}

impl std::fmt::Display for EnvReferenceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvReferenceKind::Read => write!(f, "READ"),
            EnvReferenceKind::Write => write!(f, "WRITE"),
            EnvReferenceKind::Defaulted => write!(f, "DEFAULTED"),
            EnvReferenceKind::Dynamic => write!(f, "DYNAMIC"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EnvDeclaration {
    pub var_name: String,
    pub source_kind: EnvSourceKind,
    pub required: bool,
    pub default_value_redacted: Option<String>,
    pub description: Option<String>,
    pub confidence: f64,
}

impl Eq for EnvDeclaration {}

impl PartialOrd for EnvDeclaration {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EnvDeclaration {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.var_name
            .cmp(&other.var_name)
            .then_with(|| self.source_kind.cmp(&other.source_kind))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EnvReference {
    pub var_name: String,
    pub reference_kind: EnvReferenceKind,
    pub confidence: f64,
}

impl Eq for EnvReference {}

impl PartialOrd for EnvReference {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EnvReference {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.var_name
            .cmp(&other.var_name)
            .then_with(|| self.reference_kind.cmp(&other.reference_kind))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct EnvVarDep {
    pub var_name: String,
    pub declared: bool,
    pub evidence: String,
}

/// Determines whether a value looks like a placeholder.
fn is_placeholder_value(value: &str) -> bool {
    let lower = value.to_lowercase();
    PLACEHOLDER_VALUES.iter().any(|pat| lower.contains(pat))
}

/// Determines whether a variable name suggests it holds a secret.
fn is_secret_name(name: &str) -> bool {
    let upper = name.to_uppercase();
    SECRET_PATTERNS.iter().any(|pat| upper.contains(pat))
}

/// Redacts a default value according to security rules.
/// Secret name detection takes priority over placeholder detection:
/// if the variable name suggests a secret, always use POSSIBLE_SECRET_REDACTED.
fn redact_default(var_name: &str, value: &str) -> String {
    if value.is_empty() {
        return EMPTY_DEFAULT.to_string();
    }
    if is_secret_name(var_name) {
        return POSSIBLE_SECRET_REDACTED.to_string();
    }
    if is_placeholder_value(value) {
        return PLACEHOLDER_DEFAULT.to_string();
    }
    HAS_DEFAULT.to_string()
}

// --- Regex patterns for env var references in source code ---

static RUST_ENV_VAR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"std::env::var\("([^"]+)"\)"#).expect("valid regex"));
static RUST_ENV_MACRO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"env!\("([^"]+)"\)"#).expect("valid regex"));
static RUST_ENV_VAR_DEFAULT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"std::env::var\("([^"]+)"\)\.(?:unwrap_or|ok|unwrap_or_else)"#)
        .expect("valid regex")
});
static TS_ENV_DOT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"process\.env\.([A-Z_][A-Z0-9_]*)"#).expect("valid regex"));
static TS_ENV_INDEXED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"process\.env\[['"]([^'"]+)['"]\]"#).expect("valid regex"));
static TS_ENV_DEFAULT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"process\.env\.([A-Z_][A-Z0-9_]*)\s*\|\|"#).expect("valid regex")
});
static PY_ENV_GET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"os\.(?:environ\.get|getenv)\(['"]([^'"]+)['"]\)"#).expect("valid regex")
});
static PY_ENV_GET_DEFAULT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"os\.(?:environ\.get|getenv)\(['"]([^'"]+)['"]\s*,\s*"#).expect("valid regex")
});
static PY_ENV_INDEXED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"os\.environ\[['"]([^'"]+)['"]\]"#).expect("valid regex"));
static RUST_SET_ENV: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"std::env::set_var\("([^"]+)""#).expect("valid regex"));
static TS_SET_ENV: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"process\.env\[?\.?([A-Z_][A-Z0-9_]*)\]?\s*="#).expect("valid regex")
});

fn collect_env_captures(regex: &Regex, content: &str, out: &mut Vec<String>) {
    for capture in regex.captures_iter(content) {
        if let Some(m) = capture.get(1) {
            out.push(m.as_str().to_string());
        }
    }
}

pub struct EnvSchemaExtractor;

impl EnvSchemaExtractor {
    /// Parse KEY=VALUE pairs from .env.example/.env.template files.
    /// Redacts values per security rules. Detects secrets by name patterns.
    pub fn extract_from_dotenv(content: &str) -> Vec<EnvDeclaration> {
        let mut declarations = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Try to split on first '='
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim().to_string();
                let value = trimmed[eq_pos + 1..].trim().to_string();

                if key.is_empty() {
                    continue;
                }

                // Keys starting with # after export are comments
                if key.starts_with('#') {
                    continue;
                }

                // Strip "export " prefix if present
                let key = key
                    .strip_prefix("export ")
                    .unwrap_or(&key)
                    .trim()
                    .to_string();

                if key.is_empty() {
                    continue;
                }

                // Strip surrounding quotes from value
                let value = if (value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\''))
                {
                    value[1..value.len() - 1].to_string()
                } else {
                    value
                };

                // Strip inline comments from value
                let value = if let Some(comment_pos) = value.find(" #") {
                    value[..comment_pos].trim().to_string()
                } else {
                    value
                };

                let default_value_redacted = if value.is_empty() {
                    Some(EMPTY_DEFAULT.to_string())
                } else {
                    Some(redact_default(&key, &value))
                };

                let required = value.is_empty();

                declarations.push(EnvDeclaration {
                    var_name: key,
                    source_kind: EnvSourceKind::DotenvExample,
                    required,
                    default_value_redacted,
                    description: None,
                    confidence: 1.0,
                });
            } else {
                // KEY with no = sign (declaration without default)
                let key = trimmed.trim().to_string();
                let key = key
                    .strip_prefix("export ")
                    .unwrap_or(&key)
                    .trim()
                    .to_string();

                if key.is_empty() || key.starts_with('#') {
                    continue;
                }

                // Only treat as env var if it looks like one (ALL_CAPS with underscores)
                if key
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
                    && !key.is_empty()
                {
                    declarations.push(EnvDeclaration {
                        var_name: key,
                        source_kind: EnvSourceKind::DotenvExample,
                        required: true,
                        default_value_redacted: None,
                        description: None,
                        confidence: 0.7,
                    });
                }
            }
        }

        declarations.sort_unstable();
        declarations.dedup();
        declarations
    }

    /// Parse config.toml for keys that look like env var declarations.
    /// Looks for keys with `env = true` or keys in `[env]` sections.
    pub fn extract_from_toml(content: &str) -> Vec<EnvDeclaration> {
        let mut declarations = Vec::new();
        let mut in_env_section = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Section header detection
            if trimmed.starts_with('[') {
                let section = trimmed.trim_start_matches('[').trim_end_matches(']');
                in_env_section = section == "env" || section.starts_with("env.");
                continue;
            }

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // In [env] section, every key is an env var declaration
            if in_env_section {
                if let Some(eq_pos) = trimmed.find('=') {
                    let key = trimmed[..eq_pos].trim().to_string();
                    if key.is_empty() || key.starts_with('#') {
                        continue;
                    }

                    let value = trimmed[eq_pos + 1..].trim().to_string();
                    let value = value.trim_matches('"').trim_matches('\'').to_string();

                    let default_value_redacted = if value.is_empty() {
                        Some(EMPTY_DEFAULT.to_string())
                    } else {
                        Some(redact_default(&key, &value))
                    };

                    declarations.push(EnvDeclaration {
                        var_name: key,
                        source_kind: EnvSourceKind::Config,
                        required: false,
                        default_value_redacted,
                        description: None,
                        confidence: 0.85,
                    });
                }
                continue;
            }

            // Outside [env] section: look for `env = true` or `_ENV` suffix patterns
            if trimmed.contains("= true") || trimmed.contains("=True") {
                // key = true pattern might indicate an env flag
                if let Some(eq_pos) = trimmed.find('=') {
                    let key = trimmed[..eq_pos].trim().to_string();
                    let value_str = trimmed[eq_pos + 1..].trim().to_string();
                    if value_str == "true" || value_str == "True" {
                        // Check if the key name suggests it's an env var
                        let upper_key = key.to_uppercase();
                        if upper_key.contains("ENV")
                            || key.contains('_')
                                && key.chars().all(|c| {
                                    c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit()
                                })
                        {
                            declarations.push(EnvDeclaration {
                                var_name: key,
                                source_kind: EnvSourceKind::Config,
                                required: false,
                                default_value_redacted: Some(HAS_DEFAULT.to_string()),
                                description: None,
                                confidence: 0.5,
                            });
                        }
                    }
                }
            }
        }

        declarations.sort_unstable();
        declarations.dedup();
        declarations
    }

    /// Parse config.json for env-like keys (nested keys with env var patterns).
    pub fn extract_from_json(content: &str) -> Vec<EnvDeclaration> {
        let mut declarations = Vec::new();

        // Simple line-based extraction: look for keys that look like env vars
        // Pattern: "ENV_VAR_NAME": "value" or "ENV_VAR_NAME": { ... }
        for line in content.lines() {
            let trimmed = line.trim();

            // Look for "KEY": "value" patterns where KEY looks like an env var
            if let Some(key_match) = extract_json_key_value(trimmed) {
                let (key, value) = key_match;

                // Only consider keys that look like env var names
                if !key
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
                    || key.is_empty()
                {
                    continue;
                }

                let default_value_redacted = if value.is_empty() {
                    Some(EMPTY_DEFAULT.to_string())
                } else {
                    Some(redact_default(&key, &value))
                };

                declarations.push(EnvDeclaration {
                    var_name: key,
                    source_kind: EnvSourceKind::Config,
                    required: false,
                    default_value_redacted,
                    description: None,
                    confidence: 0.7,
                });
            }
        }

        declarations.sort_unstable();
        declarations.dedup();
        declarations
    }

    /// Extract env var references from source code files.
    /// Reuses regex patterns from runtime_usage but also classifies reference_kind.
    pub fn extract_references_from_source(path: &Path, content: &str) -> Vec<EnvReference> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default();

        let mut references = Vec::new();
        let mut defaulted_vars = Vec::new();
        let mut set_vars = Vec::new();

        match extension {
            "rs" => {
                // Read references
                collect_env_captures(&RUST_ENV_VAR, content, &mut references);
                collect_env_captures(&RUST_ENV_MACRO, content, &mut references);
                // Defaulted references
                collect_env_captures(&RUST_ENV_VAR_DEFAULT, content, &mut defaulted_vars);
                // Write references
                collect_env_captures(&RUST_SET_ENV, content, &mut set_vars);
            }
            "ts" | "tsx" | "js" | "jsx" => {
                // Read references (without default)
                collect_env_captures(&TS_ENV_DOT, content, &mut references);
                collect_env_captures(&TS_ENV_INDEXED, content, &mut references);
                // Defaulted references
                collect_env_captures(&TS_ENV_DEFAULT, content, &mut defaulted_vars);
                // Write references
                collect_env_captures(&TS_SET_ENV, content, &mut set_vars);
            }
            "py" => {
                // Read references
                collect_env_captures(&PY_ENV_GET, content, &mut references);
                collect_env_captures(&PY_ENV_INDEXED, content, &mut references);
                // Defaulted references
                collect_env_captures(&PY_ENV_GET_DEFAULT, content, &mut defaulted_vars);
            }
            _ => return Vec::new(),
        }

        let mut result = Vec::new();

        // Classify read references
        let defaulted_set: std::collections::HashSet<String> = defaulted_vars.into_iter().collect();
        let set_set: std::collections::HashSet<String> = set_vars.into_iter().collect();

        let mut seen = std::collections::HashSet::new();
        for var_name in &references {
            let kind = if set_set.contains(var_name) {
                EnvReferenceKind::Write
            } else if defaulted_set.contains(var_name) {
                EnvReferenceKind::Defaulted
            } else {
                EnvReferenceKind::Read
            };
            if seen.insert((var_name.clone(), kind.clone())) {
                result.push(EnvReference {
                    var_name: var_name.clone(),
                    reference_kind: kind,
                    confidence: 1.0,
                });
            }
        }

        // Add write-only references
        for var_name in &set_set {
            if !seen.contains(&(var_name.clone(), EnvReferenceKind::Write)) {
                seen.insert((var_name.clone(), EnvReferenceKind::Write));
                result.push(EnvReference {
                    var_name: var_name.clone(),
                    reference_kind: EnvReferenceKind::Write,
                    confidence: 1.0,
                });
            }
        }

        // Add defaulted-only references (not also in read set)
        for var_name in &defaulted_set {
            if !seen.contains(&(var_name.clone(), EnvReferenceKind::Defaulted))
                && !references.iter().any(|r| r == var_name)
            {
                seen.insert((var_name.clone(), EnvReferenceKind::Defaulted));
                result.push(EnvReference {
                    var_name: var_name.clone(),
                    reference_kind: EnvReferenceKind::Defaulted,
                    confidence: 0.9,
                });
            }
        }

        // Check for dynamic env access patterns
        let dynamic_patterns = match extension {
            "rs" => content.contains("std::env::vars") || content.contains("std::env::vars_os"),
            "ts" | "tsx" | "js" | "jsx" => {
                content.contains("Object.keys(process.env)")
                    || content.contains("Object.entries(process.env)")
            }
            "py" => content.contains("os.environ"),
            _ => false,
        };

        if dynamic_patterns {
            result.push(EnvReference {
                var_name: "*".to_string(),
                reference_kind: EnvReferenceKind::Dynamic,
                confidence: 0.5,
            });
        }

        result.sort_unstable();
        result.dedup();
        result
    }

    /// Find env vars referenced in source code that aren't declared in any config.
    pub fn find_undeclared(
        references: &[EnvReference],
        declarations: &[EnvDeclaration],
    ) -> Vec<EnvVarDep> {
        let declared_names: std::collections::HashSet<String> =
            declarations.iter().map(|d| d.var_name.clone()).collect();

        let mut undeclared = Vec::new();

        for reference in references {
            // Skip dynamic references (wildcard)
            if reference.var_name == "*" {
                continue;
            }

            if !declared_names.contains(&reference.var_name) {
                undeclared.push(EnvVarDep {
                    var_name: reference.var_name.clone(),
                    declared: false,
                    evidence: format!(
                        "Referenced as {} but not declared in any env config",
                        reference.reference_kind
                    ),
                });
            }
        }

        undeclared.sort_unstable();
        undeclared.dedup();
        undeclared
    }
}

/// Helper to extract a JSON key-value pair from a line like `"KEY": "value"`.
fn extract_json_key_value(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();

    // Must start with "
    if !trimmed.starts_with('"') {
        return None;
    }

    // Find closing quote for key
    let key_end = trimmed[1..].find('"')?;
    let key = trimmed[1..key_end + 1].to_string();

    // Find the colon separator
    let after_key = trimmed[key_end + 2..].trim_start();
    if !after_key.starts_with(':') {
        return None;
    }

    let after_colon = after_key[1..].trim_start();

    // Extract value
    let value = if let Some(stripped) = after_colon.strip_prefix('"') {
        // String value
        if let Some(end) = stripped.find('"') {
            stripped[..end].to_string()
        } else {
            stripped
                .trim_end_matches(',')
                .trim_end_matches('"')
                .to_string()
        }
    } else if after_colon.starts_with('{') || after_colon.starts_with('[') {
        // Object or array value - not an env var value
        return None;
    } else {
        // Primitive value (number, boolean, null)
        after_colon
            .split(',')
            .next()
            .unwrap_or("")
            .trim()
            .to_string()
    };

    Some((key, value))
}

// --- Env Schema Indexer (database integration) ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EnvSchemaStats {
    pub total_declarations: usize,
    pub total_references: usize,
    pub dotenv_declarations: usize,
    pub config_declarations: usize,
    pub files_processed: usize,
}

struct EnvDeclarationRow {
    source_file_id: i64,
    var_name: String,
    source_kind: String,
    required: bool,
    default_value_redacted: Option<String>,
    description: Option<String>,
    confidence: f64,
}

struct EnvReferenceRow {
    file_id: i64,
    symbol_id: Option<i64>,
    var_name: String,
    reference_kind: String,
    confidence: f64,
    line_start: Option<i64>,
}

const ENV_SCHEMA_BATCH_SIZE: usize = 500;

pub struct EnvSchemaIndexer<'a> {
    storage: &'a StorageManager,
    repo_path: std::path::PathBuf,
}

impl<'a> EnvSchemaIndexer<'a> {
    pub fn new(storage: &'a StorageManager, repo_path: std::path::PathBuf) -> Self {
        Self { storage, repo_path }
    }

    pub fn extract(&self) -> Result<EnvSchemaStats> {
        // 1. Discover env config files
        let env_files = self.discover_env_files();

        // 2. Clear existing data before re-indexing
        {
            let conn = self.storage.get_connection();
            conn.execute("DELETE FROM env_declarations", [])
                .into_diagnostic()?;
            conn.execute("DELETE FROM env_references", [])
                .into_diagnostic()?;
        }

        let now = chrono::Utc::now().to_rfc3339();
        let mut total_declarations = 0usize;
        let mut dotenv_declarations = 0usize;
        let mut config_declarations = 0usize;
        let mut files_processed = 0usize;
        let mut decl_batch: Vec<EnvDeclarationRow> = Vec::new();

        // 3. Extract declarations from env config files
        for (relative_path, source_type) in &env_files {
            let full_path = self.repo_path.join(relative_path);
            let content = match std::fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(e) => {
                    warn!("Failed to read env config file {}: {}", relative_path, e);
                    continue;
                }
            };

            let file_id = self.ensure_file_entry(relative_path, &content, &now)?;

            let declarations = match source_type.as_str() {
                "dotenv" => {
                    let decls = EnvSchemaExtractor::extract_from_dotenv(&content);
                    dotenv_declarations += decls.len();
                    decls
                }
                "toml" => {
                    let decls = EnvSchemaExtractor::extract_from_toml(&content);
                    config_declarations += decls.len();
                    decls
                }
                "json" => {
                    let decls = EnvSchemaExtractor::extract_from_json(&content);
                    config_declarations += decls.len();
                    decls
                }
                _ => Vec::new(),
            };

            for decl in &declarations {
                decl_batch.push(EnvDeclarationRow {
                    source_file_id: file_id,
                    var_name: decl.var_name.clone(),
                    source_kind: decl.source_kind.to_string(),
                    required: decl.required,
                    default_value_redacted: decl.default_value_redacted.clone(),
                    description: decl.description.clone(),
                    confidence: decl.confidence,
                });

                total_declarations += 1;

                if decl_batch.len() >= ENV_SCHEMA_BATCH_SIZE {
                    self.insert_declaration_batch(&decl_batch, &now)?;
                    decl_batch.clear();
                }
            }

            files_processed += 1;
        }

        // Flush remaining declarations
        if !decl_batch.is_empty() {
            self.insert_declaration_batch(&decl_batch, &now)?;
        }

        // 4. Extract env var references from source files already in project_symbols
        let mut total_references = 0usize;
        let mut ref_batch: Vec<EnvReferenceRow> = Vec::new();

        let source_files = self.get_source_files_with_symbols()?;
        for (file_id, file_path) in &source_files {
            let full_path = self.repo_path.join(file_path);
            let content = match std::fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let path = std::path::PathBuf::from(file_path);
            let references = EnvSchemaExtractor::extract_references_from_source(&path, &content);

            for env_ref in &references {
                // Find the symbol_id for this file if the reference appears near a known symbol
                let symbol_id = self.find_symbol_id_for_file(*file_id, &env_ref.var_name)?;

                ref_batch.push(EnvReferenceRow {
                    file_id: *file_id,
                    symbol_id,
                    var_name: env_ref.var_name.clone(),
                    reference_kind: env_ref.reference_kind.to_string(),
                    confidence: env_ref.confidence,
                    line_start: None, // Line-level resolution not yet available
                });

                total_references += 1;

                if ref_batch.len() >= ENV_SCHEMA_BATCH_SIZE {
                    self.insert_reference_batch(&ref_batch, &now)?;
                    ref_batch.clear();
                }
            }
        }

        // Flush remaining references
        if !ref_batch.is_empty() {
            self.insert_reference_batch(&ref_batch, &now)?;
        }

        info!(
            "Env schema extraction complete: {} declarations from {} files, {} references ({} dotenv, {} config)",
            total_declarations,
            files_processed,
            total_references,
            dotenv_declarations,
            config_declarations
        );

        Ok(EnvSchemaStats {
            total_declarations,
            total_references,
            dotenv_declarations,
            config_declarations,
            files_processed,
        })
    }

    fn discover_env_files(&self) -> Vec<(String, String)> {
        let mut env_files = Vec::new();

        // .env.example, .env.template, .env.sample
        for name in &[
            ".env.example",
            ".env.template",
            ".env.sample",
            ".env.local.example",
        ] {
            let path = self.repo_path.join(name);
            if path.exists() {
                env_files.push((name.to_string(), "dotenv".to_string()));
            }
        }

        // config.toml
        let config_toml = self.repo_path.join("config.toml");
        if config_toml.exists() {
            env_files.push(("config.toml".to_string(), "toml".to_string()));
        }

        // config.json
        let config_json = self.repo_path.join("config.json");
        if config_json.exists() {
            env_files.push(("config.json".to_string(), "json".to_string()));
        }

        env_files
    }

    fn ensure_file_entry(&self, relative_path: &str, content: &str, now: &str) -> Result<i64> {
        let content_hash = blake3::hash(content.as_bytes()).to_hex().to_string();

        let conn = self.storage.get_connection();
        let existing_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM project_files WHERE file_path = ?1",
                [relative_path],
                |row| row.get(0),
            )
            .ok();

        if let Some(id) = existing_id {
            return Ok(id);
        }

        let conn = self.storage.get_connection();
        let tx = conn.unchecked_transaction().into_diagnostic()?;
        tx.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, parser_version, parse_status, last_indexed_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                relative_path,
                "Dotenv", // Most env config files
                content_hash,
                content.len() as i64,
                "1",
                "OK",
                now,
            ],
        )
        .into_diagnostic()?;

        let id = tx.last_insert_rowid();
        tx.commit().into_diagnostic()?;
        Ok(id)
    }

    fn get_source_files_with_symbols(&self) -> Result<Vec<(i64, String)>> {
        let conn = self.storage.get_connection();
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT pf.id, pf.file_path FROM project_files pf \
                 JOIN project_symbols ps ON pf.id = ps.file_id \
                 WHERE pf.parse_status != 'DELETED'",
            )
            .into_diagnostic()?;

        let rows: Vec<(i64, String)> = stmt
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .into_diagnostic()?
            .collect::<Result<Vec<_>, _>>()
            .into_diagnostic()?;

        Ok(rows)
    }

    fn find_symbol_id_for_file(&self, file_id: i64, _var_name: &str) -> Result<Option<i64>> {
        // For now, return None as we don't have line-level resolution
        // In a future iteration, we could match env var references to specific symbols
        let _ = (file_id, _var_name);
        Ok(None)
    }

    fn insert_declaration_batch(&self, rows: &[EnvDeclarationRow], now: &str) -> Result<()> {
        let conn = self.storage.get_connection();
        let tx = conn.unchecked_transaction().into_diagnostic()?;

        for row in rows {
            tx.execute(
                "INSERT OR IGNORE INTO env_declarations (var_name, source_file_id, source_kind, required, default_value_redacted, description, confidence, last_indexed_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    row.var_name,
                    row.source_file_id,
                    row.source_kind,
                    row.required as i32,
                    row.default_value_redacted,
                    row.description,
                    row.confidence,
                    now,
                ],
            )
            .into_diagnostic()?;
        }

        tx.commit().into_diagnostic()?;
        Ok(())
    }

    fn insert_reference_batch(&self, rows: &[EnvReferenceRow], now: &str) -> Result<()> {
        let conn = self.storage.get_connection();
        let tx = conn.unchecked_transaction().into_diagnostic()?;

        for row in rows {
            tx.execute(
                "INSERT OR IGNORE INTO env_references (file_id, symbol_id, var_name, reference_kind, confidence, line_start, last_indexed_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    row.file_id,
                    row.symbol_id,
                    row.var_name,
                    row.reference_kind,
                    row.confidence,
                    row.line_start,
                    now,
                ],
            )
            .into_diagnostic()?;
        }

        tx.commit().into_diagnostic()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extract_from_dotenv_basic() {
        let content = r#"
# Database configuration
DATABASE_URL=postgres://localhost:5432/mydb
API_KEY=your-api-key-here
EMPTY_VAR=
# Commented out
# DISABLED_VAR=value
NO_EQUALS_VAR
export EXPORTED_VAR=value
"#;
        let decls = EnvSchemaExtractor::extract_from_dotenv(content);
        let db_decl = decls.iter().find(|d| d.var_name == "DATABASE_URL").unwrap();
        assert_eq!(db_decl.source_kind, EnvSourceKind::DotenvExample);
        assert_eq!(db_decl.default_value_redacted.as_deref(), Some(HAS_DEFAULT));
        assert!(!db_decl.required);

        let api_decl = decls.iter().find(|d| d.var_name == "API_KEY").unwrap();
        assert_eq!(
            api_decl.default_value_redacted.as_deref(),
            Some(POSSIBLE_SECRET_REDACTED)
        );

        let empty_decl = decls.iter().find(|d| d.var_name == "EMPTY_VAR").unwrap();
        assert_eq!(
            empty_decl.default_value_redacted.as_deref(),
            Some(EMPTY_DEFAULT)
        );
        assert!(empty_decl.required);

        let no_eq = decls
            .iter()
            .find(|d| d.var_name == "NO_EQUALS_VAR")
            .unwrap();
        assert!(no_eq.required);
        assert!(no_eq.default_value_redacted.is_none());
        assert!((no_eq.confidence - 0.7).abs() < f64::EPSILON);

        let exported = decls.iter().find(|d| d.var_name == "EXPORTED_VAR").unwrap();
        assert_eq!(
            exported.default_value_redacted.as_deref(),
            Some(HAS_DEFAULT)
        );
    }

    #[test]
    fn test_extract_from_dotenv_placeholder_values() {
        let content = r#"
SESSION_SECRET=change_me
REDIS_URL=replace-with-your-url
APP_PORT=3000
"#;
        let decls = EnvSchemaExtractor::extract_from_dotenv(content);
        let secret_decl = decls
            .iter()
            .find(|d| d.var_name == "SESSION_SECRET")
            .unwrap();
        assert_eq!(
            secret_decl.default_value_redacted.as_deref(),
            Some(POSSIBLE_SECRET_REDACTED)
        );

        let redis_decl = decls.iter().find(|d| d.var_name == "REDIS_URL").unwrap();
        assert_eq!(
            redis_decl.default_value_redacted.as_deref(),
            Some(PLACEHOLDER_DEFAULT)
        );

        let port_decl = decls.iter().find(|d| d.var_name == "APP_PORT").unwrap();
        assert_eq!(
            port_decl.default_value_redacted.as_deref(),
            Some(HAS_DEFAULT)
        );
    }

    #[test]
    fn test_extract_from_dotenv_quoted_values() {
        let content = r#"
GREETING="Hello, World!"
FAREWELL='Goodbye!'
"#;
        let decls = EnvSchemaExtractor::extract_from_dotenv(content);
        let greeting = decls.iter().find(|d| d.var_name == "GREETING").unwrap();
        assert_eq!(
            greeting.default_value_redacted.as_deref(),
            Some(HAS_DEFAULT)
        );
    }

    #[test]
    fn test_extract_from_toml_env_section() {
        let content = r#"
[server]
port = 8080

[env]
DATABASE_URL = "postgres://localhost/mydb"
REDIS_URL = ""
"#;
        let decls = EnvSchemaExtractor::extract_from_toml(content);
        let db_decl = decls.iter().find(|d| d.var_name == "DATABASE_URL").unwrap();
        assert_eq!(db_decl.source_kind, EnvSourceKind::Config);
        assert_eq!(db_decl.default_value_redacted.as_deref(), Some(HAS_DEFAULT));

        let redis_decl = decls.iter().find(|d| d.var_name == "REDIS_URL").unwrap();
        assert_eq!(
            redis_decl.default_value_redacted.as_deref(),
            Some(EMPTY_DEFAULT)
        );
    }

    #[test]
    fn test_extract_from_json_basic() {
        let content = r#"
{
    "DATABASE_URL": "postgres://localhost/mydb",
    "API_KEY": "sk-test-key",
    "PORT": 3000,
    "DEBUG": true,
    "NESTED": {
        "inner": "value"
    }
}
"#;
        let decls = EnvSchemaExtractor::extract_from_json(content);
        let db_decl = decls.iter().find(|d| d.var_name == "DATABASE_URL").unwrap();
        assert_eq!(db_decl.source_kind, EnvSourceKind::Config);
        assert_eq!(db_decl.default_value_redacted.as_deref(), Some(HAS_DEFAULT));

        let api_decl = decls.iter().find(|d| d.var_name == "API_KEY").unwrap();
        assert_eq!(
            api_decl.default_value_redacted.as_deref(),
            Some(POSSIBLE_SECRET_REDACTED)
        );

        // PORT and DEBUG should be found
        assert!(decls.iter().any(|d| d.var_name == "PORT"));
        assert!(decls.iter().any(|d| d.var_name == "DEBUG"));
    }

    #[test]
    fn test_extract_references_from_source_rust() {
        let content = r#"
let db_url = std::env::var("DATABASE_URL").unwrap_or("sqlite://default");
let api_key = std::env::var("API_TOKEN");
let debug = env!("DEBUG_MODE");
std::env::set_var("RUNTIME_FLAG", "1");
"#;
        let refs = EnvSchemaExtractor::extract_references_from_source(
            &PathBuf::from("src/main.rs"),
            content,
        );

        assert!(refs.iter().any(
            |r| r.var_name == "DATABASE_URL" && r.reference_kind == EnvReferenceKind::Defaulted
        ));
        assert!(
            refs.iter()
                .any(|r| r.var_name == "API_TOKEN" && r.reference_kind == EnvReferenceKind::Read)
        );
        assert!(
            refs.iter()
                .any(|r| r.var_name == "DEBUG_MODE" && r.reference_kind == EnvReferenceKind::Read)
        );
        assert!(
            refs.iter().any(
                |r| r.var_name == "RUNTIME_FLAG" && r.reference_kind == EnvReferenceKind::Write
            )
        );
    }

    #[test]
    fn test_extract_references_from_source_typescript() {
        let content = r#"
const dbUrl = process.env.DATABASE_URL || 'sqlite://default';
const apiKey = process.env.API_KEY;
process.env.RUNTIME_FLAG = '1';
"#;
        let refs = EnvSchemaExtractor::extract_references_from_source(
            &PathBuf::from("src/app.ts"),
            content,
        );

        assert!(refs.iter().any(
            |r| r.var_name == "DATABASE_URL" && r.reference_kind == EnvReferenceKind::Defaulted
        ));
        assert!(
            refs.iter()
                .any(|r| r.var_name == "API_KEY" && r.reference_kind == EnvReferenceKind::Read)
        );
        assert!(
            refs.iter().any(
                |r| r.var_name == "RUNTIME_FLAG" && r.reference_kind == EnvReferenceKind::Write
            )
        );
    }

    #[test]
    fn test_extract_references_from_source_python() {
        let content = r#"
db_url = os.getenv("DATABASE_URL")
debug_mode = os.environ.get("DEBUG_MODE", "false")
"#;
        let refs =
            EnvSchemaExtractor::extract_references_from_source(&PathBuf::from("app.py"), content);

        assert!(refs.iter().any(|r| r.var_name == "DATABASE_URL" && r.reference_kind == EnvReferenceKind::Read));
        assert!(
            refs.iter()
                .any(|r| r.var_name == "DEBUG_MODE"
                    && r.reference_kind == EnvReferenceKind::Defaulted)
        );
    }

    #[test]
    fn test_find_undeclared_basic() {
        let references = vec![
            EnvReference {
                var_name: "DATABASE_URL".to_string(),
                reference_kind: EnvReferenceKind::Read,
                confidence: 1.0,
            },
            EnvReference {
                var_name: "NEW_SECRET".to_string(),
                reference_kind: EnvReferenceKind::Read,
                confidence: 1.0,
            },
            EnvReference {
                var_name: "*".to_string(),
                reference_kind: EnvReferenceKind::Dynamic,
                confidence: 0.5,
            },
        ];
        let declarations = vec![EnvDeclaration {
            var_name: "DATABASE_URL".to_string(),
            source_kind: EnvSourceKind::DotenvExample,
            required: true,
            default_value_redacted: Some(HAS_DEFAULT.to_string()),
            description: None,
            confidence: 1.0,
        }];

        let undeclared = EnvSchemaExtractor::find_undeclared(&references, &declarations);
        assert_eq!(undeclared.len(), 1);
        assert_eq!(undeclared[0].var_name, "NEW_SECRET");
        assert!(!undeclared[0].declared);
        assert!(undeclared[0].evidence.contains("READ"));
    }

    #[test]
    fn test_find_undeclared_all_declared() {
        let references = vec![EnvReference {
            var_name: "DATABASE_URL".to_string(),
            reference_kind: EnvReferenceKind::Read,
            confidence: 1.0,
        }];
        let declarations = vec![EnvDeclaration {
            var_name: "DATABASE_URL".to_string(),
            source_kind: EnvSourceKind::DotenvExample,
            required: true,
            default_value_redacted: Some(HAS_DEFAULT.to_string()),
            description: None,
            confidence: 1.0,
        }];

        let undeclared = EnvSchemaExtractor::find_undeclared(&references, &declarations);
        assert!(undeclared.is_empty());
    }

    #[test]
    fn test_find_undeclared_empty() {
        let undeclared = EnvSchemaExtractor::find_undeclared(&[], &[]);
        assert!(undeclared.is_empty());
    }

    #[test]
    fn test_redact_default() {
        assert_eq!(redact_default("PORT", ""), EMPTY_DEFAULT);
        assert_eq!(redact_default("PORT", "3000"), HAS_DEFAULT);
        assert_eq!(
            redact_default("API_KEY", "sk-abc123"),
            POSSIBLE_SECRET_REDACTED
        );
        assert_eq!(
            redact_default("APP_URL", "your-url-here"),
            PLACEHOLDER_DEFAULT
        );
        assert_eq!(
            redact_default("SESSION_SECRET", "change_me"),
            POSSIBLE_SECRET_REDACTED
        );
    }

    #[test]
    fn test_is_secret_name() {
        assert!(is_secret_name("API_KEY"));
        assert!(is_secret_name("DATABASE_PASSWORD"));
        assert!(is_secret_name("AUTH_TOKEN"));
        assert!(is_secret_name("SECRET_VALUE"));
        assert!(is_secret_name("PRIVATE_KEY"));
        assert!(!is_secret_name("PORT"));
        assert!(!is_secret_name("APP_NAME"));
    }

    #[test]
    fn test_is_placeholder_value() {
        assert!(is_placeholder_value("your-api-key-here"));
        assert!(is_placeholder_value("xxx"));
        assert!(is_placeholder_value("change_me"));
        assert!(is_placeholder_value("<insert-key>"));
        assert!(!is_placeholder_value("3000"));
        assert!(!is_placeholder_value("postgres://localhost/db"));
    }

    #[test]
    fn test_env_source_kind_display() {
        assert_eq!(EnvSourceKind::DotenvExample.to_string(), "DOTENV_EXAMPLE");
        assert_eq!(EnvSourceKind::Config.to_string(), "CONFIG");
        assert_eq!(EnvSourceKind::Docs.to_string(), "DOCS");
    }

    #[test]
    fn test_env_reference_kind_display() {
        assert_eq!(EnvReferenceKind::Read.to_string(), "READ");
        assert_eq!(EnvReferenceKind::Write.to_string(), "WRITE");
        assert_eq!(EnvReferenceKind::Defaulted.to_string(), "DEFAULTED");
        assert_eq!(EnvReferenceKind::Dynamic.to_string(), "DYNAMIC");
    }

    #[test]
    fn test_extract_from_dotenv_inline_comment() {
        let content = "REDIS_URL=redis://localhost # local dev\n";
        let decls = EnvSchemaExtractor::extract_from_dotenv(content);
        let redis_decl = decls.iter().find(|d| d.var_name == "REDIS_URL").unwrap();
        assert_eq!(
            redis_decl.default_value_redacted.as_deref(),
            Some(HAS_DEFAULT)
        );
    }

    #[test]
    fn test_extract_references_dynamic_pattern() {
        let content = r#"
for (key, value) in std::env::vars() {
    println!("{}={}", key, value);
}
"#;
        let refs = EnvSchemaExtractor::extract_references_from_source(
            &PathBuf::from("src/main.rs"),
            content,
        );
        assert!(
            refs.iter()
                .any(|r| r.var_name == "*" && r.reference_kind == EnvReferenceKind::Dynamic)
        );
    }

    #[test]
    fn test_env_var_dep_ordering() {
        let dep1 = EnvVarDep {
            var_name: "API_KEY".to_string(),
            declared: false,
            evidence: "Referenced as READ".to_string(),
        };
        let dep2 = EnvVarDep {
            var_name: "DATABASE_URL".to_string(),
            declared: false,
            evidence: "Referenced as READ".to_string(),
        };
        assert!(dep1 < dep2); // A comes before D
    }
}
