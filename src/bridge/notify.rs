use crate::bridge::ipc::IpcClient;
use crate::bridge::model::BridgeRecord;
use crate::verify::results::VerificationResult;
use std::thread;
use std::time::Duration;

pub fn push_verify_results(results: &[VerificationResult]) {
    let records: Vec<BridgeRecord> = results
        .iter()
        .map(|res| BridgeRecord::VerifyOutcome {
            success: res.exit_code == 0,
            command: res.command.clone(),
            error_snippet: if res.exit_code != 0 {
                let err = if !res.stderr_summary.is_empty() {
                    &res.stderr_summary
                } else {
                    &res.stdout_summary
                };
                Some(err.chars().take(200).collect::<String>())
            } else {
                None
            },
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
