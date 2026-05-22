use crate::embed::similarity::cosine_sim;
use crate::semantic::chunker::AstChunk;
use crate::state::storage_cozo::CozoStorage;
use cozo::{DataValue, Num};
use miette::{Result, miette};
use tracing::{info, warn};

pub struct VectorStore<'a> {
    storage: &'a CozoStorage,
    dim: usize,
    skip_hnsw: bool,
}

impl<'a> VectorStore<'a> {
    pub fn new(storage: &'a CozoStorage, dim: usize, skip_hnsw: bool) -> Result<Self> {
        let store = Self {
            storage,
            dim,
            skip_hnsw,
        };
        store.setup_schema()?;
        Ok(store)
    }

    /// Creates a VectorStore without building the HNSW index.
    /// Intended for testing the cos_dist fallback path and for environments
    /// where the index will be created separately (e.g., after migration).
    #[doc(hidden)]
    pub fn new_without_hnsw(storage: &'a CozoStorage, dim: usize) -> Result<Self> {
        let store = Self {
            storage,
            dim,
            skip_hnsw: true,
        };
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

            if !self.skip_hnsw {
                let hnsw_script = format!(
                    "::hnsw create snippet_embedding:snippet_idx {{dim:{},dtype:F32,fields:[embedding],distance:L2,m:16,ef_construction:200}}",
                    self.dim
                );
                self.storage.run_script(&hnsw_script)?;
                info!("HNSW index snippet_embedding:snippet_idx created");
            }

            // --- Track 54-1: FTS Index for Snippets ---
            self.storage.run_script(
                "::fts create snippet_embedding:fts_idx {extractor: name, tokenizer: Simple}",
            )?;
        } else {
            // Verify existing dimension
            self.storage
                .verify_embedding_dimension("snippet_embedding", self.dim)?;

            if !self.skip_hnsw {
                let indices = self.storage.get_indices("snippet_embedding")?;
                if !indices.contains(&"snippet_idx".to_string()) {
                    let hnsw_script = format!(
                        "::hnsw create snippet_embedding:snippet_idx {{dim:{},dtype:F32,fields:[embedding],distance:L2,m:16,ef_construction:200}}",
                        self.dim
                    );
                    self.storage.run_script(&hnsw_script)?;
                    info!("HNSW index snippet_embedding:snippet_idx created on existing relation");
                }
            }
        }
        Ok(())
    }

    pub fn get_vector_count(&self) -> Result<usize> {
        let relations = self.storage.get_relations()?;
        if !relations.contains(&"snippet_embedding".to_string()) {
            return Ok(0);
        }
        let script = "?[count(file_path)] := *snippet_embedding{file_path}";
        let res = self.storage.run_script(script)?;
        if let Some(row) = res.rows.first()
            && let Some(DataValue::Num(Num::Int(count))) = row.first()
        {
            return Ok(*count as usize);
        }
        Ok(0)
    }

    pub fn index_chunks(&self, chunks: Vec<AstChunk>, embeddings: Vec<Vec<f32>>) -> Result<()> {
        if chunks.len() != embeddings.len() {
            return Err(miette!("Mismatch between chunks and embeddings length"));
        }

        use cozo::ScriptMutability;
        use std::collections::BTreeMap;

        let mut data_rows = Vec::new();
        for (chunk, embedding) in chunks.into_iter().zip(embeddings) {
            let embedding = normalize_vector(embedding);
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

        let query_vector = normalize_vector(query_vector);
        let mut params = BTreeMap::new();
        params.insert(
            "query_vec".to_string(),
            DataValue::Vec(Box::new(cozo::Vector::F32(query_vector.clone().into()))),
        );

        // Tier 1: HNSW candidate generation with exact Cozo-side cosine reranking.
        let candidate_k = k.saturating_mul(10).max(50);
        let hnsw_script = format!(
            "?[file_path,name,line_offset,dist] := ~snippet_embedding:snippet_idx{{file_path,name,line_offset|query:$query_vec,k:{candidate_k},ef:100}}, *snippet_embedding{{file_path,name,line_offset,embedding}}, dist = cos_dist(embedding, $query_vec) :order +dist :limit {k}"
        );
        let res = self.storage.run_script_with_params(
            &hnsw_script,
            params.clone(),
            ScriptMutability::Immutable,
        );

        match res {
            Ok(r) => {
                info!("Semantic query served by HNSW index");
                return parse_hnsw_results(r);
            }
            Err(e)
                if e.to_string().contains("hnsw_index_not_found")
                    || e.to_string().contains("no_implementation") =>
            {
                warn!("HNSW index unavailable, falling back to Cozo-native cos_dist.");
                // Fall through to Tier 2
            }
            Err(e) => return Err(e),
        }

        // Tier 2: CozoDB-native cos_dist query (no materialization needed)
        let cos_dist_script = format!(
            "?[file_path,name,line_offset,dist] := *snippet_embedding{{file_path,name,line_offset,embedding}}, dist = cos_dist(embedding, $query_vec) :order +dist :limit {}",
            k
        );
        let cos_res = self.storage.run_script_with_params(
            &cos_dist_script,
            params.clone(),
            ScriptMutability::Immutable,
        );

        match cos_res {
            Ok(r) => {
                info!("Semantic query served by Cozo-native cos_dist");
                return parse_hnsw_results(r);
            }
            Err(e) if e.to_string().contains("no_implementation") => {
                warn!("Cozo-native cos_dist unavailable, falling back to Rust-side cosine_sim.");
                // Fall through to Tier 3
            }
            Err(e) => return Err(e),
        }

        // Tier 3: Rust-side cosine_sim loop (last-resort safety net)
        warn!(
            "Serving semantic query via Rust-side cosine_sim (slow path) — consider running 'changeguard update --migrate' and 'changeguard index --semantic'."
        );
        let all_script = "?[file_path,name,line_offset,embedding] := *snippet_embedding{file_path,name,line_offset,embedding}";
        let all_res = self.storage.run_script(all_script)?;

        let mut scored_results = Vec::new();
        for row in all_res.rows {
            if let (
                Some(DataValue::Str(file_path)),
                Some(DataValue::Str(name)),
                Some(DataValue::Num(Num::Int(offset))),
                Some(DataValue::Vec(v)),
            ) = (row.first(), row.get(1), row.get(2), row.get(3))
            {
                let candidate_vec: Vec<f32> = match &**v {
                    cozo::Vector::F32(vec) => vec.to_vec(),
                    cozo::Vector::F64(vec) => vec.iter().map(|&x| x as f32).collect(),
                };

                if let Ok(sim) = cosine_sim(&query_vector, &candidate_vec) {
                    scored_results.push((
                        file_path.to_string(),
                        name.to_string(),
                        *offset as usize,
                        sim,
                    ));
                }
            }
        }

        scored_results.sort_by(|a, b| {
            b.3.partial_cmp(&a.3)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        if scored_results.len() > k {
            scored_results.truncate(k);
        }

        // Return cos_dist values (1.0 - sim) for consistency with the HNSW/cos_dist paths
        Ok(scored_results
            .into_iter()
            .map(|(f, n, o, s)| (f, n, o, 1.0 - s))
            .collect())
    }
}

fn normalize_vector(mut vector: Vec<f32>) -> Vec<f32> {
    let norm = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in &mut vector {
            *value /= norm;
        }
    }
    vector
}

fn parse_hnsw_results(res: cozo::NamedRows) -> Result<Vec<(String, String, usize, f32)>> {
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
