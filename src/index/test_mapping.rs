use crate::state::storage::StorageManager;
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TestMappingStats {
    pub total_mappings: usize,
    pub import_mappings: usize,
    pub naming_convention_mappings: usize,
    pub files_processed: usize,
}

struct TestMappingRow {
    test_symbol_id: i64,
    test_file_id: i64,
    tested_symbol_id: Option<i64>,
    tested_file_id: Option<i64>,
    confidence: f64,
    mapping_kind: String,
    evidence: Option<String>,
}

const TEST_MAPPING_BATCH_SIZE: usize = 500;

pub struct TestMapper<'a> {
    storage: &'a StorageManager,
    repo_path: PathBuf,
}

impl<'a> TestMapper<'a> {
    pub fn new(storage: &'a StorageManager, repo_path: PathBuf) -> Self {
        Self { storage, repo_path }
    }

    pub fn extract(&self) -> Result<TestMappingStats> {
        let conn = self.storage.get_connection();

        // 1. Query all test symbols (where symbol_kind = 'Function' AND name starts with 'test_'
        //    OR symbol_kind = 'Function' AND file is in a test directory)
        let mut test_stmt = conn
            .prepare(
                "SELECT ps.id, ps.symbol_name, ps.qualified_name, ps.file_id, pf.file_path, pf.language
                 FROM project_symbols ps
                 JOIN project_files pf ON ps.file_id = pf.id
                 WHERE ps.symbol_kind = 'Function'
                 AND pf.parse_status != 'DELETED'",
            )
            .into_diagnostic()?;

        let all_function_rows: Vec<(i64, String, String, i64, String, Option<String>)> = test_stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                ))
            })
            .into_diagnostic()?
            .collect::<Result<Vec<_>, _>>()
            .into_diagnostic()?;

        drop(test_stmt);

        // 2. Filter to test functions
        let test_functions: Vec<(i64, String, String, i64, String, Option<String>)> =
            all_function_rows
                .iter()
                .filter(|(_, name, _, _, path, language)| {
                    is_test_function(name, path, language.as_deref())
                })
                .cloned()
                .collect();

        if test_functions.is_empty() {
            info!("Test mapping extraction: no test functions found");
            return Ok(TestMappingStats {
                total_mappings: 0,
                import_mappings: 0,
                naming_convention_mappings: 0,
                files_processed: 0,
            });
        }

        // 3. Build a symbol lookup: symbol_name -> Vec<(id, file_id, qualified_name)>
        let mut symbol_lookup: HashMap<String, Vec<(i64, i64, String)>> = HashMap::new();
        let mut sym_stmt = conn
            .prepare(
                "SELECT ps.id, ps.symbol_name, ps.file_id, ps.qualified_name
                 FROM project_symbols ps
                 JOIN project_files pf ON ps.file_id = pf.id
                 WHERE ps.symbol_kind = 'Function'
                 AND pf.parse_status != 'DELETED'",
            )
            .into_diagnostic()?;

        let symbol_rows: Vec<(i64, String, i64, String)> = sym_stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .into_diagnostic()?
            .collect::<Result<Vec<_>, _>>()
            .into_diagnostic()?;

        drop(sym_stmt);

        for (id, name, file_id, qualified_name) in &symbol_rows {
            symbol_lookup.entry(name.clone()).or_default().push((
                *id,
                *file_id,
                qualified_name.clone(),
            ));
        }

        // Build file_id -> file_path lookup
        let mut file_stmt = conn
            .prepare("SELECT id, file_path FROM project_files WHERE parse_status != 'DELETED'")
            .into_diagnostic()?;

        let file_rows: Vec<(i64, String)> = file_stmt
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .into_diagnostic()?
            .collect::<Result<Vec<_>, _>>()
            .into_diagnostic()?;

        drop(file_stmt);

        let _file_id_to_path: HashMap<i64, String> = file_rows.into_iter().collect();

        // 4. Delete existing test_mapping data before re-indexing
        {
            let conn = self.storage.get_connection();
            conn.execute("DELETE FROM test_mapping", [])
                .into_diagnostic()?;
        }

        // 5. For each test function, find mappings
        let mut total_mappings = 0usize;
        let mut import_mappings = 0usize;
        let mut naming_convention_mappings = 0usize;
        let mut batch: Vec<TestMappingRow> = Vec::new();
        let mut processed_test_files: std::collections::HashSet<i64> =
            std::collections::HashSet::new();

        for (test_sym_id, test_name, _qualified, test_file_id, test_file_path, _test_lang) in
            &test_functions
        {
            processed_test_files.insert(*test_file_id);

            // a. Read the file content to extract import/use statements
            let full_path = self.repo_path.join(test_file_path.replace('\\', "/"));
            let content = match std::fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // b. Extract import/use statements and resolve against project_symbols
            let imported_names = extract_imported_names(&content, test_file_path);

            for imported in &imported_names {
                // Try to resolve the imported name against project_symbols
                if let Some(candidates) = symbol_lookup.get(imported) {
                    for (tested_sym_id, tested_file_id, _qualified_name) in candidates {
                        // Don't map test symbols to other test symbols in the same file
                        if *tested_sym_id == *test_sym_id {
                            continue;
                        }

                        batch.push(TestMappingRow {
                            test_symbol_id: *test_sym_id,
                            test_file_id: *test_file_id,
                            tested_symbol_id: Some(*tested_sym_id),
                            tested_file_id: Some(*tested_file_id),
                            confidence: 1.0,
                            mapping_kind: "IMPORT".to_string(),
                            evidence: Some(format!("import: {}", imported)),
                        });

                        import_mappings += 1;

                        if batch.len() >= TEST_MAPPING_BATCH_SIZE {
                            total_mappings += batch.len();
                            self.insert_batch(&batch)?;
                            batch.clear();
                        }
                    }
                }
            }

            // c. Check naming conventions (test_foo -> foo)
            let stripped_name = strip_test_prefix(test_name);
            if stripped_name != test_name.as_str() {
                // The test is named test_foo, so it likely tests foo
                if let Some(candidates) = symbol_lookup.get(stripped_name) {
                    for (tested_sym_id, tested_file_id, _qualified_name) in candidates {
                        if *tested_sym_id == *test_sym_id {
                            continue;
                        }

                        batch.push(TestMappingRow {
                            test_symbol_id: *test_sym_id,
                            test_file_id: *test_file_id,
                            tested_symbol_id: Some(*tested_sym_id),
                            tested_file_id: Some(*tested_file_id),
                            confidence: 0.5,
                            mapping_kind: "NAMING_CONVENTION".to_string(),
                            evidence: Some(format!("naming: {} -> {}", test_name, stripped_name)),
                        });

                        naming_convention_mappings += 1;

                        if batch.len() >= TEST_MAPPING_BATCH_SIZE {
                            total_mappings += batch.len();
                            self.insert_batch(&batch)?;
                            batch.clear();
                        }
                    }
                }
            }
        }

        // Flush remaining
        if !batch.is_empty() {
            total_mappings += batch.len();
            self.insert_batch(&batch)?;
        }

        let files_processed = processed_test_files.len();

        info!(
            "Test mapping extraction complete: {} mappings ({} import, {} naming convention) from {} test files",
            total_mappings, import_mappings, naming_convention_mappings, files_processed
        );

        Ok(TestMappingStats {
            total_mappings,
            import_mappings,
            naming_convention_mappings,
            files_processed,
        })
    }

    fn insert_batch(&self, rows: &[TestMappingRow]) -> Result<()> {
        let conn = self.storage.get_connection();
        let tx = conn.unchecked_transaction().into_diagnostic()?;
        let now = chrono::Utc::now().to_rfc3339();

        for row in rows {
            tx.execute(
                "INSERT OR IGNORE INTO test_mapping \
                 (test_symbol_id, test_file_id, tested_symbol_id, tested_file_id, confidence, mapping_kind, evidence, last_indexed_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    row.test_symbol_id,
                    row.test_file_id,
                    row.tested_symbol_id,
                    row.tested_file_id,
                    row.confidence,
                    row.mapping_kind,
                    row.evidence,
                    now,
                ],
            )
            .into_diagnostic()?;
        }

        tx.commit().into_diagnostic()?;
        Ok(())
    }
}

