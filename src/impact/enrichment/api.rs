use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::{ApiRoute, ImpactPacket};
use miette::{IntoDiagnostic, Result};
use tracing::info;

pub struct ApiProvider;

impl EnrichmentProvider for ApiProvider {
    fn name(&self) -> &'static str {
        "API Enrichment Provider"
    }

    fn enrich(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        if !context.storage.table_exists_and_has_data("api_routes")? {
            info!("Skipping API enrichment: api_routes table is empty or missing.");
            return Ok(());
        }

        let conn = context.storage.get_connection();
        
        for changed_file in &mut packet.changes {
            let Some(&file_id) = context.file_id_map.get(&changed_file.path) else {
                continue;
            };

            let mut stmt = conn
                .prepare(
                    "SELECT method, path_pattern, handler_symbol_name, framework, route_source, mount_prefix, is_dynamic, route_confidence, evidence 
                     FROM api_routes WHERE handler_file_id = ?1"
                )
                .into_diagnostic()?;

            let routes = stmt
                .query_map([file_id], |row| {
                    Ok(ApiRoute {
                        method: row.get(0)?,
                        path_pattern: row.get(1)?,
                        handler_symbol_name: row.get(2)?,
                        framework: row.get(3)?,
                        route_source: row.get(4)?,
                        mount_prefix: row.get(5)?,
                        is_dynamic: row.get::<_, i32>(6)? != 0,
                        route_confidence: row.get(7)?,
                        evidence: row.get(8)?,
                    })
                })
                .into_diagnostic()?;

            for route in routes {
                changed_file.api_routes.push(route.into_diagnostic()?);
            }
        }

        Ok(())
    }
}
