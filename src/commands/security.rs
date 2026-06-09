use crate::commands::helpers::get_layout;
use crate::output::table::Table;
use crate::state::storage::StorageManager;
use clap::{Args, Subcommand};
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::collections::HashSet;

#[derive(Args, Debug)]
pub struct SecurityArgs {
    #[command(subcommand)]
    pub command: SecuritySubcommands,
}

#[derive(Subcommand, Debug)]
pub enum SecuritySubcommands {
    /// Show security impact of recent changes
    Impact {
        /// Filter by changed policies only
        #[arg(long)]
        changed: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// List security boundaries, roles, and policies
    Boundaries {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

/// Extract changed file paths from the impact packet as a HashSet of normalized paths.
fn collect_changed_files() -> Result<HashSet<String>> {
    let packet = crate::commands::impact::execute_impact_silent()?;
    let changed: HashSet<String> = packet
        .changes
        .iter()
        .map(|c| c.path.to_string_lossy().replace('\\', "/"))
        .collect();
    Ok(changed)
}

/// Open CozoDB storage in read-only mode and return the Cozo engine.
fn open_cozo(root: &camino::Utf8Path) -> Result<crate::state::storage_cozo::CozoStorage> {
    let storage = StorageManager::open_read_only(root)?;
    storage
        .cozo
        .ok_or_else(|| miette::miette!("CozoDB not available"))
}

/// Truncate a string to `max_len` characters, appending "…" if it was cut.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len])
    }
}

fn execute_impact(changed: bool, json: bool, layout: &crate::state::layout::Layout) -> Result<()> {
    let changed_files = collect_changed_files()?;
    let cozo = open_cozo(&layout.root)?;

    // Query all policy nodes from Cozo
    let query = "?[id, label, metadata] := *node{id, label, category: 'policy', metadata}";
    let res = cozo.run_script(query)?;

    let mut impacted = Vec::new();
    for row in res.rows {
        if let (
            Some(cozo::DataValue::Str(id)),
            Some(cozo::DataValue::Str(label)),
            Some(cozo::DataValue::Json(meta)),
        ) = (row.first(), row.get(1), row.get(2))
        {
            let is_impacted = changed_files.iter().any(|cf| id.contains(cf));

            if !changed || is_impacted {
                impacted.push(serde_json::json!({
                    "id": id,
                    "label": label,
                    "raw": meta.get("raw").and_then(|v| v.as_str()).unwrap_or(""),
                    "effect": meta.get("effect").and_then(|v| v.as_str()).unwrap_or(""),
                    "is_changed": is_impacted,
                }));
            }
        }
    }

    let total = impacted.len();
    let changed_count = impacted
        .iter()
        .filter(|i| i["is_changed"].as_bool().unwrap_or(false))
        .count();

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&impacted).into_diagnostic()?
        );
    } else {
        println!("{}", "Security Policy Impact Analysis".bold().red());
        let mut table = Table::new();
        table.set_header(vec!["Policy ID", "Effect", "Changed?"]);

        for item in &impacted {
            table.add_row(vec![
                item["id"].as_str().unwrap_or("").to_string(),
                item["effect"].as_str().unwrap_or_default().to_string(),
                if item["is_changed"].as_bool().unwrap_or(false) {
                    "YES".yellow().bold().to_string()
                } else {
                    "NO".to_string()
                },
            ]);
        }

        println!("{}", table);
        // Summary counts
        if changed {
            println!(
                "  {} of {} policies match changed files",
                changed_count.to_string().yellow().bold(),
                total.to_string().bold(),
            );
        } else {
            println!(
                "  {} policies evaluated, {} changed by this diff",
                total.to_string().bold(),
                changed_count.to_string().yellow().bold(),
            );
        }
    }

    Ok(())
}

