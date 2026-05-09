use crate::config::model::Config;
use crate::impact::packet::ImpactPacket;
use crate::impact::providers::{RiskImpact, RiskProvider};
use crate::policy::rules::Rules;
use miette::Result;
use tracing::debug;

/// Provider that assesses risk based on changes to observability and error handling.
pub struct ObservabilityProvider;

impl RiskProvider for ObservabilityProvider {
    fn name(&self) -> &str {
        "Observability & Error Handling Provider"
    }

    fn analyze(
        &self,
        packet: &ImpactPacket,
        _rules: &Rules,
        config: &Config,
    ) -> Result<RiskImpact> {
        let mut weight = 0;
        let mut reasons = Vec::new();

        // 1. Logging Coverage Reduction
        let observability_weight_per_file = 25;
        let observability_weight_cap = 25;
        let mut observability_total = 0;
        for delta in &packet.logging_coverage_delta {
            if delta.current_count < delta.previous_count
                && observability_total + observability_weight_per_file <= observability_weight_cap
            {
                observability_total += observability_weight_per_file;
                let reduction = delta.previous_count - delta.current_count;
                reasons.push(format!(
                    "Logging coverage reduced in {}: {} statements removed",
                    delta.file_path, reduction
                ));
                debug!(
                    "Risk Factor: Logging coverage reduced ({}) +{}",
                    delta.file_path, observability_weight_per_file
                );
            }
        }
        weight += observability_total;

        // 2. Error Handling Reduction
        let error_handling_weight_per_file = 25;
        let error_handling_weight_cap = 25;
        let mut error_handling_total = 0;
        for delta in &packet.error_handling_delta {
            if delta.current_count < delta.previous_count
                && error_handling_total + error_handling_weight_per_file
                    <= error_handling_weight_cap
            {
                error_handling_total += error_handling_weight_per_file;
                let reduction = delta.previous_count - delta.current_count;
                reasons.push(format!(
                    "Error handling reduced in {}: {} patterns removed",
                    delta.file_path, reduction
                ));
                debug!(
                    "Risk Factor: Error handling reduced ({}) +{}",
                    delta.file_path, error_handling_weight_per_file
                );
            }
        }
        weight += error_handling_total;

        // 3. Telemetry Reduction Risk
        let telemetry_weight_per_file = 25;
        let telemetry_weight_cap = 25;
        let mut telemetry_total = 0;
        for delta in &packet.telemetry_coverage_delta {
            if delta.current_count < delta.previous_count
                && telemetry_total + telemetry_weight_per_file <= telemetry_weight_cap
            {
                telemetry_total += telemetry_weight_per_file;
                let reduction = delta.previous_count - delta.current_count;
                reasons.push(format!(
                    "Telemetry coverage reduced in {}: {} instrumentation points removed",
                    delta.file_path, reduction
                ));
                debug!(
                    "Risk Factor: Telemetry coverage reduced ({}) +{}",
                    delta.file_path, telemetry_weight_per_file
                );
            }
        }
        weight += telemetry_total;

        // 4. Trace Config Drift
        let trace_config_weight_per_file = config.coverage.traces.risk_weight_per_config_file;
        let trace_config_weight_cap = config.coverage.traces.risk_cap;
        let mut trace_config_total = 0;
        for change in &packet.trace_config_drift {
            if trace_config_total + trace_config_weight_per_file <= trace_config_weight_cap {
                trace_config_total += trace_config_weight_per_file;
                reasons.push(format!("Observability config drift: {:?}", change.file));
                debug!(
                    "Risk Factor: Trace config drift ({:?}) +{}",
                    change.file, trace_config_weight_per_file
                );
            }
        }
        weight += trace_config_total;

        // 5. Trace Env Var Changes
        let trace_env_weight_per_var = config.coverage.traces.risk_weight_per_env_var;
        let trace_env_weight_cap = config.coverage.traces.risk_cap;
        let mut trace_env_total = 0;
        for change in &packet.trace_env_vars {
            if trace_env_total + trace_env_weight_per_var <= trace_env_weight_cap {
                trace_env_total += trace_env_weight_per_var;
                reasons.push(format!("Observability env var change: {}", change.var_name));
                debug!(
                    "Risk Factor: Trace env var change ({}) +{}",
                    change.var_name, trace_env_weight_per_var
                );
            }
        }
        weight += trace_env_total;

        Ok(RiskImpact { weight, reasons })
    }
}
