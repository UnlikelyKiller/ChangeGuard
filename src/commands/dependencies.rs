use clap::{Args, Subcommand};
use miette::{IntoDiagnostic, Result};
use crate::commands::helpers::get_layout;
use crate::state::storage::StorageManager;
use crate::output::table::Table;
use owo_colors::OwoColorize;

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
    let _storage = StorageManager::open_read_only(&layout.root)?;

    match args.command {
        DependencySubcommands::List { json } => {
            // Simplified: read from Cargo.lock if exists, or query KG
            println!("Dependency listing is coming soon.");
            if json { /* stub */ }
        }
        DependencySubcommands::Audit { input, json } => {
            let path = std::path::Path::new(&input);
            if !path.exists() {
                return Err(miette::miette!("Input file not found: {}", input));
            }

            let result = crate::index::advisories::OsvImporter::import_from_json(path)?;
            
            if json {
                println!("{}", serde_json::to_string_pretty(&result).into_diagnostic()?);
            } else {
                println!("{}", "Security Advisory Audit (OSV)".bold().red());
                let mut table = Table::new();
                table.set_header(vec!["Package", "Version", "Vulnerability", "Summary"]);

                for pkg_res in result.results {
                    if let Some(vulns) = pkg_res.vulnerabilities {
                        for vuln in vulns {
                            table.add_row(vec![
                                pkg_res.package.name.clone(),
                                pkg_res.package.version.clone(),
                                vuln.id.red().to_string(),
                                vuln.summary.unwrap_or_else(|| "-".to_string()),
                            ]);
                        }
                    }
                }
                println!("{}", table);
            }
        }
    }

    Ok(())
}
