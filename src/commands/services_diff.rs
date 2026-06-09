use crate::commands::helpers::get_layout;
use crate::output::table::Table;
use crate::state::storage::StorageManager;
use clap::Args;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

#[derive(Args, Debug)]
pub struct ServicesDiffArgs {
    /// Show full topology
    #[arg(short, long)]
    pub full: bool,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

pub fn execute_services_diff(
    args: ServicesDiffArgs,
    config: &crate::config::model::Config,
) -> Result<()> {
    let layout = get_layout()?;
    let storage = StorageManager::open_read_only(&layout.root)?;
    let conn = storage.get_connection();

    // Query for services and their boundaries
    let mut stmt = conn
        .prepare(
            "SELECT pf.service_name, count(pf.id), count(ar.id)
         FROM project_files pf
         LEFT JOIN api_routes ar ON pf.id = ar.handler_file_id
         WHERE pf.service_name IS NOT NULL
         GROUP BY pf.service_name",
        )
        .into_diagnostic()?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .into_diagnostic()?;

    if args.json {
        let mut results = Vec::new();
        for row in rows {
            let (name, files, routes) = row.into_diagnostic()?;
            results.push(serde_json::json!({
                "service": name,
                "file_count": files,
                "route_count": routes,
            }));
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&results).into_diagnostic()?
        );
    } else {
        println!("{}", "Service Boundary Summary".bold().cyan());
        let mut table = Table::new();
        table.set_header(vec!["Service", "Files", "Endpoints", "Status"]);

        for row in rows {
            let (name, files, routes) = row.into_diagnostic()?;

            // Check if declared in config
            let is_declared = config.services.definitions.iter().any(|d| d.name == name);
            let status = if is_declared {
                "Declared".green().to_string()
            } else {
                "Inferred".yellow().to_string()
            };

            table.add_row(vec![
                name.bold().to_string(),
                files.to_string(),
                routes.to_string(),
                status,
            ]);
        }
        println!("{}", table);
    }

    Ok(())
}
