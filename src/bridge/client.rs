use crate::bridge::ipc::IpcClient;
use crate::bridge::model::BridgeRecord;
use miette::Result;
use std::time::Duration;

mod client_cli;
pub use client_cli::query_external_cli;

pub fn query_unified(query: &str) -> Result<Vec<BridgeRecord>> {
    // 1. Try IPC
    if let Ok(_client) = IpcClient::connect_with_timeout(Duration::from_millis(200)) {
        // For now, IPC query might be just sending the query as a specific record type
        // if supported, or just falling back.
        // The spec says IPC is preferred.
        // Let's assume we can query over IPC later.
    }

    // 2. Fallback to CLI
    query_external_cli(query)
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