fn execute_boundaries(json: bool, layout: &crate::state::layout::Layout) -> Result<()> {
    let cozo = open_cozo(&layout.root)?;

    // Query 1: policy + principal/action/resource authorisation nodes
    let auth_res = cozo.run_script(
        "?[id, label, category] := *node{id, label, category}, \
         category in ['policy', 'principal', 'action', 'resource']",
    )?;

    // Query 2: cross-surface boundary edges — policy → service/endpoint/config/deploy/adr
    let boundary_res = cozo.run_script(
        "?[policy_id, policy_label, relation, target_id, target_label, target_cat] := \
         *node{id: policy_id, label: policy_label, category: 'policy'}, \
         *edge{source: policy_id, target: target_id, relation: rel}, \
         *node{id: target_id, label: target_label, category: target_cat}, \
         target_cat in ['service', 'endpoint', 'config_key', 'deploy_surface', 'adr'], \
         relation = rel",
    )?;

    if json {
        let mut auth_nodes = Vec::new();
        for row in auth_res.rows {
            if let (
                Some(cozo::DataValue::Str(id)),
                Some(cozo::DataValue::Str(label)),
                Some(cozo::DataValue::Str(cat)),
            ) = (row.first(), row.get(1), row.get(2))
            {
                auth_nodes.push(serde_json::json!({
                    "id": id, "label": label, "category": cat,
                }));
            }
        }
        let mut boundary_edges = Vec::new();
        for row in boundary_res.rows {
            if let (
                Some(cozo::DataValue::Str(pid)),
                Some(cozo::DataValue::Str(plabel)),
                Some(cozo::DataValue::Str(rel)),
                Some(cozo::DataValue::Str(tid)),
                Some(cozo::DataValue::Str(tlabel)),
                Some(cozo::DataValue::Str(tcat)),
            ) = (
                row.first(),
                row.get(1),
                row.get(2),
                row.get(3),
                row.get(4),
                row.get(5),
            ) {
                boundary_edges.push(serde_json::json!({
                    "policy_id": pid, "policy_label": plabel,
                    "relation": rel,
                    "target_id": tid, "target_label": tlabel, "target_category": tcat,
                }));
            }
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "auth_nodes": auth_nodes,
                "boundary_edges": boundary_edges,
            }))
            .into_diagnostic()?
        );
    } else {
        println!("{}", "Security Boundaries & Policies".bold().red());

        // --- Auth nodes table ---
        let auth_count = auth_res.rows.len();
        println!(
            "\n{} ({} total)",
            "Authorization Nodes (policy/principal/action/resource):".bold(),
            auth_count.to_string().bold(),
        );
        let mut auth_table = Table::new();
        auth_table.set_header(vec!["Category", "Label", "ID"]);
        for row in auth_res.rows {
            if let (
                Some(cozo::DataValue::Str(id)),
                Some(cozo::DataValue::Str(label)),
                Some(cozo::DataValue::Str(cat)),
            ) = (row.first(), row.get(1), row.get(2))
            {
                auth_table.add_row(vec![cat.to_string(), truncate(label, 60), truncate(id, 80)]);
            }
        }
        println!("{}", auth_table);

        // --- Boundary links table ---
        let boundary_count = boundary_res.rows.len();
        println!(
            "\n{} ({} total)",
            "Cross-Surface Boundary Links (policy → protected entity):".bold(),
            boundary_count.to_string().bold(),
        );
        if boundary_res.rows.is_empty() {
            println!(
                "{}",
                "  No cross-surface links found. Run `changeguard index --incremental` to refresh."
                    .dimmed()
            );
        } else {
            let mut boundary_table = Table::new();
            boundary_table.set_header(vec!["Policy", "Relation", "Target", "Target Category"]);
            for row in boundary_res.rows {
                if let (
                    Some(cozo::DataValue::Str(_pid)),
                    Some(cozo::DataValue::Str(plabel)),
                    Some(cozo::DataValue::Str(rel)),
                    Some(cozo::DataValue::Str(_tid)),
                    Some(cozo::DataValue::Str(tlabel)),
                    Some(cozo::DataValue::Str(tcat)),
                ) = (
                    row.first(),
                    row.get(1),
                    row.get(2),
                    row.get(3),
                    row.get(4),
                    row.get(5),
                ) {
                    boundary_table.add_row(vec![
                        truncate(plabel, 50),
                        rel.to_string(),
                        truncate(tlabel, 50),
                        tcat.to_string(),
                    ]);
                }
            }
            println!("{}", boundary_table);
        }
    }

    Ok(())
}

pub fn execute_security(args: SecurityArgs) -> Result<()> {
    let layout = get_layout()?;

    match args.command {
        SecuritySubcommands::Impact { changed, json } => execute_impact(changed, json, &layout),
        SecuritySubcommands::Boundaries { json } => execute_boundaries(json, &layout),
    }
}
