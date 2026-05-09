use crate::config::model::LocalModelConfig;
use crate::embed::client::{embed_batch, embed_long_text, MAX_BATCH_SIZE};
use miette::Result;

pub struct SemanticEmbedder {
    config: LocalModelConfig,
}

impl SemanticEmbedder {
    pub fn new(config: LocalModelConfig) -> Self {
        Self { config }
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        embed_long_text(&self.config, text)
            .map_err(|e| miette::miette!(e))
    }

    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut all_vectors = Vec::with_capacity(texts.len());
        for chunk in texts.chunks(MAX_BATCH_SIZE) {
            let batch_vectors = embed_batch(
                &self.config.base_url,
                &self.config.embedding_model,
                chunk,
                self.config.timeout_secs,
            ).map_err(|e| miette::miette!(e))?;
            all_vectors.extend(batch_vectors);
        }
        Ok(all_vectors)
    }
}
