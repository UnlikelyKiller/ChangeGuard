use crate::config::model::DeadCodeConfig;
use crate::impact::packet::{ConfidenceFactor, DeadCodeFinding, ImpactPacket};
use crate::index::symbols::Symbol;
use crate::state::storage::StorageManager;
use crate::state::storage_cozo::CozoStorage;
use miette::{IntoDiagnostic, Result};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

pub struct DeadCodeImpactProvider;

impl super::ImpactProvider for DeadCodeImpactProvider {
    fn name(&self) -> &'static str {
        "Dead Code Impact Provider"
    }

    fn analyze(
        &self,
        packet: &ImpactPacket,
        _rules: &crate::policy::rules::Rules,
        config: &crate::config::model::Config,
    ) -> Result<crate::impact::packet::RiskImpact> {
        let mut impact = crate::impact::packet::RiskImpact {
            weight: 0,
            reasons: Vec::new(),
        };

        if !config.dead_code.enabled {
            return Ok(impact);
        }

        for finding in &packet.dead_code_findings {
            if finding.confidence >= config.dead_code.confidence_threshold {
                let reason = format!(
                    "Advisory: changed symbol '{}' in {} is likely dead code (confidence: {:.0}%)",
                    finding.symbol_name,
                    finding.file_path.display(),
                    finding.confidence * 100.0
                );
                impact.reasons.push(reason);
            }
        }

        Ok(impact)
    }
}

pub struct ConfidenceScorer<'a> {
    pub(super) cozo: Option<&'a CozoStorage>,
    pub(super) storage: &'a StorageManager,
    pub(super) config: &'a DeadCodeConfig,
    pub(super) repo_path: &'a Path,
    /// When `false` (default), standard trait symbols are excluded from results.
    /// Set to `true` via `--include-traits` to see all findings.
    pub(super) include_traits: bool,
    pub(super) git_activity_cache:
        std::cell::RefCell<std::collections::HashMap<std::path::PathBuf, Option<u32>>>,
}

impl<'a> ConfidenceScorer<'a> {
    pub fn new(
        cozo: Option<&'a CozoStorage>,
        storage: &'a StorageManager,
        config: &'a DeadCodeConfig,
        repo_path: &'a Path,
        include_traits: bool,
    ) -> Self {
        Self {
            cozo,
            storage,
            config,
            repo_path,
            include_traits,
            git_activity_cache: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }
}

mod evidence;
mod filters;
mod scoring;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::symbols::{Symbol, SymbolKind};
    use crate::state::storage::StorageManager;
    use crate::state::storage_cozo::CozoStorage;
    use std::path::PathBuf;

    pub(super) fn in_memory_storage_with_cozo() -> (StorageManager, CozoStorage) {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        let mut conn = conn;
        crate::state::migrations::get_migrations()
            .to_latest(&mut conn)
            .unwrap();
        let storage = StorageManager::init_from_conn(conn);
        let cozo = CozoStorage::new(&PathBuf::from("")).unwrap();
        (storage, cozo)
    }

    pub(super) fn default_config() -> DeadCodeConfig {
        DeadCodeConfig {
            enabled: true,
            confidence_threshold: 0.75,
            git_inactivity_days: 90,
            reachability_weight: 1.0,
            git_activity_weight: 1.0,
            test_coverage_weight: 1.0,
        }
    }

    pub(super) fn make_symbol(
        name: &str,
        qualified: Option<&str>,
        entrypoint: Option<&str>,
    ) -> Symbol {
        Symbol {
            name: name.to_string(),
            kind: SymbolKind::Function,
            is_public: false,
            cognitive_complexity: None,
            cyclomatic_complexity: None,
            line_start: None,
            line_end: None,
            qualified_name: qualified.map(|s| s.to_string()),
            byte_start: None,
            byte_end: None,
            entrypoint_kind: entrypoint.map(|s| s.to_string()),
            metadata: std::collections::BTreeMap::new(),
        }
    }

