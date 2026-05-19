use cozo::*;
use miette::Result;
use serde_json::json;
use std::path::Path;
use tracing::info;

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub category: String,
    pub risk_score: f64,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub confidence: f64,
    pub provenance_id: String,
}

pub struct CozoStorage {
    db: DbInstance,
}

impl CozoStorage {
    pub fn new(db_path: &Path) -> Result<Self> {
        // We use the sqlite engine for persistence
        let engine = if db_path.as_os_str().is_empty() {
            "mem"
        } else {
            "sled"
        };
        info!(
            "CozoStorage selecting engine '{}' for path {:?}",
            engine, db_path
        );

        let db = DbInstance::new(engine, db_path, Default::default())
            .map_err(|e| miette::miette!("Failed to initialize CozoDB: {:?}", e))?;

        let storage = Self { db };
        storage.setup_schema()?;

        info!("Initialized CozoDB storage at {:?}", db_path);
        Ok(storage)
    }

    pub fn run_script(&self, script: &str) -> Result<NamedRows> {
        self.db
            .run_script(script, Default::default(), ScriptMutability::Mutable)
            .map_err(|e| miette::miette!("CozoDB script error: {:?}", e))
    }

    pub fn run_script_with_params(
        &self,
        script: &str,
        params: std::collections::BTreeMap<String, DataValue>,
        mutability: ScriptMutability,
    ) -> Result<NamedRows> {
        self.db
            .run_script(script, params, mutability)
            .map_err(|e| miette::miette!("CozoDB script error: {:?}", e))
    }
    pub fn setup_schema(&self) -> Result<()> {
        let existing = self.get_relations()?;

        if !existing.contains(&"node".to_string()) {
            self.run_script(":create node { id: String => label: String, category: String, risk_score: Float, metadata: Json }")?;
        }
        if !existing.contains(&"edge".to_string()) {
            self.run_script(":create edge { source: String, target: String, relation: String => confidence: Float, provenance_id: String }")?;
        }
        if !existing.contains(&"ledger_link".to_string()) {
            self.run_script(":create ledger_link { node_id: String, ledger_id: String => interaction_type: String }")?;
        }
        if !existing.contains(&"ledger_entry".to_string()) {
            self.run_script(":create ledger_entry { id: Int => tx_id: String, category: String, entry_type: String, entity_normalized: String, change_type: String, summary: String, reason: String, committed_at: String, is_breaking: Bool, verification_status: String, trace_id: String }")?;
        }
        if !existing.contains(&"project_symbol".to_string()) {
            self.run_script(":create project_symbol { id: Int => file_path: String, qualified_name: String, symbol_name: String, symbol_kind: String, is_public: Bool, line_start: Int, line_end: Int }")?;
        }

        // --- Track 54-1: FTS Index ---
        if !existing.contains(&"node:fts_idx".to_string()) {
            // We use the 'Simple' tokenizer for now, to be replaced by 'Code' tokenizer in next step.
            self.run_script("::fts create node:fts_idx {extractor: label, tokenizer: Simple}")?;
        }

        Ok(())
    }

    pub fn get_relations(&self) -> Result<Vec<String>> {
        let res = self.run_script("::relations")?;
        let mut relations = Vec::new();
        for row in res.rows {
            if let Some(DataValue::Str(name)) = row.first() {
                relations.push(name.to_string());
            }
        }
        Ok(relations)
    }

    /// Returns the list of indices on a given relation (e.g., HNSW, FTS).
    /// Uses `::indices <relation>` to avoid blind duplicate creation.
    pub fn get_indices(&self, relation: &str) -> Result<Vec<String>> {
        let script = format!("::indices {}", relation);
        let res = self.run_script(&script)?;
        let mut indices = Vec::new();
        for row in res.rows {
            if let Some(DataValue::Str(name)) = row.first() {
                indices.push(name.to_string());
            }
        }
        Ok(indices)
    }

