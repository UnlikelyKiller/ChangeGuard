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

pub struct SemanticDiscovery<'a> {
    embedder: SemanticEmbedder,
    vector_store: VectorStore<'a>,
}

impl<'a> SemanticDiscovery<'a> {
    pub fn new(config: LocalModelConfig, storage: &'a CozoStorage) -> Result<Self> {
        let dim = if config.dimensions == 0 {
            384
        } else {
            config.dimensions
        };
        let embedder = SemanticEmbedder::new(config);
        let vector_store = VectorStore::new(storage, dim)?;
        Ok(Self {
            embedder,
            vector_store,
        })
    }

    pub fn index_file(&self, path: &Path, content: &str) -> Result<()> {
        let chunks = AstChunker::chunk_file(path, content)?;
        if chunks.is_empty() {
            return Ok(());
        }

        let texts: Vec<String> = chunks.iter().map(|c| c.to_embedding_text()).collect();
        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();

        let embeddings = self.embedder.embed_batch(&text_refs)?;
        self.vector_store.index_chunks(chunks, embeddings)?;

        Ok(())
    }

    pub fn query(&self, query_text: &str, k: usize) -> Result<Vec<(String, String, usize, f32)>> {
        let query_vector = self.embedder.embed(query_text)?;
        self.vector_store.query(query_vector, k)
    }
}
