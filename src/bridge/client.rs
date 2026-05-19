use crate::bridge::ipc::IpcClient;
use crate::bridge::model::BridgeRecord;
use miette::Result;
use std::time::Duration;

mod client_cli;
pub use client_cli::query_external_cli;

pub fn query_unified(query: &str) -> Result<Vec<BridgeRecord>> {
    // 1. Try IPC
    if let Ok(mut client) = IpcClient::connect_with_timeout(Duration::from_millis(200)) {
        let req = BridgeRecord::Query {
            text: query.to_string(),
        };
        if client.send_record(&req).is_ok()
            && let Ok(records) = client.receive_records()
            && !records.is_empty()
        {
            return Ok(records);
        }
    }

    // 2. Fallback to CLI
    query_external_cli(query)
}
pub fn execute_query(query: String) -> Result<()> {
    let records = query_unified(&query)?;
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