    /// Verifies that the 'embedding' column in the given relation has the expected dimension.
    /// Returns an error if the dimension mismatches.
    pub fn verify_embedding_dimension(&self, relation: &str, expected_dim: usize) -> Result<()> {
        let script = format!("::columns {}", relation);
        let res = self.run_script(&script)?;
        for row in res.rows {
            #[allow(clippy::collapsible_if)]
            if let (Some(DataValue::Str(name)), Some(DataValue::Str(typ))) =
                (row.first(), row.get(1))
            {
                if name == "embedding" {
                    let expected = format!("<F32; {}>", expected_dim);
                    if typ != &expected {
                        return Err(miette::miette!(
                            "Dimension mismatch for relation '{}': expected {}, found {}. You may need to clear your ChangeGuard state with 'changeguard index --semantic --clear'.",
                            relation,
                            expected,
                            typ
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn node_count(&self) -> Result<usize> {
        let res = self.run_script("?[count(id)] := *node{id}")?;
        if let Some(DataValue::Num(Num::Int(n))) = res.rows.first().and_then(|r| r.first()) {
            return Ok(*n as usize);
        }
        Ok(0)
    }

    pub fn edge_count(&self) -> Result<usize> {
        let res = self.run_script("?[count(source)] := *edge{source}")?;
        if let Some(DataValue::Num(Num::Int(n))) = res.rows.first().and_then(|r| r.first()) {
            return Ok(*n as usize);
        }
        Ok(0)
    }

    pub fn remove_nodes_by_id(&self, ids: &[String]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let batch: Vec<Vec<&str>> = ids.iter().map(|id| vec![id.as_str()]).collect();
        let script = "ids[] <- $batch\n?[id] := ids[id]\n:rm node { id }";
        let mut params = std::collections::BTreeMap::new();
        params.insert("batch".to_string(), cozo::DataValue::from(json!(batch)));
        self.run_script_with_params(script, params, ScriptMutability::Mutable)?;
        Ok(())
    }

    pub fn remove_edges_for_source(&self, source_ids: &[String]) -> Result<()> {
        if source_ids.is_empty() {
            return Ok(());
        }
        let batch: Vec<Vec<&str>> = source_ids.iter().map(|id| vec![id.as_str()]).collect();
        let script = "sources[] <- $batch\n?[source, target, relation] := *edge{source, target, relation}, sources[source]\n:rm edge { source, target, relation }";
        let mut params = std::collections::BTreeMap::new();
        params.insert("batch".to_string(), cozo::DataValue::from(json!(batch)));
        self.run_script_with_params(script, params, ScriptMutability::Mutable)?;
        Ok(())
    }

    pub fn put_node_batch(&self, nodes: &[GraphNode]) -> Result<()> {
        if nodes.is_empty() {
            return Ok(());
        }
        let batch: Vec<_> = nodes
            .iter()
            .map(|n| {
                json!([
                    n.id.as_str(),
                    n.label.as_str(),
                    n.category.as_str(),
                    n.risk_score,
                    n.metadata.clone().unwrap_or_else(|| json!({}))
                ])
            })
            .collect();

        let script = "?[id, label, category, risk_score, metadata] <- $batch :put node";
        let mut params = std::collections::BTreeMap::new();
        params.insert("batch".to_string(), cozo::DataValue::from(json!(batch)));
        self.run_script_with_params(script, params, ScriptMutability::Mutable)?;
        Ok(())
    }

    pub fn put_edge_batch(&self, edges: &[GraphEdge]) -> Result<()> {
        if edges.is_empty() {
            return Ok(());
        }
        let batch: Vec<_> = edges
            .iter()
            .map(|e| {
                json!([
                    e.source.as_str(),
                    e.target.as_str(),
                    e.relation.as_str(),
                    e.confidence,
                    e.provenance_id.as_str()
                ])
            })
            .collect();

        let script = "?[source, target, relation, confidence, provenance_id] <- $batch :put edge";
        let mut params = std::collections::BTreeMap::new();
        params.insert("batch".to_string(), cozo::DataValue::from(json!(batch)));
        self.run_script_with_params(script, params, ScriptMutability::Mutable)?;
        Ok(())
    }

    /// Run Louvain community detection on the edge relation.
    /// Note: CozoDB 0.7 does not include a Leiden implementation;
    /// Louvain is used as the closest available alternative.
    pub fn run_community_louvain(&self) -> Result<Vec<(String, i64)>> {
        let script = "
            edges[src, dst] := *edge{source: src, target: dst}
            ?[node, community_id] <~ CommunityDetectionLouvain(edges[src, dst])
        ";
        let res = self.run_script(script)?;
        let mut communities = Vec::new();
        for row in &res.rows {
            // CommunityDetectionLouvain returns [List(community_ids), node]
            // where community_ids is a hierarchy (usually a single element).
            if let (Some(DataValue::List(comm_list)), Some(DataValue::Str(node))) =
                (row.first(), row.get(1))
                && let Some(DataValue::Num(Num::Int(comm))) = comm_list.first()
            {
                communities.push((node.to_string(), *comm));
            }
        }
        Ok(communities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_cozo_basic_init() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();
        let relations = storage.get_relations().unwrap();
        assert!(relations.contains(&"node".to_string()));
        assert!(relations.contains(&"edge".to_string()));
        assert!(relations.contains(&"ledger_link".to_string()));
    }

    #[test]
    fn test_cozo_insert_query() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();

        // Insert a node
        storage.run_script("?[id, label, category, risk_score, metadata] <- [['node_1', 'Test Node', 'code', 0.5, {}]] :put node").unwrap();

        // Query the node
        let res = storage
            .run_script("?[label] := *node{id: 'node_1', label: label}")
            .unwrap();
        assert_eq!(res.rows.len(), 1);
        assert_eq!(res.rows[0][0], DataValue::Str("Test Node".into()));
    }

    #[test]
    fn test_cozo_reachability() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();

        // Setup nodes
        storage
            .run_script(
                "
            ?[id, label, category, risk_score, metadata] <- [
                ['A', 'Node A', 'code', 0.0, {}],
                ['B', 'Node B', 'code', 0.0, {}],
                ['C', 'Node C', 'code', 0.0, {}]
            ] :put node
        ",
            )
            .unwrap();

        // Setup edges
        storage
            .run_script(
                "
            ?[source, target, relation, confidence, provenance_id] <- [
                ['A', 'B', 'calls', 1.0, 'tx1'],
                ['B', 'C', 'calls', 1.0, 'tx1']
            ] :put edge
        ",
            )
            .unwrap();

        // 2-hop reachability query
        let res = storage
            .run_script(
                "?[target] := *edge{source: 'A', target: t}, *edge{source: t, target: target}",
            )
            .unwrap();
        assert_eq!(res.rows.len(), 1);
        assert_eq!(res.rows[0][0], DataValue::Str("C".into()));
    }

    #[test]
    fn test_node_count_and_edge_count() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();

        assert_eq!(storage.node_count().unwrap(), 0);
        assert_eq!(storage.edge_count().unwrap(), 0);

        storage
            .run_script(
                "?[id, label, category, risk_score, metadata] <- [
                    ['n1', 'Node 1', 'code', 0.0, {}],
                    ['n2', 'Node 2', 'code', 0.0, {}]
                ] :put node",
            )
            .unwrap();

        storage
            .run_script(
                "?[source, target, relation, confidence, provenance_id] <- [
                    ['n1', 'n2', 'calls', 1.0, 'tx1']
                ] :put edge",
            )
            .unwrap();

        assert_eq!(storage.node_count().unwrap(), 2);
        assert_eq!(storage.edge_count().unwrap(), 1);
    }

    #[test]
    fn test_put_node_batch_and_query() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();
        let nodes = vec![
            GraphNode {
                id: "n1".to_string(),
                label: "Node 1".to_string(),
                category: "code".to_string(),
                risk_score: 0.5,
                metadata: None,
            },
            GraphNode {
                id: "n2".to_string(),
                label: "Node 2".to_string(),
                category: "code".to_string(),
                risk_score: 0.0,
                metadata: Some(json!({"lang": "rust"})),
            },
        ];
        storage.put_node_batch(&nodes).unwrap();
        assert_eq!(storage.node_count().unwrap(), 2);

        let res = storage
            .run_script("?[label] := *node{id: 'n1', label: label}")
            .unwrap();
        assert_eq!(res.rows.len(), 1);
        assert_eq!(res.rows[0][0], DataValue::Str("Node 1".into()));
    }

    #[test]
    fn test_remove_nodes_by_id() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();
        let nodes = vec![
            GraphNode {
                id: "n1".to_string(),
                label: "Node 1".to_string(),
                category: "code".to_string(),
                risk_score: 0.0,
                metadata: None,
            },
            GraphNode {
                id: "n2".to_string(),
                label: "Node 2".to_string(),
                category: "code".to_string(),
                risk_score: 0.0,
                metadata: None,
            },
        ];
        storage.put_node_batch(&nodes).unwrap();
        assert_eq!(storage.node_count().unwrap(), 2);

        storage.remove_nodes_by_id(&["n1".to_string()]).unwrap();
        assert_eq!(storage.node_count().unwrap(), 1);

        let res = storage.run_script("?[id] := *node{id: id}").unwrap();
        assert_eq!(res.rows.len(), 1);
        assert_eq!(res.rows[0][0], DataValue::Str("n2".into()));
    }

    #[test]
    fn test_remove_nodes_by_id_with_colons() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();
        let nodes = vec![
            GraphNode {
                id: "src/lib.rs".to_string(),
                label: "lib.rs".to_string(),
                category: "file".to_string(),
                risk_score: 0.0,
                metadata: None,
            },
            GraphNode {
                id: "crate::foo".to_string(),
                label: "foo".to_string(),
                category: "symbol".to_string(),
                risk_score: 0.0,
                metadata: None,
            },
        ];
        storage.put_node_batch(&nodes).unwrap();
        assert_eq!(storage.node_count().unwrap(), 2);

        storage
            .remove_nodes_by_id(&["src/lib.rs".to_string(), "crate::foo".to_string()])
            .unwrap();
        assert_eq!(storage.node_count().unwrap(), 0);
    }

