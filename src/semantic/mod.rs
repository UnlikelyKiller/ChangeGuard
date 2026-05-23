pub mod chunker;
pub mod embedder;
pub mod hotspots;
pub mod vector_store;

use crate::config::model::LocalModelConfig;
use crate::semantic::chunker::AstChunker;
use crate::semantic::embedder::SemanticEmbedder;
use crate::semantic::vector_store::VectorStore;
use crate::state::storage_cozo::CozoStorage;
use miette::Result;
use std::path::Path;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SemanticReadiness {
    pub endpoint_available: bool,
    pub model_name: String,
    pub dimensions: usize,
    pub vector_count: usize,
    pub is_stale: bool,
    pub dimension_mismatch: bool,
}

pub struct SemanticDiscovery<'a> {
    embedder: SemanticEmbedder,
    vector_store: VectorStore<'a>,
    config: LocalModelConfig,
}

impl<'a> SemanticDiscovery<'a> {
    pub fn new(mut config: LocalModelConfig, storage: &'a CozoStorage) -> Result<Self> {
        if config.dimensions == 0 && !config.base_url.is_empty() {
            match crate::embed::client::check_local_model(&config) {
                Ok(dims) if dims.dimensions > 0 => {
                    tracing::info!(
                        "Probed local model: {} ({} dimensions)",
                        dims.model_name, dims.dimensions
                    );
                    config.dimensions = dims.dimensions;
                }
                Err(e) => {
                    tracing::warn!("Failed to probe local model at {}: {}. Defaulting to 384.", config.base_url, e);
                    config.dimensions = 384;
                }
                _ => {
                    tracing::warn!("Probed model returned zero dimensions. Defaulting to 384.");
                    config.dimensions = 384;
                }
            }
        } else if config.dimensions == 0 {
            config.dimensions = 384;
        }

        let dim = config.dimensions;
        let skip_hnsw = config.disable_hnsw;
        tracing::info!("Initializing VectorStore with {} dimensions", dim);
        let embedder = SemanticEmbedder::new(config.clone());
        let vector_store = VectorStore::new(storage, dim, skip_hnsw)?;
        Ok(Self {
            embedder,
            vector_store,
            config,
        })
    }

    pub fn check_readiness(&self) -> Result<SemanticReadiness> {
        let probe = crate::embed::client::check_local_model(&self.config);
        let endpoint_available = probe.is_ok();
        let model_name = probe
            .as_ref()
            .map(|d| d.model_name.clone())
            .unwrap_or_else(|_| self.config.embedding_model.clone());
        let model_dims = probe.as_ref().map(|d| d.dimensions).unwrap_or(0);

        let vector_count = self.vector_store.get_vector_count().unwrap_or(0);

        // Check for dimension mismatch between model and store
        let dimension_mismatch = if model_dims > 0 && self.config.dimensions > 0 {
            model_dims != self.config.dimensions
        } else {
            false
        };

        Ok(SemanticReadiness {
            endpoint_available,
            model_name,
            dimensions: self.config.dimensions,
            vector_count,
            is_stale: false, // Stale check handled at command level
            dimension_mismatch,
        })
    }

    pub fn index_file(&self, path: &Path, content: &str) -> Result<()> {
        let (chunks, embeddings) = self.process_file(path, content)?;
        if !chunks.is_empty() {
            self.vector_store.index_chunks(chunks, embeddings)?;
        }
        Ok(())
    }

    pub fn process_file(
        &self,
        path: &Path,
        content: &str,
    ) -> Result<(Vec<crate::semantic::chunker::AstChunk>, Vec<Vec<f32>>)> {
        let chunks = AstChunker::chunk_file(path, content)?;
        if chunks.is_empty() {
            return Ok((vec![], vec![]));
        }

        let texts: Vec<String> = chunks.iter().map(|c| c.to_embedding_text()).collect();
        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();

        tracing::debug!("Embedding {} chunks for {}", chunks.len(), path.display());
        let embeddings = self.embedder.embed_batch(&text_refs)?;

        if !embeddings.is_empty() {
            tracing::info!("Received {} embeddings of dimension {}", embeddings.len(), embeddings[0].len());
        }

        // Verify we got non-zero embeddings
        let zero_count = embeddings.iter().filter(|v| v.iter().all(|&x| x == 0.0)).count();
        if zero_count > 0 {
            tracing::warn!("Found {} zero-magnitude embeddings for {}", zero_count, path.display());
        }

        Ok((chunks, embeddings))
    }

    pub fn index_chunks_batched(
        &self,
        chunks: Vec<crate::semantic::chunker::AstChunk>,
        embeddings: Vec<Vec<f32>>,
    ) -> Result<()> {
        self.vector_store.index_chunks(chunks, embeddings)
    }

    pub fn query(&self, query_text: &str, k: usize) -> Result<Vec<(String, String, usize, f32)>> {
        let query_vector = self.embedder.embed(query_text)?;
        self.vector_store.query(query_vector, k)
    }
}
