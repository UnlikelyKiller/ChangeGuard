use crate::config::model::Config;
use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::ImpactPacket;
use crate::state::storage::StorageManager;
use miette::Result;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

pub struct ImpactOrchestrator {
    providers: Vec<Box<dyn EnrichmentProvider>>,
}

impl ImpactOrchestrator {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn with_builtins() -> Self {
        let mut orch = Self::new();
        orch.register_provider(Box::new(crate::impact::enrichment::federated::FederatedProvider));
        orch.register_provider(Box::new(crate::impact::enrichment::api::ApiProvider));
        orch.register_provider(Box::new(crate::impact::enrichment::data_models::DataModelProvider));
        orch.register_provider(Box::new(crate::impact::enrichment::contracts::ContractProvider));
        orch.register_provider(Box::new(crate::impact::enrichment::ci_gates::CIGateProvider));
        orch.register_provider(Box::new(crate::impact::enrichment::infrastructure::InfrastructureProvider));
        orch.register_provider(Box::new(crate::impact::enrichment::environment::EnvironmentProvider));
        orch.register_provider(Box::new(crate::impact::enrichment::observability::ObservabilityProvider));
        orch.register_provider(Box::new(crate::impact::enrichment::coupling::CouplingProvider));
        orch.register_provider(Box::new(crate::impact::enrichment::deploy::DeployProvider));
        orch.register_provider(Box::new(crate::impact::enrichment::hotspots::HotspotProvider));
        orch.register_provider(Box::new(crate::impact::enrichment::coverage::CoverageProvider));
        orch.register_provider(Box::new(crate::impact::enrichment::services::ServiceProvider));
        orch.register_provider(Box::new(crate::impact::enrichment::runtime_usage::RuntimeUsageProvider));
        orch.register_provider(Box::new(crate::impact::enrichment::knowledge::KnowledgeProvider));
        orch.register_provider(Box::new(crate::impact::enrichment::risk::RiskProvider));
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
