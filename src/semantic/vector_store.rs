use crate::semantic::chunker::AstChunk;
use crate::state::storage_cozo::CozoStorage;
use cozo::{DataValue, Num};
use miette::{IntoDiagnostic, Result, miette};

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
                ":create snippet_embedding {{file_path,name,line_offset=>embedding:<F32; {}>}}",
                self.dim
            );
            self.storage.run_script(&script)?;

            let hnsw_script = format!(
                "::hnsw create snippet_embedding:snippet_idx {{dim:{},fields:[embedding],distance:Cosine,m:16,ef_construction:200}}",
                self.dim
            );
            self.storage.run_script(&hnsw_script)?;

            // --- Track 54-1: FTS Index for Snippets ---
            self.storage.run_script("::fts create snippet_embedding:fts_idx {extractor: name, tokenizer: Simple}")?;
        }
        Ok(())
    }

    pub fn index_chunks(&self, chunks: Vec<AstChunk>, embeddings: Vec<Vec<f32>>) -> Result<()> {
        if chunks.len() != embeddings.len() {
            return Err(miette!("Mismatch between chunks and embeddings length"));
        }

        use cozo::ScriptMutability;
        use std::collections::BTreeMap;

        let mut data_rows = Vec::new();
        for (chunk, embedding) in chunks.into_iter().zip(embeddings) {
            let row = vec![
                DataValue::from(chunk.file_path),
                DataValue::from(chunk.name),
                DataValue::from(chunk.offset as i64),
                DataValue::Vec(Box::new(cozo::Vector::F32(embedding.into()))),
            ];
            data_rows.push(DataValue::from(row));
        }

        if data_rows.is_empty() {
            return Ok(());
        }

        let mut params = BTreeMap::new();
        params.insert("data".to_string(), DataValue::from(data_rows));

        let script = "?[file_path, name, line_offset, embedding] <- $data :put snippet_embedding";
        self.storage
            .run_script_with_params(script, params, ScriptMutability::Mutable)?;
        Ok(())
    }

    pub fn query(
        &self,
        query_vector: Vec<f32>,
        k: usize,
    ) -> Result<Vec<(String, String, usize, f32)>> {
        use cozo::ScriptMutability;
        use std::collections::BTreeMap;

        let mut params = BTreeMap::new();
        params.insert(
            "query_vec".to_string(),
            DataValue::Vec(Box::new(cozo::Vector::F32(query_vector.into()))),
        );

        let script = format!(
            "?[file_path,name,line_offset,dist]:=~snippet_embedding:snippet_idx{{file_path,name,line_offset|query:$query_vec,k:{},ef:100,bind_distance:dist}}",
            k
        );
        let res =
            self.storage
                .run_script_with_params(&script, params, ScriptMutability::Immutable)?;

        let mut results = Vec::new();
        for row in res.rows {
            if let (
                Some(DataValue::Str(file_path)),
                Some(DataValue::Str(name)),
                Some(DataValue::Num(Num::Int(offset))),
                Some(DataValue::Num(Num::Float(dist))),
            ) = (row.first(), row.get(1), row.get(2), row.get(3))
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
