use cozo::*;
use miette::Result;
use std::path::Path;
use tracing::{debug, info, warn};

use crate::state::storage_cozo::CozoStorage;
use crate::state::cozo::queries::*;

pub fn setup_schema(storage: &CozoStorage) -> Result<()> {
    let existing = storage.get_relations()?;

    if !existing.contains(&"node".to_string()) {
        storage.run_script(CREATE_NODE_TABLE)?;
    }
    if !existing.contains(&"edge".to_string()) {
        storage.run_script(CREATE_EDGE_TABLE)?;
    }
    if !existing.contains(&"ledger_link".to_string()) {
        storage.run_script(CREATE_LEDGER_LINK_TABLE)?;
    }
    if !existing.contains(&"project_symbol".to_string()) {
        storage.run_script(CREATE_PROJECT_SYMBOL_TABLE)?;
    }

    // FTS Index
    if !existing.contains(&"node:fts_idx".to_string()) {
        storage.run_script(CREATE_FTS_INDEX)?;
    }

    // AI-Brains relations
    if !existing.contains(&"Turn".to_string()) { storage.run_script(CREATE_TURN_TABLE)?; }
    if !existing.contains(&"Session".to_string()) { storage.run_script(CREATE_SESSION_TABLE)?; }
    if !existing.contains(&"Memory".to_string()) { storage.run_script(CREATE_MEMORY_TABLE)?; }
    if !existing.contains(&"Decision".to_string()) { storage.run_script(CREATE_DECISION_TABLE)?; }

    // Metadata & Migration
    if !existing.contains(&"cozo_meta".to_string()) {
        storage.run_script(":create cozo_meta { key: String => value: String }")?;
        storage.run_script("?[key, value] <- [[\"cozo_schema_version\", \"2\"]] :put cozo_meta")?;
    }

    // Now ensure ledger_entry exists
    let existing = storage.get_relations()?;
    if !existing.contains(&"ledger_entry".to_string()) {
        storage.run_script(CREATE_LEDGER_ENTRY_TABLE)?;
    }

    migrate_cozo_schema(storage)?;

    Ok(())
}

pub fn migrate_cozo_schema(storage: &CozoStorage) -> Result<()> {
    let res = storage.run_script("?[value] := *cozo_meta{key: \"cozo_schema_version\", value: value}")?;
    let version = res
        .rows
        .first()
        .and_then(|r| r.first())
        .and_then(|v| {
            if let DataValue::Str(s) = v {
                s.parse::<u32>().ok()
            } else {
                None
            }
        })
        .unwrap_or(1);

    if version < 2 {
        info!("[migrate] Upgrading CozoDB schema from v1 to v2");
        
        // 1. Check if ledger_entry exists before trying to migrate data
        let existing = storage.get_relations()?;
        if existing.contains(&"ledger_entry".to_string()) {
             let current_entries = match storage.run_script("?[id, tx_id, category, entry_type, entity_normalized, change_type, summary, reason, committed_at, is_breaking, verification_status, trace_id] := *ledger_entry{id, tx_id, category, entry_type, entity_normalized, change_type, summary, reason, committed_at, is_breaking, verification_status, trace_id}") {
                 Ok(res) => res,
                 Err(e) => {
                     warn!("Failed to query old ledger_entry for migration: {}. Skipping data migration.", e);
                     NamedRows { headers: Vec::new(), rows: Vec::new(), next: None }
                 }
             };

             storage.run_script("::remove ledger_entry")?;
             storage.run_script(CREATE_LEDGER_ENTRY_TABLE)?;

             if !current_entries.rows.is_empty() {
                let mut script = String::from("?[id, tx_id, category, entry_type, entity_normalized, change_type, summary, reason, committed_at, is_breaking, verification_status, trace_id, signature, public_key, risk, related_tickets] <- [\n");
                for (i, row) in current_entries.rows.iter().enumerate() {
                    let mut row_vals = Vec::new();
                    for val in row {
                        match val {
                            DataValue::Str(s) => {
                                let escaped = serde_json::to_string(s.as_str()).unwrap_or_else(|_| "\"\"".to_string());
                                row_vals.push(escaped);
                            }
                            DataValue::Num(Num::Int(n)) => row_vals.push(n.to_string()),
                            DataValue::Bool(b) => row_vals.push(b.to_string()),
                            _ => row_vals.push("null".to_string()),
                        }
                    }

                    // Append empty strings for new signature and public_key, 
                    // 'Low' for risk, and '[]' for related_tickets
                    script.push_str(&format!(
                        "  [{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, \"\", \"\", \"Low\", \"[]\"]{}\n",
                        row_vals[0], row_vals[1], row_vals[2], row_vals[3], row_vals[4], row_vals[5],
                        row_vals[6], row_vals[7], row_vals[8], row_vals[9], row_vals[10], row_vals[11],
                        if i == current_entries.rows.len() - 1 { "" } else { "," }
                    ));
                }

                script.push_str("] :put ledger_entry");
                storage.run_script(&script)?;
             }
        } else {
             // Just create it if it didn't exist
             storage.run_script(CREATE_LEDGER_ENTRY_TABLE)?;
        }

        storage.run_script("?[key, value] <- [[\"cozo_schema_version\", \"2\"]] :put cozo_meta")?;
    }

    Ok(())
}

pub fn initialize_instance(db_path: &Path, read_only: bool) -> Result<DbInstance> {
    let engine = if db_path.as_os_str().is_empty() {
        "mem"
    } else {
        "sled"
    };
    debug!(
        "CozoStorage selecting engine '{}' for path {:?} (read_only: {})",
        engine, db_path, read_only
    );

    let is_new = engine == "sled" && !db_path.exists();
    
    // Bounded Retry Loop for Locks
    let mut retries = 0;
    let max_retries = if read_only { 15 } else { 5 };
    let base_delay_ms = 100;

    let db = loop {
        match DbInstance::new(engine, db_path, Default::default()) {
            Ok(db) => break db,
            Err(e) if engine == "sled" && retries < max_retries => {
                let err_debug = format!("{:?}", e).to_lowercase();
                if err_debug.contains("lock") || err_debug.contains("access is denied") || err_debug.contains("os error 33") {
                    retries += 1;
                    let delay = base_delay_ms * (2u64.pow(retries - 1));
                    debug!("CozoDB is locked, retrying in {}ms", delay);
                    std::thread::sleep(std::time::Duration::from_millis(delay));
                    continue;
                }
                return Err(miette::miette!("Failed to initialize CozoDB: {:?}", e));
            }
            Err(e) => return Err(miette::miette!("Failed to initialize CozoDB: {:?}", e)),
        }
    };

    // Cold Start Verification
    if engine == "sled" && !is_new {
        if let Err(e) = db.run_script(GET_RELATIONS, Default::default(), ScriptMutability::Immutable) {
            return Err(miette::miette!("CozoDB Cold Start Verification failed: {}", e));
        }
    }

    Ok(db)
}
