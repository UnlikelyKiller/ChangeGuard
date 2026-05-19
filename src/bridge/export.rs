use crate::bridge::model::{BridgeRecord, serialize_record};
use crate::impact::packet::ImpactPacket;
use crate::state::storage::StorageManager;
use miette::{IntoDiagnostic, Result};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

pub fn execute_export(out_path: String) -> Result<()> {
    let storage_path = PathBuf::from(".changeguard/state/ledger.db");
    let storage = StorageManager::init(&storage_path)?;
    let conn = storage.get_connection();
    let ledger_db = crate::ledger::db::LedgerDb::new(conn);

    let mut records = Vec::new();

    // 1. Export Hotspots from latest-impact.json
    let impact_path = Path::new(".changeguard/reports/latest-impact.json");
    if impact_path.exists() {
        let file = File::open(impact_path).into_diagnostic()?;
        let packet: ImpactPacket = serde_json::from_reader(file).into_diagnostic()?;
        for hotspot in packet.hotspots {
            records.push(BridgeRecord::Hotspot {
                path: hotspot.path.to_string_lossy().into_owned(),
                score: hotspot.score as f64,
                reason: format!(
                    "Complexity: {:.2}, Frequency: {}",
                    hotspot.complexity, hotspot.frequency
                ),
            });
        }
    }

    // 2. Export Ledger Deltas (Recent commits)
    let entries = ledger_db
        .get_all_committed_ledger_entries()
        .into_diagnostic()?;
    for entry in entries {
        records.push(BridgeRecord::LedgerDelta {
            tx_id: entry.tx_id.clone(),
            intent: entry.summary.clone(),
            files_changed: 1, // LedgerEntry is per-entity
        });
    }

    // 3. Write NDJSON
    let out_file = File::create(out_path).into_diagnostic()?;
    let mut writer = BufWriter::new(out_file);
    for record in records {
        let line = serialize_record(&record).into_diagnostic()?;
        writer.write_all(line.as_bytes()).into_diagnostic()?;
        writer.write_all(b"\n").into_diagnostic()?;
    }
    writer.flush().into_diagnostic()?;

    println!("Exported records to bridge NDJSON.");
    Ok(())
}
