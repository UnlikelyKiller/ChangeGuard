use crate::impact::analysis::analyze_risk;
use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::ImpactPacket;
use crate::policy::load::load_rules;
use crate::state::layout::Layout;
use miette::Result;
use tracing::{info, warn};

pub struct RiskProvider;

impl EnrichmentProvider for RiskProvider {
    fn name(&self) -> &'static str {
        "Risk Analysis Enrichment Provider"
    }

    fn enrich(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        info!("Performing risk analysis...");

        let layout = Layout::new(context.project_root.to_string_lossy().as_ref());
        let rules = match load_rules(&layout) {
            Ok(r) => r,
            Err(e) => {
                warn!("Failed to load rules: {e}");
                context.add_warning(format!("Risk analysis skipped: could not load rules: {e}"));
                return Ok(());
            }
        };

        analyze_risk(packet, &rules, context.config)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impact::packet::{ChangedFile, FileAnalysisStatus};
    use crate::state::migrations::get_migrations;
    use crate::state::storage::StorageManager;
    use rusqlite::Connection;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    #[test]
    fn enrich_analyzes_risk() {
        let mut conn = Connection::open_in_memory().unwrap();
        get_migrations().to_latest(&mut conn).unwrap();
        let storage = StorageManager::init_from_conn(conn);
        let config = crate::config::model::Config::default();
        let context = EnrichmentContext {
            storage: &storage,
            config: &config,
            file_id_map: HashMap::new(),
            project_root: PathBuf::from(r"C:\dev\changeguard"),
            warnings: Arc::new(Mutex::new(Vec::new())),
        };
        let mut packet = ImpactPacket {
            changes: vec![
                ChangedFile {
                    path: PathBuf::from("src/a.rs"),
                    status: "Modified".to_string(),
                    old_path: None,
                    is_staged: false,
                    symbols: None,
                    imports: None,
                    runtime_usage: None,
                    analysis_status: FileAnalysisStatus::default(),
                    analysis_warnings: Vec::new(),
                    api_routes: Vec::new(),
                    data_models: Vec::new(),
                    ci_gates: Vec::new(),
                },
                ChangedFile {
                    path: PathBuf::from("src/b.rs"),
                    status: "Modified".to_string(),
                    old_path: None,
                    is_staged: false,
                    symbols: None,
                    imports: None,
                    runtime_usage: None,
                    analysis_status: FileAnalysisStatus::default(),
                    analysis_warnings: Vec::new(),
                    api_routes: Vec::new(),
                    data_models: Vec::new(),
                    ci_gates: Vec::new(),
                },
                ChangedFile {
                    path: PathBuf::from("src/c.rs"),
                    status: "Modified".to_string(),
                    old_path: None,
                    is_staged: false,
                    symbols: None,
                    imports: None,
                    runtime_usage: None,
                    analysis_status: FileAnalysisStatus::default(),
                    analysis_warnings: Vec::new(),
                    api_routes: Vec::new(),
                    data_models: Vec::new(),
                    ci_gates: Vec::new(),
                },
                ChangedFile {
                    path: PathBuf::from("src/d.rs"),
                    status: "Modified".to_string(),
                    old_path: None,
                    is_staged: false,
                    symbols: None,
                    imports: None,
                    runtime_usage: None,
                    analysis_status: FileAnalysisStatus::default(),
                    analysis_warnings: Vec::new(),
                    api_routes: Vec::new(),
                    data_models: Vec::new(),
                    ci_gates: Vec::new(),
                },
                ChangedFile {
                    path: PathBuf::from("src/e.rs"),
                    status: "Modified".to_string(),
                    old_path: None,
                    is_staged: false,
                    symbols: None,
                    imports: None,
                    runtime_usage: None,
                    analysis_status: FileAnalysisStatus::default(),
                    analysis_warnings: Vec::new(),
                    api_routes: Vec::new(),
                    data_models: Vec::new(),
                    ci_gates: Vec::new(),
                },
                ChangedFile {
                    path: PathBuf::from("src/f.rs"),
                    status: "Modified".to_string(),
                    old_path: None,
                    is_staged: false,
                    symbols: None,
                    imports: None,
                    runtime_usage: None,
                    analysis_status: FileAnalysisStatus::default(),
                    analysis_warnings: Vec::new(),
                    api_routes: Vec::new(),
                    data_models: Vec::new(),
                    ci_gates: Vec::new(),
                },
            ],
            ..Default::default()
        };

        RiskProvider.enrich(&context, &mut packet).unwrap();

        assert!(!packet.risk_reasons.is_empty());
        let has_volume_reason = packet
            .risk_reasons
            .iter()
            .any(|r| r.contains("High volume"));
        assert!(has_volume_reason);
    }
}
