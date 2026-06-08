use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::{ImpactPacket, KGImpact};
use crate::ui::spinner::Spinner;
use miette::Result;
use std::time::Instant;
use tracing::{debug, warn};

use crate::state::graph_kinds::NodeKind;
use crate::platform::urn::build_urn;

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

        debug!("Enriching impact packet with Knowledge Graph data...");
        let spinner = Spinner::new("Enriching Knowledge Graph...");
        let start_time = Instant::now();
        let timeout_secs = context.config.coverage.kg_timeout_secs as u64;

        let check_timeout = |context: &EnrichmentContext| -> bool {
            if start_time.elapsed().as_secs() >= timeout_secs {
                context.add_warning("Knowledge Graph enrichment timed out".to_string());
                return true;
            }
            false
        };

        // 1. Sync Hotspots to KG risk scores
        if !packet.hotspots.is_empty() {
            if check_timeout(context) {
                spinner.finish();
                return Ok(());
            }

            let mut risk_updates = Vec::new();
            for hotspot in &packet.hotspots {
                let id = build_urn(NodeKind::File, &hotspot.path.to_string_lossy());
                risk_updates.push(vec![
                    cozo::DataValue::Str(id.into()),
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
            if check_timeout(context) {
                spinner.finish();
                return Ok(());
            }

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
            if check_timeout(context) {
                spinner.finish();
                return Ok(());
            }

            // Find nodes associated with this file
            let file_path = file.path.to_string_lossy();
            let file_urn = build_urn(NodeKind::File, &file_path);
            
            // Query for symbol nodes associated with this file in SQLite project_symbols,
            // then find their corresponding node IDs in Cozo (which are URNs).
            let query = format!(
                "?[id] := *project_symbol{{file_path: '{}', qualified_name: qn}}, *node{{id: id}}, id == concat('urn:changeguard:symbol:', qn)",
                file_path
            );

            // Also check the file node directly
            seed_nodes.push(vec![file_urn]);

            if let Ok(res) = cozo.run_script(&query) {
                for row in res.rows {
                    if let Some(cozo::DataValue::Str(id)) = row.first() {
                        seed_nodes.push(vec![id.to_string()]);
                    }
                }
            }
        }

        if seed_nodes.is_empty() {
            debug!("No seed nodes found in KG for changes");
            spinner.finish();
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
            if check_timeout(context) {
                spinner.finish();
                return Ok(());
            }

            if let Ok(res) = cozo.run_script(&query) {
                for row in res.rows {
                    if let (
                        Some(cozo::DataValue::Str(target)),
                        Some(cozo::DataValue::Str(rel)),
                        Some(cozo::DataValue::Num(cozo::Num::Int(len))),
                    ) = (row.first(), row.get(1), row.get(2))
                    {
                        let impacted_category = target.split(':').nth(2).unwrap_or("unknown").to_string();
                        packet.knowledge_graph.push(KGImpact {
                            source_node: "change_seed".to_string(),
                            source_category: "seed".to_string(),
                            impacted_node: target.to_string(),
                            impacted_category,
                            relation: rel.to_string(),
                            path_length: *len as usize,
                            reason: format!("KG reachability via {} ({} hops)", rel, len),
                        });
                    }
                }
            }
        }

        spinner.finish();
        debug!(
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
    use crate::state::graph_kinds::{EdgeKind, NodeKind};
    use crate::state::storage::StorageManager;
    use crate::state::storage_cozo::{CozoStorage, GraphEdge, GraphNode};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_kg_enrichment() {
        let cozo = CozoStorage::new(&PathBuf::from("")).unwrap();

        // Setup KG data
        let nodes = vec![
            GraphNode {
                id: build_urn(NodeKind::File, "file_1.rs"),
                label: "file_1.rs".to_string(),
                category: NodeKind::File,
                risk_score: 0.0,
                metadata: None,
            },
            GraphNode {
                id: build_urn(NodeKind::File, "file_2.rs"),
                label: "file_2.rs".to_string(),
                category: NodeKind::File,
                risk_score: 0.0,
                metadata: None,
            },
            GraphNode {
                id: build_urn(NodeKind::File, "file_3.rs"),
                label: "file_3.rs".to_string(),
                category: NodeKind::File,
                risk_score: 0.0,
                metadata: None,
            },
        ];
        cozo.insert_nodes(&nodes).unwrap();

        let edges = vec![
            GraphEdge {
                source: build_urn(NodeKind::File, "file_1.rs"),
                target: build_urn(NodeKind::File, "file_2.rs"),
                relation: EdgeKind::DependsOn,
                confidence: 1.0,
                provenance_id: "tx1".to_string(),
            },
            GraphEdge {
                source: build_urn(NodeKind::File, "file_2.rs"),
                target: build_urn(NodeKind::File, "file_3.rs"),
                relation: EdgeKind::DependsOn,
                confidence: 1.0,
                provenance_id: "tx2".to_string(),
            },
        ];
        cozo.insert_edges(&edges).unwrap();

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
        assert!(nodes.contains(&build_urn(NodeKind::File, "file_2.rs")));
        assert!(nodes.contains(&build_urn(NodeKind::File, "file_3.rs")));

        // Verify categories are populated
        for impact in &packet.knowledge_graph {
            assert_eq!(impact.impacted_category, "file");
            assert_eq!(impact.source_category, "seed");
        }
    }
}
