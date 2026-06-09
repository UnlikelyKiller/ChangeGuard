use crate::commands::helpers::get_layout;
use crate::output::table::Table;
use crate::state::storage::StorageManager;
use clap::{Args, Subcommand};
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::Serialize;

#[derive(Args, Debug)]
pub struct DependenciesArgs {
    #[command(subcommand)]
    pub command: DependencySubcommands,
}

#[derive(Subcommand, Debug)]
pub enum DependencySubcommands {
    /// List all project dependencies and their versions
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Audit dependencies for known vulnerabilities (requires OSV-Scanner JSON)
    Audit {
        /// Path to OSV-Scanner JSON output
        #[arg(short, long)]
        input: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

pub fn execute_dependencies(args: DependenciesArgs) -> Result<()> {
    let layout = get_layout()?;

    match args.command {
        DependencySubcommands::List { json } => {
            let storage = StorageManager::open_read_only(&layout.root)?;
            let cozo = storage
                .cozo
                .as_ref()
                .ok_or_else(|| miette::miette!("CozoDB storage is unavailable"))?;
            let res = cozo.run_script("?[id, name, metadata] := *node{id: id, label: name, category: 'package', metadata: metadata}")?;

            #[derive(Serialize)]
            struct ListedDep {
                name: String,
                version: String,
                ecosystem: String,
            }

            let mut deps = Vec::new();
            for row in res.rows {
                if let (
                    Some(cozo::DataValue::Str(_id)),
                    Some(cozo::DataValue::Str(name)),
                    Some(cozo::DataValue::Json(meta)),
                ) = (row.first(), row.get(1), row.get(2))
                {
                    let version = meta
                        .get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("-")
                        .to_string();
                    let ecosystem = meta
                        .get("ecosystem")
                        .and_then(|v| v.as_str())
                        .unwrap_or("-")
                        .to_string();
                    deps.push(ListedDep {
                        name: name.to_string(),
                        version,
                        ecosystem,
                    });
                }
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&deps).into_diagnostic()?);
            } else {
                println!(
                    "{}",
                    "Project Dependencies (from Knowledge Graph)".bold().green()
                );
                let mut table = Table::new();
                table.set_header(vec!["Package", "Version", "Ecosystem"]);
                for dep in deps {
                    table.add_row(vec![dep.name, dep.version, dep.ecosystem]);
                }
                println!("{}", table);
            }
        }
        DependencySubcommands::Audit { input, json } => {
            let path = std::path::Path::new(&input);
            if !path.exists() {
                return Err(miette::miette!("Input file not found: {}", input));
            }

            let result = crate::index::advisories::OsvImporter::import_from_json(path)?;

            // Open writeable storage to populate KG
            let db_path = layout.state_subdir().join("ledger.db");
            let storage = StorageManager::init(db_path.as_std_path())?;
            if let Some(cozo) = &storage.cozo {
                crate::index::advisories::OsvImporter::populate_kg(cozo, &result, "audit-tx")?;
            }

            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result).into_diagnostic()?
                );
            } else {
                println!("{}", "Security Advisory Audit (OSV)".bold().red());
                let mut table = Table::new();
                table.set_header(vec!["Package", "Version", "Vulnerability", "Summary"]);

                for src_res in &result.results {
                    for pkg_res in &src_res.packages {
                        if let Some(vulns) = &pkg_res.vulnerabilities {
                            for vuln in vulns {
                                table.add_row(vec![
                                    pkg_res.package.name.clone(),
                                    pkg_res.package.version.clone(),
                                    vuln.id.red().to_string(),
                                    vuln.summary.as_deref().unwrap_or("-").to_string(),
                                ]);
                            }
                        }
                    }
                }
                println!("{}", table);
            }
        }
    }

    Ok(())
}
