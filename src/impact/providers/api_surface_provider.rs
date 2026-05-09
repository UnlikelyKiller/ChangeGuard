use crate::config::model::Config;
use crate::impact::packet::{ImpactPacket, RiskImpact};
use crate::policy::rules::Rules;
use miette::Result;
use tracing::debug;
pub struct ApiSurfaceProvider;

impl crate::impact::providers::RiskProvider for ApiSurfaceProvider {
    fn name(&self) -> &str {
        "API Surface Provider"
    }

    fn analyze(&self, packet: &ImpactPacket, _rules: &Rules, _config: &Config) -> Result<RiskImpact> {
        let mut total_weight = 0;
        let mut reasons = Vec::new();

        // 3. Symbol Visibility & Entrypoint Risk
        for file in &packet.changes {
            if let Some(symbols) = &file.symbols {
                for symbol in symbols {
                    if symbol.is_public {
                        let weight = 30;
                        total_weight += weight;
                        reasons.push(format!(
                            "Public symbol modified: {} ({})",
                            symbol.name,
                            file.path.display()
                        ));
                        debug!(
                            "Risk Factor: Public symbol modified ({}) +{}",
                            symbol.name, weight
                        );
                    }

                    // Entrypoint-based risk (API Surface category, max 35 points)
                    if let Some(ref kind) = symbol.entrypoint_kind {
                        match kind.as_str() {
                            "ENTRYPOINT" => {
                                let weight = 35;
                                total_weight += weight;
                                reasons.push(format!(
                                    "Entry point changed: {} ({})",
                                    symbol.name,
                                    file.path.display()
                                ));
                                debug!(
                                    "Risk Factor: Entry point changed ({}) +{}",
                                    symbol.name, weight
                                );
                            }
                            "HANDLER" => {
                                let weight = 30;
                                total_weight += weight;
                                reasons.push(format!(
                                    "Handler changed: {} ({})",
                                    symbol.name,
                                    file.path.display()
                                ));
                                debug!("Risk Factor: Handler changed ({}) +{}", symbol.name, weight);
                            }
                            "PUBLIC_API" => {
                                let weight = 20;
                                total_weight += weight;
                                reasons.push(format!(
                                    "Public API changed: {} ({})",
                                    symbol.name,
                                    file.path.display()
                                ));
                                debug!(
                                    "Risk Factor: Public API changed ({}) +{}",
                                    symbol.name, weight
                                );
                            }
                            // TEST — no additional weight for test entry points
                            _ => {}
                        }
                    }
                }
            }
        }

        // 3c. Route Handler Risk
        // Add 30 weight per file that has route handlers (max 30 total, not per-route).
        // This stacks with but doesn't duplicate the entrypoint HANDLER weight.
        let route_weight = 30;
        let route_weight_cap = 30;
        let mut route_total = 0;
        for file in &packet.changes {
            if !file.api_routes.is_empty() && route_total + route_weight <= route_weight_cap {
                route_total += route_weight;
                // Add a risk reason for the first route (summarize all routes for this file)
                let first_route = &file.api_routes[0];
                reasons.push(format!(
                    "Public API route: {} {}",
                    first_route.method, first_route.path_pattern
                ));
                debug!(
                    "Risk Factor: Route handler in {} ({} routes) +{}",
                    file.path.display(),
                    file.api_routes.len(),
                    route_weight
                );
            }
        }
        total_weight += route_total;

        Ok(RiskImpact {
            weight: total_weight,
            reasons,
        })
    }
}
