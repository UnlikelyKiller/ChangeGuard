use crate::bridge::export::ExportArgs;
use clap::Subcommand;
use miette::Result;

#[derive(Subcommand, Debug)]
pub enum BridgeCommands {
    /// Export ChangeGuard state for AI-Brains
    Export {
        /// Output path
        #[arg(long, short)]
        out: Option<String>,
        /// Include hotspots
        #[arg(long)]
        hotspots: bool,
        /// Include ledger entries
        #[arg(long)]
        ledger: bool,
        /// Path scope for hotspots
        #[arg(long)]
        scope: Option<String>,
        /// Export structured MADR fields
        #[arg(long)]
        madr: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Import AI-Brains insights into ChangeGuard
    Import {
        /// Input path (NDJSON)
        #[arg(long, short)]
        input: String,
    },
    /// Query AI-Brains for context
    Query {
        /// Query string
        query: String,
    },
}

pub fn execute(command: BridgeCommands) -> Result<()> {
    match command {
        BridgeCommands::Export {
            out,
            hotspots,
            ledger,
            scope,
            madr,
            json,
        } => {
            let scope_vec = scope.map(|s| s.split(',').map(|p| p.trim().to_string()).collect());
            let args = ExportArgs {
                out_path: out,
                hotspots,
                ledger,
                scope: scope_vec,
                madr,
                json,
            };
            crate::bridge::export::execute_export(args)
        }
        BridgeCommands::Import { input } => crate::bridge::import::execute_import(input),
        BridgeCommands::Query { query } => crate::bridge::client::execute_query(query),
    }
}
