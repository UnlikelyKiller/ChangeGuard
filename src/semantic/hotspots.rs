use crate::state::storage_cozo::CozoStorage;
use cozo::{DataValue, Num};
use miette::Result;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SemanticMatch {
    pub file1: String,
    pub name1: String,
    pub offset1: usize,
    pub file2: String,
    pub name2: String,
    pub offset2: usize,
    pub similarity: f32,
}

pub fn find_semantic_hotspots(storage: &CozoStorage, threshold: f32) -> Result<Vec<SemanticMatch>> {
    // Find snippets with high cosine similarity (> threshold).
    // Similarity = 1.0 - Cosine Distance.
    // We use a self-join on snippet_embedding.
    let script = format!(
        "?[f1, n1, o1, f2, n2, o2, similarity] := 
            *snippet_embedding{{file_path: f1, name: n1, offset: o1, embedding: v1}},
            *snippet_embedding{{file_path: f2, name: n2, offset: o2, embedding: v2}},
            (f1 < f2) or (f1 == f2 and o1 < o2),
            dist = vec_cosine_distance(v1, v2),
            similarity = 1.0 - dist,
            similarity > {}",
        threshold
    );

    let res = storage.run_script(&script)?;
    let mut results = Vec::new();
    for row in res.rows {
        if let (
            Some(DataValue::Str(f1)),
            Some(DataValue::Str(n1)),
            Some(DataValue::Num(Num::Int(o1))),
            Some(DataValue::Str(f2)),
            Some(DataValue::Str(n2)),
            Some(DataValue::Num(Num::Int(o2))),
            Some(DataValue::Num(num)),
        ) = (
            row.first(),
            row.get(1),
            row.get(2),
            row.get(3),
            row.get(4),
            row.get(5),
            row.get(6),
        ) {
            let sim = match num {
                Num::Float(f) => *f as f32,
                Num::Int(i) => *i as f32,
            };
            results.push(SemanticMatch {
                file1: f1.to_string(),
                name1: n1.to_string(),
                offset1: *o1 as usize,
                file2: f2.to_string(),
                name2: n2.to_string(),
                offset2: *o2 as usize,
                similarity: sim,
            });
        }
    }
    Ok(results)
}
