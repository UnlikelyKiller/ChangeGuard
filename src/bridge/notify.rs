use crate::bridge::ipc::IpcClient;
use crate::bridge::model::{BridgeDirection, BridgePayload, BridgeRecord, BridgeVerifyOutcome};
use crate::state::layout::Layout;
use std::thread;
use std::time::Duration;

pub fn push_verify_results(results: Vec<BridgeVerifyOutcome>) {
    let current_dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(_) => return,
    };
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let project_id = layout.get_project_id();

    let records: Vec<BridgeRecord> = results
        .into_iter()
        .map(|outcome| {
            BridgeRecord::new(
                BridgeDirection::Outbound,
                project_id.clone(),
                "verify_outcome",
                BridgePayload::VerifyOutcome(outcome),
            )
        })
        .collect();

    // Fire and forget in a separate thread to avoid delaying CLI exit
    thread::spawn(move || {
        if let Ok(mut client) = IpcClient::connect_with_timeout(Duration::from_millis(100)) {
            for record in records {
                let _ = client.send_record(&record);
            }
        }
    });
}
