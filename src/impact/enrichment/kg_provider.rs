use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::{ImpactPacket, KGImpact};
use miette::Result;
use tracing::{debug, info, warn};

pub struct KGProvider;

impl EnrichmentProvider for KGProvider {
    fn name(&self) -> &'static str {
        "KnowledgeGraph"
    }

    fn enrich(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        let Some(cozo) = &context.storage.cozo else {
            debug!("CozoStorage not available, skipping KG enrichment");
            return Ok(());
        };

        info!("Enriching impact packet with Knowledge Graph data...");

        // 1. Sync Hotspots to KG risk scores
        if !packet.hotspots.is_empty() {
            let mut risk_updates = Vec::new();
            for hotspot in &packet.hotspots {
                risk_updates.push(vec![
                    cozo::DataValue::Str(hotspot.path.to_string_lossy().to_string().into()),
                    cozo::DataValue::Num(cozo::Num::Float(hotspot.score as f64)),
                ]);
            }

            let risk_json =
                serde_json::to_string(&risk_updates).unwrap_or_else(|_| "[]".to_string());
            let sync_script = format!(
                "updates[id, score] <- {}\n?[id, label, category, risk_score, metadata] := *node{{id, label, category, metadata}}, updates[id, risk_score]\n:put node",
                risk_json
            );
            if let Err(e) = cozo.run_script(&sync_script) {
                warn!("Failed to sync hotspots to KG: {}", e);
            } else {
                debug!("Synced {} hotspots to KG", risk_updates.len());
            }

            // 1.1 Simple propagation (1-hop)
            let propagation_script = "
                diffused[id, s] := *node{id: src, risk_score: src_s}, *edge{source: src, target: id}, s = src_s * 0.5
                ?[id, label, category, risk_score, metadata] := *node{id, label, category, risk_score: current, metadata}, diffused[id, s], s > current, risk_score = s
                :put node
            ";
            if let Err(e) = cozo.run_script(propagation_script) {
                warn!("Failed to propagate risk in KG: {}", e);
            }
        }

        // 2. Identify changed files/symbols in KG
        let mut seed_nodes: Vec<Vec<String>> = Vec::new();
        for file in &packet.changes {
            // Find nodes associated with this file
            let file_path = file.path.to_string_lossy();
            let query = format!(
                "?[id] := *project_symbol{{file_path: '{}', id: symbol_id}}, *node{{id: id}}, *ledger_link{{node_id: id, ledger_id: _}}",
                file_path
            );

            // Wait, I should also check 'node' directly if I used file_path as ID for file nodes
            let query_file = format!("?[id] := *node{{id: id, label: '{}'}}", file_path);

            if let Ok(res) = cozo.run_script(&query) {
                for row in res.rows {
                    if let Some(cozo::DataValue::Str(id)) = row.first() {
                        seed_nodes.push(vec![id.to_string()]);
                    }
                }
            }
            if let Ok(res) = cozo.run_script(&query_file) {
                for row in res.rows {
                    if let Some(cozo::DataValue::Str(id)) = row.first() {
                        println!("Found seed node: {}", id);
                        seed_nodes.push(vec![id.to_string()]);
                    }
                }
            }
        }

        println!("Seed nodes: {:?}", seed_nodes);

        if seed_nodes.is_empty() {
            debug!("No seed nodes found in KG for changes");
            return Ok(());
        }

        // 2. Perform reachability analysis (up to 2 hops for now)
        let seed_list = serde_json::to_string(&seed_nodes).unwrap_or_else(|_| "[]".to_string());
        let queries = vec![
            format!(
                "seeds[id] <- {}\n?[t, r, len] := seeds[s], *edge{{source: s, target: t, relation: r}}, len = 1",
                seed_list
            ),
            format!(
                "seeds[id] <- {}\n?[t, r, len] := seeds[s], *edge{{source: s, target: m}}, *edge{{source: m, target: t, relation: r}}, len = 2",
                seed_list
            ),
        ];

        for query in queries {
            if let Ok(res) = cozo.run_script(&query) {
                for row in res.rows {
                    if let (
                        Some(cozo::DataValue::Str(target)),
                        Some(cozo::DataValue::Str(rel)),
                        Some(cozo::DataValue::Num(cozo::Num::Int(len))),
                    ) = (row.first(), row.get(1), row.get(2))
                    {
                        packet.knowledge_graph.push(KGImpact {
                            source_node: "change_seed".to_string(),
                            impacted_node: target.to_string(),
                            relation: rel.to_string(),
                            path_length: *len as usize,
                            reason: format!("KG reachability via {} ({} hops)", rel, len),
                        });
                    }
                }
            }
        }

        info!(
            "KG enrichment added {} impact links",
            packet.knowledge_graph.len()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impact::enrichment::EnrichmentContext;
    use crate::impact::packet::{ChangedFile, ImpactPacket};
    use crate::state::storage::StorageManager;
    use crate::state::storage_cozo::CozoStorage;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_kg_enrichment() {
        let cozo = CozoStorage::new(&PathBuf::from("")).unwrap();

        // Setup KG data
        cozo.run_script(
            "
            ?[id, label, category, risk_score, metadata] <- [
                ['file_1', 'file_1.rs', 'code', 0.0, {}],
                ['file_2', 'file_2.rs', 'code', 0.0, {}],
                ['file_3', 'file_3.rs', 'code', 0.0, {}]
            ] :put node
        ",
        )
        .unwrap();

        cozo.run_script(
            "
            ?[source, target, relation, confidence, provenance_id] <- [
                ['file_1', 'file_2', 'depends_on', 1.0, 'tx1'],
                ['file_2', 'file_3', 'imports', 1.0, 'tx2']
            ] :put edge
        ",
        )
        .unwrap();

        let mut storage =
            StorageManager::init_from_conn(rusqlite::Connection::open_in_memory().unwrap());
        storage.cozo = Some(cozo);

        let context = EnrichmentContext {
            storage: &storage,
            config: &crate::config::model::Config::default(),
            file_id_map: HashMap::new(),
            project_root: PathBuf::from("."),
            warnings: Arc::new(Mutex::new(Vec::new())),
        };

        let mut packet = ImpactPacket {
            changes: vec![ChangedFile {
                path: PathBuf::from("file_1.rs"),
                status: "Modified".to_string(),
                is_staged: true,
                ..Default::default()
            }],
            ..Default::default()
        };

        let provider = KGProvider;
        provider.enrich(&context, &mut packet).unwrap();

        // Should find file_2 (1 hop) and file_3 (2 hops)
        assert!(packet.knowledge_graph.len() >= 2);
        let nodes: Vec<String> = packet
            .knowledge_graph
            .iter()
            .map(|k| k.impacted_node.clone())
            .collect();
        assert!(nodes.contains(&"file_2".to_string()));
        assert!(nodes.contains(&"file_3".to_string()));
    }
}
