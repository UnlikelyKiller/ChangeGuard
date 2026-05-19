use crate::bridge::ipc::IpcClient;
use crate::bridge::model::{BridgeDirection, BridgePayload, BridgeRecord};
use crate::state::layout::Layout;
use miette::{IntoDiagnostic, Result};
use std::time::Duration;

mod client_cli;
pub use client_cli::query_external_cli;

pub fn query_unified(query: &str) -> Result<Vec<BridgeRecord>> {
    let current_dir = std::env::current_dir().into_diagnostic()?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let project_id = layout.get_project_id();

    // 1. Try IPC
    if let Ok(mut client) = IpcClient::connect_with_timeout(Duration::from_millis(200)) {
        let payload = BridgePayload::Query {
            text: query.to_string(),
        };
        let req = BridgeRecord::new(
            BridgeDirection::Outbound,
            project_id.clone(),
            "query",
            payload,
        );
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
            match record.payload {
                BridgePayload::Insight {
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
