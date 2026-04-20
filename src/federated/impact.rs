use crate::federated::schema::FederatedSchema;
use crate::federated::storage::get_federated_links;
use crate::impact::packet::ImpactPacket;
use crate::state::storage::StorageManager;
use miette::{IntoDiagnostic, Result};
use std::collections::HashMap;
use std::fs;

pub fn check_cross_repo_impact(packet: &mut ImpactPacket, storage: &StorageManager) -> Result<()> {
    let links = get_federated_links(storage.get_connection())?;
    if links.is_empty() {
        return Ok(());
    }

    let mut sibling_schemas = HashMap::new();
    for (name, path, _) in links {
        let schema_path = std::path::Path::new(&path)
            .join(".changeguard")
            .join("schema.json");
        if schema_path.exists() {
            let content = fs::read_to_string(schema_path).into_diagnostic()?;
            if let Ok(schema) = serde_json::from_str::<FederatedSchema>(&content) {
                sibling_schemas.insert(name, schema);
            }
        }
    }

    // Engineering Standard: We currently only check if a local file depends on a sibling interface
    // based on our stored federated_dependencies table.
    // This requires a discovery phase during 'impact' or a background task.
    // For Track 28, we'll implement a simple "Changed Sibling" warning.

    for (name, _schema) in sibling_schemas {
        // In a real implementation, we would compare the current sibling schema
        // with the one we had when we last recorded federated dependencies.
        // If the sibling schema changed (e.g. a symbol removed or type changed),
        // we find which local symbols depend on it.

        // For the current scope, let's add a generic diagnostic if a sibling is known.
        packet.risk_reasons.push(format!(
            "Cross-repo monitoring active for sibling: {}. Run 'changeguard federate scan' to refresh.",
            name
        ));
    }

    Ok(())
}
