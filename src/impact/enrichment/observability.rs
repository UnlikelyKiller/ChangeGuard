use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::{CoverageDelta, ImpactPacket};
use crate::index::languages::{extract_error_handling, extract_logging_patterns, extract_telemetry_patterns};
use miette::{IntoDiagnostic, Result};
use std::fs;
use tracing::info;

pub struct ObservabilityProvider;

impl EnrichmentProvider for ObservabilityProvider {
    fn name(&self) -> &'static str {
        "Observability Enrichment Provider"
    }

    fn enrich(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        if !context.storage.table_exists_and_has_data("observability_patterns")? {
            info!("Skipping observability enrichment: observability_patterns table is empty or missing.");
            return Ok(());
        }

        let conn = context.storage.get_connection();
        
        for change in &packet.changes {
            let Some(&file_id) = context.file_id_map.get(&change.path) else {
                continue;
            };

            let full_path = context.project_root.join(&change.path);
            let content = match fs::read_to_string(&full_path) {
                Ok(c) => Some(c),
                Err(_) => None,
            };

            // 1. Error Handling
            self.enrich_coverage(
                conn,
                file_id,
                change,
                content.as_deref(),
                "ERROR_HANDLE",
                &mut packet.error_handling_delta,
                |p, c| extract_error_handling(p, c).map(|v| v.iter().filter(|x| !x.in_test).count()),
            )?;

            // 2. Logging
            self.enrich_coverage(
                conn,
                file_id,
                change,
                content.as_deref(),
                "LOG",
                &mut packet.logging_coverage_delta,
                |p, c| extract_logging_patterns(p, c).map(|v| v.iter().filter(|x| !x.in_test).count()),
            )?;

            // 3. Telemetry
            self.enrich_coverage(
                conn,
                file_id,
                change,
                content.as_deref(),
                "TRACE",
                &mut packet.telemetry_coverage_delta,
                |p, c| extract_telemetry_patterns(p, c).map(|v| v.iter().filter(|x| !x.in_test).count()),
            )?;
        }

        // 4. Enrich specific observability signals (M7 logic)
        crate::observability::enrich_observability(packet, context.config, context.storage.get_connection())
            .map_err(|e| miette::miette!(e))?;

        Ok(())
    }
}

impl ObservabilityProvider {
    fn enrich_coverage<F>(
        &self,
        conn: &rusqlite::Connection,
        file_id: i64,
        change: &crate::impact::packet::ChangedFile,
        content: Option<&str>,
        kind: &str,
        deltas: &mut Vec<CoverageDelta>,
        extractor: F,
    ) -> Result<()>
    where
        F: Fn(&std::path::Path, &str) -> Result<usize>,
    {
        let previous_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM observability_patterns WHERE file_id = ?1 AND pattern_kind = ?2 AND in_test = 0",
                rusqlite::params![file_id, kind],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let current_count = if let Some(content) = content {
            match extractor(&change.path, content) {
                Ok(count) => count as i64,
                Err(_) => previous_count,
            }
        } else {
            previous_count
        };

        if current_count < previous_count {
            let delta = (previous_count - current_count) as usize;
            deltas.push(CoverageDelta {
                file_path: change.path.to_string_lossy().to_string(),
                pattern_kind: kind.to_string(),
                previous_count: previous_count as usize,
                current_count: current_count as usize,
                message: format!(
                    "{} reduced in {}: {} patterns removed",
                    kind,
                    change.path.display(),
                    delta
                ),
            });
        }
        Ok(())
    }
}
