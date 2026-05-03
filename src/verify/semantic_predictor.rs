use crate::config::model::LocalModelConfig;
use crate::embed::embed_and_store;
use crate::impact::packet::ImpactPacket;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
}

impl TestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TestStatus::Passed => "pass",
            TestStatus::Failed => "fail",
            TestStatus::Skipped => "skip",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestOutcome {
    pub test_name: String,
    pub test_file: String,
    pub commit_hash: String,
    pub status: TestStatus,
    pub duration_ms: u64,
    pub diff_summary: String,
}

pub fn build_diff_text(packet: &ImpactPacket) -> String {
    let mut items: Vec<String> = Vec::new();

    for file in &packet.changes {
        items.push(file.path.to_string_lossy().to_string());
        if let Some(ref symbols) = file.symbols {
            for sym in symbols {
                items.push(sym.name.clone());
            }
        }
    }

    items.truncate(200);
    items.join(" ")
}

pub fn record_test_outcomes(
    conn: &Connection,
    embed_config: &LocalModelConfig,
    outcomes: &[TestOutcome],
    diff_text: &str,
) -> Result<usize, String> {
    if embed_config.base_url.is_empty() {
        warn!("Semantic prediction: base_url is empty; skipping outcome recording");
        return Ok(0);
    }

    if diff_text.is_empty() {
        warn!("Semantic prediction: diff_text is empty; skipping outcome recording");
        return Ok(0);
    }

    if outcomes.is_empty() {
        return Ok(0);
    }

    let commit_hash = &outcomes[0].commit_hash;

    embed_and_store(embed_config, conn, "test_diff", commit_hash, diff_text)?;

    let embedding_id: i64 = conn
        .query_row(
            "SELECT id FROM embeddings WHERE entity_type = 'test_diff' AND entity_id = ?1 AND model_name = ?2",
            rusqlite::params![commit_hash, embed_config.embedding_model],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    for outcome in outcomes {
        conn.execute(
            "INSERT INTO test_outcome_history (diff_embedding_id, test_file, outcome, commit_hash) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![embedding_id, outcome.test_file, outcome.status.as_str(), outcome.commit_hash],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(outcomes.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impact::packet::{ChangedFile, FileAnalysisStatus};
    use crate::index::symbols::Symbol;
    use crate::state::migrations::get_migrations;
    use httpmock::prelude::*;
    use std::path::PathBuf;
    fn setup_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();
        conn
    }

    fn make_packet() -> ImpactPacket {
        ImpactPacket {
            head_hash: Some("abc123def".to_string()),
            changes: vec![
                ChangedFile {
                    path: PathBuf::from("src/main.rs"),
                    status: "Modified".to_string(),
                    is_staged: true,
                    symbols: Some(vec![
                        Symbol {
                            name: "main".to_string(),
                            kind: crate::index::symbols::SymbolKind::Function,
                            is_public: true,
                            cognitive_complexity: None,
                            cyclomatic_complexity: None,
                            line_start: None,
                            line_end: None,
                            qualified_name: None,
                            byte_start: None,
                            byte_end: None,
                            entrypoint_kind: None,
                        },
                        Symbol {
                            name: "helper".to_string(),
                            kind: crate::index::symbols::SymbolKind::Function,
                            is_public: false,
                            cognitive_complexity: None,
                            cyclomatic_complexity: None,
                            line_start: None,
                            line_end: None,
                            qualified_name: None,
                            byte_start: None,
                            byte_end: None,
                            entrypoint_kind: None,
                        },
                    ]),
                    imports: None,
                    runtime_usage: None,
                    analysis_status: FileAnalysisStatus::default(),
                    analysis_warnings: Vec::new(),
                    api_routes: Vec::new(),
                    data_models: Vec::new(),
                    ci_gates: Vec::new(),
                },
                ChangedFile {
                    path: PathBuf::from("src/lib.rs"),
                    status: "Modified".to_string(),
                    is_staged: true,
                    symbols: Some(vec![
                        Symbol {
                            name: "init".to_string(),
                            kind: crate::index::symbols::SymbolKind::Function,
                            is_public: true,
                            cognitive_complexity: None,
                            cyclomatic_complexity: None,
                            line_start: None,
                            line_end: None,
                            qualified_name: None,
                            byte_start: None,
                            byte_end: None,
                            entrypoint_kind: None,
                        },
                        Symbol {
                            name: "run".to_string(),
                            kind: crate::index::symbols::SymbolKind::Function,
                            is_public: false,
                            cognitive_complexity: None,
                            cyclomatic_complexity: None,
                            line_start: None,
                            line_end: None,
                            qualified_name: None,
                            byte_start: None,
                            byte_end: None,
                            entrypoint_kind: None,
                        },
                    ]),
                    imports: None,
                    runtime_usage: None,
                    analysis_status: FileAnalysisStatus::default(),
                    analysis_warnings: Vec::new(),
                    api_routes: Vec::new(),
                    data_models: Vec::new(),
                    ci_gates: Vec::new(),
                },
                ChangedFile {
                    path: PathBuf::from("src/utils.rs"),
                    status: "Added".to_string(),
                    is_staged: true,
                    symbols: Some(vec![Symbol {
                        name: "normalize".to_string(),
                        kind: crate::index::symbols::SymbolKind::Function,
                        is_public: true,
                        cognitive_complexity: None,
                        cyclomatic_complexity: None,
                        line_start: None,
                        line_end: None,
                        qualified_name: None,
                        byte_start: None,
                        byte_end: None,
                        entrypoint_kind: None,
                    }]),
                    imports: None,
                    runtime_usage: None,
                    analysis_status: FileAnalysisStatus::default(),
                    analysis_warnings: Vec::new(),
                    api_routes: Vec::new(),
                    data_models: Vec::new(),
                    ci_gates: Vec::new(),
                },
            ],
            ..ImpactPacket::default()
        }
    }

    #[test]
    fn test_status_as_str() {
        assert_eq!(TestStatus::Passed.as_str(), "pass");
        assert_eq!(TestStatus::Failed.as_str(), "fail");
        assert_eq!(TestStatus::Skipped.as_str(), "skip");
    }

    #[test]
    fn build_diff_text_with_changes_contains_paths_and_symbols() {
        let packet = make_packet();
        let text = build_diff_text(&packet);
        assert!(text.contains("src/main.rs"));
        assert!(text.contains("src/lib.rs"));
        assert!(text.contains("src/utils.rs"));
        assert!(text.contains("main"));
        assert!(text.contains("helper"));
        assert!(text.contains("init"));
        assert!(text.contains("run"));
        assert!(text.contains("normalize"));
    }

    #[test]
    fn build_diff_text_empty_packet_returns_empty_string() {
        let packet = ImpactPacket::default();
        let text = build_diff_text(&packet);
        assert_eq!(text, "");
    }

    #[test]
    fn record_test_outcomes_stores_outcomes_in_db() {
        let conn = setup_db();
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(httpmock::Method::POST).path("/v1/embeddings");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(serde_json::json!({
                    "data": [{"embedding": [0.1, 0.2, 0.3]}]
                }));
        });

        let config = LocalModelConfig {
            base_url: server.base_url(),
            embedding_model: "test-model".to_string(),
            dimensions: 3,
            context_window: 8192,
            timeout_secs: 30,
            ..LocalModelConfig::default()
        };

        let outcomes = vec![
            TestOutcome {
                test_name: "cargo test".to_string(),
                test_file: "tests/test_a.rs".to_string(),
                commit_hash: "abc123".to_string(),
                status: TestStatus::Passed,
                duration_ms: 150,
                diff_summary: "changed src/lib.rs".to_string(),
            },
            TestOutcome {
                test_name: "cargo test".to_string(),
                test_file: "tests/test_b.rs".to_string(),
                commit_hash: "abc123".to_string(),
                status: TestStatus::Failed,
                duration_ms: 200,
                diff_summary: "changed src/lib.rs".to_string(),
            },
            TestOutcome {
                test_name: "cargo clippy".to_string(),
                test_file: "lint".to_string(),
                commit_hash: "abc123".to_string(),
                status: TestStatus::Passed,
                duration_ms: 50,
                diff_summary: "changed src/lib.rs".to_string(),
            },
        ];

        let count = record_test_outcomes(&conn, &config, &outcomes, "changed src/lib.rs").unwrap();
        assert_eq!(count, 3);

        let row_count: i64 = conn
            .query_row("SELECT count(*) FROM test_outcome_history", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(row_count, 3);

        let pass_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM test_outcome_history WHERE outcome = 'pass'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(pass_count, 2);

        let fail_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM test_outcome_history WHERE outcome = 'fail'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(fail_count, 1);
    }

    #[test]
    fn record_test_outcomes_returns_zero_when_base_url_empty() {
        let conn = setup_db();
        let config = LocalModelConfig::default();
        let outcomes = vec![TestOutcome {
            test_name: "cargo test".to_string(),
            test_file: "tests/test.rs".to_string(),
            commit_hash: "abc".to_string(),
            status: TestStatus::Passed,
            duration_ms: 100,
            diff_summary: String::new(),
        }];

        let count = record_test_outcomes(&conn, &config, &outcomes, "some diff").unwrap();
        assert_eq!(count, 0);

        let row_count: i64 = conn
            .query_row("SELECT count(*) FROM test_outcome_history", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(row_count, 0);
    }

    #[test]
    fn record_test_outcomes_skips_when_diff_text_empty() {
        let conn = setup_db();
        let config = LocalModelConfig {
            base_url: "http://localhost:9999".to_string(),
            embedding_model: "test-model".to_string(),
            dimensions: 3,
            ..LocalModelConfig::default()
        };
        let outcomes = vec![TestOutcome {
            test_name: "cargo test".to_string(),
            test_file: "tests/test.rs".to_string(),
            commit_hash: "abc".to_string(),
            status: TestStatus::Passed,
            duration_ms: 100,
            diff_summary: String::new(),
        }];

        let count = record_test_outcomes(&conn, &config, &outcomes, "").unwrap();
        assert_eq!(count, 0);

        let row_count: i64 = conn
            .query_row("SELECT count(*) FROM test_outcome_history", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(row_count, 0);
    }
}
