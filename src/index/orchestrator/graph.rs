use super::ProjectIndexer;
use miette::Result;
use tracing::{info, warn};

pub fn build_kg_native(
    indexer: &ProjectIndexer,
    local_model_config: &crate::config::model::LocalModelConfig,
    gemini_config: &crate::config::model::GeminiConfig,
    enable_semantic: bool,
    fast: bool,
) -> Result<()> {
    let Some(cozo) = &indexer.storage.cozo else {
        info!("CozoDB not available, skipping native KG build");
        return Ok(());
    };

    let stats = crate::index::graph_loader::build_native_graph(
        &indexer.storage,
        cozo,
        "native_kg",
        &indexer.config,
    )?;

    if enable_semantic {
        match super::discovery::get_semantic_sample_files(indexer) {
            Ok(sample_files) if !sample_files.is_empty() => {
                info!(
                    "Running semantic enrichment on {} sample files via LLM...",
                    sample_files.len()
                );
                let extractor = crate::ai::semantic_extractor::SemanticExtractor::new(
                    crate::ai::semantic_extractor::SemanticExtractorConfig {
                        fast,
                        ..Default::default()
                    },
                );
                match extractor.extract_batch(sample_files, local_model_config, gemini_config) {
                    Ok(result) => {
                        info!(
                            "Semantic extraction complete: {} nodes, {} edges ({} input tokens, {} output tokens)",
                            result.nodes.len(),
                            result.edges.len(),
                            result.input_tokens,
                            result.output_tokens,
                        );
                        if let Err(e) =
                            crate::ai::semantic_extractor::SemanticExtractor::ingest_into_cozo(
                                &result,
                                cozo,
                                "semantic_kg",
                            )
                        {
                            warn!("Semantic extraction ingestion failed: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("Semantic extraction failed: {}", e);
                    }
                }
            }
            Ok(_) => {
                info!("No parsed source files found; skipping semantic enrichment.");
            }
            Err(e) => {
                warn!("Failed to collect semantic sample files: {}", e);
            }
        }
    } else {
        info!("Semantic enrichment skipped (pass --semantic to enable LLM-based extraction).");
    }

    let communities = crate::index::graph_loader::run_community_louvain(cozo)?;
    let node_count = cozo.node_count()?;
    let edge_count = cozo.edge_count()?;

    info!(
        "Native KG build complete: {} nodes, {} edges, {} communities ({} files, {} symbols)",
        node_count,
        edge_count,
        communities.len(),
        stats.files_indexed,
        stats.symbols_indexed
    );

    Ok(())
}