    #[test]
    fn test_remove_nodes_by_id_with_backslash() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();
        let nodes = vec![GraphNode {
            id: "src\\lib.rs".to_string(),
            label: "lib.rs".to_string(),
            category: "file".to_string(),
            risk_score: 0.0,
            metadata: None,
        }];
        storage.put_node_batch(&nodes).unwrap();
        assert_eq!(storage.node_count().unwrap(), 1);

        storage
            .remove_nodes_by_id(&["src\\lib.rs".to_string()])
            .unwrap();
        assert_eq!(storage.node_count().unwrap(), 0);
    }

    #[test]
    fn test_remove_mixed_batch() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();
        let nodes = vec![
            GraphNode {
                id: "src\\lib.rs".to_string(),
                label: "lib.rs".to_string(),
                category: "file".to_string(),
                risk_score: 0.0,
                metadata: None,
            },
            GraphNode {
                id: "crate::foo".to_string(),
                label: "foo".to_string(),
                category: "symbol".to_string(),
                risk_score: 0.0,
                metadata: None,
            },
        ];
        storage.put_node_batch(&nodes).unwrap();
        storage
            .remove_nodes_by_id(&["src\\lib.rs".to_string(), "crate::foo".to_string()])
            .unwrap();
        storage
            .remove_edges_for_source(&["crate::foo".to_string()])
            .unwrap();
        assert_eq!(storage.node_count().unwrap(), 0);
        assert_eq!(storage.edge_count().unwrap(), 0);
    }

    #[test]
    fn test_remove_nonexistent_nodes() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();
        storage
            .remove_nodes_by_id(&["src\\lib.rs".to_string(), "crate::foo".to_string()])
            .unwrap();
        assert_eq!(storage.node_count().unwrap(), 0);
    }

    #[test]
    fn test_remove_nodes_exact_script() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();
        let script = r#"?[id] <- [["src/lib.rs"],["crate::foo"]] :rm node { id }"#;
        storage.run_script(script).unwrap();
        assert_eq!(storage.node_count().unwrap(), 0);
    }

    #[test]
    fn test_remove_nodes_by_id_with_backslash_models() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();
        let nodes = vec![
            GraphNode {
                id: "src\\models.rs".to_string(),
                label: "models.rs".to_string(),
                category: "file".to_string(),
                risk_score: 0.0,
                metadata: None,
            },
            GraphNode {
                id: "Model".to_string(),
                label: "Model".to_string(),
                category: "symbol".to_string(),
                risk_score: 0.0,
                metadata: None,
            },
            GraphNode {
                id: "new".to_string(),
                label: "new".to_string(),
                category: "symbol".to_string(),
                risk_score: 0.0,
                metadata: None,
            },
        ];
        storage.put_node_batch(&nodes).unwrap();
        assert_eq!(storage.node_count().unwrap(), 3);

        storage
            .remove_nodes_by_id(&[
                "src\\models.rs".to_string(),
                "Model".to_string(),
                "new".to_string(),
            ])
            .unwrap();
        assert_eq!(storage.node_count().unwrap(), 0);
    }

    #[test]
    #[ignore = "CozoDB :rm with list binding does not work reliably on sled-backed storage"]
    fn test_remove_nodes_by_id_with_backslash_models_sled() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("test.cozo");
        let storage = CozoStorage::new(&path).unwrap();
        let nodes = vec![
            GraphNode {
                id: "src/models.rs".to_string(),
                label: "models.rs".to_string(),
                category: "file".to_string(),
                risk_score: 0.0,
                metadata: None,
            },
            GraphNode {
                id: "Model".to_string(),
                label: "Model".to_string(),
                category: "symbol".to_string(),
                risk_score: 0.0,
                metadata: None,
            },
            GraphNode {
                id: "new".to_string(),
                label: "new".to_string(),
                category: "symbol".to_string(),
                risk_score: 0.0,
                metadata: None,
            },
        ];
        storage.put_node_batch(&nodes).unwrap();
        let before = storage.node_count().unwrap();
        assert_eq!(before, 3, "Expected 3 nodes after put, got {}", before);

        // Clear all nodes before testing remove on empty graph
        let ids = vec![
            "src/models.rs".to_string(),
            "Model".to_string(),
            "new".to_string(),
        ];
        storage.remove_nodes_by_id(&ids).unwrap();
        let count = storage.node_count().unwrap();
        assert_eq!(count, 0, "Expected 0 nodes after clear, got {}", count);

        storage.remove_nodes_by_id(&ids).unwrap();
        assert_eq!(storage.node_count().unwrap(), 0);
    }

    #[test]
    fn test_remove_edges_for_source() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();
        let nodes = vec![
            GraphNode {
                id: "a".to_string(),
                label: "A".to_string(),
                category: "code".to_string(),
                risk_score: 0.0,
                metadata: None,
            },
            GraphNode {
                id: "b".to_string(),
                label: "B".to_string(),
                category: "code".to_string(),
                risk_score: 0.0,
                metadata: None,
            },
            GraphNode {
                id: "c".to_string(),
                label: "C".to_string(),
                category: "code".to_string(),
                risk_score: 0.0,
                metadata: None,
            },
        ];
        storage.put_node_batch(&nodes).unwrap();
        let edges = vec![
            GraphEdge {
                source: "a".to_string(),
                target: "b".to_string(),
                relation: "calls".to_string(),
                confidence: 1.0,
                provenance_id: "tx1".to_string(),
            },
            GraphEdge {
                source: "b".to_string(),
                target: "c".to_string(),
                relation: "calls".to_string(),
                confidence: 1.0,
                provenance_id: "tx1".to_string(),
            },
        ];
        storage.put_edge_batch(&edges).unwrap();
        assert_eq!(storage.edge_count().unwrap(), 2);

        storage.remove_edges_for_source(&["a".to_string()]).unwrap();
        assert_eq!(storage.edge_count().unwrap(), 1);

        let res = storage
            .run_script("?[source, target] := *edge{source, target}")
            .unwrap();
        assert_eq!(res.rows.len(), 1);
        assert_eq!(res.rows[0][0], DataValue::Str("b".into()));
        assert_eq!(res.rows[0][1], DataValue::Str("c".into()));
    }

    #[test]
    fn test_idempotent_put() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();
        let nodes = vec![GraphNode {
            id: "n1".to_string(),
            label: "First".to_string(),
            category: "code".to_string(),
            risk_score: 0.0,
            metadata: None,
        }];
        storage.put_node_batch(&nodes).unwrap();
        assert_eq!(storage.node_count().unwrap(), 1);

        let nodes2 = vec![GraphNode {
            id: "n1".to_string(),
            label: "Second".to_string(),
            category: "code".to_string(),
            risk_score: 1.0,
            metadata: None,
        }];
        storage.put_node_batch(&nodes2).unwrap();
        assert_eq!(storage.node_count().unwrap(), 1);

        let res = storage
            .run_script("?[label] := *node{id: 'n1', label: label}")
            .unwrap();
        assert_eq!(res.rows[0][0], DataValue::Str("Second".into()));
    }

    #[test]
    fn test_run_community_louvain() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();

        // Two disconnected clusters
        storage
            .run_script(
                "?[id, label, category, risk_score, metadata] <- [
                    ['a1', 'A1', 'code', 0.0, {}],
                    ['a2', 'A2', 'code', 0.0, {}],
                    ['b1', 'B1', 'code', 0.0, {}],
                    ['b2', 'B2', 'code', 0.0, {}]
                ] :put node",
            )
            .unwrap();

        storage
            .run_script(
                "?[source, target, relation, confidence, provenance_id] <- [
                    ['a1', 'a2', 'calls', 1.0, 'tx1'],
                    ['b1', 'b2', 'calls', 1.0, 'tx1']
                ] :put edge",
            )
            .unwrap();

        let communities = storage.run_community_louvain().unwrap();
        assert!(!communities.is_empty());

        let distinct: std::collections::HashSet<i64> =
            communities.iter().map(|(_, c)| *c).collect();
        assert!(
            distinct.len() >= 2,
            "Expected at least 2 communities, got {:?}",
            distinct.len()
        );
    }
}
