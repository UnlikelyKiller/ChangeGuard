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

pub fn execute_export(
    out_path: String,
    hotspots: bool,
    targets: Option<Vec<String>>,
    scope: Option<Vec<String>>,
    ledger: bool,
    madr: bool,
) -> Result<()> {
    execute_export_in_dir(out_path, hotspots, targets, scope, ledger, madr, None)
}

/// Internal implementation that accepts an optional base directory override.
/// When `base_dir` is `None`, uses `std::env::current_dir()`.
fn execute_export_in_dir(
    out_path: String,
    hotspots: bool,
    targets: Option<Vec<String>>,
    scope: Option<Vec<String>>,
    ledger: bool,
    madr: bool,
    base_dir: Option<&str>,
) -> Result<()> {
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
    let export_all = !hotspots && !ledger && !madr;

    // 1. Export Hotspots from latest-impact.json
    if hotspots || export_all {
        let impact_path = layout.reports_dir().join("latest-impact.json");
        if impact_path.exists() {
            let file = File::open(impact_path).into_diagnostic()?;
            let packet: ImpactPacket = serde_json::from_reader(file).into_diagnostic()?;

            // Determine which filter strategy to use
            let use_scoped = scope.is_some();
            let filter_paths = scope.as_ref().or(targets.as_ref());

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
                    // Scoped analysis: only consider couplings where both files are
                    // within the scope. This gives targeted temporal coupling scores
                    // rather than global coupling noise.
                    if let Some(ref scope_paths) = scope {
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
                    // Global: consider all temporal couplings
                    for tc in &packet.temporal_couplings {
                        if (tc.file_a == hotspot.path || tc.file_b == hotspot.path)
                            && (tc.score as f64) > max_coupling
                        {
                            max_coupling = tc.score as f64;
                        }
                    }
                }

                // Compute failure risk probability
                let max_risk = if use_scoped {
                    // CIPrediction lacks per-file mapping, so we weight the
                    // global max CI risk by scoped temporal coupling relevance.
                    // Files with high temporal coupling are more likely to be
                    // affected by CI failures in coupled paths.
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
                .or_insert_with(|| (entry.summary.clone(), std::collections::HashSet::new()));
            group.1.insert(entry.entity_normalized.clone());
        }

        // Sort tx_ids for deterministic output
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

    // 3. Export MADR fields (structured, not pre-formatted markdown)
    if madr {
        let adr_entries = ledger_db.get_adr_entries(None).into_diagnostic()?;

        // Sort entries by committed_at ascending for deterministic order,
        // then by id as a stable tiebreaker
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
                    frequency: 5,
                    centrality: Some(3),
                },
                Hotspot {
                    path: PathBuf::from("src/bridge/model.rs"),
                    score: 0.7,
                    display_score: 0.0,
                    complexity: 8,
                    frequency: 3,
                    centrality: Some(2),
                },
                Hotspot {
                    path: PathBuf::from("src/ledger/db.rs"),
                    score: 0.85,
                    display_score: 0.0,
                    complexity: 15,
                    frequency: 7,
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
        let (_dir, root, layout) = setup_test_output_dir();

        // Write a test impact packet
        let packet = make_test_packet();
        let impact_path = layout.reports_dir().join("latest-impact.json");
        let json = serde_json::to_string_pretty(&packet).expect("failed to serialize");
        fs::write(&impact_path, json).expect("failed to write impact");

        let out_path = layout
            .reports_dir()
            .join("scoped-export.ndjson")
            .to_string();

        // Call with scope targeting only src/bridge/
        let scope = Some(vec!["src/bridge/".to_string()]);
        execute_export_in_dir(
            out_path.clone(),
            true,
            None,
            scope,
            false,
            false,
            Some(&root),
        )
        .expect("scoped export failed");

        // Read back the NDJSON output
        let content = fs::read_to_string(&out_path).expect("failed to read output");
        let records: Vec<_> = content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| deserialize_record(l).ok())
            .collect();

        // Should only contain bridge/ files, not ledger/
        assert!(!records.is_empty(), "expected at least one hotspot record");
        for record in &records {
            if let BridgePayload::Hotspot { path, .. } = &record.payload {
                assert!(
                    path.starts_with("src/bridge/"),
                    "scoped export included out-of-scope path: {path}"
                );
            }
        }

        // Verify deterministic ordering: paths should be sorted
        let hotspot_paths: Vec<&str> = records
            .iter()
            .filter_map(|r| {
                if let BridgePayload::Hotspot { path, .. } = &r.payload {
                    Some(path.as_str())
                } else {
                    None
                }
            })
            .collect();
        let mut sorted_paths = hotspot_paths.clone();
        sorted_paths.sort();
        assert_eq!(
            hotspot_paths, sorted_paths,
            "hotspots not sorted deterministically"
        );

        // Verify temporal coupling is scoped: export.rs max coupling should be
        // 0.85 (with model.rs, both in scope) not 0.6 (with ledger/db.rs, out of scope)
        let export_record = records.iter().find(|r| {
            if let BridgePayload::Hotspot { path, .. } = &r.payload {
                path == "src/bridge/export.rs"
            } else {
                false
            }
        });
        assert!(
            export_record.is_some(),
            "export.rs should be in scoped output"
        );
        if let BridgePayload::Hotspot {
            temporal_coupling, ..
        } = &export_record.unwrap().payload
        {
            assert!(
                (*temporal_coupling - 0.85).abs() < 1e-6,
                "expected scoped coupling 0.85, got {temporal_coupling}"
            );
        }
    }

    #[test]
    fn test_unscoped_export_preserves_global_behavior() {
        let (_dir, root, layout) = setup_test_output_dir();

        let packet = make_test_packet();
        let impact_path = layout.reports_dir().join("latest-impact.json");
        let json = serde_json::to_string_pretty(&packet).expect("failed to serialize");
        fs::write(&impact_path, json).expect("failed to write impact");

        let out_path = layout
            .reports_dir()
            .join("global-export.ndjson")
            .to_string();

        // Call without scope (global export)
        execute_export_in_dir(
            out_path.clone(),
            true,
            None,
            None,
            false,
            false,
            Some(&root),
        )
        .expect("global export failed");

        let content = fs::read_to_string(&out_path).expect("failed to read output");
        let records: Vec<_> = content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| deserialize_record(l).ok())
            .collect();

        // Should contain all three hotspots
        assert_eq!(
            records.len(),
            3,
            "global export should include all 3 hotspots"
        );

        // Verify global coupling includes cross-scope: export.rs should have
        // max coupling 0.85 (with model.rs, which has higher score than 0.6 with db.rs)
        let export_record = records.iter().find(|r| {
            if let BridgePayload::Hotspot { path, .. } = &r.payload {
                path == "src/bridge/export.rs"
            } else {
                false
            }
        });
        assert!(
            export_record.is_some(),
            "export.rs should be in global output"
        );
        if let BridgePayload::Hotspot {
            temporal_coupling, ..
        } = &export_record.unwrap().payload
        {
            assert!(
                (*temporal_coupling - 0.85).abs() < 1e-6,
                "expected global coupling 0.85, got {temporal_coupling}"
            );
        }

        // Verify deterministic ordering
        let hotspot_paths: Vec<&str> = records
            .iter()
            .filter_map(|r| {
                if let BridgePayload::Hotspot { path, .. } = &r.payload {
                    Some(path.as_str())
                } else {
                    None
                }
            })
            .collect();
        let mut sorted_paths = hotspot_paths.clone();
        sorted_paths.sort();
        assert_eq!(
            hotspot_paths, sorted_paths,
            "global hotspots not sorted deterministically"
        );
    }

    #[test]
    fn test_madr_export_emits_structured_fields_not_markdown() {
        let (_dir, root, layout) = setup_test_output_dir();

        // Initialize the ledger database with test ADR entries
        let storage_path = layout.state_subdir().join("ledger.db");
        let storage =
            StorageManager::init(storage_path.as_std_path()).expect("failed to init storage");
        let conn = storage.get_connection();
        let ledger_db = crate::ledger::db::LedgerDb::new(conn);

        use crate::ledger::types::{Category, ChangeType, EntryType, LedgerEntry, Transaction};
        use chrono::Utc;

        // Insert a transaction first
        let tx = Transaction {
            tx_id: "tx-madr-001".to_string(),
            operation_id: None,
            status: "COMMITTED".to_string(),
            category: Category::Architecture,
            entity: "src/bridge/model.rs".to_string(),
            entity_normalized: "src/bridge/model.rs".to_string(),
            planned_action: Some("Add MADR payload variant".to_string()),
            session_id: "test-session".to_string(),
            source: "LOCAL".to_string(),
            started_at: Utc::now().to_rfc3339(),
            resolved_at: Some(Utc::now().to_rfc3339()),
            detected_at: None,
            drift_count: 0,
            first_seen_at: None,
            last_seen_at: None,
            issue_ref: None,
        };
        ledger_db
            .insert_transaction(&tx)
            .expect("failed to insert transaction");

        // Insert an architecture entry
        let entry = LedgerEntry {
            id: 0, // auto-assigned
            tx_id: "tx-madr-001".to_string(),
            category: Category::Architecture,
            entry_type: EntryType::Architecture,
            entity: "src/bridge/model.rs".to_string(),
            entity_normalized: "src/bridge/model.rs".to_string(),
            change_type: ChangeType::Modify,
            summary: "Add MADR structured fields to bridge protocol".to_string(),
            reason: "AI-Brains needs structured decision records for nightly ingestion without markdown pre-formatting.".to_string(),
            is_breaking: false,
            committed_at: Utc::now().to_rfc3339(),
            verification_status: None,
            verification_basis: None,
            outcome_notes: Some("MADR fields are sent as structured JSON, not pre-formatted markdown.".to_string()),
            origin: "LOCAL".to_string(),
            trace_id: None,
        };
        ledger_db
            .insert_ledger_entry(&entry)
            .expect("failed to insert ledger entry");

        // Also insert a breaking change entry
        let tx2 = Transaction {
            tx_id: "tx-madr-002".to_string(),
            operation_id: None,
            status: "COMMITTED".to_string(),
            category: Category::Architecture,
            entity: "src/bridge/ipc.rs".to_string(),
            entity_normalized: "src/bridge/ipc.rs".to_string(),
            planned_action: Some("Bump bridge version".to_string()),
            session_id: "test-session".to_string(),
            source: "LOCAL".to_string(),
            started_at: Utc::now().to_rfc3339(),
            resolved_at: Some(Utc::now().to_rfc3339()),
            detected_at: None,
            drift_count: 0,
            first_seen_at: None,
            last_seen_at: None,
            issue_ref: None,
        };
        ledger_db
            .insert_transaction(&tx2)
            .expect("failed to insert transaction 2");

        let entry2 = LedgerEntry {
            id: 0,
            tx_id: "tx-madr-002".to_string(),
            category: Category::Architecture,
            entry_type: EntryType::Architecture,
            entity: "src/bridge/ipc.rs".to_string(),
            entity_normalized: "src/bridge/ipc.rs".to_string(),
            change_type: ChangeType::Modify,
            summary: "Bump bridge wire protocol to v0.2".to_string(),
            reason: "Version bump required for new MADR payload variant.".to_string(),
            is_breaking: true,
            committed_at: Utc::now().to_rfc3339(),
            verification_status: None,
            verification_basis: None,
            outcome_notes: Some("All existing clients must upgrade to v0.2.".to_string()),
            origin: "LOCAL".to_string(),
            trace_id: None,
        };
        ledger_db
            .insert_ledger_entry(&entry2)
            .expect("failed to insert ledger entry 2");

        // Also write a minimal impact packet so the export doesn't fail on missing file
        let packet = ImpactPacket::default();
        let impact_path = layout.reports_dir().join("latest-impact.json");
        let json = serde_json::to_string_pretty(&packet).expect("failed to serialize");
        fs::write(&impact_path, json).expect("failed to write impact");

        let out_path = layout.reports_dir().join("madr-export.ndjson").to_string();

        // Export MADR only
        execute_export_in_dir(
            out_path.clone(),
            false,
            None,
            None,
            false,
            true,
            Some(&root),
        )
        .expect("MADR export failed");

        let content = fs::read_to_string(&out_path).expect("failed to read output");
        let records: Vec<_> = content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| deserialize_record(l).ok())
            .collect();

        // Should have 2 MADR records (both architecture entries)
        assert_eq!(
            records.len(),
            2,
            "expected 2 MADR records, got {}",
            records.len()
        );

        for record in &records {
            // Verify record kind
            assert_eq!(record.record_kind, "madr_field");

            // Verify payload is Madr with structured fields
            if let BridgePayload::Madr {
                title,
                context,
                decision,
                consequences,
            } = &record.payload
            {
                // Verify fields are present and non-empty
                assert!(!title.is_empty(), "MADR title should not be empty");
                assert!(!context.is_empty(), "MADR context should not be empty");
                assert!(!decision.is_empty(), "MADR decision should not be empty");
                // consequences can be empty for entries without outcome_notes

                // Verify NO markdown pre-formatting
                assert!(
                    !title.contains('#'),
                    "MADR title should not contain markdown headers: {title}"
                );
                assert!(
                    !context.contains("##"),
                    "MADR context should not contain markdown headers: {context}"
                );
                assert!(
                    !decision.contains("**"),
                    "MADR decision should not contain markdown formatting: {decision}"
                );
                assert!(
                    !consequences.contains("##"),
                    "MADR consequences should not contain markdown headers: {consequences}"
                );
            } else {
                panic!("expected Madr payload, got {:?}", record.payload);
            }
        }

        // Verify deterministic ordering: entries sorted by committed_at, then id
        let titles: Vec<&str> = records
            .iter()
            .filter_map(|r| {
                if let BridgePayload::Madr { title, .. } = &r.payload {
                    Some(title.as_str())
                } else {
                    None
                }
            })
            .collect();
        // "Add MADR..." comes before "Bump bridge..." alphabetically assuming same timestamp
        assert_eq!(titles[0], "Add MADR structured fields to bridge protocol");
        assert_eq!(titles[1], "Bump bridge wire protocol to v0.2");
    }

    #[test]
    fn test_madr_flag_does_not_affect_export_all_behavior() {
        let (_dir, root, layout) = setup_test_output_dir();

        // Write a minimal impact packet
        let packet = ImpactPacket {
            hotspots: vec![Hotspot {
                path: PathBuf::from("src/main.rs"),
                score: 0.5,
                display_score: 0.0,
                complexity: 5,
                frequency: 2,
                centrality: None,
            }],
            ..ImpactPacket::default()
        };
        let impact_path = layout.reports_dir().join("latest-impact.json");
        let json = serde_json::to_string_pretty(&packet).expect("failed to serialize");
        fs::write(&impact_path, json).expect("failed to write impact");

        // Initialize ledger with one ADR entry
        let storage_path = layout.state_subdir().join("ledger.db");
        let storage =
            StorageManager::init(storage_path.as_std_path()).expect("failed to init storage");
        let conn = storage.get_connection();
        let ledger_db = crate::ledger::db::LedgerDb::new(conn);

        use crate::ledger::types::{Category, ChangeType, EntryType, LedgerEntry, Transaction};
        use chrono::Utc;

        let tx = Transaction {
            tx_id: "tx-export-all-001".to_string(),
            operation_id: None,
            status: "COMMITTED".to_string(),
            category: Category::Architecture,
            entity: "src/main.rs".to_string(),
            entity_normalized: "src/main.rs".to_string(),
            planned_action: None,
            session_id: "test".to_string(),
            source: "LOCAL".to_string(),
            started_at: Utc::now().to_rfc3339(),
            resolved_at: Some(Utc::now().to_rfc3339()),
            detected_at: None,
            drift_count: 0,
            first_seen_at: None,
            last_seen_at: None,
            issue_ref: None,
        };
        ledger_db.insert_transaction(&tx).expect("insert tx");

        let entry = LedgerEntry {
            id: 0,
            tx_id: "tx-export-all-001".to_string(),
            category: Category::Architecture,
            entry_type: EntryType::Architecture,
            entity: "src/main.rs".to_string(),
            entity_normalized: "src/main.rs".to_string(),
            change_type: ChangeType::Create,
            summary: "Initial architecture".to_string(),
            reason: "Starting point.".to_string(),
            is_breaking: false,
            committed_at: Utc::now().to_rfc3339(),
            verification_status: None,
            verification_basis: None,
            outcome_notes: None,
            origin: "LOCAL".to_string(),
            trace_id: None,
        };
        ledger_db.insert_ledger_entry(&entry).expect("insert entry");

        // Export with NO flags (export_all behavior)
        let out_path_all = layout.reports_dir().join("export-all.ndjson").to_string();
        execute_export_in_dir(
            out_path_all.clone(),
            false,
            None,
            None,
            false,
            false,
            Some(&root),
        )
        .expect("export_all failed");

        let content_all = fs::read_to_string(&out_path_all).expect("read output");
        let records_all: Vec<_> = content_all
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| deserialize_record(l).ok())
            .collect();

        // export_all (no flags) should have hotspots + ledger, NOT MADR
        let has_hotspot = records_all
            .iter()
            .any(|r| matches!(r.payload, BridgePayload::Hotspot { .. }));
        let has_madr = records_all
            .iter()
            .any(|r| matches!(r.payload, BridgePayload::Madr { .. }));
        assert!(has_hotspot, "export_all should include hotspots");
        assert!(
            !has_madr,
            "export_all should NOT include MADR without --madr flag"
        );

        // Export with --madr flag
        let out_path_madr = layout.reports_dir().join("export-madr.ndjson").to_string();
        execute_export_in_dir(
            out_path_madr.clone(),
            false,
            None,
            None,
            false,
            true,
            Some(&root),
        )
        .expect("madr export failed");

        let content_madr = fs::read_to_string(&out_path_madr).expect("read output");
        let records_madr: Vec<_> = content_madr
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| deserialize_record(l).ok())
            .collect();

        let has_madr_flag = records_madr
            .iter()
            .any(|r| matches!(r.payload, BridgePayload::Madr { .. }));
        assert!(has_madr_flag, "--madr flag should export MADR records");
    }

    #[test]
    fn test_scoped_vs_global_returns_different_results() {
        let (_dir, root, layout) = setup_test_output_dir();

        let packet = make_test_packet();
        let impact_path = layout.reports_dir().join("latest-impact.json");
        let json = serde_json::to_string_pretty(&packet).expect("failed to serialize");
        fs::write(&impact_path, json).expect("failed to write impact");

        // Global export
        let global_path = layout.reports_dir().join("global.ndjson").to_string();
        execute_export_in_dir(
            global_path.clone(),
            true,
            None,
            None,
            false,
            false,
            Some(&root),
        )
        .expect("global export failed");

        // Scoped export
        let scoped_path = layout.reports_dir().join("scoped.ndjson").to_string();
        execute_export_in_dir(
            scoped_path.clone(),
            true,
            None,
            Some(vec!["src/bridge/".to_string()]),
            false,
            false,
            Some(&root),
        )
        .expect("scoped export failed");

        let global_content = fs::read_to_string(&global_path).expect("read global");
        let scoped_content = fs::read_to_string(&scoped_path).expect("read scoped");

        let global_records: Vec<_> = global_content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| deserialize_record(l).ok())
            .collect();
        let scoped_records: Vec<_> = scoped_content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| deserialize_record(l).ok())
            .collect();

        // Global should have 3 hotspots, scoped should have only 2 (bridge/ files)
        assert_eq!(global_records.len(), 3, "global should have 3 hotspots");
        assert_eq!(scoped_records.len(), 2, "scoped should have 2 hotspots");

        // The sets should be different
        let global_paths: std::collections::HashSet<String> = global_records
            .iter()
            .filter_map(|r| {
                if let BridgePayload::Hotspot { path, .. } = &r.payload {
                    Some(path.clone())
                } else {
                    None
                }
            })
            .collect();
        let scoped_paths: std::collections::HashSet<String> = scoped_records
            .iter()
            .filter_map(|r| {
                if let BridgePayload::Hotspot { path, .. } = &r.payload {
                    Some(path.clone())
                } else {
                    None
                }
            })
            .collect();
        assert_ne!(
            global_paths, scoped_paths,
            "scoped and global exports should return different results"
        );
    }
}
