use clap::{Args, Subcommand};
use miette::{IntoDiagnostic, Result};
use crate::commands::helpers::get_layout;
use crate::state::storage::StorageManager;
use crate::output::table::Table;
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
    let storage = StorageManager::open_read_only(&layout.root)?;
    let cozo = storage.cozo.as_ref().ok_or_else(|| miette::miette!("CozoDB not available"))?;

    match args.command {
        SecuritySubcommands::Impact { changed: _, json } => {
            // Query Cozo for changed nodes that are security-related
            println!("Security impact analysis is coming soon.");
            if json { /* stub */ }
        }
        SecuritySubcommands::Boundaries { json } => {
            let query = "
                ?[id, label, category] := *node{id, label, category}, \
                 category in ['policy', 'principal', 'action', 'resource']
            ";

            let res = cozo.run_script(query)?;

            if json {
                let mut results = Vec::new();
                for row in res.rows {
                    if let (Some(cozo::DataValue::Str(id)), Some(cozo::DataValue::Str(label)), Some(cozo::DataValue::Str(cat))) = 
                        (row.get(0), row.get(1), row.get(2))
                    {
                        results.push(serde_json::json!({
                            "id": id,
                            "label": label,
                            "category": cat,
                        }));
                    }
                }
                println!("{}", serde_json::to_string_pretty(&results).into_diagnostic()?);
            } else {
                println!("{}", "Security Boundaries & Policies".bold().red());
                let mut table = Table::new();
                table.set_header(vec!["ID", "Label", "Category"]);

                for row in res.rows {
                    if let (Some(cozo::DataValue::Str(id)), Some(cozo::DataValue::Str(label)), Some(cozo::DataValue::Str(cat))) = 
                        (row.get(0), row.get(1), row.get(2))
                    {
                        table.add_row(vec![
                            id.to_string(),
                            label.to_string(),
                            cat.to_string(),
                        ]);
                    }
                }
                println!("{}", table);
            }
        }
    }

    Ok(())
}
