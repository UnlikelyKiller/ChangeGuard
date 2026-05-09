use crate::semantic::chunker::AstChunk;
use crate::state::storage_cozo::CozoStorage;
use cozo::{DataValue, Num};
use miette::{miette, Result};

pub struct VectorStore<'a> {
    storage: &'a CozoStorage,
    dim: usize,
}

impl<'a> VectorStore<'a> {
    pub fn new(storage: &'a CozoStorage, dim: usize) -> Result<Self> {
        let store = Self { storage, dim };
        store.setup_schema()?;
        Ok(store)
    }

    fn setup_schema(&self) -> Result<()> {
        let relations = self.storage.get_relations()?;
        if !relations.contains(&"snippet_embedding".to_string()) {
            let script = format!(
                ":create snippet_embedding {{ file_path: String, name: String, offset: Int => embedding: <F32; {}> }}",
                self.dim
            );
            self.storage.run_script(&script)?;

            let hnsw_script = format!(
                "::hnsw create snippet_embedding:snippet_idx {{ dim: {}, fields: [embedding], distance: Cosine }}",
                self.dim
            );
            self.storage.run_script(&hnsw_script)?;
        }
        Ok(())
    }

    pub fn index_chunks(&self, chunks: Vec<AstChunk>, embeddings: Vec<Vec<f32>>) -> Result<()> {
        if chunks.len() != embeddings.len() {
            return Err(miette!("Mismatch between chunks and embeddings length"));
        }

        let mut rows = Vec::new();
        for (chunk, embedding) in chunks.into_iter().zip(embeddings.into_iter()) {
            let embedding_json = serde_json::to_string(&embedding).unwrap();
            let row = format!(
                "['{}', '{}', {}, <F32; {}> {}]",
                chunk.file_path.replace("'", "''"),
                chunk.name.replace("'", "''"),
                chunk.offset,
                self.dim,
                embedding_json
            );
            rows.push(row);
        }

        if rows.is_empty() {
            return Ok(());
        }

        let script = format!(
            "?[file_path, name, offset, embedding] <- [{}] :put snippet_embedding",
            rows.join(", ")
        );
        self.storage.run_script(&script)?;
        Ok(())
    }

    pub fn query(
        &self,
        query_vector: Vec<f32>,
        k: usize,
    ) -> Result<Vec<(String, String, usize, f32)>> {
        let query_vec_json = serde_json::to_string(&query_vector).unwrap();
        let script = format!(
            "?[file_path, name, offset, dist] := ~snippet_embedding:snippet_idx {{ file_path, name, offset | query: <F32; {}> {}, k: {}, bind_distance: dist }}",
            self.dim, query_vec_json, k
        );
        let res = self.storage.run_script(&script)?;

        let mut results = Vec::new();
        for row in res.rows {
            if let (
                Some(DataValue::Str(file_path)),
                Some(DataValue::Str(name)),
                Some(DataValue::Num(Num::Int(offset))),
                Some(DataValue::Num(Num::Float(dist))),
            ) = (row.get(0), row.get(1), row.get(2), row.get(3))
            {
                results.push((
                    file_path.to_string(),
                    name.to_string(),
                    *offset as usize,
                    *dist as f32,
                ));
            }
        }
        Ok(results)
    }
}
