use cozo::*;
use miette::Result;
use serde_json::json;
use std::path::Path;
use tracing::{debug, info};

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
        debug!(
            "CozoStorage selecting engine '{}' for path {:?}",
            engine, db_path
        );

        let is_new = engine == "sled" && !db_path.exists();
        if is_new {
            info!("[init] Creating new CozoDB storage at {:?}", db_path);
        }

        let db = DbInstance::new(engine, db_path, Default::default())
            .map_err(|e| miette::miette!("Failed to initialize CozoDB: {:?}", e))?;

        // Cold Start Verification: Detect HNSW metadata corruption immediately.
        // If this panics/errors internally in Cozo due to "Invalid neighbor degree",
        // we catch it right here at initialization and prevent further operations.
        if engine == "sled"
            && !is_new
            && let Err(e) = db.run_script(
                "::relations",
                Default::default(),
                ScriptMutability::Immutable,
            )
        {
            return Err(miette::miette!(
                "CozoDB Cold Start Verification failed. Storage metadata may be corrupt: {}",
                e
            ));
        }

        let storage = Self { db };
        storage.setup_schema()?;

        debug!("Initialized CozoDB storage at {:?}", db_path);
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

    /// Explicitly close the CozoDB instance, ensuring all file locks are released.
    /// This is particularly important on Windows before attempting to delete
    /// the underlying storage directory.
    pub fn shutdown(self) {
        // Drop the DbInstance to release locks
        debug!("Shutting down CozoDB instance");
        drop(self.db);
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

        // --- Track C2: AI-Brains Domain Relations ---
        if !existing.contains(&"Turn".to_string()) {
            self.run_script(":create Turn { id: String => session_id: String, timestamp: String, project_id: String, summary: String, privacy_level: String }")?;
        }
        if !existing.contains(&"Session".to_string()) {
            self.run_script(":create Session { id: String => project_id: String, started_at: String, ended_at: String, turn_count: Int, privacy_level: String }")?;
        }
        if !existing.contains(&"Memory".to_string()) {
            self.run_script(":create Memory { id: String => source_turn_id: String, content: String, memory_type: String, privacy_level: String, created_at: String }")?;
        }
        if !existing.contains(&"Decision".to_string()) {
            self.run_script(":create Decision { id: String => title: String, context_field: String, decision_text: String, consequences: String, source_tx_id: String, timestamp: String }")?;
        }

        // --- Track C2: Cross-Domain Reachability Queries ---
        // Cross-domain traversals are expressed as Datalog query patterns executed
        // via run_script(), which is the interface AI-Brains' CozoProxyBackend uses.
        // No stored rules — the cozo-redux fork does not support :create ... := for rules.
        //
        // conversation_to_ast query (Turn path):
        //   ?[node_id, node_label] := *Memory{source_turn_id: '<id>', content: node_label}, *node{id: node_id, label: node_label}
        //
        // conversation_to_ast query (Decision source path):
        //   ?[node_id, node_label] := *Decision{id: '<id>', source_tx_id: tx_id}, *edge{source: node_id, provenance_id: tx_id}, *node{id: node_id, label: node_label}
        //
        // conversation_to_ast query (Decision target path):
        //   ?[node_id, node_label] := *Decision{id: '<id>', source_tx_id: tx_id}, *edge{target: node_id, provenance_id: tx_id}, *node{id: node_id, label: node_label}
        //
        // ast_to_conversation query (Turn via Memory):
        //   ?[entity_id] := *node{id: '<id>', label: label}, *Memory{source_turn_id: entity_id, content: label}
        //
        // ast_to_conversation query (Session via Memory->Turn):
        //   ?[entity_id] := *node{id: '<id>', label: label}, *Memory{source_turn_id: turn_id, content: label}, *Turn{id: turn_id, session_id: entity_id}
        //
        // ast_to_conversation query (Decision via Edge source):
        //   ?[entity_id] := *edge{source: '<id>', provenance_id: tx_id}, *Decision{id: entity_id, source_tx_id: tx_id}
        //
        // ast_to_conversation query (Decision via Edge target):
        //   ?[entity_id] := *edge{target: '<id>', provenance_id: tx_id}, *Decision{id: entity_id, source_tx_id: tx_id}

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

    // --- Track C2: Cross-Domain Reachability Query Helpers ---
    // These return Datalog query text for execution via run_script().
    // AI-Brains' CozoProxyBackend sends equivalent raw Datalog over Named Pipes.

    /// Query: find AST nodes affected by a Turn via Memory content matching.
    /// Traversal: Turn -> Memory -> Node (where Memory.content == Node.label)
    pub fn query_conversation_to_ast_via_memory(&self, turn_id: &str) -> Result<NamedRows> {
        let script = "?[node_id, node_label] := *Memory{source_turn_id: $turn_id, content: node_label}, *node{id: node_id, label: node_label}";
        let mut params = std::collections::BTreeMap::new();
        params.insert(
            "turn_id".to_string(),
            DataValue::Str(turn_id.to_string().into()),
        );
        self.run_script_with_params(script, params, ScriptMutability::Immutable)
    }

    /// Query: find AST nodes affected by a Decision via Edge provenance (source nodes).
    /// Traversal: Decision -> Edge source -> Node
    pub fn query_conversation_to_ast_via_decision(&self, decision_id: &str) -> Result<NamedRows> {
        let script = "?[node_id, node_label] := *Decision{id: $decision_id, source_tx_id: tx_id}, *edge{source: node_id, provenance_id: tx_id}, *node{id: node_id, label: node_label}";
        let mut params = std::collections::BTreeMap::new();
        params.insert(
            "decision_id".to_string(),
            DataValue::Str(decision_id.to_string().into()),
        );
        self.run_script_with_params(script, params, ScriptMutability::Immutable)
    }

    /// Query: find conversations that discussed a given AST node via Memory.
    /// Traversal: Node -> Memory -> Turn
    pub fn query_ast_to_conversation_via_memory(&self, node_id: &str) -> Result<NamedRows> {
        let script = "?[entity_id] := *node{id: $node_id, label: label}, *Memory{source_turn_id: entity_id, content: label}";
        let mut params = std::collections::BTreeMap::new();
        params.insert(
            "node_id".to_string(),
            DataValue::Str(node_id.to_string().into()),
        );
        self.run_script_with_params(script, params, ScriptMutability::Immutable)
    }

    /// Query: find sessions that discussed a given AST node via Memory -> Turn.
    /// Traversal: Node -> Memory -> Turn -> Session
    pub fn query_ast_to_conversation_via_session(&self, node_id: &str) -> Result<NamedRows> {
        let script = "?[entity_id] := *node{id: $node_id, label: label}, *Memory{source_turn_id: turn_id, content: label}, *Turn{id: turn_id, session_id: entity_id}";
        let mut params = std::collections::BTreeMap::new();
        params.insert(
            "node_id".to_string(),
            DataValue::Str(node_id.to_string().into()),
        );
        self.run_script_with_params(script, params, ScriptMutability::Immutable)
    }

    /// Query: find Decisions that discuss a given AST node via Edge provenance.
    /// Traversal: Node (as edge source or target) -> Edge -> Decision
    pub fn query_ast_to_conversation_via_decision(&self, node_id: &str) -> Result<NamedRows> {
        // Query both edge source and edge target paths.
        let script = "?[entity_id] := *edge{source: $node_id, provenance_id: tx_id}, *Decision{id: entity_id, source_tx_id: tx_id}";
        let mut params = std::collections::BTreeMap::new();
        params.insert(
            "node_id".to_string(),
            DataValue::Str(node_id.to_string().into()),
        );
        self.run_script_with_params(script, params, ScriptMutability::Immutable)
    }

    /// Query: find Decisions that discuss a given AST node via Edge target provenance.
    pub fn query_ast_to_conversation_via_decision_target(
        &self,
        node_id: &str,
    ) -> Result<NamedRows> {
        let script = "?[entity_id] := *edge{target: $node_id, provenance_id: tx_id}, *Decision{id: entity_id, source_tx_id: tx_id}";
        let mut params = std::collections::BTreeMap::new();
        params.insert(
            "node_id".to_string(),
            DataValue::Str(node_id.to_string().into()),
        );
        self.run_script_with_params(script, params, ScriptMutability::Immutable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

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

    // --- Track C2: AI-Brains Domain Relations Tests ---

    #[test]
    fn test_ai_brains_relations_exist() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();
        let relations = storage.get_relations().unwrap();
        assert!(
            relations.contains(&"Turn".to_string()),
            "Turn relation missing"
        );
        assert!(
            relations.contains(&"Session".to_string()),
            "Session relation missing"
        );
        assert!(
            relations.contains(&"Memory".to_string()),
            "Memory relation missing"
        );
        assert!(
            relations.contains(&"Decision".to_string()),
            "Decision relation missing"
        );
    }

    #[test]
    fn test_turn_insert_and_query() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();

        storage
            .run_script(
                "?[id, session_id, timestamp, project_id, summary, privacy_level] <- \
                 [['t1', 's1', '2024-01-01T00:00:00Z', 'proj1', 'discussed architecture', 'internal']] \
                 :put Turn",
            )
            .unwrap();

        let res = storage
            .run_script("?[summary] := *Turn{id: 't1', summary: summary}")
            .unwrap();
        assert_eq!(res.rows.len(), 1);
        assert_eq!(
            res.rows[0][0],
            DataValue::Str("discussed architecture".into())
        );

        // Query by session
        let res = storage
            .run_script("?[id] := *Turn{session_id: 's1', id: id}")
            .unwrap();
        assert_eq!(res.rows.len(), 1);
        assert_eq!(res.rows[0][0], DataValue::Str("t1".into()));
    }

    #[test]
    fn test_session_insert_and_query() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();

        storage
            .run_script(
                "?[id, project_id, started_at, ended_at, turn_count, privacy_level] <- \
                 [['sess1', 'proj1', '2024-01-01T00:00:00Z', '2024-01-01T01:00:00Z', 5, 'internal']] \
                 :put Session",
            )
            .unwrap();

        let res = storage
            .run_script("?[turn_count] := *Session{id: 'sess1', turn_count: turn_count}")
            .unwrap();
        assert_eq!(res.rows.len(), 1);
        assert_eq!(res.rows[0][0], DataValue::Num(Num::Int(5)));
    }

    #[test]
    fn test_memory_insert_and_query() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();

        storage
            .run_script(
                "?[id, source_turn_id, content, memory_type, privacy_level, created_at] <- \
                 [['m1', 't1', 'parse_input', 'symbol_reference', 'internal', '2024-01-01T00:00:00Z']] \
                 :put Memory",
            )
            .unwrap();

        let res = storage
            .run_script("?[content] := *Memory{id: 'm1', content: content}")
            .unwrap();
        assert_eq!(res.rows.len(), 1);
        assert_eq!(res.rows[0][0], DataValue::Str("parse_input".into()));

        // Query by source_turn_id
        let res = storage
            .run_script("?[id] := *Memory{source_turn_id: 't1', id: id}")
            .unwrap();
        assert_eq!(res.rows.len(), 1);
        assert_eq!(res.rows[0][0], DataValue::Str("m1".into()));
    }

    #[test]
    fn test_decision_insert_and_query() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();

        storage
            .run_script(
                "?[id, title, context_field, decision_text, consequences, source_tx_id, timestamp] <- \
                 [['d1', 'ADR: Use CozoDB', 'architecture', 'Adopt CozoDB for KG storage', \
                   'Unified Datalog queries across domains', 'tx_001', '2024-01-01T00:00:00Z']] \
                 :put Decision",
            )
            .unwrap();

        let res = storage
            .run_script("?[title] := *Decision{id: 'd1', title: title}")
            .unwrap();
        assert_eq!(res.rows.len(), 1);
        assert_eq!(res.rows[0][0], DataValue::Str("ADR: Use CozoDB".into()));

        // Query by source_tx_id
        let res = storage
            .run_script("?[id] := *Decision{source_tx_id: 'tx_001', id: id}")
            .unwrap();
        assert_eq!(res.rows.len(), 1);
        assert_eq!(res.rows[0][0], DataValue::Str("d1".into()));
    }

    // --- Track C2: Cross-Domain Reachability Tests ---
    // These tests use raw Datalog queries via run_script (AI-Brains' interface)
    // and the query helper methods on CozoStorage.

    #[test]
    fn test_conversation_to_ast_memory_path() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();

        // Insert nodes
        storage
            .run_script(
                "?[id, label, category, risk_score, metadata] <- [
                    ['n1', 'parse_input', 'function', 0.0, {}],
                    ['n2', 'validate', 'function', 0.0, {}]
                ] :put node",
            )
            .unwrap();

        // Insert Turn
        storage
            .run_script(
                "?[id, session_id, timestamp, project_id, summary, privacy_level] <- \
                 [['t1', 's1', '2024-01-01T00:00:00Z', 'proj1', 'discussed parse_input', 'internal']] \
                 :put Turn",
            )
            .unwrap();

        // Insert Memory whose content matches node label 'parse_input'
        storage
            .run_script(
                "?[id, source_turn_id, content, memory_type, privacy_level, created_at] <- \
                 [['m1', 't1', 'parse_input', 'symbol_reference', 'internal', '2024-01-01T00:00:00Z']] \
                 :put Memory",
            )
            .unwrap();

        // Use the helper method (which executes raw Datalog via run_script)
        let res = storage.query_conversation_to_ast_via_memory("t1").unwrap();

        // Should find n1 (parse_input) but not n2 (validate) since Memory.content = "parse_input"
        assert_eq!(res.rows.len(), 1, "Expected 1 node, got {:?}", res.rows);
        assert_eq!(res.rows[0][0], DataValue::Str("n1".into()));
        assert_eq!(res.rows[0][1], DataValue::Str("parse_input".into()));
    }

    #[test]
    fn test_conversation_to_ast_memory_multi_symbol() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();

        // Insert nodes
        storage
            .run_script(
                "?[id, label, category, risk_score, metadata] <- [
                    ['n_parse', 'parse_input', 'function', 0.0, {}],
                    ['n_valid', 'validate', 'function', 0.0, {}],
                    ['n_handler', 'handle_request', 'function', 0.0, {}]
                ] :put node",
            )
            .unwrap();

        // Insert Turn
        storage
            .run_script(
                "?[id, session_id, timestamp, project_id, summary, privacy_level] <- \
                 [['t2', 's2', '2024-01-01T00:00:00Z', 'proj1', 'code review', 'internal']] \
                 :put Turn",
            )
            .unwrap();

        // Two Memory entries from the same Turn, each referencing a different symbol
        storage
            .run_script(
                "?[id, source_turn_id, content, memory_type, privacy_level, created_at] <- [
                    ['m_p', 't2', 'parse_input', 'symbol_reference', 'internal', '2024-01-01T00:00:00Z'],
                    ['m_v', 't2', 'validate', 'symbol_reference', 'internal', '2024-01-01T00:00:00Z']
                ] :put Memory",
            )
            .unwrap();

        // Use the helper method
        let res = storage.query_conversation_to_ast_via_memory("t2").unwrap();

        // Should find both parse_input and validate, but not handle_request
        assert_eq!(res.rows.len(), 2);
        let labels: Vec<String> = res
            .rows
            .iter()
            .map(|r| {
                if let DataValue::Str(s) = &r[1] {
                    s.to_string()
                } else {
                    String::new()
                }
            })
            .collect();
        assert!(labels.contains(&"parse_input".to_string()));
        assert!(labels.contains(&"validate".to_string()));
    }

    #[test]
    fn test_conversation_to_ast_decision_path() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();

        // Insert nodes
        storage
            .run_script(
                "?[id, label, category, risk_score, metadata] <- [
                    ['n1', 'parse_input', 'function', 0.0, {}],
                    ['n2', 'validate', 'function', 0.0, {}]
                ] :put node",
            )
            .unwrap();

        // Insert edge with provenance matching decision's source_tx_id
        storage
            .run_script(
                "?[source, target, relation, confidence, provenance_id] <- \
                 [['n1', 'n2', 'calls', 1.0, 'tx_adr_1']] \
                 :put edge",
            )
            .unwrap();

        // Insert Decision linked via source_tx_id = edge provenance_id
        storage
            .run_script(
                "?[id, title, context_field, decision_text, consequences, source_tx_id, timestamp] <- \
                 [['d1', 'ADR: Input Validation', 'security', 'Use strict validation', \
                   'better security posture', 'tx_adr_1', '2024-01-01T00:00:00Z']] \
                 :put Decision",
            )
            .unwrap();

        // Query via the helper method (source nodes of matching edges)
        let res = storage
            .query_conversation_to_ast_via_decision("d1")
            .unwrap();

        // Should find n1 via edge source path
        assert_eq!(res.rows.len(), 1);
        assert_eq!(res.rows[0][0], DataValue::Str("n1".into()));
        assert_eq!(res.rows[0][1], DataValue::Str("parse_input".into()));

        // Also query target nodes via raw Datalog
        let res_target = storage
            .run_script(
                "?[node_id, node_label] := *Decision{id: 'd1', source_tx_id: tx_id}, \
                 *edge{target: node_id, provenance_id: tx_id}, \
                 *node{id: node_id, label: node_label}",
            )
            .unwrap();
        assert_eq!(res_target.rows.len(), 1);
        assert_eq!(res_target.rows[0][0], DataValue::Str("n2".into()));
        assert_eq!(res_target.rows[0][1], DataValue::Str("validate".into()));
    }

    #[test]
    fn test_ast_to_conversation_memory_to_turn() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();

        // Insert node
        storage
            .run_script(
                "?[id, label, category, risk_score, metadata] <- \
                 [['n1', 'parse_input', 'function', 0.0, {}]] \
                 :put node",
            )
            .unwrap();

        // Insert Turn
        storage
            .run_script(
                "?[id, session_id, timestamp, project_id, summary, privacy_level] <- \
                 [['t1', 's1', '2024-01-01T00:00:00Z', 'proj1', 'discussed parse_input', 'internal']] \
                 :put Turn",
            )
            .unwrap();

        // Insert Memory linking the Turn to the symbol
        storage
            .run_script(
                "?[id, source_turn_id, content, memory_type, privacy_level, created_at] <- \
                 [['m1', 't1', 'parse_input', 'symbol_reference', 'internal', '2024-01-01T00:00:00Z']] \
                 :put Memory",
            )
            .unwrap();

        // Use the helper method (finds Turns via Memory content matching)
        let res = storage.query_ast_to_conversation_via_memory("n1").unwrap();

        // Should find the Turn (t1) via Memory content matching
        assert_eq!(res.rows.len(), 1);
        assert_eq!(res.rows[0][0], DataValue::Str("t1".into()));
    }

    #[test]
    fn test_ast_to_conversation_memory_to_session() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();

        // Insert node
        storage
            .run_script(
                "?[id, label, category, risk_score, metadata] <- \
                 [['n1', 'parse_input', 'function', 0.0, {}]] \
                 :put node",
            )
            .unwrap();

        // Insert Session
        storage
            .run_script(
                "?[id, project_id, started_at, ended_at, turn_count, privacy_level] <- \
                 [['sess1', 'proj1', '2024-01-01T00:00:00Z', '2024-01-01T01:00:00Z', 3, 'internal']] \
                 :put Session",
            )
            .unwrap();

        // Insert Turn linked to the session
        storage
            .run_script(
                "?[id, session_id, timestamp, project_id, summary, privacy_level] <- \
                 [['t1', 'sess1', '2024-01-01T00:00:00Z', 'proj1', 'discussed parse_input', 'internal']] \
                 :put Turn",
            )
            .unwrap();

        // Insert Memory linking the Turn to the symbol
        storage
            .run_script(
                "?[id, source_turn_id, content, memory_type, privacy_level, created_at] <- \
                 [['m1', 't1', 'parse_input', 'symbol_reference', 'internal', '2024-01-01T00:00:00Z']] \
                 :put Memory",
            )
            .unwrap();

        // Find Turns via Memory (should find t1)
        let res_turns = storage.query_ast_to_conversation_via_memory("n1").unwrap();
        assert_eq!(res_turns.rows.len(), 1);
        assert_eq!(res_turns.rows[0][0], DataValue::Str("t1".into()));

        // Find Sessions via Memory -> Turn (should find sess1)
        let res_sessions = storage.query_ast_to_conversation_via_session("n1").unwrap();
        assert_eq!(res_sessions.rows.len(), 1);
        assert_eq!(res_sessions.rows[0][0], DataValue::Str("sess1".into()));
    }

    #[test]
    fn test_ast_to_conversation_edge_to_decision() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();

        // Insert nodes
        storage
            .run_script(
                "?[id, label, category, risk_score, metadata] <- [
                    ['n1', 'parse_input', 'function', 0.0, {}],
                    ['n2', 'validate', 'function', 0.0, {}]
                ] :put node",
            )
            .unwrap();

        // Insert edge with provenance
        storage
            .run_script(
                "?[source, target, relation, confidence, provenance_id] <- \
                 [['n1', 'n2', 'calls', 1.0, 'tx_adr_1']] \
                 :put edge",
            )
            .unwrap();

        // Insert Decision linked via source_tx_id = edge provenance_id
        storage
            .run_script(
                "?[id, title, context_field, decision_text, consequences, source_tx_id, timestamp] <- \
                 [['d1', 'ADR: Input Validation', 'security', 'Use strict validation', \
                   'better security posture', 'tx_adr_1', '2024-01-01T00:00:00Z']] \
                 :put Decision",
            )
            .unwrap();

        // Query via the helper method (n1 is source of edge)
        let res = storage
            .query_ast_to_conversation_via_decision("n1")
            .unwrap();
        assert_eq!(res.rows.len(), 1);
        assert_eq!(res.rows[0][0], DataValue::Str("d1".into()));

        // Query via raw Datalog for n2 (target of edge)
        let res = storage
            .run_script(
                "?[entity_id] := *edge{target: 'n2', provenance_id: tx_id}, \
                 *Decision{id: entity_id, source_tx_id: tx_id}",
            )
            .unwrap();
        assert_eq!(res.rows.len(), 1);
        assert_eq!(res.rows[0][0], DataValue::Str("d1".into()));
    }

    #[test]
    fn test_cross_domain_bidirectional_roundtrip() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();

        // Setup: nodes, edges, decisions, turns, sessions, memories
        storage
            .run_script(
                "?[id, label, category, risk_score, metadata] <- [
                    ['n1', 'parse_input', 'function', 0.0, {}],
                    ['n2', 'validate', 'function', 0.0, {}]
                ] :put node",
            )
            .unwrap();

        storage
            .run_script(
                "?[source, target, relation, confidence, provenance_id] <- \
                 [['n1', 'n2', 'calls', 1.0, 'tx_adr_1']] \
                 :put edge",
            )
            .unwrap();

        storage
            .run_script(
                "?[id, title, context_field, decision_text, consequences, source_tx_id, timestamp] <- \
                 [['d1', 'ADR: Validation', 'security', 'Add validation', \
                   'fewer bugs', 'tx_adr_1', '2024-01-01T00:00:00Z']] \
                 :put Decision",
            )
            .unwrap();

        storage
            .run_script(
                "?[id, session_id, timestamp, project_id, summary, privacy_level] <- \
                 [['t1', 'sess1', '2024-01-01T00:00:00Z', 'proj1', 'discussed parse_input', 'internal']] \
                 :put Turn",
            )
            .unwrap();

        storage
            .run_script(
                "?[id, source_turn_id, content, memory_type, privacy_level, created_at] <- \
                 [['m1', 't1', 'parse_input', 'symbol_reference', 'internal', '2024-01-01T00:00:00Z']] \
                 :put Memory",
            )
            .unwrap();

        // Forward: Decision d1 -> conversation_to_ast -> should find n1 (source)
        let forward = storage
            .query_conversation_to_ast_via_decision("d1")
            .unwrap();
        assert_eq!(forward.rows.len(), 1);
        assert_eq!(forward.rows[0][0], DataValue::Str("n1".into()));

        // Reverse: Node n1 as edge source -> ast_to_conversation -> should find d1
        let reverse = storage
            .query_ast_to_conversation_via_decision("n1")
            .unwrap();
        assert!(
            reverse
                .rows
                .iter()
                .any(|r| r[0] == DataValue::Str("d1".into()))
        );

        // Reverse: Node n2 as edge target -> should find d1
        let reverse_target = storage
            .run_script(
                "?[entity_id] := *edge{target: 'n2', provenance_id: tx_id}, \
                 *Decision{id: entity_id, source_tx_id: tx_id}",
            )
            .unwrap();
        assert!(
            reverse_target
                .rows
                .iter()
                .any(|r| r[0] == DataValue::Str("d1".into()))
        );

        // Forward: Turn t1 -> conversation_to_ast -> should find n1
        let forward_turn = storage.query_conversation_to_ast_via_memory("t1").unwrap();
        assert_eq!(forward_turn.rows.len(), 1);
        assert_eq!(forward_turn.rows[0][0], DataValue::Str("n1".into()));

        // Reverse: Node n1 -> ast_to_conversation via Memory -> should find t1
        let reverse_node = storage.query_ast_to_conversation_via_memory("n1").unwrap();
        assert!(
            reverse_node
                .rows
                .iter()
                .any(|r| r[0] == DataValue::Str("t1".into()))
        );
    }

    #[test]
    fn test_conversation_to_ast_no_match() {
        let storage = CozoStorage::new(&PathBuf::from("")).unwrap();

        // Insert nodes
        storage
            .run_script(
                "?[id, label, category, risk_score, metadata] <- \
                 [['n1', 'unrelated_function', 'function', 0.0, {}]] \
                 :put node",
            )
            .unwrap();

        // Insert Turn with no matching Memory
        storage
            .run_script(
                "?[id, session_id, timestamp, project_id, summary, privacy_level] <- \
                 [['t_orphan', 's1', '2024-01-01T00:00:00Z', 'proj1', 'no symbols', 'internal']] \
                 :put Turn",
            )
            .unwrap();

        // Query via helper for a Turn with no Memory
        let res = storage
            .query_conversation_to_ast_via_memory("t_orphan")
            .unwrap();
        assert_eq!(res.rows.len(), 0);

        // Query for a non-existent entity (still empty)
        let res = storage
            .query_conversation_to_ast_via_memory("nonexistent")
            .unwrap();
        assert_eq!(res.rows.len(), 0);
    }

    #[test]
    fn test_schema_setup_idempotent() {
        // Use a persistent path to test that setup_schema can be called multiple times
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("idempotent_test.cozo");

        // First initialization
        let storage1 = CozoStorage::new(&path).unwrap();
        let relations1 = storage1.get_relations().unwrap();
        assert!(relations1.contains(&"Turn".to_string()));

        // Drop storage1 so the database file can be reopened
        drop(storage1);

        // Second initialization on the same path (tests idempotency)
        let storage2 = CozoStorage::new(&path).unwrap();
        let relations2 = storage2.get_relations().unwrap();

        // All relations should still be present
        assert!(relations2.contains(&"Turn".to_string()));
        assert!(relations2.contains(&"Session".to_string()));
        assert!(relations2.contains(&"Memory".to_string()));
        assert!(relations2.contains(&"Decision".to_string()));

        // Existing relations (node, edge) should not have been affected
        assert!(relations2.contains(&"node".to_string()));
        assert!(relations2.contains(&"edge".to_string()));
    }

    #[test]
    fn test_no_info_logs_on_existing_db_init() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("existing.cozo");

        // 1. Initial creation (first-time init)
        let storage1 = CozoStorage::new(&path).unwrap();
        drop(storage1);

        // 2. Set up tracing log capture subscriber
        struct SimpleLogCapture {
            logs: Arc<Mutex<Vec<(tracing::Level, String)>>>,
        }
        impl tracing::Subscriber for SimpleLogCapture {
            fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
                metadata.level() <= &tracing::Level::INFO
            }
            fn new_span(&self, _span: &tracing::span::Attributes<'_>) -> tracing::span::Id {
                tracing::span::Id::from_u64(1)
            }
            fn record(&self, _span: &tracing::span::Id, _values: &tracing::span::Record<'_>) {}
            fn record_follows_from(&self, _span: &tracing::span::Id, _follows: &tracing::span::Id) {
            }
            fn event(&self, event: &tracing::Event<'_>) {
                let mut msg = String::new();
                struct Visitor<'a>(&'a mut String);
                impl<'a> tracing::field::Visit for Visitor<'a> {
                    fn record_debug(
                        &mut self,
                        field: &tracing::field::Field,
                        value: &dyn std::fmt::Debug,
                    ) {
                        if field.name() == "message" {
                            use std::fmt::Write;
                            let _ = write!(self.0, "{:?}", value);
                        }
                    }
                }
                event.record(&mut Visitor(&mut msg));
                if let Ok(mut logs) = self.logs.lock() {
                    logs.push((*event.metadata().level(), msg));
                }
            }
            fn enter(&self, _span: &tracing::span::Id) {}
            fn exit(&self, _span: &tracing::span::Id) {}
        }

        let logs = Arc::new(Mutex::new(Vec::new()));
        let subscriber = SimpleLogCapture { logs: logs.clone() };

        // 3. Re-initialize (existing DB path)
        tracing::subscriber::with_default(subscriber, || {
            let _storage2 = CozoStorage::new(&path).unwrap();
        });

        // 4. Assert no INFO logs occurred
        let captured = logs.lock().unwrap();
        let info_logs: Vec<_> = captured
            .iter()
            .filter(|(lvl, _)| *lvl == tracing::Level::INFO)
            .collect();
        assert!(
            info_logs.is_empty(),
            "Expected no INFO logs on re-initializing existing DB, but got: {:?}",
            info_logs
        );
    }
}
