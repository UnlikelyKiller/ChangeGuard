use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::{DataModel, ImpactPacket};
use miette::{IntoDiagnostic, Result};
use tracing::info;

pub struct DataModelProvider;

impl EnrichmentProvider for DataModelProvider {
    fn name(&self) -> &'static str {
        "Data Model Enrichment Provider"
    }

    fn enrich(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        if !context.storage.table_exists_and_has_data("data_models")? {
            info!("Skipping data model enrichment: data_models table is empty or missing.");
            return Ok(());
        }

        let conn = context.storage.get_connection();

        for changed_file in &mut packet.changes {
            let Some(&file_id) = context.file_id_map.get(&changed_file.path) else {
                continue;
            };

            let mut stmt = conn
                .prepare(
                    "SELECT model_name, model_kind, confidence, evidence 
                     FROM data_models WHERE model_file_id = ?1",
                )
                .into_diagnostic()?;

            let models = stmt
                .query_map([file_id], |row| {
                    Ok(DataModel {
                        model_name: row.get(0)?,
                        model_kind: row.get(1)?,
                        confidence: row.get(2)?,
                        evidence: row.get(3)?,
                    })
                })
                .into_diagnostic()?;

            for model in models {
                changed_file.data_models.push(model.into_diagnostic()?);
            }
        }

        Ok(())
    }
}
