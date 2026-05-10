use crate::config::model::Config;
use crate::git::{ChangeType, RepoSnapshot};
use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::{ChangedFile, FileAnalysisStatus, ImpactPacket};
use crate::index::analysis::{AnalysisOutcome, analyze_file};
use crate::state::storage::StorageManager;
use crate::util::clock::SystemClock;
use indicatif::{ProgressBar, ProgressStyle};
use miette::Result;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

pub struct ImpactOrchestrator {
    providers: Vec<Box<dyn EnrichmentProvider>>,
}

impl Default for ImpactOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl ImpactOrchestrator {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn with_builtins() -> Self {
        let mut orch = Self::new();
        orch.register_provider(Box::new(
            crate::impact::enrichment::federated::FederatedProvider,
        ));
        orch.register_provider(Box::new(crate::impact::enrichment::api::ApiProvider));
        orch.register_provider(Box::new(
            crate::impact::enrichment::data_models::DataModelProvider,
        ));
        orch.register_provider(Box::new(
            crate::impact::enrichment::contracts::ContractProvider,
        ));
        orch.register_provider(Box::new(
            crate::impact::enrichment::ci_gates::CIGateProvider,
        ));
        orch.register_provider(Box::new(
            crate::impact::enrichment::infrastructure::InfrastructureProvider,
        ));
        orch.register_provider(Box::new(
            crate::impact::enrichment::environment::EnvironmentProvider,
        ));
        orch.register_provider(Box::new(
            crate::impact::enrichment::observability::ObservabilityProvider,
        ));
        orch.register_provider(Box::new(
            crate::impact::enrichment::coupling::CouplingProvider,
        ));
        orch.register_provider(Box::new(crate::impact::enrichment::deploy::DeployProvider));
        orch.register_provider(Box::new(
            crate::impact::enrichment::ci_self_awareness::CISelfAwarenessProvider,
        ));
        orch.register_provider(Box::new(
            crate::impact::enrichment::ci_predictor::CIPredictorProvider,
        ));
        orch.register_provider(Box::new(
            crate::impact::enrichment::hotspots::HotspotProvider,
        ));
        orch.register_provider(Box::new(
            crate::impact::enrichment::coverage::CoverageProvider,
        ));
        orch.register_provider(Box::new(
            crate::impact::enrichment::services::ServiceProvider,
        ));
        orch.register_provider(Box::new(
            crate::impact::enrichment::runtime_usage::RuntimeUsageProvider,
        ));
        orch.register_provider(Box::new(crate::impact::enrichment::risk::RiskProvider));
        orch.register_provider(Box::new(
            crate::impact::enrichment::dead_code::DeadCodeProvider,
        ));
        orch.register_provider(Box::new(crate::impact::enrichment::kg_provider::KGProvider));
        orch
    }

    pub fn register_provider(&mut self, provider: Box<dyn EnrichmentProvider>) {
        self.providers.push(provider);
    }

    pub fn run(
        &self,
        packet: &mut ImpactPacket,
        storage: &StorageManager,
        config: &Config,
        project_root: &Path,
    ) -> Result<()> {
        info!("Starting impact orchestration...");

        // 1. Prepare Context
        let file_id_map = storage.get_active_file_id_map()?;
        let warnings_collector = Arc::new(Mutex::new(Vec::new()));

        let context = EnrichmentContext {
            storage,
            config,
            file_id_map,
            project_root: project_root.to_path_buf(),
            warnings: Arc::clone(&warnings_collector),
        };

        // 2. Execute Providers (Resilient Execution)
        for provider in &self.providers {
            let name = provider.name();
            info!("Running enrichment provider: {}", name);

            if let Err(e) = provider.enrich(&context, packet) {
                warn!("Enrichment provider '{}' failed: {}", name, e);
                context.add_warning(format!("Provider '{}' failed: {}", name, e));
            }
        }

        // 3. Collect Warnings
        if let Ok(w) = warnings_collector.lock() {
            packet.analysis_warnings.extend(w.iter().cloned());
        }

        Ok(())
    }
}

pub(crate) fn map_snapshot_to_packet(
    snapshot: RepoSnapshot,
    base_dir: &Path,
) -> Result<ImpactPacket> {
    let mut packet = ImpactPacket {
        head_hash: snapshot.head_hash,
        branch_name: snapshot.branch_name,
        ..ImpactPacket::with_clock(&SystemClock)
    };

    let pb = ProgressBar::new(snapshot.changes.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar()),
    );
    pb.set_message("Extracting symbols...");

    packet.changes = snapshot
        .changes
        .into_iter()
        .map(|c| {
            pb.set_message(format!("Extracting symbols from {}", c.path.display()));
            let (status, old_path) = match c.change_type {
                ChangeType::Added => ("Added".to_string(), None),
                ChangeType::Modified => ("Modified".to_string(), None),
                ChangeType::Deleted => ("Deleted".to_string(), Some(c.path.clone())),
                ChangeType::Renamed { ref old_path } => {
                    ("Renamed".to_string(), Some(old_path.clone()))
                }
            };

            let outcome = if matches!(c.change_type, ChangeType::Added | ChangeType::Modified) {
                analyze_file(&c.path, base_dir)
            } else {
                AnalysisOutcome {
                    symbols: None,
                    imports: None,
                    runtime_usage: None,
                    analysis_status: FileAnalysisStatus::default(),
                    analysis_warnings: Vec::new(),
                }
            };

            pb.inc(1);
            ChangedFile {
                path: c.path,
                status,
                old_path,
                is_staged: c.is_staged,
                symbols: outcome.symbols,
                imports: outcome.imports,
                runtime_usage: outcome.runtime_usage,
                analysis_status: outcome.analysis_status,
                analysis_warnings: outcome.analysis_warnings,
                api_routes: Vec::new(),
                data_models: Vec::new(),
                ci_gates: Vec::new(),
            }
        })
        .collect();

    pb.finish_with_message("Symbol extraction complete.");
    Ok(packet)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::FileChange;

    fn make_deleted(path: &str) -> FileChange {
        FileChange {
            path: std::path::PathBuf::from(path),
            change_type: ChangeType::Deleted,
            is_staged: true,
        }
    }

    fn make_added(path: &str) -> FileChange {
        FileChange {
            path: std::path::PathBuf::from(path),
            change_type: ChangeType::Added,
            is_staged: true,
        }
    }

    #[test]
    fn test_deleted_file_old_path() {
        let snapshot = RepoSnapshot {
            head_hash: Some("abc123".to_string()),
            branch_name: Some("main".to_string()),
            is_clean: false,
            changes: vec![
                make_deleted("src/api/users/handler.rs"),
                make_added("src/api/users/new.rs"),
            ],
        };
        let temp = tempfile::tempdir().unwrap();
        let packet = map_snapshot_to_packet(snapshot, temp.path()).unwrap();
        let deleted = packet
            .changes
            .iter()
            .find(|c| c.status == "Deleted")
            .expect("Deleted file not found");
        assert_eq!(
            deleted
                .old_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            Some("src/api/users/handler.rs".to_string())
        );
        let added = packet
            .changes
            .iter()
            .find(|c| c.status == "Added")
            .expect("Added file not found");
        assert!(added.old_path.is_none());
    }
}
