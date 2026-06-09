use crate::commands::helpers::get_layout;
use crate::output::table::Table;
use crate::state::storage::StorageManager;
use clap::{Args, Subcommand};
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

#[derive(Args, Debug)]
pub struct ObservabilityArgs {
    #[command(subcommand)]
    pub command: ObservabilitySubcommands,
}

#[derive(Subcommand, Debug)]
pub enum ObservabilitySubcommands {
    /// Show observability coverage for services and endpoints
    Coverage {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show observability changes based on current diff (changed SLOs, metrics, alerts)
    Diff {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

pub fn execute_observability(args: ObservabilityArgs) -> Result<()> {
    let layout = get_layout()?;
    let storage = StorageManager::open_read_only(&layout.root)?;
    let cozo = storage
        .cozo
        .as_ref()
        .ok_or_else(|| miette::miette!("CozoDB not available"))?;
    match args.command {
        ObservabilitySubcommands::Coverage { json } => {
            let services_res = cozo.run_script(
                "?[svc_urn, service] := *node{id: svc_urn, label: service, category: 'service'}",
            )?;
            let slo_res = cozo.run_script("?[svc_urn, count(slo_urn)] := *edge{source: slo_urn, target: svc_urn, relation: 'monitors'}, *node{id: slo_urn, category: 'slo'}")?;
            let metric_res = cozo.run_script("?[svc_urn, count(m_urn)] := *edge{source: slo_urn, target: svc_urn, relation: 'monitors'}, *edge{source: slo_urn, target: m_urn, relation: 'depends_on'}, *node{id: m_urn, category: 'metric'}")?;

            let mut slo_map = std::collections::HashMap::new();
            for row in slo_res.rows {
                if let (
                    Some(cozo::DataValue::Str(svc_urn)),
                    Some(cozo::DataValue::Num(cozo::Num::Int(count))),
                ) = (row.first(), row.get(1))
                {
                    slo_map.insert(svc_urn.clone(), *count);
                }
            }

            let mut metric_map = std::collections::HashMap::new();
            for row in metric_res.rows {
                if let (
                    Some(cozo::DataValue::Str(svc_urn)),
                    Some(cozo::DataValue::Num(cozo::Num::Int(count))),
                ) = (row.first(), row.get(1))
                {
                    metric_map.insert(svc_urn.clone(), *count);
                }
            }

            let mut final_rows = Vec::new();
            for row in services_res.rows {
                if let (Some(cozo::DataValue::Str(svc_urn)), Some(cozo::DataValue::Str(service))) =
                    (row.first(), row.get(1))
                {
                    let slo_count = *slo_map.get(svc_urn).unwrap_or(&0);
                    let metric_count = *metric_map.get(svc_urn).unwrap_or(&0);
                    final_rows.push((service.clone(), slo_count, metric_count));
                }
            }

            if json {
                let mut results = Vec::new();
                for (svc, sc, mc) in &final_rows {
                    results.push(serde_json::json!({
                        "service": svc,
                        "slo_count": sc,
                        "metric_count": mc,
                    }));
                }
                println!(
                    "{}",
                    serde_json::to_string_pretty(&results).into_diagnostic()?
                );
            } else {
                println!("{}", "Observability Coverage Summary".bold().cyan());
                let mut table = Table::new();
                table.set_header(vec!["Service", "SLOs", "Metrics", "Health"]);

                for (svc, sc, mc) in &final_rows {
                    let health = if *sc > 0 {
                        "COVERED".green().to_string()
                    } else {
                        "MISSING".red().to_string()
                    };
                    table.add_row(vec![
                        svc.to_string(),
                        sc.to_string(),
                        mc.to_string(),
                        health,
                    ]);
                }
                println!("{}", table);
            }
        }
        ObservabilitySubcommands::Diff { json } => {
            // Identify changed observability files (YAML/YML in changed diff)
            // and surface which graph nodes (SLO, metric, alert) they map to.
            let packet = crate::commands::impact::execute_impact_silent()?;
            let changed_files: std::collections::HashSet<String> = packet
                .changes
                .iter()
                .map(|c| c.path.to_string_lossy().replace('\\', "/"))
                .collect();

            let cozo = storage
                .cozo
                .as_ref()
                .ok_or_else(|| miette::miette!("CozoDB not available"))?;

            // Query all observability graph nodes including metadata for source_file lookup
            let obs_res = cozo.run_script(
                "?[id, label, category, metadata] := *node{id, label, category, metadata}, \
                 category in ['slo', 'metric', 'alert', 'observability_signal']",
            )?;

            let mut changed = Vec::new();
            let mut unchanged = Vec::new();

            for row in obs_res.rows {
                if let (
                    Some(cozo::DataValue::Str(id)),
                    Some(cozo::DataValue::Str(label)),
                    Some(cozo::DataValue::Str(cat)),
                ) = (row.first(), row.get(1), row.get(2))
                {
                    // Match via source_file stored in metadata at index time.
                    // URN-based matching is unreliable because URNs use the entity name, not path.
                    let source_file: Option<String> = row.get(3).and_then(|v| {
                        if let cozo::DataValue::Json(j) = v {
                            j.get("source_file")
                                .and_then(|f| f.as_str())
                                .map(|s| s.replace('\\', "/"))
                        } else {
                            None
                        }
                    });
                    let is_changed = source_file
                        .as_deref()
                        .map(|sf| changed_files.contains(sf))
                        .unwrap_or(false);

                    let entry = serde_json::json!({
                        "id": id,
                        "label": label,
                        "category": cat,
                        "changed": is_changed,
                    });

                    if is_changed {
                        changed.push(entry);
                    } else {
                        unchanged.push(entry);
                    }
                }
            }

            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "changed": changed,
                        "unchanged_count": unchanged.len(),
                    }))
                    .into_diagnostic()?
                );
            } else {
                println!("{}", "Observability Diff".bold().cyan());
                println!("Changed files in diff: {}", changed_files.len());

                if changed.is_empty() {
                    println!(
                        "{}",
                        "No observability signals impacted by current diff.".dimmed()
                    );
                } else {
                    println!(
                        "\n{} observability signal(s) impacted:",
                        changed.len().to_string().yellow()
                    );
                    let mut table = Table::new();
                    table.set_header(vec!["Category", "Label", "ID"]);
                    for item in &changed {
                        table.add_row(vec![
                            item["category"].as_str().unwrap_or("").to_string(),
                            item["label"].as_str().unwrap_or("").to_string(),
                            item["id"].as_str().unwrap_or("").to_string(),
                        ]);
                    }
                    println!("{}", table);
                }
                println!(
                    "\n{} other observability signal(s) not impacted.",
                    unchanged.len()
                );
            }
        }
    }

    Ok(())
}
