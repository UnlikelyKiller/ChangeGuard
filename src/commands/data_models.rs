use crate::commands::helpers::get_layout;
use crate::output::table::Table;
use crate::state::storage::StorageManager;
use clap::{Args, Subcommand};
use miette::{IntoDiagnostic, Result};
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

    match args.command {
        DataModelSubcommands::List { json } => {
            let storage = StorageManager::open_read_only(&layout.root)?;
            let conn = storage.get_connection();

            let mut stmt = conn
                .prepare("SELECT model_name, language, model_kind, confidence FROM data_models")
                .into_diagnostic()?;

            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, f64>(3)?,
                    ))
                })
                .into_diagnostic()?;

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
                println!(
                    "{}",
                    serde_json::to_string_pretty(&results).into_diagnostic()?
                );
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
            let packet = crate::commands::impact::execute_impact_silent()?;

            let storage = StorageManager::open_read_only(&layout.root)?;
            let conn = storage.get_connection();

            // Collect the files that changed
            let changed_files: std::collections::HashSet<String> = packet
                .changes
                .iter()
                .map(|c| c.path.to_string_lossy().replace('\\', "/"))
                .collect();

            // Now query data models and see which ones are in changed files
            let mut stmt = conn
                .prepare(
                    "SELECT dm.model_name, pf.file_path, dm.language, dm.model_kind, dm.confidence \
                 FROM data_models dm \
                 JOIN project_files pf ON dm.model_file_id = pf.id",
                )
                .into_diagnostic()?;

            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, f64>(4)?,
                    ))
                })
                .into_diagnostic()?;

            let mut impacted = Vec::new();
            for row in rows {
                let (name, file_path, lang, kind, conf) = row.into_diagnostic()?;
                let file_path_norm = file_path.replace('\\', "/");
                let is_impacted = changed_files.contains(&file_path_norm);

                if !changed || is_impacted {
                    impacted.push(serde_json::json!({
                        "name": name,
                        "file_path": file_path_norm,
                        "language": lang,
                        "kind": kind,
                        "confidence": conf,
                        "is_changed": is_impacted,
                    }));
                }
            }

            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&impacted).into_diagnostic()?
                );
            } else {
                println!("{}", "Data Model Impact Analysis".bold().cyan());
                let mut table = Table::new();
                table.set_header(vec!["Name", "File", "Language", "Kind", "Changed?"]);
                if impacted.is_empty() {
                    let total_models: i64 = conn
                        .query_row("SELECT COUNT(*) FROM data_models", [], |row| row.get(0))
                        .into_diagnostic()?;

                    if total_models > 0 && changed {
                        println!("{}", "  No changed data models found.".dimmed());
                    } else {
                        println!(
                            "{}",
                            "  No data models indexed. Data models are extracted from ORM structs, \
                             SQL table definitions, and migration files. Run `changeguard index \
                             --incremental` if models exist, or confirm your ORM/framework is supported."
                                .dimmed()
                        );
                    }
                } else {
                    for item in impacted {
                        table.add_row(vec![
                            item["name"].as_str().unwrap_or("").bold().to_string(),
                            item["file_path"].as_str().unwrap_or("").to_string(),
                            item["language"].as_str().unwrap_or("").to_string(),
                            item["kind"].as_str().unwrap_or("").to_string(),
                            if item["is_changed"].as_bool().unwrap_or(false) {
                                "YES".red().bold().to_string()
                            } else {
                                "NO".dimmed().to_string()
                            },
                        ]);
                    }
                    println!("{}", table);
                }
            }
        }
    }

    Ok(())
}
