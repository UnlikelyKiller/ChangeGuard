use crate::bridge::model::{BridgeRecord, deserialize_record};
use crate::impact::packet::{AiInsight, ImpactPacket};
use miette::{IntoDiagnostic, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn execute_import(input_path: String) -> Result<()> {
    let input_file = File::open(input_path).into_diagnostic()?;
    let reader = BufReader::new(input_file);

    let mut insights = Vec::new();
    for line in reader.lines() {
        let line = line.into_diagnostic()?;
        if line.trim().is_empty() {
            continue;
        }

        match deserialize_record(&line) {
            Ok(BridgeRecord::Insight {
                memory_id,
                relevance,
                content,
            }) => {
                insights.push(AiInsight {
                    memory_id,
                    relevance,
                    content,
                });
            }
            Ok(_) => {
                // Ignore other record types during import for now
            }
            Err(e) => {
                tracing::warn!("Failed to parse bridge record: {}", e);
            }
        }
    }

    if insights.is_empty() {
        println!("No insights found in bridge NDJSON.");
        return Ok(());
    }

    // Enrich latest-impact.json
    let impact_path = Path::new(".changeguard/reports/latest-impact.json");
    let mut packet = if impact_path.exists() {
        let file = File::open(impact_path).into_diagnostic()?;
        serde_json::from_reader(file).into_diagnostic()?
    } else {
        println!("latest-impact.json not found. Initializing new report with insights.");
        ImpactPacket::default()
    };

    packet.ai_insights.extend(insights);

    // Ensure directory exists
    if let Some(parent) = impact_path.parent() {
        std::fs::create_dir_all(parent).into_diagnostic()?;
    }

    let out_file = File::create(impact_path).into_diagnostic()?;
    serde_json::to_writer_pretty(out_file, &packet).into_diagnostic()?;

    println!("Enriched latest-impact.json with insights.");

    Ok(())
}
