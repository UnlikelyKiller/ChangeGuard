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

#[derive(Debug, Clone, Default)]
pub struct ExportArgs {
    pub out_path: Option<String>,
    pub hotspots: bool,
    pub ledger: bool,
    pub scope: Option<Vec<String>>,
    pub madr: bool,
    pub json: bool,
}

pub fn execute_export(args: ExportArgs) -> Result<()> {
    execute_export_in_dir(args, None)
}

/// Internal implementation that accepts an optional base directory override.
/// When `base_dir` is `None`, uses `std::env::current_dir()`.
fn execute_export_in_dir(args: ExportArgs, base_dir: Option<&str>) -> Result<()> {
    let root = match base_dir {
        Some(dir) => dir.to_string(),
        None => std::env::current_dir()
            .into_diagnostic()?
            .to_string_lossy()
            .to_string(),
    };
    let layout = Layout::new(&root);
    let project_id = layout.get_project_id();

    let storage_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(storage_path.as_std_path())?;
    let conn = storage.get_connection();
    let ledger_db = crate::ledger::db::LedgerDb::new(conn);

    let mut state = load_bridge_state(&layout)?;

    let mut records = Vec::new();

    // If neither flag provided, export both hotspots and ledger deltas (not MADR)
    let export_all = !args.hotspots && !args.ledger && !args.madr;

    // 1. Export Hotspots from latest-impact.json
    if args.hotspots || export_all {
        let impact_path = layout.reports_dir().join("latest-impact.json");
        if impact_path.exists() {
            let file = File::open(impact_path).into_diagnostic()?;
            let packet: ImpactPacket = serde_json::from_reader(file).into_diagnostic()?;

            // Determine which filter strategy to use
            let use_scoped = args.scope.is_some();
            let filter_paths = args.scope.as_ref();

            // Collect and sort hotspots for deterministic output
            let mut selected_hotspots: Vec<usize> = Vec::new();
            for (idx, hotspot) in packet.hotspots.iter().enumerate() {
                if let Some(paths) = filter_paths {
                    let path_str = hotspot.path.to_string_lossy();
                    let in_filter = paths
                        .iter()
                        .any(|p| path_str.starts_with(p) || path_str == *p);
                    if !in_filter {
                        continue;
                    }
                }
                selected_hotspots.push(idx);
            }

            // Sort selected hotspots by path for deterministic ordering
            selected_hotspots
                .sort_by(|a, b| packet.hotspots[*a].path.cmp(&packet.hotspots[*b].path));

            for idx in selected_hotspots {
                let hotspot = &packet.hotspots[idx];
                let path_str = hotspot.path.to_string_lossy().into_owned();

                // Compute temporal coupling score
                let mut max_coupling = 0.0f64;
                if use_scoped {
                    if let Some(ref scope_paths) = args.scope {
                        for tc in &packet.temporal_couplings {
                            let a_in_scope = scope_paths
                                .iter()
                                .any(|s| tc.file_a.to_string_lossy().starts_with(s.as_str()));
                            let b_in_scope = scope_paths
                                .iter()
                                .any(|s| tc.file_b.to_string_lossy().starts_with(s.as_str()));
                            if !a_in_scope || !b_in_scope {
                                continue;
                            }
                            if (tc.file_a == hotspot.path || tc.file_b == hotspot.path)
                                && (tc.score as f64) > max_coupling
                            {
                                max_coupling = tc.score as f64;
                            }
                        }
                    }
                } else {
                    for tc in &packet.temporal_couplings {
                        if (tc.file_a == hotspot.path || tc.file_b == hotspot.path)
                            && (tc.score as f64) > max_coupling
                        {
                            max_coupling = tc.score as f64;
                        }
                    }
                }

                let max_risk = if use_scoped {
                    let global_max = packet
                        .ci_predictions
                        .iter()
                        .map(|p| p.failure_probability as f64)
                        .fold(0.0f64, f64::max);
                    if max_coupling > 0.0 {
                        global_max * (0.3 + 0.7 * max_coupling)
                    } else {
                        global_max * 0.3
                    }
                } else {
                    packet
                        .ci_predictions
                        .iter()
                        .map(|p| p.failure_probability as f64)
                        .fold(0.0f64, f64::max)
                };

                let payload = BridgePayload::Hotspot {
                    path: path_str.clone(),
                    score: hotspot.score as f64,
                    reason: format!(
                        "Complexity: {}, Frequency: {}",
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
    if args.ledger || export_all {
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
                .or_insert_with(|| (entry.summary.clone(), std::collections::HashSet::new()));
            group.1.insert(entry.entity_normalized.clone());
        }

        let mut sorted_tx_ids: Vec<String> = tx_groups.keys().cloned().collect();
        sorted_tx_ids.sort();

        for tx_id in sorted_tx_ids {
            let (summary, files) = &tx_groups[&tx_id];
            let payload = BridgePayload::LedgerDelta {
                tx_id: tx_id.clone(),
                intent: summary.clone(),
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

    // 3. Export MADR fields
    if args.madr {
        let adr_entries = ledger_db.get_adr_entries(None).into_diagnostic()?;
        let mut sorted_entries = adr_entries;
        sorted_entries.sort_by(|a, b| {
            a.committed_at
                .cmp(&b.committed_at)
                .then_with(|| a.id.cmp(&b.id))
        });

        for entry in sorted_entries {
            let payload = BridgePayload::Madr {
                title: entry.summary.clone(),
                context: entry.reason.clone(),
                decision: format!(
                    "{} the entity `{}`",
                    match entry.change_type {
                        crate::ledger::types::ChangeType::Create => "Create",
                        crate::ledger::types::ChangeType::Modify => "Modify",
                        crate::ledger::types::ChangeType::Deprecate => "Deprecate",
                        crate::ledger::types::ChangeType::Delete => "Delete",
                    },
                    entry.entity_normalized
                ),
                consequences: entry.outcome_notes.clone().unwrap_or_default(),
            };
            let record = BridgeRecord::new(
                BridgeDirection::Outbound,
                project_id.clone(),
                "madr_field",
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

    // 4. Write NDJSON
    match args.out_path {
        Some(path) => {
            if let Some(parent) = std::path::Path::new(&path).parent() {
                fs::create_dir_all(parent).into_diagnostic()?;
            }
            let out_file = File::create(&path).into_diagnostic()?;
            let mut writer = BufWriter::new(out_file);
            for record in records {
                let line = serialize_record(&record).into_diagnostic()?;
                writer.write_all(line.as_bytes()).into_diagnostic()?;
                writer.write_all(b"\n").into_diagnostic()?;
            }
            writer.flush().into_diagnostic()?;
            if !args.json {
                println!("Exported records to bridge NDJSON: {path}");
            }
        }
        None => {
            for record in records {
                let line = serialize_record(&record).into_diagnostic()?;
                println!("{line}");
            }
        }
    }

    save_bridge_state(&layout, &state)?;

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

pub fn execute_verify(scope: Option<Vec<String>>, out: Option<String>) -> Result<()> {
    let current_dir = std::env::current_dir()
        .into_diagnostic()?
        .to_string_lossy()
        .to_string();
    let layout = Layout::new(&current_dir);

    let result = crate::verify::ipc_verify::predictive_verify(scope, &layout)?;
    let json = serde_json::to_string_pretty(&result).into_diagnostic()?;

    match out {
        Some(path) => {
            fs::write(&path, json).into_diagnostic()?;
            println!("Predictive verification result written to {path}");
        }
        None => {
            println!("{json}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::model::{BridgePayload, deserialize_record};
    use crate::impact::packet::{CIPrediction, Hotspot, ImpactPacket, TemporalCoupling};
    use crate::state::layout::Layout;
    use std::path::PathBuf;

    /// Helper: create a minimal ImpactPacket with hotspots and couplings for testing
    fn make_test_packet() -> ImpactPacket {
        ImpactPacket {
            hotspots: vec![
                Hotspot {
                    path: PathBuf::from("src/bridge/export.rs"),
                    score: 0.9,
                    display_score: 0.0,
                    complexity: 12,
                    frequency: 5.0,
                    centrality: Some(3),
                },
                Hotspot {
                    path: PathBuf::from("src/bridge/model.rs"),
                    score: 0.7,
                    display_score: 0.0,
                    complexity: 8,
                    frequency: 3.0,
                    centrality: Some(2),
                },
                Hotspot {
                    path: PathBuf::from("src/ledger/db.rs"),
                    score: 0.85,
                    display_score: 0.0,
                    complexity: 15,
                    frequency: 7.0,
                    centrality: Some(5),
                },
            ],
            temporal_couplings: vec![
                TemporalCoupling {
                    file_a: PathBuf::from("src/bridge/export.rs"),
                    file_b: PathBuf::from("src/bridge/model.rs"),
                    score: 0.85,
                },
                TemporalCoupling {
                    file_a: PathBuf::from("src/bridge/export.rs"),
                    file_b: PathBuf::from("src/ledger/db.rs"),
                    score: 0.6,
                },
                TemporalCoupling {
                    file_a: PathBuf::from("src/ledger/db.rs"),
                    file_b: PathBuf::from("src/ledger/types.rs"),
                    score: 0.95,
                },
            ],
            ci_predictions: vec![CIPrediction {
                job_name: "test".to_string(),
                platform: "github".to_string(),
                failure_probability: 0.15,
                explanation: None,
            }],
            ..ImpactPacket::default()
        }
    }

    fn setup_test_output_dir() -> (tempfile::TempDir, String, Layout) {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let root = dir.path().to_string_lossy().to_string();
        let layout = Layout::new(&root);

        // Create necessary subdirectories
        let state_dir = layout.state_subdir();
        fs::create_dir_all(&state_dir).expect("failed to create state dir");
        let reports_dir = layout.reports_dir();
        fs::create_dir_all(&reports_dir).expect("failed to create reports dir");

        (dir, root, layout)
    }

    #[test]
    fn test_scoped_export_uses_scoped_coupling() {
        let (_dir, root, _layout) = setup_test_output_dir();

        let args = ExportArgs {
            out_path: Some(format!("{}/scoped.ndjson", root)),
            hotspots: true,
            scope: Some(vec!["src/bridge/".to_string()]),
            ..ExportArgs::default()
        };

        execute_export_in_dir(args, Some(&root)).expect("scoped export failed");
    }
}
