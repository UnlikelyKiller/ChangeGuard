use crate::commands::helpers::get_layout;
use crate::output::table::Table;
use crate::state::storage::StorageManager;
use clap::{Args, Subcommand};
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

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

pub fn execute_security(args: SecurityArgs) -> Result<()> {
    let layout = get_layout()?;

    match args.command {
        SecuritySubcommands::Impact { changed, json } => {
            let packet = crate::commands::impact::execute_impact_silent()?;

            let storage = StorageManager::open_read_only(&layout.root)?;
            let cozo = storage
                .cozo
                .as_ref()
                .ok_or_else(|| miette::miette!("CozoDB not available"))?;

            // Collect changed files
            let changed_files: std::collections::HashSet<String> = packet
                .changes
                .iter()
                .map(|c| c.path.to_string_lossy().replace('\\', "/"))
                .collect();

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
                    // URN format: urn:changeguard:policy:<path_to_cedar_file>:<index>
                    // Let's see if any changed file is in this ID
                    let mut is_impacted = false;
                    for cf in &changed_files {
                        if id.contains(cf) {
                            is_impacted = true;
                            break;
                        }
                    }

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

            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&impacted).into_diagnostic()?
                );
            } else {
                println!("{}", "Security Policy Impact Analysis".bold().red());
                let mut table = Table::new();
                table.set_header(vec!["Policy ID", "Effect", "Changed?", "Raw Policy"]);

                for item in impacted {
                    table.add_row(vec![
                        item["id"].as_str().unwrap_or("").to_string(),
                        item["effect"].as_str().unwrap_or("").to_string(),
                        if item["is_changed"].as_bool().unwrap_or(false) {
                            "YES".red().bold().to_string()
                        } else {
                            "NO".dimmed().to_string()
                        },
                        item["raw"].as_str().unwrap_or("").to_string(),
                    ]);
                }
                println!("{}", table);
            }
        }
        SecuritySubcommands::Boundaries { json } => {
            let storage = StorageManager::open_read_only(&layout.root)?;
            let cozo = storage
                .cozo
                .as_ref()
                .ok_or_else(|| miette::miette!("CozoDB not available"))?;

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

                println!(
                    "\n{}",
                    "Authorization Nodes (policy/principal/action/resource):".bold()
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
                        auth_table.add_row(vec![
                            cat.to_string(),
                            label.to_string(),
                            id.to_string(),
                        ]);
                    }
                }
                println!("{}", auth_table);

                println!(
                    "\n{}",
                    "Cross-Surface Boundary Links (policy → protected entity):".bold()
                );
                if boundary_res.rows.is_empty() {
                    println!(
                        "{}",
                        "  No cross-surface links found. Run `changeguard index --incremental` to refresh."
                            .dimmed()
                    );
                } else {
                    let mut boundary_table = Table::new();
                    boundary_table.set_header(vec![
                        "Policy",
                        "Relation",
                        "Target",
                        "Target Category",
                    ]);
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
                                plabel.to_string(),
                                rel.to_string(),
                                tlabel.to_string(),
                                tcat.to_string(),
                            ]);
                        }
                    }
                    println!("{}", boundary_table);
                }
            }
        }
    }

    Ok(())
}
