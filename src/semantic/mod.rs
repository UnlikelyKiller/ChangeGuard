pub mod chunker;
pub mod embedder;
pub mod hotspots;
pub mod vector_store;

use crate::config::model::LocalModelConfig;
use crate::semantic::chunker::AstChunker;
use crate::semantic::embedder::SemanticEmbedder;
use crate::semantic::vector_store::VectorStore;
use crate::state::storage_cozo::CozoStorage;
use crate::search::code_tokenizer::{get_rust_tokenizer, get_typescript_tokenizer, get_go_tokenizer};
use miette::Result;
use std::path::Path;

pub struct SemanticDiscovery<'a> {
    embedder: SemanticEmbedder,
    vector_store: VectorStore<'a>,
}

impl<'a> SemanticDiscovery<'a> {
    pub fn new(mut config: LocalModelConfig, storage: &'a CozoStorage) -> Result<Self> {
        if config.dimensions == 0 {
            config.dimensions = 384;
        }
        let dim = config.dimensions;
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

        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let tokens = match extension {
            "rs" => Some(get_rust_tokenizer().tokenize(content)),
            "ts" | "tsx" => Some(get_typescript_tokenizer().tokenize(content)),
            "go" => Some(get_go_tokenizer().tokenize(content)),
            _ => None,
        };

        if let Some(tokens) = tokens {
            // Index tokens in CozoDB FTS (or just use them to enrich the metadata)
            // For now, we are just implementing the logic. 
            // The VectorStore::index_chunks could be updated to accept tokens.
            tracing::info!("Extracted {} tokens from {}", tokens.len(), path.display());
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
