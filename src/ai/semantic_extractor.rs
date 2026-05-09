use crate::config::model::LocalModelConfig;
use crate::local_model::client::{ChatMessage, CompletionOptions, complete};
use crate::state::storage_cozo::CozoStorage;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::warn;

#[derive(Debug, Clone)]
pub struct SemanticNode {
    pub id: String,
    pub label: String,
    pub category: String,
    pub source_file: String,
    pub source_location: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SemanticEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct ExtractionResult {
    pub nodes: Vec<SemanticNode>,
    pub edges: Vec<SemanticEdge>,
    pub input_tokens: usize,
    pub output_tokens: usize,
}

#[derive(Debug, Clone)]
pub struct SemanticExtractorConfig {
    pub max_tokens_per_chunk: usize,
    pub model_context_window: usize,
    pub overlap_chars: usize,
    pub max_retries: usize,
    pub enable_adaptive_recursion: bool,
}

impl Default for SemanticExtractorConfig {
    fn default() -> Self {
        Self {
            max_tokens_per_chunk: 30_000,
            model_context_window: 4096,
            overlap_chars: 500,
            max_retries: 3,
            enable_adaptive_recursion: true,
        }
    }
}

pub struct SemanticExtractor {
    config: SemanticExtractorConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct LlmNode {
    id: String,
    label: String,
    category: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct LlmEdge {
    source: String,
    target: String,
    relation: String,
    confidence: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct LlmResponse {
    nodes: Vec<LlmNode>,
    edges: Vec<LlmEdge>,
}

const EXTRACTION_PROMPT: &str = r#"Analyze the following source code and extract semantic nodes and edges.

Return ONLY valid JSON matching this exact schema:

{
  "nodes": [
    {"id": "qualified::name", "label": "brief semantic description", "category": "function_concept|data_model|business_logic|infrastructure|utility"}
  ],
  "edges": [
    {"source": "id1", "target": "id2", "relation": "depends_on|implements|orchestrates|reads_from|calls", "confidence": 0.95}
  ]
}

Categories:
- function_concept: A function, method, or callable concept
- data_model: A struct, enum, type alias, or database schema
- business_logic: Core domain logic, rules, or workflows
- infrastructure: Configuration, build scripts, deployment, or tooling
- utility: Helper functions, formatting, logging, or generic utilities

Relations:
- depends_on: One concept depends on another
- implements: A concept implements an interface or trait
- orchestrates: A concept coordinates or manages other concepts
- reads_from: A concept reads data from another
- calls: A function or method calls another

Source code:
```"#;

impl SemanticExtractor {
    pub fn new(config: SemanticExtractorConfig) -> Self {
        Self { config }
    }

    pub fn extract_from_file(
        &self,
        path: &Path,
        content: &str,
        local_model_config: &LocalModelConfig,
    ) -> Result<ExtractionResult, String> {
        let chunks = self.chunk_for_extraction(content);
        let mut all_nodes = Vec::new();
        let mut all_edges = Vec::new();
        let mut total_input_tokens = 0;
        let mut total_output_tokens = 0;

        for chunk in chunks {
            let chunk_input_tokens = chunk.chars().count().div_ceil(4);
            total_input_tokens += chunk_input_tokens;

            match self.call_llm(path, &chunk, local_model_config) {
                Ok((partial, output_tokens)) => {
                    total_output_tokens += output_tokens;
                    all_nodes.extend(partial.nodes);
                    all_edges.extend(partial.edges);
                }
                Err(e) => {
                    warn!(
                        "LLM extraction failed for chunk in {}: {}",
                        path.display(),
                        e
                    );
                }
            }
        }

        let (nodes, edges) = deduplicate(all_nodes, all_edges);
        Ok(ExtractionResult {
            nodes,
            edges,
            input_tokens: total_input_tokens,
            output_tokens: total_output_tokens,
        })
    }

    pub fn extract_batch(
        &self,
        files: Vec<(PathBuf, String)>,
        local_model_config: &LocalModelConfig,
    ) -> Result<ExtractionResult, String> {
        let mut all_nodes = Vec::new();
        let mut all_edges = Vec::new();
        let mut total_input_tokens = 0;
        let mut total_output_tokens = 0;

        for (path, content) in files {
            let result = self.extract_from_file(&path, &content, local_model_config)?;
            total_input_tokens += result.input_tokens;
            total_output_tokens += result.output_tokens;
            all_nodes.extend(result.nodes);
            all_edges.extend(result.edges);
        }

        let (nodes, edges) = deduplicate(all_nodes, all_edges);
        Ok(ExtractionResult {
            nodes,
            edges,
            input_tokens: total_input_tokens,
            output_tokens: total_output_tokens,
        })
    }

    pub fn ingest_into_cozo(
        result: &ExtractionResult,
        cozo: &CozoStorage,
        provenance_id: &str,
    ) -> miette::Result<()> {
        let mut node_batch = Vec::new();
        for node in &result.nodes {
            let metadata = json!({
                "source_file": node.source_file,
                "source_location": node.source_location
            });
            node_batch.push(json!([
                node.id.clone(),
                node.label.clone(),
                node.category.clone(),
                0.0,
                metadata
            ]));
        }

        if !node_batch.is_empty() {
            let script = format!(
                "?[id, label, category, risk_score, metadata] <- {} :put node",
                serde_json::to_string(&node_batch)
                    .map_err(|e| miette::miette!("Failed to serialize nodes: {}", e))?
            );
            cozo.run_script(&script)?;
        }

        let mut edge_batch = Vec::new();
        for edge in &result.edges {
            edge_batch.push(json!([
                edge.source.clone(),
                edge.target.clone(),
                edge.relation.clone(),
                edge.confidence,
                provenance_id
            ]));
        }

        if !edge_batch.is_empty() {
            let script = format!(
                "?[source, target, relation, confidence, provenance_id] <- {} :put edge",
                serde_json::to_string(&edge_batch)
                    .map_err(|e| miette::miette!("Failed to serialize edges: {}", e))?
            );
            cozo.run_script(&script)?;
        }

        Ok(())
    }

    fn chunk_for_extraction(&self, content: &str) -> Vec<String> {
        let max_chars = self.config.max_tokens_per_chunk * 4;
        if content.len() <= max_chars {
            return vec![content.to_string()];
        }
        chunk_content(content, max_chars, self.config.overlap_chars)
    }

    fn call_llm(
        &self,
        path: &Path,
        chunk: &str,
        local_model_config: &LocalModelConfig,
    ) -> Result<(ExtractionResult, usize), String> {
        let system_msg = ChatMessage {
            role: "system".to_string(),
            content: "You are a semantic code analysis engine that returns only JSON.".to_string(),
        };
        let options = CompletionOptions {
            max_tokens: self.config.model_context_window,
            temperature: 0.1,
        };

        let mut last_error = String::new();
        let mut attempt = 0;
        let mut current_chunk = chunk.to_string();

        while attempt < self.config.max_retries {
            attempt += 1;
            let prompt = format!("{}{}\n```", EXTRACTION_PROMPT, current_chunk);
            let messages = vec![
                system_msg.clone(),
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ];

            match complete(local_model_config, &messages, &options) {
                Ok(response) => {
                    let output_tokens = response.chars().count().div_ceil(4);
                    let trimmed = response.trim_end();
                    if self.config.enable_adaptive_recursion
                        && (!trimmed.ends_with('}') && !trimmed.ends_with(']'))
                    {
                        warn!("LLM response appears truncated, retrying with smaller chunk");
                        if current_chunk.len() > 1000 {
                            current_chunk = current_chunk[..current_chunk.len() / 2].to_string();
                            continue;
                        }
                    }
                    match parse_llm_response(&response, path) {
                        Ok((nodes, edges)) => {
                            let partial = ExtractionResult {
                                nodes,
                                edges,
                                input_tokens: current_chunk.chars().count().div_ceil(4),
                                output_tokens,
                            };
                            return Ok((partial, output_tokens));
                        }
                        Err(e) => {
                            warn!("Failed to parse LLM JSON response: {}", e);
                            last_error = e;
                        }
                    }
                }
                Err(e) => {
                    last_error = e.clone();
                    if e.contains("503") || e.contains("rate limited") {
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                }
            }
        }

        Err(format!(
            "LLM extraction failed after {} attempts: {}",
            self.config.max_retries, last_error
        ))
    }
}

fn parse_llm_response(
    response: &str,
    path: &Path,
) -> Result<(Vec<SemanticNode>, Vec<SemanticEdge>), String> {
    let cleaned = response.trim();
    let cleaned = if cleaned.starts_with("```json") {
        cleaned
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim()
    } else if cleaned.starts_with("```") {
        cleaned
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
    } else {
        cleaned
    };

    let parsed: LlmResponse =
        serde_json::from_str(cleaned).map_err(|e| format!("JSON parse error: {}", e))?;

    let nodes: Vec<SemanticNode> = parsed
        .nodes
        .into_iter()
        .map(|n| SemanticNode {
            id: n.id,
            label: n.label,
            category: n.category,
            source_file: path.to_string_lossy().to_string(),
            source_location: None,
        })
        .collect();

    let edges: Vec<SemanticEdge> = parsed
        .edges
        .into_iter()
        .map(|e| SemanticEdge {
            source: e.source,
            target: e.target,
            relation: e.relation,
            confidence: e.confidence.clamp(0.0, 1.0),
        })
        .collect();

    Ok((nodes, edges))
}

fn chunk_content(content: &str, max_chars: usize, overlap_chars: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < content.len() {
        let end = (start + max_chars).min(content.len());
        let end = content.floor_char_boundary(end);
        let chunk = content[start..end].to_string();
        chunks.push(chunk);
        if end >= content.len() {
            break;
        }
        let next_start = end.saturating_sub(overlap_chars);
        let mut next_start = content.floor_char_boundary(next_start);
        if next_start <= start {
            next_start = content.floor_char_boundary(start + 1);
        }
        if next_start >= content.len() {
            break;
        }
        start = next_start;
    }
    chunks
}

fn deduplicate(
    nodes: Vec<SemanticNode>,
    edges: Vec<SemanticEdge>,
) -> (Vec<SemanticNode>, Vec<SemanticEdge>) {
    let mut seen_nodes: HashSet<String> = HashSet::new();
    let mut deduped_nodes = Vec::new();
    for node in nodes {
        if seen_nodes.insert(node.id.clone()) {
            deduped_nodes.push(node);
        }
    }
    deduped_nodes.sort_by(|a, b| a.id.cmp(&b.id));

    let mut seen_edges: HashSet<(String, String, String)> = HashSet::new();
    let mut deduped_edges = Vec::new();
    for edge in edges {
        let key = (
            edge.source.clone(),
            edge.target.clone(),
            edge.relation.clone(),
        );
        if seen_edges.insert(key) {
            deduped_edges.push(edge);
        }
    }
    deduped_edges.sort_by(|a, b| {
        (&a.source, &a.target, &a.relation).cmp(&(&b.source, &b.target, &b.relation))
    });

    (deduped_nodes, deduped_edges)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_chunking_splits_long_content() {
        let extractor = SemanticExtractor::new(SemanticExtractorConfig {
            max_tokens_per_chunk: 10,
            model_context_window: 4096,
            overlap_chars: 5,
            max_retries: 3,
            enable_adaptive_recursion: true,
        });
        let content = "A".repeat(100);
        let chunks = extractor.chunk_for_extraction(&content);
        assert!(
            chunks.len() > 1,
            "Expected multiple chunks, got {}",
            chunks.len()
        );
    }

    #[test]
    fn test_prompt_includes_schema() {
        assert!(EXTRACTION_PROMPT.contains("\"nodes\""));
        assert!(EXTRACTION_PROMPT.contains("\"edges\""));
        assert!(EXTRACTION_PROMPT.contains("\"id\""));
        assert!(EXTRACTION_PROMPT.contains("\"label\""));
        assert!(EXTRACTION_PROMPT.contains("\"category\""));
        assert!(EXTRACTION_PROMPT.contains("\"source\""));
        assert!(EXTRACTION_PROMPT.contains("\"target\""));
        assert!(EXTRACTION_PROMPT.contains("\"relation\""));
        assert!(EXTRACTION_PROMPT.contains("\"confidence\""));
        assert!(EXTRACTION_PROMPT.contains("function_concept"));
        assert!(EXTRACTION_PROMPT.contains("depends_on"));
    }

    #[test]
    fn test_deduplicate_nodes_and_edges() {
        let nodes = vec![
            SemanticNode {
                id: "a".to_string(),
                label: "A".to_string(),
                category: "x".to_string(),
                source_file: "f".to_string(),
                source_location: None,
            },
            SemanticNode {
                id: "a".to_string(),
                label: "A2".to_string(),
                category: "x".to_string(),
                source_file: "f".to_string(),
                source_location: None,
            },
            SemanticNode {
                id: "b".to_string(),
                label: "B".to_string(),
                category: "y".to_string(),
                source_file: "f".to_string(),
                source_location: None,
            },
        ];
        let edges = vec![
            SemanticEdge {
                source: "a".to_string(),
                target: "b".to_string(),
                relation: "calls".to_string(),
                confidence: 0.9,
            },
            SemanticEdge {
                source: "a".to_string(),
                target: "b".to_string(),
                relation: "calls".to_string(),
                confidence: 0.8,
            },
            SemanticEdge {
                source: "b".to_string(),
                target: "c".to_string(),
                relation: "reads".to_string(),
                confidence: 0.7,
            },
        ];
        let (deduped_nodes, deduped_edges) = deduplicate(nodes, edges);
        assert_eq!(deduped_nodes.len(), 2);
        assert_eq!(deduped_edges.len(), 2);
        assert_eq!(deduped_nodes[0].id, "a");
        assert_eq!(deduped_nodes[1].id, "b");
    }

    #[test]
    fn test_token_budget_enforcement() {
        let extractor = SemanticExtractor::new(SemanticExtractorConfig {
            max_tokens_per_chunk: 5,
            model_context_window: 4096,
            overlap_chars: 2,
            max_retries: 3,
            enable_adaptive_recursion: true,
        });
        let content = "B".repeat(50);
        let chunks = extractor.chunk_for_extraction(&content);
        assert!(
            chunks.len() > 1,
            "Expected multiple chunks due to budget, got {}",
            chunks.len()
        );
        for chunk in &chunks {
            assert!(
                chunk.len() <= 5 * 4,
                "Chunk exceeds char budget: {} > {}",
                chunk.len(),
                5 * 4
            );
        }
    }

    #[test]
    fn test_ingest_into_cozo() {
        let cozo = CozoStorage::new(&PathBuf::from("")).unwrap();
        let result = ExtractionResult {
            nodes: vec![
                SemanticNode {
                    id: "node1".to_string(),
                    label: "Node 1".to_string(),
                    category: "function_concept".to_string(),
                    source_file: "test.rs".to_string(),
                    source_location: None,
                },
                SemanticNode {
                    id: "node2".to_string(),
                    label: "Node 2".to_string(),
                    category: "data_model".to_string(),
                    source_file: "test.rs".to_string(),
                    source_location: Some("line 5".to_string()),
                },
            ],
            edges: vec![SemanticEdge {
                source: "node1".to_string(),
                target: "node2".to_string(),
                relation: "calls".to_string(),
                confidence: 0.95,
            }],
            input_tokens: 100,
            output_tokens: 50,
        };
        SemanticExtractor::ingest_into_cozo(&result, &cozo, "tx_test").unwrap();

        let res = cozo.run_script("?[id] := *node{id: id}").unwrap();
        assert_eq!(res.rows.len(), 2);

        let res = cozo
            .run_script("?[source, target] := *edge{source: source, target: target}")
            .unwrap();
        assert_eq!(res.rows.len(), 1);
    }
}
