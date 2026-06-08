use clap::{Args, Subcommand};
use miette::{IntoDiagnostic, Result};
use crate::commands::helpers::get_layout;
use crate::state::storage::StorageManager;
use crate::output::table::Table;
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
        DeploySubcommands::Impact { changed: _, json } => {
            let mut stmt = conn.prepare(
                "SELECT file_path, manifest_type, risk_tier, service_name, owner FROM deploy_manifests"
            ).into_diagnostic()?;
            
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i32>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            }).into_diagnostic()?;

            if json {
                let mut results = Vec::new();
                for row in rows {
                    let (path, mtype, risk, service, owner) = row.into_diagnostic()?;
                    results.push(serde_json::json!({
                        "path": path,
                        "type": mtype,
                        "risk_tier": risk,
                        "service": service,
                        "owner": owner,
                    }));
                }
                println!("{}", serde_json::to_string_pretty(&results).into_diagnostic()?);
            } else {
                println!("{}", "Deployment Manifest Impact".bold().cyan());
                let mut table = Table::new();
                table.set_header(vec!["Manifest", "Type", "Risk", "Service", "Owner"]);
                
                for row in rows {
                    let (path, mtype, risk, service, owner) = row.into_diagnostic()?;
                    let risk_str = match risk {
                        3 => risk.to_string().red().to_string(),
                        2 => risk.to_string().yellow().to_string(),
                        _ => risk.to_string().green().to_string(),
                    };

                    table.add_row(vec![
                        path,
                        mtype,
                        risk_str,
                        service.unwrap_or_else(|| "-".to_string()),
                        owner.unwrap_or_else(|| "-".to_string()),
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
            let mut stmt = conn.prepare(
                "SELECT platform, job_name, workflow_name, environment FROM ci_gates"
            ).into_diagnostic()?;
            
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            }).into_diagnostic()?;

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
                println!("{}", serde_json::to_string_pretty(&results).into_diagnostic()?);
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
