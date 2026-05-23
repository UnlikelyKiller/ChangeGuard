use cozo::*;
use miette::Result;
use serde_json::json;
use std::path::Path;
use tracing::debug;

use crate::state::cozo::{init, queries};

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
        Self::new_with_options(db_path, false)
    }

    pub fn new_read_only(db_path: &Path) -> Result<Self> {
        Self::new_with_options(db_path, true)
    }

    pub fn new_in_memory() -> Result<Self> {
        Self::new_with_options(Path::new(""), false)
    }

    fn new_with_options(db_path: &Path, read_only: bool) -> Result<Self> {
        let db = init::initialize_instance(db_path, read_only)?;
        let storage = Self { db };
        if !read_only {
            storage.setup_schema()?;
        }
        Ok(storage)
    }

    pub fn run_script(&self, script: &str) -> Result<NamedRows> {
        self.db
            .run_script(script, Default::default(), ScriptMutability::Mutable)
            .map_err(|e| miette::miette!("CozoDB script error: {:?}. Script was: '{}'", e, script))
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

    pub fn shutdown(self) {
        debug!("Shutting down CozoDB instance");
        drop(self.db);
    }

    pub fn setup_schema(&self) -> Result<()> {
        init::setup_schema(self)
    }

    pub fn migrate_cozo_schema(&self) -> Result<()> {
        init::migrate_cozo_schema(self)
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

    pub fn verify_embedding_dimension(&self, relation: &str, expected_dim: usize) -> Result<()> {
        let script = format!("::columns {}", relation);
        let res = self.run_script(&script)?;
        for row in res.rows {
            if let (Some(DataValue::Str(name)), Some(DataValue::Str(typ))) =
                (row.first(), row.get(1))
            {
                if name == "embedding" {
                    let expected = format!("<F32; {}>", expected_dim);
                    if typ != &expected {
                        return Err(miette::miette!(
                            "Dimension mismatch for relation '{}': expected {}, found {}.",
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
        let res = self.run_script(queries::node_count_query())?;
        if let Some(row) = res.rows.first() {
            if let Some(DataValue::Num(Num::Int(count))) = row.first() {
                return Ok(*count as usize);
            }
        }
        Ok(0)
    }

    pub fn edge_count(&self) -> Result<usize> {
        let res = self.run_script(queries::edge_count_query())?;
        if let Some(row) = res.rows.first() {
            if let Some(DataValue::Num(Num::Int(count))) = row.first() {
                return Ok(*count as usize);
            }
        }
        Ok(0)
    }

    pub fn put_node_batch(&self, nodes: &[GraphNode]) -> Result<()> {
        self.insert_nodes(nodes)
    }

    pub fn put_edge_batch(&self, edges: &[GraphEdge]) -> Result<()> {
        self.insert_edges(edges)
    }

    pub fn remove_nodes_by_id(&self, ids: &[String]) -> Result<()> {
        if ids.is_empty() { return Ok(()); }
        let mut script = String::from("?[id] <- [\n");
        for (i, id) in ids.iter().enumerate() {
            script.push_str(&format!("  ['{}']{}\n", id, if i == ids.len() - 1 { "" } else { "," }));
        }
        script.push_str("] :rm node {id}");
        self.run_script(&script)?;
        Ok(())
    }

    pub fn remove_edges_for_source(&self, source_ids: &[String]) -> Result<()> {
        if source_ids.is_empty() { return Ok(()); }
        let mut script = String::from("?[source] <- [\n");
        for (i, id) in source_ids.iter().enumerate() {
            script.push_str(&format!("  ['{}']{}\n", id, if i == source_ids.len() - 1 { "" } else { "," }));
        }
        script.push_str("] :rm edge {source}");
        self.run_script(&script)?;
        Ok(())
    }

    pub fn insert_nodes(&self, nodes: &[GraphNode]) -> Result<()> {
        if nodes.is_empty() { return Ok(()); }
        let mut script = String::from("?[id, label, category, risk_score, metadata] <- [\n");
        for (i, node) in nodes.iter().enumerate() {
            let metadata = node.metadata.as_ref().cloned().unwrap_or(json!({}));
            script.push_str(&format!(
                "  ['{}', '{}', '{}', {}, {}]{}\n",
                node.id, node.label, node.category, node.risk_score,
                metadata,
                if i == nodes.len() - 1 { "" } else { "," }
            ));
        }
        script.push_str("] :put node");
        self.run_script(&script)?;
        Ok(())
    }

    pub fn insert_edges(&self, edges: &[GraphEdge]) -> Result<()> {
        if edges.is_empty() { return Ok(()); }
        let mut script = String::from("?[source, target, relation, confidence, provenance_id] <- [\n");
        for (i, edge) in edges.iter().enumerate() {
            script.push_str(&format!(
                "  ['{}', '{}', '{}', {}, '{}']{}\n",
                edge.source, edge.target, edge.relation, edge.confidence, edge.provenance_id,
                if i == edges.len() - 1 { "" } else { "," }
            ));
        }
        script.push_str("] :put edge");
        self.run_script(&script)?;
        Ok(())
    }

    pub fn run_community_louvain(&self) -> Result<Vec<(String, i64)>> {
        let script = "::algo community_louvain {node: node, edge: edge, output: community}";
        let res = self.run_script(script)?;
        let mut results = Vec::new();
        for row in res.rows {
            if let (Some(DataValue::Str(node)), Some(DataValue::Num(Num::Int(comm)))) = (row.first(), row.get(1)) {
                results.push((node.to_string(), *comm));
            }
        }
        Ok(results)
    }
}
