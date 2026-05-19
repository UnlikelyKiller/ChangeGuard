use crate::bridge::model::{BridgeRecord, deserialize_record};
use miette::Result;
use std::process::Command;

pub fn query_external_cli(query: &str) -> Result<Vec<BridgeRecord>> {
    let mut cmd = Command::new("ai-brains");
    cmd.args(["recall", query, "--format", "ndjson"]);

    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            tracing::warn!(
                "Failed to invoke ai-brains CLI: {}. AI-Brains integration is degraded.",
                e
            );
            return Ok(Vec::new());
        }
    };

    if !output.status.success() {
        tracing::warn!(
            "ai-brains CLI returned error: {}. AI-Brains integration is degraded.",
            String::from_utf8_lossy(&output.stderr)
        );
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut records = Vec::new();
    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match deserialize_record(line) {
            Ok(record) => records.push(record),
            Err(e) => {
                tracing::warn!("Failed to parse ai-brains record: {}", e);
            }
        }
    }

    Ok(records)
}

pub fn execute_query(query: String) -> Result<()> {
    let records = query_external_cli(&query)?;
    if records.is_empty() {
        println!("No memories recalled from AI-Brains.");
    } else {
        println!("Recalled {} memories from AI-Brains:", records.len());
        for record in records {
            match record {
                BridgeRecord::Insight {
                    content, relevance, ..
                } => {
                    println!("- [{:.2}] {}", relevance, content);
                }
                _ => {
                    // Other record types not handled here
                }
            }
        }
    }
    Ok(())
}
