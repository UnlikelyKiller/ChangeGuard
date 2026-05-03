use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::ImpactPacket;
use crate::index::env_schema::EnvVarDep;
use miette::{IntoDiagnostic, Result};
use std::collections::HashSet;
use tracing::info;

pub struct EnvironmentProvider;

impl EnrichmentProvider for EnvironmentProvider {
    fn name(&self) -> &'static str {
        "Environment Enrichment Provider"
    }

    fn enrich(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        if !context
            .storage
            .table_exists_and_has_data("env_references")?
        {
            info!("Skipping environment enrichment: env_references table is empty or missing.");
            return Ok(());
        }

        let conn = context.storage.get_connection();

        // 1. Collect all declared var names
        let mut decl_stmt = conn
            .prepare("SELECT var_name FROM env_declarations")
            .into_diagnostic()?;

        let declared_names: HashSet<String> = decl_stmt
            .query_map([], |row| row.get::<_, String>(0))
            .into_diagnostic()?
            .collect::<rusqlite::Result<HashSet<_>>>()
            .into_diagnostic()?;

        // 2. For each changed file, find env var references and check if declared
        for change in &packet.changes {
            let Some(&file_id) = context.file_id_map.get(&change.path) else {
                continue;
            };

            let mut ref_stmt = conn
                .prepare("SELECT var_name, reference_kind FROM env_references WHERE file_id = ?1")
                .into_diagnostic()?;

            let refs = ref_stmt
                .query_map([file_id], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .into_diagnostic()?;

            for res in refs {
                let (var_name, reference_kind) = res.into_diagnostic()?;

                if var_name == "*" {
                    continue;
                }

                if !declared_names.contains(&var_name) {
                    packet.env_var_deps.push(EnvVarDep {
                        var_name: var_name.clone(),
                        declared: false,
                        evidence: format!(
                            "Referenced as {} in {} but not declared in any env config",
                            reference_kind,
                            change.path.display()
                        ),
                    });
                }
            }
        }

        Ok(())
    }
}
