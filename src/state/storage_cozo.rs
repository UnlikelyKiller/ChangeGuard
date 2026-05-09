use cozo::*;
use miette::Result;
use std::path::Path;
use tracing::info;

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