/// Determine if a function is a test function based on name and file path.
fn is_test_function(name: &str, file_path: &str, language: Option<&str>) -> bool {
    // Rust: #[test] annotated functions are already detected by entrypoint classification.
    // Here we use heuristics:
    // 1. Function name starts with "test_" (Rust, Python convention)
    // 2. Function name ends with "_test" (Go convention)
    // 3. File is in a test directory (tests/, test/, __tests__/)
    if name.starts_with("test_") || name.ends_with("_test") {
        return true;
    }

    // Check if file is in a test directory
    let normalized_path = file_path.replace('\\', "/");
    if language == Some("Rust") {
        // Rust test files are typically in tests/ directory or are *tests.rs modules
        if normalized_path.starts_with("tests/")
            || normalized_path.contains("/tests/")
            || normalized_path.ends_with("_test.rs")
            || normalized_path.ends_with("_tests.rs")
        {
            return true;
        }
    }

    if normalized_path.starts_with("tests/")
        || normalized_path.starts_with("test/")
        || normalized_path.contains("/tests/")
        || normalized_path.contains("/test/")
        || normalized_path.contains("/__tests__/")
        || normalized_path.contains("/spec/")
    {
        return true;
    }

    // TypeScript/JavaScript test files
    if (language == Some("TypeScript") || language == Some("JavaScript"))
        && (normalized_path.ends_with(".test.ts")
            || normalized_path.ends_with(".test.tsx")
            || normalized_path.ends_with(".test.js")
            || normalized_path.ends_with(".test.jsx")
            || normalized_path.ends_with(".spec.ts")
            || normalized_path.ends_with(".spec.tsx")
            || normalized_path.ends_with(".spec.js")
            || normalized_path.ends_with(".spec.jsx"))
    {
        return true;
    }

    // Python test files
    if language == Some("Python")
        && (normalized_path.starts_with("tests/")
            || normalized_path.starts_with("test/")
            || normalized_path.contains("/tests/")
            || normalized_path.contains("/test/")
            || normalized_path.ends_with("_test.py")
            || normalized_path.starts_with("test_"))
    {
        return true;
    }

    false
}

