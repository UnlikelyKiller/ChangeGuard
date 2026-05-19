use crate::cli::BridgeCommands;
use miette::Result;

pub fn execute(subcommand: BridgeCommands) -> Result<()> {
    match subcommand {
        BridgeCommands::Export { out } => super::super::bridge::export::execute_export(out),
        BridgeCommands::Import { input } => super::super::bridge::import::execute_import(input),
        BridgeCommands::Query { query } => super::super::bridge::client::execute_query(query),
    }
}
