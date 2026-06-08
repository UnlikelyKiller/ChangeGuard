use clap::Args;
use miette::{IntoDiagnostic, Result};
use crate::commands::helpers::get_layout;
use crate::state::storage::StorageManager;
use crate::output::table::Table;
use owo_colors::OwoColorize;

#[derive(Args, Debug)]
pub struct LedgerGraphArgs {
    /// Transaction ID (or prefix)
    pub tx_id: String,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

pub fn execute_ledger_graph(args: LedgerGraphArgs) -> Result<()> {
    let layout = get_layout()?;
    let storage = StorageManager::open_read_only(&layout.root)?;
    let cozo = storage.cozo.as_ref().ok_or_else(|| miette::miette!("CozoDB not available"))?;

    // Resolve prefix
    let db_path = layout.state_subdir().join("ledger.db");
    let mut sqlite_storage = StorageManager::init(db_path.as_std_path())?;
    let config = crate::config::load::load_config(&layout).unwrap_or_default();
    let manager = crate::ledger::transaction::TransactionManager::new(
        sqlite_storage.get_connection_mut(),
        layout.root.clone().into(),
        config,
    );
    let full_id = manager.resolve_tx_id(&args.tx_id).map_err(|e| miette::miette!("{}", e))?;

    let tx_urn = format!("urn:changeguard:transaction:{}", full_id);

    // Query Cozo for entities governed by this transaction
    let query = format!(
        "?[entity_id, label, category, relation] := *node{{id: entity_id, label: label, category: category}}, \
         *edge{{source: '{}', target: entity_id, relation: relation}}",
        tx_urn
    );

    let res = cozo.run_script(&query)?;

    if args.json {
        let mut results = Vec::new();
        for row in res.rows {
            if let (Some(cozo::DataValue::Str(id)), Some(cozo::DataValue::Str(label)), Some(cozo::DataValue::Str(cat)), Some(cozo::DataValue::Str(rel))) = 
                (row.get(0), row.get(1), row.get(2), row.get(3))
            {
                results.push(serde_json::json!({
                    "entity_id": id,
                    "label": label,
                    "category": cat,
                    "relation": rel,
                }));
            }
        }
        println!("{}", serde_json::to_string_pretty(&results).into_diagnostic()?);
    } else {
        println!("{} {}", "Graph neighborhood for transaction:".bold(), full_id.cyan());
        let mut table = Table::new();
        table.set_header(vec!["Entity ID", "Label", "Category", "Relation"]);

        for row in res.rows {
            if let (Some(cozo::DataValue::Str(id)), Some(cozo::DataValue::Str(label)), Some(cozo::DataValue::Str(cat)), Some(cozo::DataValue::Str(rel))) = 
                (row.get(0), row.get(1), row.get(2), row.get(3))
            {
                table.add_row(vec![
                    id.to_string(),
                    label.to_string(),
                    cat.to_string(),
                    rel.to_string(),
                ]);
            }
        }
        println!("{}", table);
    }

    Ok(())
}
