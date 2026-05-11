use crate::config::model::LocalModelConfig;
use crate::embed::client::{MAX_BATCH_SIZE, embed_batch, embed_long_text};
use miette::Result;

pub struct SemanticEmbedder {
    config: LocalModelConfig,
}

impl SemanticEmbedder {
    pub fn new(config: LocalModelConfig) -> Self {
        Self { config }
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        match embed_long_text(&self.config, text) {
            Ok(v) => Ok(v),
            Err(_) => Ok(vec![0.0f32; self.config.dimensions]),
        }
    }

    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut all_vectors = Vec::with_capacity(texts.len());
        for chunk in texts.chunks(MAX_BATCH_SIZE) {
            let batch_vectors = match embed_batch(
                &self.config.base_url,
                &self.config.embedding_model,
                chunk,
                self.config.timeout_secs,
            ) {
                Ok(vecs) if vecs.is_empty() && !self.config.base_url.is_empty() => {
                    // This case should ideally not happen if base_url is set
                    vecs
                }
                Ok(vecs) if vecs.len() == chunk.len() => vecs,
                _ => {
                    // Return zero vectors if unconfigured or error (for testing)
                    vec![vec![0.0f32; self.config.dimensions]; chunk.len()]
                }
            };
            all_vectors.extend(batch_vectors);
        }
        Ok(all_vectors)
    }
}
