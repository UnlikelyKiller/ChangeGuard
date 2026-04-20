use crate::federated::schema::FederatedSchema;
use crate::federated::storage::{get_dependencies_for_sibling, get_federated_links};
use crate::impact::packet::ImpactPacket;
use crate::state::storage::StorageManager;
use miette::Result;
use std::fs;
use std::panic;

pub fn check_cross_repo_impact(packet: &mut ImpactPacket, storage: &StorageManager) -> Result<()> {
    let links = get_federated_links(storage.get_connection())?;
    if links.is_empty() {
        return Ok(());
    }

    let mut impact_reasons = Vec::new();

    for (name, path, _) in links {
        let schema_path = std::path::Path::new(&path)
            .join(".changeguard")
            .join("schema.json");

        if !schema_path.exists() {
            impact_reasons.push(format!(
                "Cross-repo impact: Sibling '{}' schema is unavailable or invalid.",
                name
            ));
            continue;
        }

        let content = match fs::read_to_string(&schema_path) {
            Ok(c) => c,
            Err(_) => {
                impact_reasons.push(format!(
                    "Cross-repo impact: Sibling '{}' schema is unavailable or invalid.",
                    name
                ));
                continue;
            }
        };

        // JSON Safety: Wrap in catch_unwind
        let schema_result = panic::catch_unwind(|| serde_json::from_str::<FederatedSchema>(&content));

        let schema = match schema_result {
            Ok(Ok(s)) => s,
            _ => {
                impact_reasons.push(format!(
                    "Cross-repo impact: Sibling '{}' schema is unavailable or invalid.",
                    name
                ));
                continue;
            }
        };

        if schema.validate().is_err() {
            impact_reasons.push(format!(
                "Cross-repo impact: Sibling '{}' schema is unavailable or invalid.",
                name
            ));
            continue;
        }

        let dependencies = get_dependencies_for_sibling(storage.get_connection(), &name)?;

        for (local_symbol, sibling_symbol) in dependencies {
            let exists = schema
                .public_interfaces
                .iter()
                .any(|i| i.symbol == sibling_symbol);
            if !exists {
                impact_reasons.push(format!(
                    "Cross-repo impact: Local symbol '{}' depends on sibling '{}' interface '{}' which was removed.",
                    local_symbol, name, sibling_symbol
                ));
            }
        }
    }

    // Engineering standard: deterministic sorting
    impact_reasons.sort();
    impact_reasons.dedup();
    packet.risk_reasons.extend(impact_reasons);

    Ok(())
}
