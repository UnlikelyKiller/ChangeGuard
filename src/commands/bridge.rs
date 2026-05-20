use crate::cli::BridgeCommands;
use miette::Result;

pub fn execute(subcommand: BridgeCommands) -> Result<()> {
    match subcommand {
        BridgeCommands::Export {
            out,
            hotspots,
            targets,
            scope,
            ledger,
            madr,
        } => crate::bridge::export::execute_export(out, hotspots, targets, scope, ledger, madr),
        BridgeCommands::Import { from, input } => {
            let path = from.or(input).ok_or_else(|| {
                miette::miette!("Either --from or --in must be provided for bridge import.")
            })?;
            crate::bridge::import::execute_import(path)
        }
        BridgeCommands::Query { query } => crate::bridge::client::execute_query(query),
        BridgeCommands::Verify { scope, out } => {
            crate::bridge::export::execute_verify(scope, out)
        }
    }
}
