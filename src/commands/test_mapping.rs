use crate::commands::helpers::get_layout;
use crate::output::table::Table;
use crate::state::storage::StorageManager;
use clap::Args;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

#[derive(Args, Debug)]
pub struct TestsForEntityArgs {
    /// Entity ID (URN, path, or symbol name)
    pub entity: String,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

pub fn execute_tests_for_entity(args: TestsForEntityArgs) -> Result<()> {
    let layout = get_layout()?;
    let storage = StorageManager::open_read_only(&layout.root)?;
    let cozo = storage
        .cozo
        .as_ref()
        .ok_or_else(|| miette::miette!("CozoDB not available"))?;

    // Try to resolve entity to URN if it's not already one
    let entity_urn = if args.entity.starts_with("urn:") {
        args.entity.clone()
    } else {
        // Guess kind based on input
        if args.entity.contains('/') || args.entity.ends_with(".rs") {
            crate::platform::urn::build_urn(crate::state::graph_kinds::NodeKind::File, &args.entity)
        } else {
            crate::platform::urn::build_urn(
                crate::state::graph_kinds::NodeKind::Symbol,
                &args.entity,
            )
        }
    };

    let query = "?[test_id, label, confidence, evidence] := *node{id: test_id, label: label, category: 'test', metadata: meta}, \
                 *edge{source: test_id, target: $entity, relation: 'validates'}, \
                 confidence = get(meta, 'confidence'), \
                 evidence = get(meta, 'evidence')";

    let mut params = std::collections::BTreeMap::new();
    params.insert(
        "entity".to_string(),
        cozo::DataValue::Str(entity_urn.clone().into()),
    );
    let res = cozo.run_script_with_params(query, params, cozo::ScriptMutability::Immutable)?;

    if !args.json && res.rows.is_empty() {
        println!(
            "  {}",
            format!("No test mappings found for '{}'.", args.entity).yellow()
        );
        println!(
            "  Run {} to populate test mappings.",
            "changeguard index".cyan().bold()
        );
        return Ok(());
    }

    if args.json {
        let mut results = Vec::new();
        for row in res.rows {
            if let (
                Some(cozo::DataValue::Str(id)),
                Some(cozo::DataValue::Str(label)),
                Some(conf),
                Some(ev),
            ) = (row.first(), row.get(1), row.get(2), row.get(3))
            {
                results.push(serde_json::json!({
                    "test_id": id,
                    "label": label,
                    "confidence": conf,
                    "evidence": ev,
                }));
            }
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&results).into_diagnostic()?
        );
    } else {
        println!("{} {}", "Tests validating".bold(), entity_urn.cyan());
        let mut table = Table::new();
        table.set_header(vec!["Test URN", "Label", "Confidence", "Evidence"]);

        for row in res.rows {
            if let (
                Some(cozo::DataValue::Str(id)),
                Some(cozo::DataValue::Str(label)),
                Some(cozo::DataValue::Num(cozo::Num::Float(conf))),
                Some(cozo::DataValue::Str(ev)),
            ) = (row.first(), row.get(1), row.get(2), row.get(3))
            {
                table.add_row(vec![
                    id.to_string(),
                    label.to_string(),
                    format!("{:.2}", conf),
                    ev.to_string(),
                ]);
            }
        }
        println!("{}", table);
    }

    Ok(())
}
