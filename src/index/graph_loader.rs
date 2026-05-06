use crate::state::storage_cozo::CozoStorage;
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::Path;
use tracing::info;

#[derive(Debug, Deserialize, Serialize)]
pub struct GraphJsonNode {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub file_type: String,
    #[serde(default)]
    pub source_file: Option<String>,
    #[serde(default)]
    pub source_location: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GraphJsonEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
    #[serde(default = "default_confidence")]
    pub confidence: String,
}

fn default_confidence() -> String {
    "EXTRACTED".to_string()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GraphJson {
    pub nodes: Vec<GraphJsonNode>,
    pub edges: Vec<GraphJsonEdge>,
}

pub fn ingest_graphify_json(
    json_path: &Path,
    cozo: &CozoStorage,
    provenance_id: &str,
) -> Result<()> {
    let content = fs::read_to_string(json_path).into_diagnostic()?;
    let graph: GraphJson = serde_json::from_str(&content).into_diagnostic()?;

    info!(
        "Ingesting graph from {:?}: {} nodes, {} edges",
        json_path,
        graph.nodes.len(),
        graph.edges.len()
    );

    // 1. Ingest Nodes
    let mut node_batch = Vec::new();
    for node in &graph.nodes {
        let category = if node.file_type.is_empty() {
            "code"
        } else {
            &node.file_type
        };
        let metadata = json!({
            "source_file": node.source_file,
            "source_location": node.source_location
        });
        node_batch.push(json!([
            node.id, node.label, category, 0.0, // initial risk score
            metadata
        ]));
    }

    if !node_batch.is_empty() {
        let script = format!(
            "?[id, label, category, risk_score, metadata] <- {} :put node",
            serde_json::to_string(&node_batch).into_diagnostic()?
        );
        cozo.run_script(&script)?;
    }

    // 2. Ingest Edges
    let mut edge_batch = Vec::new();
    for edge in &graph.edges {
        let confidence_val = match edge.confidence.as_str() {
            "EXTRACTED" => 1.0,
            "INFERRED" => 0.7,
            "AMBIGUOUS" => 0.4,
            _ => 1.0,
        };
        edge_batch.push(json!([
            edge.source,
            edge.target,
            edge.relation,
            confidence_val,
            provenance_id
        ]));
    }

    if !edge_batch.is_empty() {
        // We use :put to insert or update
        let script = format!(
            "?[source, target, relation, confidence, provenance_id] <- {} :put edge",
            serde_json::to_string(&edge_batch).into_diagnostic()?
        );
        cozo.run_script(&script)?;
    }

    info!("Graph ingestion complete.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::storage_cozo::CozoStorage;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_ingest_graph_json() {
        let cozo = CozoStorage::new(&PathBuf::from("")).unwrap();
        let dir = tempdir().unwrap();
        let json_path = dir.path().join("graph.json");

        let json_content = r#"{
            "nodes": [
                {"id": "node_1", "label": "Label 1", "file_type": "code"},
                {"id": "node_2", "label": "Label 2", "file_type": "doc"}
            ],
            "edges": [
                {"source": "node_1", "target": "node_2", "relation": "references", "confidence": "INFERRED"}
            ]
        }"#;
        fs::write(&json_path, json_content).unwrap();

        ingest_graphify_json(&json_path, &cozo, "tx_999").unwrap();

        // Verify nodes
        let res = cozo.run_script("?[id] := *node{id: id}").unwrap();
        assert_eq!(res.rows.len(), 2);

        // Verify edge
        let res = cozo
            .run_script(
                "?[target, conf] := *edge{source: 'node_1', target: target, confidence: conf}",
            )
            .unwrap();
        assert_eq!(res.rows.len(), 1);
        if let cozo::DataValue::Str(s) = &res.rows[0][0] {
            assert_eq!(s.as_str(), "node_2");
        } else {
            panic!("Expected String target");
        }
        // 0.7 for INFERRED
        match &res.rows[0][1] {
            cozo::DataValue::Num(cozo::Num::Float(f)) => assert!((f - 0.7).abs() < 0.001),
            _ => panic!("Expected Num::Float confidence, got {:?}", res.rows[0][1]),
        }
    }
}