/// Strip test prefix from a function name to get the tested name.
/// test_foo -> foo, test_bar_baz -> bar_baz, it_returns_something -> returns_something
fn strip_test_prefix(name: &str) -> &str {
    if let Some(stripped) = name.strip_prefix("test_") {
        return stripped;
    }
    if let Some(stripped) = name.strip_suffix("_test") {
        return stripped;
    }
    // Jest-style: it_does_something or should_do_something
    if let Some(stripped) = name.strip_prefix("it_") {
        return stripped;
    }
    if let Some(stripped) = name.strip_prefix("should_") {
        return stripped;
    }
    name
}

/// Extract imported names from source content based on language.
fn extract_imported_names(content: &str, file_path: &str) -> Vec<String> {
    let extension = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match extension {
        "rs" => extract_rust_imported_names(content),
        "ts" | "tsx" | "js" | "jsx" => extract_typescript_imported_names(content),
        "py" => extract_python_imported_names(content),
        _ => Vec::new(),
    }
}

/// Extract names from Rust use statements.
fn extract_rust_imported_names(content: &str) -> Vec<String> {
    let mut names = Vec::new();

    // Simple regex-based extraction of use statement final segments
    // e.g., "use crate::foo::bar;" -> "bar"
    // e.g., "use crate::foo::{baz, quux};" -> "baz", "quux"
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("use ") {
            continue;
        }

        // Remove "use " prefix and trailing ";"
        let use_path = trimmed
            .strip_prefix("use ")
            .unwrap_or("")
            .trim()
            .trim_end_matches(';')
            .trim();

        // Handle grouped imports: use crate::foo::{bar, baz}
        if let Some(start) = use_path.find("::{") {
            let _prefix = &use_path[..start];
            // Find the closing brace - skip past "::{ " which is 3 chars
            let brace_start = start + 3; // skip past "::{"
            let group_content = if let Some(end) = use_path[brace_start..].find('}') {
                &use_path[brace_start..brace_start + end]
            } else {
                &use_path[brace_start..]
            };
            for item in group_content.split(',') {
                let name = item.trim().to_string();
                if !name.is_empty() {
                    names.push(name);
                }
            }
        } else {
            // Simple import: use crate::foo::bar -> "bar"
            if let Some(last_segment) = use_path.rsplit("::").next() {
                let name = last_segment.trim().to_string();
                if !name.is_empty() && name != "self" && name != "super" {
                    names.push(name);
                }
            }
        }
    }

    names.sort_unstable();
    names.dedup();
    names
}

/// Extract names from TypeScript/JavaScript import statements.
fn extract_typescript_imported_names(content: &str) -> Vec<String> {
    let mut names = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("import ") {
            continue;
        }

        // Extract names from: import { foo, bar } from '...'
        // and: import foo from '...'
        // and: import * as foo from '...'
        if let Some(brace_start) = trimmed.find('{') {
            if let Some(brace_end) = trimmed.find('}') {
                let group = &trimmed[brace_start + 1..brace_end];
                for item in group.split(',') {
                    let name = item.split_whitespace().next().unwrap_or("").to_string();
                    if !name.is_empty() {
                        names.push(name);
                    }
                }
            }
        } else if trimmed.starts_with("import * as ") {
            // import * as foo from '...'
            let after_as = trimmed.strip_prefix("import * as ").unwrap_or("");
            if let Some(name) = after_as.split_whitespace().next() {
                names.push(name.to_string());
            }
        } else if trimmed.starts_with("import ") && !trimmed.contains(" from ") {
            // Side-effect import, skip
        } else {
            // Default import: import foo from '...'
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 && parts[0] == "import" && parts[1] != "type" {
                let name = parts[1].to_string();
                if !name.is_empty() && name != "from" && name != "{" {
                    names.push(name);
                }
            }
        }
    }

    names.sort_unstable();
    names.dedup();
    names
}

