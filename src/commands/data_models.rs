use clap::{Args, Subcommand};
use miette::{IntoDiagnostic, Result};
use crate::commands::helpers::get_layout;
use crate::state::storage::StorageManager;
use crate::output::table::Table;
use owo_colors::OwoColorize;

#[derive(Args, Debug)]
pub struct DataModelsArgs {
    #[command(subcommand)]
    pub command: DataModelSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum DataModelSubcommands {
    /// List all extracted data models and their mapping to tables
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show impact of changes on data models
    Impact {
        /// Filter by changed models only
        #[arg(long)]
        changed: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

pub fn execute_data_models(args: DataModelsArgs) -> Result<()> {
    let layout = get_layout()?;
    let storage = StorageManager::open_read_only(&layout.root)?;
    let conn = storage.get_connection();

    match args.command {
        DataModelSubcommands::List { json } => {
            let mut stmt = conn.prepare(
                "SELECT model_name, language, model_kind, confidence FROM data_models"
            ).into_diagnostic()?;
            
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, f64>(3)?,
                ))
            }).into_diagnostic()?;

            if json {
                let mut results = Vec::new();
                for row in rows {
                    let (name, lang, kind, conf) = row.into_diagnostic()?;
                    results.push(serde_json::json!({
                        "name": name,
                        "language": lang,
                        "kind": kind,
                        "confidence": conf,
                    }));
                }
                println!("{}", serde_json::to_string_pretty(&results).into_diagnostic()?);
            } else {
                println!("{}", "Data Models".bold().cyan());
                let mut table = Table::new();
                table.set_header(vec!["Name", "Language", "Kind", "Confidence"]);
                
                for row in rows {
                    let (name, lang, kind, conf) = row.into_diagnostic()?;
                    table.add_row(vec![
                        name.bold().to_string(),
                        lang,
                        kind,
                        format!("{:.2}", conf),
                    ]);
                }
                println!("{}", table);
            }
        }
        DataModelSubcommands::Impact { changed, json } => {
            // Implementation for data model impact
            // This would query Cozo for changed nodes with DataModel kind
            println!("Data model impact analysis is coming soon.");
            if changed || json { /* stubs */ }
        }
    }

    Ok(())
}
