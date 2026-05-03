use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::{ImpactPacket, RelevantDecision};
use crate::retrieval::query::query_docs;
use crate::retrieval::rerank::rerank;
use miette::Result;
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

pub struct KnowledgeProvider;

impl EnrichmentProvider for KnowledgeProvider {
    fn name(&self) -> &'static str {
        "Knowledge Enrichment Provider"
    }

    fn enrich(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        let lm_config = &context.config.local_model;

        // Skip if no embedding model configured
        if lm_config.base_url.is_empty() {
            return Ok(());
        }

        // Skip if no doc paths configured
        if context.config.docs.include.is_empty() {
            return Ok(());
        }

        let conn = context.storage.get_connection();
        
        // Check if doc_chunks table has any rows
        if !context.storage.table_exists_and_has_data("doc_chunks")? {
            info!("Skipping knowledge enrichment: doc_chunks table is empty or missing.");
            return Ok(());
        }

        // Build query text from changed files
        let mut query_parts = Vec::new();
        for change in &packet.changes {
            let path = change.path.to_string_lossy().to_string();
            query_parts.push(format!("{} ({})", path, change.status));
        }
        
        if query_parts.is_empty() {
            return Ok(());
        }

        let query_text = query_parts.join("; ");

        // Truncate to reasonable query length
        let query_text = if query_text.len() > 2000 {
            format!("{}...", &query_text[..2000])
        } else {
            query_text.clone()
        };

        let top_n = context.config.docs.retrieval_top_k;

        // Query docs
        let retrieved = match query_docs(lm_config, conn, &query_text, top_n) {
            Ok(r) => r,
            Err(e) => {
                warn!("Doc retrieval query failed: {e}");
                context.add_warning(format!("Doc retrieval failed: {e}"));
                return Ok(());
            }
        };

        if retrieved.is_empty() {
            return Ok(());
        }

        // Rerank if configured
        let final_chunks = if !lm_config.rerank_model.is_empty() {
            rerank(
                &lm_config.base_url,
                &lm_config.rerank_model,
                &query_text,
                retrieved,
                lm_config.timeout_secs,
            )
        } else {
            retrieved
        };

        // Map to RelevantDecision, taking top top_n
        let decisions: Vec<RelevantDecision> = final_chunks
            .into_iter()
            .take(top_n)
            .map(|chunk| {
                let excerpt: String = chunk.content.chars().take(200).collect();
                let mut staleness_days = None;

                // Populate staleness_days from filesystem mtime
                let full_path = PathBuf::from(chunk.file_path.clone());
                if let Ok(metadata) = fs::metadata(&full_path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            staleness_days = Some((elapsed.as_secs() / 86400) as u32);
                        }
                    }
                }

                RelevantDecision {
                    file_path: PathBuf::from(chunk.file_path),
                    heading: chunk.heading,
                    excerpt,
                    similarity: chunk.similarity,
                    rerank_score: None,
                    staleness_days,
                }
            })
            .collect();

        packet.relevant_decisions = decisions;

        Ok(())
    }
}