/// Extract names from Python import statements.
fn extract_python_imported_names(content: &str) -> Vec<String> {
    let mut names = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // from foo import bar, baz
        if trimmed.starts_with("from ") {
            if let Some(import_idx) = trimmed.find(" import ") {
                let after_import = &trimmed[import_idx + 8..];
                for item in after_import.split(',') {
                    let name = item.split_whitespace().next().unwrap_or("").to_string();
                    if !name.is_empty() {
                        names.push(name);
                    }
                }
            }
        }
        // import foo, bar
        else if trimmed.starts_with("import ") {
            let after_import = trimmed.strip_prefix("import ").unwrap_or("");
            for item in after_import.split(',') {
                let name = item.split_whitespace().next().unwrap_or("").to_string();
                if !name.is_empty() {
                    names.push(name);
                }
            }
        }
    }

    names.sort_unstable();
    names.dedup();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_test_function_prefix() {
        assert!(is_test_function("test_foo", "src/lib.rs", Some("Rust")));
        assert!(is_test_function(
            "test_bar_baz",
            "tests/test_bar.rs",
            Some("Rust")
        ));
    }

    #[test]
    fn test_is_test_function_suffix() {
        assert!(is_test_function("foo_test", "src/foo_test.go", None));
    }

    #[test]
    fn test_is_test_function_test_directory() {
        assert!(is_test_function(
            "my_function",
            "tests/integration.rs",
            Some("Rust")
        ));
        assert!(is_test_function(
            "my_function",
            "test/unit.py",
            Some("Python")
        ));
    }

    #[test]
    fn test_is_test_function_ts_spec_file() {
        assert!(is_test_function(
            "it_works",
            "src/app.test.ts",
            Some("TypeScript")
        ));
        assert!(is_test_function(
            "should_work",
            "src/app.spec.ts",
            Some("TypeScript")
        ));
    }

    #[test]
    fn test_is_test_function_non_test() {
        assert!(!is_test_function("calculate", "src/math.rs", Some("Rust")));
        assert!(!is_test_function(
            "process",
            "src/handler.ts",
            Some("TypeScript")
        ));
    }

    #[test]
    fn test_strip_test_prefix() {
        assert_eq!(strip_test_prefix("test_foo"), "foo");
        assert_eq!(strip_test_prefix("test_bar_baz"), "bar_baz");
        assert_eq!(strip_test_prefix("foo_test"), "foo");
        assert_eq!(strip_test_prefix("it_works"), "works");
        assert_eq!(strip_test_prefix("should_return_ok"), "return_ok");
        assert_eq!(strip_test_prefix("calculate"), "calculate");
    }

    #[test]
    fn test_extract_rust_imported_names() {
        let content = r#"
use crate::foo;
use crate::bar::baz;
use crate::models::{User, Product};
use super::helper;
"#;
        let names = extract_rust_imported_names(content);
        assert!(names.contains(&"foo".to_string()));
        assert!(names.contains(&"baz".to_string()));
        assert!(names.contains(&"User".to_string()));
        assert!(names.contains(&"Product".to_string()));
        assert!(names.contains(&"helper".to_string()));
    }

    #[test]
    fn test_extract_typescript_imported_names() {
        let content = r#"
import { foo, bar } from './utils';
import * as utils from './utils';
import baz from './baz';
"#;
        let names = extract_typescript_imported_names(content);
        assert!(names.contains(&"foo".to_string()));
        assert!(names.contains(&"bar".to_string()));
        assert!(names.contains(&"utils".to_string()));
        assert!(names.contains(&"baz".to_string()));
    }

    #[test]
    fn test_extract_python_imported_names() {
        let content = r#"
from myapp.models import User, Product
import os
import sys
"#;
        let names = extract_python_imported_names(content);
        assert!(names.contains(&"User".to_string()));
        assert!(names.contains(&"Product".to_string()));
        assert!(names.contains(&"os".to_string()));
        assert!(names.contains(&"sys".to_string()));
    }

    #[test]
    fn test_test_mapping_stats_serialization() {
        let stats = TestMappingStats {
            total_mappings: 100,
            import_mappings: 80,
            naming_convention_mappings: 20,
            files_processed: 15,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("totalMappings"));
        assert!(json.contains("importMappings"));
        assert!(json.contains("namingConventionMappings"));
        assert!(json.contains("filesProcessed"));
    }
}