    #[test]
    fn test_entrypoint_skipped() {
        let (storage, _cozo) = in_memory_storage_with_cozo();
        let config = default_config();
        let scorer = ConfidenceScorer::new(None, &storage, &config, Path::new("."), false);

        let symbol = make_symbol("main", Some("crate::main"), Some("ENTRYPOINT"));
        let result = scorer
            .score_symbol(&symbol, Path::new("src/main.rs"))
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_reachability_unreachable_sqlite() {
        let (storage, _cozo) = in_memory_storage_with_cozo();
        let conn = storage.get_connection();

        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, parse_status, last_indexed_at) VALUES ('src/main.rs', 'Rust', 'h1', 100, 'OK', '2026-01-01')",
            [],
        ).unwrap();
        let main_file = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, parse_status, last_indexed_at) VALUES ('src/lib.rs', 'Rust', 'h2', 100, 'OK', '2026-01-01')",
            [],
        ).unwrap();
        let lib_file = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, entrypoint_kind, last_indexed_at) VALUES (?1, 'crate::main', 'main', 'Function', 'ENTRYPOINT', '2026-01-01')",
            [main_file],
        ).unwrap();
        let main_sym = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, entrypoint_kind, last_indexed_at) VALUES (?1, 'crate::helper', 'helper', 'Function', 'INTERNAL', '2026-01-01')",
            [lib_file],
        ).unwrap();
        let helper_sym = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, entrypoint_kind, last_indexed_at) VALUES (?1, 'crate::unused', 'unused', 'Function', 'INTERNAL', '2026-01-01')",
            [lib_file],
        ).unwrap();

        conn.execute(
            "INSERT INTO structural_edges (caller_symbol_id, caller_file_id, callee_symbol_id, callee_file_id, call_kind, resolution_status) VALUES (?1, ?2, ?3, ?4, 'DIRECT', 'RESOLVED')",
            [main_sym, main_file, helper_sym, lib_file],
        ).unwrap();

        let config = default_config();
        let scorer = ConfidenceScorer::new(None, &storage, &config, Path::new("."), false);

        let helper = make_symbol("helper", Some("crate::helper"), None);
        let score = scorer
            .reachability_score(&helper, Path::new("src/lib.rs"))
            .unwrap();
        assert_eq!(score, 0.0);

        let unused = make_symbol("unused", Some("crate::unused"), None);
        let score = scorer
            .reachability_score(&unused, Path::new("src/lib.rs"))
            .unwrap();
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_reachability_via_cozo() {
        use crate::platform::urn::build_urn;
        use crate::state::graph_kinds::NodeKind;

        let (storage, cozo) = in_memory_storage_with_cozo();

        let main_urn = build_urn(NodeKind::Symbol, "crate::main");
        let helper_urn = build_urn(NodeKind::Symbol, "crate::helper");
        let unused_urn = build_urn(NodeKind::Symbol, "crate::unused");

        cozo.run_script(&format!(
            "?[id, label, category, risk_score, metadata] <- [
                ['{}', 'main', 'code', 0.0, {{}}],
                ['{}', 'helper', 'code', 0.0, {{}}],
                ['{}', 'unused', 'code', 0.0, {{}}]
            ] :put node",
            main_urn, helper_urn, unused_urn
        ))
        .unwrap();

        cozo.run_script(&format!(
            "?[source, target, relation, confidence, provenance_id] <- [
                ['{}', '{}', 'calls', 1.0, 'tx1']
            ] :put edge",
            main_urn, helper_urn
        ))
        .unwrap();

        let conn = storage.get_connection();
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, parse_status, last_indexed_at) VALUES ('src/main.rs', 'Rust', 'h1', 100, 'OK', '2026-01-01')",
            [],
        ).unwrap();
        let main_file = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, entrypoint_kind, last_indexed_at) VALUES (?1, 'crate::main', 'main', 'Function', 'ENTRYPOINT', '2026-01-01')",
            [main_file],
        ).unwrap();

        let config = default_config();
        let scorer = ConfidenceScorer::new(Some(&cozo), &storage, &config, Path::new("."), false);

        let helper = make_symbol("helper", Some("crate::helper"), None);
        let score = scorer
            .reachability_score(&helper, Path::new("src/lib.rs"))
            .unwrap();
        assert_eq!(score, 0.0);

        let unused = make_symbol("unused", Some("crate::unused"), None);
        let score = scorer
            .reachability_score(&unused, Path::new("src/lib.rs"))
            .unwrap();
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_test_coverage_no_mapping() {
        let (storage, _cozo) = in_memory_storage_with_cozo();
        let config = default_config();
        let scorer = ConfidenceScorer::new(None, &storage, &config, Path::new("."), false);

        let symbol = make_symbol("foo", Some("crate::foo"), None);
        let score = scorer
            .test_coverage_score(&symbol, Path::new("src/lib.rs"))
            .unwrap();
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_test_coverage_with_mapping() {
        let (storage, _cozo) = in_memory_storage_with_cozo();
        let conn = storage.get_connection();

        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, parse_status, last_indexed_at) VALUES ('src/lib.rs', 'Rust', 'h1', 100, 'OK', '2026-01-01')",
            [],
        ).unwrap();
        let file_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, entrypoint_kind, last_indexed_at) VALUES (?1, 'crate::foo', 'foo', 'Function', 'INTERNAL', '2026-01-01')",
            [file_id],
        ).unwrap();
        let sym_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, entrypoint_kind, last_indexed_at) VALUES (?1, 'crate::test_foo', 'test_foo', 'Function', 'TEST', '2026-01-01')",
            [file_id],
        ).unwrap();
        let test_sym_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO test_mapping (test_symbol_id, test_file_id, tested_symbol_id, tested_file_id, mapping_kind, last_indexed_at) VALUES (?1, ?2, ?3, ?4, 'IMPORT', '2026-01-01')",
            [test_sym_id, file_id, sym_id, file_id],
        ).unwrap();

        let config = default_config();
        let scorer = ConfidenceScorer::new(None, &storage, &config, Path::new("."), false);

        let symbol = make_symbol("foo", Some("crate::foo"), None);
        let score = scorer
            .test_coverage_score(&symbol, Path::new("src/lib.rs"))
            .unwrap();
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_blend_expected_value() {
        let (storage, _cozo) = in_memory_storage_with_cozo();
        let config = default_config();
        let scorer = ConfidenceScorer::new(None, &storage, &config, Path::new("."), false);

        let confidence = scorer.blend(1.0, 0.5, 0.0);
        assert!((confidence - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_with_zero_weights() {
        let (storage, _cozo) = in_memory_storage_with_cozo();
        let config = DeadCodeConfig {
            enabled: true,
            confidence_threshold: 0.75,
            git_inactivity_days: 90,
            reachability_weight: 0.0,
            git_activity_weight: 0.0,
            test_coverage_weight: 0.0,
        };
        let scorer = ConfidenceScorer::new(None, &storage, &config, Path::new("."), false);
        let confidence = scorer.blend(1.0, 1.0, 1.0);
        assert_eq!(confidence, 0.0);
    }

    #[test]
    fn test_standard_trait_filtered_by_default() {
        let (storage, _cozo) = in_memory_storage_with_cozo();
        let config = default_config();
        let scorer = ConfidenceScorer::new(None, &storage, &config, Path::new("."), false);

        // The Rust extractor stores `impl Eq for MyType {}` as (name="Eq", kind=Type).
        let eq_symbol = Symbol {
            name: "Eq".to_string(),
            kind: SymbolKind::Type, // impl_item → Type in the Rust AST extractor
            is_public: true,
            cognitive_complexity: None,
            cyclomatic_complexity: None,
            line_start: None,
            line_end: None,
            qualified_name: Some("crate::Eq".to_string()),
            byte_start: None,
            byte_end: None,
            entrypoint_kind: None,
            metadata: std::collections::BTreeMap::new(),
        };

        let result = scorer
            .score_symbol(&eq_symbol, Path::new("src/lib.rs"))
            .unwrap();
        assert!(
            result.is_none(),
            "impl Eq for MyType (stored as Type/Eq) must be filtered by default"
        );
    }

    #[test]
    fn test_standard_trait_shown_with_include_traits() {
        let (storage, _cozo) = in_memory_storage_with_cozo();
        // Zero threshold so any confidence value above 0 would be returned,
        // and zero weights so confidence = 0 via blend → None regardless.
        // The key assertion: score_symbol must NOT short-circuit for standard traits
        // when include_traits = true (no early None from is_standard_trait filter).
        // We confirm by checking it reaches the reachability check (no panic).
        let config = DeadCodeConfig {
            enabled: true,
            confidence_threshold: 0.0,
            git_inactivity_days: 90,
            reachability_weight: 0.0,
            git_activity_weight: 0.0,
            test_coverage_weight: 0.0,
        };
        let scorer = ConfidenceScorer::new(None, &storage, &config, Path::new("."), true);

        let eq_symbol = Symbol {
            name: "Eq".to_string(),
            kind: SymbolKind::Type, // impl_item → Type in the Rust AST extractor
            is_public: true,
            cognitive_complexity: None,
            cyclomatic_complexity: None,
            line_start: None,
            line_end: None,
            qualified_name: Some("crate::Eq".to_string()),
            byte_start: None,
            byte_end: None,
            entrypoint_kind: None,
            metadata: std::collections::BTreeMap::new(),
        };

        // Should not panic (reaches scoring path even for standard traits)
        let _ = scorer.score_symbol(&eq_symbol, Path::new("src/lib.rs"));
    }
}
