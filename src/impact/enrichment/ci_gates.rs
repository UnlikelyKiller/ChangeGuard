use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::{CIGate, ImpactPacket};
use miette::{IntoDiagnostic, Result};
use tracing::info;

pub struct CIGateProvider;

impl EnrichmentProvider for CIGateProvider {
    fn name(&self) -> &'static str {
        "CI Gate Enrichment Provider"
    }

    fn enrich(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        if !context.storage.table_exists_and_has_data("ci_gates")? {
            info!("Skipping CI gate enrichment: ci_gates table is empty or missing.");
            return Ok(());
        }

        let conn = context.storage.get_connection();

        for changed_file in &mut packet.changes {
            let Some(&file_id) = context.file_id_map.get(&changed_file.path) else {
                continue;
            };

            let mut stmt = conn
                .prepare(
                    "SELECT platform, workflow_path, job_id, step_name, event_type, is_blocking 
                     FROM ci_gates WHERE ci_file_id = ?1",
                )
                .into_diagnostic()?;

            let gates = stmt
                .query_map([file_id], |row| {
                    Ok(CIGate {
                        platform: row.get(0)?,
                        job_name: row.get(1)?,
                        trigger: row.get(2)?,
                    })
                })
                .into_diagnostic()?;

            for gate in gates {
                changed_file.ci_gates.push(gate.into_diagnostic()?);
            }
        }

        Ok(())
    }
}
