use crate::bridge::model::{
    BridgeDirection, BridgePayload, BridgeRecord, Privacy, calculate_hash, serialize_record,
};
use crate::impact::packet::ImpactPacket;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufWriter, Write};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct BridgeState {
    last_inbound_hash: Option<String>,
    last_outbound_hash: Option<String>,
    privacy: Option<Privacy>,
}

pub fn execute_export(out_path: String, hotspots: bool, targets: Option<Vec<String>>, ledger: bool) -> Result<()> {
    let current_dir = std::env::current_dir().into_diagnostic()?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let project_id = layout.get_project_id();

    let storage_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(storage_path.as_std_path())?;
    let conn = storage.get_connection();
    let ledger_db = crate::ledger::db::LedgerDb::new(conn);

    let mut state = load_bridge_state(&layout)?;

    let mut records = Vec::new();

    // If neither flag provided, export both
    let export_all = !hotspots && !ledger;

    // 1. Export Hotspots from latest-impact.json
    if hotspots || export_all {
        let impact_path = layout.reports_dir().join("latest-impact.json");
        if impact_path.exists() {
            let file = File::open(impact_path).into_diagnostic()?;
            let packet: ImpactPacket = serde_json::from_reader(file).into_diagnostic()?;
            for hotspot in packet.hotspots {
                if let Some(ref t) = targets {
                    let path_str = hotspot.path.to_string_lossy();
                    if !t.iter().any(|target| path_str.starts_with(target)) {
                        continue;
                    }
                }
                let mut max_coupling = 0.0f64;
                for tc in &packet.temporal_couplings {
                    if tc.file_a == hotspot.path || tc.file_b == hotspot.path {
                        if (tc.score as f64) > max_coupling {
                            max_coupling = tc.score as f64;
                        }
                    }
                }
                
                let max_risk = packet.ci_predictions.iter()
                    .map(|p| p.failure_probability as f64)
                    .fold(0.0f64, f64::max);

                let payload = BridgePayload::Hotspot {
                    path: hotspot.path.to_string_lossy().into_owned(),
                    score: hotspot.score as f64,
                    reason: format!(
                        "Complexity: {:.2}, Frequency: {}",
                        hotspot.complexity, hotspot.frequency
                    ),
                    temporal_coupling: max_coupling,
                    failure_risk_probability: max_risk,
                };
                let record = BridgeRecord::new(
                    BridgeDirection::Outbound,
                    project_id.clone(),
                    "hotspot_delta",
                    payload,
                );
                if record.privacy == Privacy::Private || record.privacy == Privacy::Sealed {
                    continue;
                }
                let mut record = record;
                record.parent_hash = state.last_outbound_hash.clone();
                state.last_outbound_hash = Some(calculate_hash(&record));
                records.push(record);
            }
        }
    }

    // 2. Export Ledger Deltas (Recent commits)
    if ledger || export_all {
        let entries = ledger_db
            .get_all_committed_ledger_entries()
            .into_diagnostic()?;

        let mut tx_groups: std::collections::HashMap<
            String,
            (String, std::collections::HashSet<String>),
        > = std::collections::HashMap::new();
        for entry in entries {
            let group = tx_groups
                .entry(entry.tx_id.clone())
                .or_insert((entry.summary.clone(), std::collections::HashSet::new()));
            group.1.insert(entry.entity_normalized.clone());
        }

        for (tx_id, (summary, files)) in tx_groups {
            let payload = BridgePayload::LedgerDelta {
                tx_id: tx_id.clone(),
                intent: summary,
                files_changed: files.len(),
            };
            let mut record = BridgeRecord::new(
                BridgeDirection::Outbound,
                project_id.clone(),
                "ledger_delta",
                payload,
            );
            record.tx_id = Some(tx_id);
            if record.privacy == Privacy::Private || record.privacy == Privacy::Sealed {
                continue;
            }
            record.parent_hash = state.last_outbound_hash.clone();
            state.last_outbound_hash = Some(calculate_hash(&record));
            records.push(record);
        }
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

    save_bridge_state(&layout, &state)?;

    println!("Exported records to bridge NDJSON.");
    Ok(())
}

fn load_bridge_state(layout: &Layout) -> Result<BridgeState> {
    let path = layout.bridge_state_file();
    if path.exists() {
        let content = fs::read_to_string(path).into_diagnostic()?;
        serde_json::from_str(&content).into_diagnostic()
    } else {
        Ok(BridgeState::default())
    }
}

fn save_bridge_state(layout: &Layout, state: &BridgeState) -> Result<()> {
    let path = layout.bridge_state_file();
    let json = serde_json::to_string_pretty(state).into_diagnostic()?;
    fs::write(path, json).into_diagnostic()
}
