use crate::commands::helpers::get_layout;
use crate::output::table::Table;
use crate::state::storage::StorageManager;
use clap::{Args, Subcommand};
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

#[derive(Args, Debug)]
pub struct DeployArgs {
    #[command(subcommand)]
    pub command: DeploySubcommands,
}

#[derive(Subcommand, Debug)]
pub enum DeploySubcommands {
    /// Show impact of changes on deployment manifests
    Impact {
        /// Filter by changed manifests only
        #[arg(long)]
        changed: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

pub fn execute_deploy(args: DeployArgs) -> Result<()> {
    let layout = get_layout()?;
    let storage = StorageManager::open_read_only(&layout.root)?;
    let conn = storage.get_connection();

    match args.command {
        DeploySubcommands::Impact { changed, json } => {
            let changed_files: Option<std::collections::HashSet<String>> = if changed {
                let packet = crate::commands::impact::execute_impact_silent()?;
                let set = packet
                    .changes
                    .iter()
                    .map(|c| c.path.to_string_lossy().replace('\\', "/"))
                    .collect();
                Some(set)
            } else {
                None
            };

            let mut stmt = conn
                .prepare(
                    "SELECT file_path, manifest_type, risk_tier, service_name, owner FROM deploy_manifests",
                )
                .into_diagnostic()?;

            #[allow(clippy::type_complexity)]
            let all_rows: Vec<(String, String, i32, Option<String>, Option<String>)> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i32>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                    ))
                })
                .into_diagnostic()?
                .collect::<std::result::Result<Vec<_>, _>>()
                .into_diagnostic()?;

            let rows: Vec<_> = if let Some(ref cf) = changed_files {
                all_rows
                    .into_iter()
                    .filter(|(fp, _, _, _, _)| cf.contains(&fp.replace('\\', "/")))
                    .collect()
            } else {
                all_rows
            };

            if !json && rows.is_empty() {
                println!(
                    "  {}",
                    "No deployment impact detected for current changes.".yellow()
                );
                return Ok(());
            }

            if json {
                let mut results = Vec::new();
                for (path, mtype, risk, service, owner) in &rows {
                    results.push(serde_json::json!({
                        "path": path,
                        "type": mtype,
                        "risk_tier": risk,
                        "service": service,
                        "owner": owner,
                    }));
                }
                println!(
                    "{}",
                    serde_json::to_string_pretty(&results).into_diagnostic()?
                );
            } else {
                println!("{}", "Deployment Manifest Impact".bold().cyan());
                let mut table = Table::new();
                table.set_header(vec!["Manifest", "Type", "Risk", "Service", "Owner"]);

                for (path, mtype, risk, service, owner) in &rows {
                    let risk_str = match risk {
                        3 => risk.to_string().red().to_string(),
                        2 => risk.to_string().yellow().to_string(),
                        _ => risk.to_string().green().to_string(),
                    };

                    table.add_row(vec![
                        path.clone(),
                        mtype.clone(),
                        risk_str,
                        service.clone().unwrap_or_else(|| "-".to_string()),
                        owner.clone().unwrap_or_else(|| "-".to_string()),
                    ]);
                }
                println!("{}", table);
            }
        }
    }

    Ok(())
}

#[derive(Args, Debug)]
pub struct CiArgs {
    #[command(subcommand)]
    pub command: CiSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum CiSubcommands {
    /// Show differences in CI configuration and gates
    Diff {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

pub fn execute_ci(args: CiArgs) -> Result<()> {
    let layout = get_layout()?;
    let storage = StorageManager::open_read_only(&layout.root)?;
    let conn = storage.get_connection();

    match args.command {
        CiSubcommands::Diff { json } => {
            let mut stmt = conn
                .prepare("SELECT platform, job_name, workflow_name, environment FROM ci_gates")
                .into_diagnostic()?;

            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                    ))
                })
                .into_diagnostic()?;

            if json {
                let mut results = Vec::new();
                for row in rows {
                    let (plat, job, workflow, env) = row.into_diagnostic()?;
                    results.push(serde_json::json!({
                        "platform": plat,
                        "job": job,
                        "workflow": workflow,
                        "environment": env,
                    }));
                }
                println!(
                    "{}",
                    serde_json::to_string_pretty(&results).into_diagnostic()?
                );
            } else {
                println!("{}", "CI Gate Summary".bold().cyan());
                let mut table = Table::new();
                table.set_header(vec!["Platform", "Job", "Workflow", "Environment"]);

                for row in rows {
                    let (plat, job, workflow, env) = row.into_diagnostic()?;
                    table.add_row(vec![
                        plat,
                        job,
                        workflow.unwrap_or_else(|| "-".to_string()),
                        env.unwrap_or_else(|| "-".to_string()),
                    ]);
                }
                println!("{}", table);
            }
        }
    }

    Ok(())
}
