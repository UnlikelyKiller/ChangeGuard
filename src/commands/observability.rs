use clap::{Args, Subcommand};
use miette::{IntoDiagnostic, Result};
use crate::commands::helpers::get_layout;
use crate::state::storage::StorageManager;
use crate::output::table::Table;
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
}

pub fn execute_observability(args: ObservabilityArgs) -> Result<()> {
    let layout = get_layout()?;
    let storage = StorageManager::open_read_only(&layout.root)?;
    let cozo = storage.cozo.as_ref().ok_or_else(|| miette::miette!("CozoDB not available"))?;

    match args.command {
        ObservabilitySubcommands::Coverage { json } => {
            // Query Cozo for services and their linked SLOs/Metrics
            let query = "
                ?[service, slo_count, metric_count] := *node{id: svc_urn, label: service, category: 'service'}, \
                 slo_count = count(slo_urn) { *edge{source: slo_urn, target: svc_urn, relation: 'monitors'}, *node{id: slo_urn, category: 'slo'} }, \
                 metric_count = count(m_urn) { *edge{source: slo_urn, target: svc_urn, relation: 'monitors'}, *edge{source: slo_urn, target: m_urn, relation: 'depends_on'}, *node{id: m_urn, category: 'metric'} }
            ";

            let res = cozo.run_script(query)?;

            if json {
                let mut results = Vec::new();
                for row in res.rows {
                    if let (Some(cozo::DataValue::Str(svc)), Some(cozo::DataValue::Num(cozo::Num::Int(sc))), Some(cozo::DataValue::Num(cozo::Num::Int(mc)))) = 
                        (row.get(0), row.get(1), row.get(2))
                    {
                        results.push(serde_json::json!({
                            "service": svc,
                            "slo_count": sc,
                            "metric_count": mc,
                        }));
                    }
                }
                println!("{}", serde_json::to_string_pretty(&results).into_diagnostic()?);
            } else {
                println!("{}", "Observability Coverage Summary".bold().cyan());
                let mut table = Table::new();
                table.set_header(vec!["Service", "SLOs", "Metrics", "Health"]);

                for row in res.rows {
                    if let (Some(cozo::DataValue::Str(svc)), Some(cozo::DataValue::Num(cozo::Num::Int(sc))), Some(cozo::DataValue::Num(cozo::Num::Int(mc)))) = 
                        (row.get(0), row.get(1), row.get(2))
                    {
                        let health = if *sc > 0 { "COVERED".green().to_string() } else { "MISSING".red().to_string() };
                        table.add_row(vec![
                            svc.to_string(),
                            sc.to_string(),
                            mc.to_string(),
                            health,
                        ]);
                    }
                }
                println!("{}", table);
            }
        }
    }

    Ok(())
}
