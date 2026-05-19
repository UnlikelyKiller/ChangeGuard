use crate::bridge::model::{BridgeRecord, deserialize_record};
use miette::Result;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub fn query_external_cli(query: &str) -> Result<Vec<BridgeRecord>> {
    let (tx, rx) = mpsc::channel();
    let query_owned = query.to_string();

    thread::spawn(move || {
        let mut cmd = Command::new("ai-brains");
        cmd.args(["recall", &query_owned, "--format", "ndjson"]);
        let res = cmd.output();
        let _ = tx.send(res);
    });

    let output = match rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => {
            tracing::warn!(
                "Failed to invoke ai-brains CLI: {}. AI-Brains integration is degraded.",
                e
            );
            return Ok(Vec::new());
        }
        Err(_) => {
            tracing::warn!("ai-brains CLI query timed out. AI-Brains integration is degraded.");
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
