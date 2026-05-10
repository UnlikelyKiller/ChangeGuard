use crate::config::model::Config;
use crate::impact::packet::{ImpactPacket, RiskImpact};
use crate::policy::rules::Rules;
use miette::Result;

pub mod adr_staleness_provider;
pub mod advisory_provider;
pub mod api_surface_provider;
pub mod centrality_provider;
pub mod ci_provider;
pub mod coupling_provider;
pub mod data_contract_provider;
pub mod dead_code_provider;
pub mod environment_provider;
pub mod infra_provider;
pub mod observability_provider;
pub mod path_provider;
pub mod sdk_provider;
pub mod volume_provider;

/// Trait for modular risk analysis components.
pub trait RiskProvider: Send + Sync {
    /// The unique name of this provider (for debugging and tracing).
    fn name(&self) -> &str;

    /// Analyzes the impact packet and returns the calculated risk impact.
    fn analyze(&self, packet: &ImpactPacket, rules: &Rules, config: &Config) -> Result<RiskImpact>;
}

/// Registry that orchestrates multiple risk providers.
pub struct RiskRegistry {
    pub providers: Vec<Box<dyn RiskProvider>>,
}

impl Default for RiskRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(path_provider::PathProvider));
        registry.register(Box::new(volume_provider::VolumeProvider));
        registry.register(Box::new(api_surface_provider::ApiSurfaceProvider));
        registry.register(Box::new(coupling_provider::CouplingProvider));
        registry.register(Box::new(data_contract_provider::DataContractProvider));
        registry.register(Box::new(centrality_provider::CentralityProvider));
        registry.register(Box::new(environment_provider::EnvironmentProvider));
        registry.register(Box::new(observability_provider::ObservabilityProvider));
        registry.register(Box::new(infra_provider::InfraProvider));
        registry.register(Box::new(ci_provider::CiProvider));
        registry.register(Box::new(sdk_provider::SdkProvider));
        registry.register(Box::new(adr_staleness_provider::ADRStalenessProvider));
        registry.register(Box::new(advisory_provider::AdvisoryProvider));
        registry.register(Box::new(dead_code_provider::DeadCodeProvider));
        registry
    }
}

impl RiskRegistry {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Register a new risk provider.
    pub fn register(&mut self, provider: Box<dyn RiskProvider>) {
        self.providers.push(provider);
    }

    /// Run all registered providers and apply their impacts to the packet.
    pub fn run(&self, packet: &mut ImpactPacket, rules: &Rules, config: &Config) -> Result<()> {
        let mut total_weight = 0;

        // Capture state before providers run to handle "provisional baseline" logic correctly.
        let has_prior_risk_signal = packet.risk_level == crate::impact::packet::RiskLevel::High
            || !packet.risk_reasons.is_empty();

        for provider in &self.providers {
            let impact = provider.analyze(packet, rules, config)?;
            packet.apply_risk_impact(impact, &mut total_weight);
        }

        packet.finalize_risk_level(total_weight, has_prior_risk_signal);

        Ok(())
    }
}
