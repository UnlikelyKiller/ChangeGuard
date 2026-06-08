use crate::state::storage::StorageManager;
use miette::{IntoDiagnostic, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::LazyLock;

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
    pub is_secret: bool,
    pub default_value_redacted: Option<String>,
    pub description: Option<String>,
    pub owner: Option<String>,
    pub environment: Option<String>,
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

// --- Regex patterns ---
static RUST_ENV_VAR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"std::env::var\("([^"]+)"\)"#).unwrap());
static RUST_ENV_MACRO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"env!\("([^"]+)"\)"#).unwrap());
#[allow(dead_code)]
static RUST_ENV_VAR_DEFAULT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"std::env::var\("([^"]+)"\)\.(?:unwrap_or|ok|unwrap_or_else)"#).unwrap());
static TS_ENV_DOT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"process\.env\.([A-Z_][A-Z0-9_]*)"#).unwrap());
#[allow(dead_code)]
static TS_ENV_INDEXED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"process\.env\[['"]([^'"]+)['"]\]"#).unwrap());
#[allow(dead_code)]
static TS_ENV_DEFAULT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"process\.env\.([A-Z_][A-Z0-9_]*)\s*\|\|"#).unwrap());
static PY_ENV_GET: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"os\.(?:environ\.get|getenv)\(['"]([^'"]+)['"]\)"#).unwrap());
#[allow(dead_code)]
static PY_ENV_GET_DEFAULT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"os\.(?:environ\.get|getenv)\(['"]([^'"]+)['"]\s*,\s*"#).unwrap());
#[allow(dead_code)]
static PY_ENV_INDEXED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"os\.environ\[['"]([^'"]+)['"]\]"#).unwrap());
#[allow(dead_code)]
static RUST_SET_ENV: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"std::env::set_var\("([^"]+)""#).unwrap());
#[allow(dead_code)]
static TS_SET_ENV: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"process\.env\[?\.?([A-Z_][A-Z0-9_]*)\]?\s*="#).unwrap());

fn collect_env_captures(regex: &Regex, content: &str, out: &mut Vec<String>) {
    for capture in regex.captures_iter(content) {
        if let Some(m) = capture.get(1) {
            out.push(m.as_str().to_string());
        }
    }
}

pub struct EnvSchemaExtractor;

impl EnvSchemaExtractor {
    pub fn extract_from_dotenv(content: &str) -> Vec<EnvDeclaration> {
        let mut declarations = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim().strip_prefix("export ").unwrap_or(trimmed[..eq_pos].trim()).trim().to_string();
                let value = trimmed[eq_pos + 1..].trim().trim_matches('"').trim_matches('\'').to_string();
                if key.is_empty() { continue; }
                let default_value_redacted = if value.is_empty() { Some(EMPTY_DEFAULT.to_string()) } else { Some(redact_default(&key, &value)) };
                declarations.push(EnvDeclaration {
                    var_name: key.clone(),
                    source_kind: EnvSourceKind::DotenvExample,
                    required: value.is_empty(),
                    is_secret: is_secret_name(&key),
                    default_value_redacted,
                    description: None,
                    owner: None,
                    environment: None,
                    confidence: 1.0,
                });
            }
        }
        declarations.sort_unstable();
        declarations.dedup();
        declarations
    }

    pub fn extract_from_toml(content: &str) -> Vec<EnvDeclaration> {
        let mut declarations = Vec::new();
        let mut in_env_section = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') {
                let section = trimmed.trim_start_matches('[').trim_end_matches(']');
                in_env_section = section == "env" || section.starts_with("env.");
                continue;
            }
            if in_env_section && let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim().to_string();
                let value = trimmed[eq_pos + 1..].trim().trim_matches('"').trim_matches('\'').to_string();
                declarations.push(EnvDeclaration {
                    var_name: key.clone(),
                    source_kind: EnvSourceKind::Config,
                    required: false,
                    is_secret: is_secret_name(&key),
                    default_value_redacted: Some(redact_default(&key, &value)),
                    description: None,
                    owner: None,
                    environment: None,
                    confidence: 0.85,
                });
            }
        }
        declarations
    }

    pub fn extract_from_json(content: &str) -> Vec<EnvDeclaration> {
        let mut declarations = Vec::new();
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(obj) = val.as_object() {
                for (key, _val) in obj {
                    if key.chars().all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit()) {
                        declarations.push(EnvDeclaration {
                            var_name: key.clone(),
                            source_kind: EnvSourceKind::Config,
                            required: false,
                            is_secret: is_secret_name(key),
                            default_value_redacted: Some(HAS_DEFAULT.to_string()),
                            description: None,
                            owner: None,
                            environment: None,
                            confidence: 0.7,
                        });
                    }
                }
            }
        }
        declarations
    }

    pub fn extract_references_from_source(path: &Path, content: &str) -> Vec<EnvReference> {
        let mut result = Vec::new();
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or_default();
        let mut names = Vec::new();
        match extension {
            "rs" => {
                collect_env_captures(&RUST_ENV_VAR, content, &mut names);
                collect_env_captures(&RUST_ENV_MACRO, content, &mut names);
            }
            "ts" | "js" => {
                collect_env_captures(&TS_ENV_DOT, content, &mut names);
            }
            "py" => {
                collect_env_captures(&PY_ENV_GET, content, &mut names);
            }
            _ => {}
        }
        for name in names {
            result.push(EnvReference { var_name: name, reference_kind: EnvReferenceKind::Read, confidence: 1.0 });
        }
        result
    }

    pub fn find_undeclared(references: &[EnvReference], declarations: &[EnvDeclaration]) -> Vec<EnvVarDep> {
        let declared: std::collections::HashSet<_> = declarations.iter().map(|d| &d.var_name).collect();
        references.iter().filter(|r| !declared.contains(&r.var_name)).map(|r| EnvVarDep {
            var_name: r.var_name.clone(),
            declared: false,
            evidence: format!("Referenced as {:?}", r.reference_kind),
        }).collect()
    }
}

pub struct EnvSchemaIndexer<'a> {
    storage: &'a StorageManager,
    repo_path: std::path::PathBuf,
}

struct EnvDeclarationRow {
    source_file_id: i64,
    var_name: String,
    source_kind: String,
    required: bool,
    is_secret: bool,
    default_value_redacted: Option<String>,
    description: Option<String>,
    owner: Option<String>,
    environment: Option<String>,
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

impl<'a> EnvSchemaIndexer<'a> {
    pub fn new(storage: &'a StorageManager, repo_path: std::path::PathBuf) -> Self {
        Self { storage, repo_path }
    }

    pub fn extract(&self) -> Result<EnvSchemaStats> {
        let now = chrono::Utc::now().to_rfc3339();
        let mut decls = Vec::new();
        // ... (simplified discovery for now to keep implementation focused on persistence)
        let example_path = self.repo_path.join(".env.example");
        if example_path.exists() {
            let content = std::fs::read_to_string(&example_path).into_diagnostic()?;
            decls.extend(EnvSchemaExtractor::extract_from_dotenv(&content));
        }

        let stats = EnvSchemaStats {
            total_declarations: decls.len(),
            total_references: 0,
            dotenv_declarations: decls.len(),
            config_declarations: 0,
            files_processed: 1,
        };

        let rows: Vec<EnvDeclarationRow> = decls.into_iter().map(|d| EnvDeclarationRow {
            source_file_id: 1, // Placeholder
            var_name: d.var_name,
            source_kind: d.source_kind.to_string(),
            required: d.required,
            is_secret: d.is_secret,
            default_value_redacted: d.default_value_redacted,
            description: d.description,
            owner: d.owner,
            environment: d.environment,
            confidence: d.confidence,
        }).collect();

        self.insert_declaration_batch(&rows, &now)?;
        Ok(stats)
    }

    fn insert_declaration_batch(&self, rows: &[EnvDeclarationRow], now: &str) -> Result<()> {
        let conn = self.storage.get_connection();
        let tx = conn.unchecked_transaction().into_diagnostic()?;
        for row in rows {
            tx.execute(
                "INSERT OR IGNORE INTO env_declarations (var_name, source_file_id, source_kind, required, is_secret, default_value_redacted, description, owner, environment, confidence, last_indexed_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![row.var_name, row.source_file_id, row.source_kind, row.required as i32, row.is_secret as i32, row.default_value_redacted, row.description, row.owner, row.environment, row.confidence, now],
            ).into_diagnostic()?;
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
                rusqlite::params![row.file_id, row.symbol_id, row.var_name, row.reference_kind, row.confidence, row.line_start, now],
            ).into_diagnostic()?;
        }
        tx.commit().into_diagnostic()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EnvSchemaStats {
    pub total_declarations: usize,
    pub total_references: usize,
    pub dotenv_declarations: usize,
    pub config_declarations: usize,
    pub files_processed: usize,
}
