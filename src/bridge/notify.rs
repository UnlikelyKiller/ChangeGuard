use crate::bridge::ipc::IpcClient;
use crate::bridge::model::{BridgeRecord, BridgeVerifyOutcome};
use std::thread;
use std::time::Duration;

pub fn push_verify_results(results: Vec<BridgeVerifyOutcome>) {
    let records: Vec<BridgeRecord> = results
        .into_iter()
        .map(BridgeRecord::VerifyOutcome)
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
