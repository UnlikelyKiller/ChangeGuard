use crate::config::model::LocalModelConfig;
use crate::embed::client::embed_long_text;
use crate::embed::similarity::pairwise_cosine;
use crate::embed::storage::load_candidates;
use rusqlite::Connection;

#[derive(Debug, Clone, PartialEq)]
pub struct RetrievedChunk {
    pub entity_id: String,
    pub similarity: f32,
    pub content: String,
    pub heading: Option<String>,
    pub file_path: String,
}

impl Eq for RetrievedChunk {}

impl PartialOrd for RetrievedChunk {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RetrievedChunk {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.similarity
            .partial_cmp(&other.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| self.entity_id.cmp(&other.entity_id))
    }
}

pub fn retrieve_top_k(
    conn: &Connection,
    query_vec: &[f32],
    entity_type: &str,
    model_name: &str,
    k: usize,
) -> Result<Vec<RetrievedChunk>, String> {
    if k == 0 {
        return Ok(Vec::new());
    }

    let candidates = load_candidates(conn, entity_type, model_name)?;
    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    let scores = pairwise_cosine(query_vec, &candidates);

    // Over-fetch: top k*3 for reranker
    let overfetch = (k * 3).min(scores.len());
    let top_scores: Vec<_> = scores.into_iter().take(overfetch).collect();

    let mut results = Vec::with_capacity(top_scores.len());
    for (entity_id, similarity) in top_scores {
        if entity_type == "doc_chunk" {
            if let Ok(Some((content, heading, file_path))) = resolve_doc_chunk(conn, &entity_id) {
                results.push(RetrievedChunk {
                    entity_id,
                    similarity,
                    content,
                    heading,
                    file_path,
                });
            }
        } else {
            results.push(RetrievedChunk {
                entity_id,
                similarity,
                content: String::new(),
                heading: None,
                file_path: String::new(),
            });
        }
    }

    results.sort_unstable_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.entity_id.cmp(&b.entity_id))
    });

    Ok(results)
}

fn resolve_doc_chunk(
    conn: &Connection,
    entity_id: &str,
) -> Result<Option<(String, Option<String>, String)>, String> {
    let (file_path, chunk_index) = entity_id
        .rsplit_once("::")
        .ok_or_else(|| format!("Invalid entity_id format: {entity_id}"))?;

    let chunk_index: i64 = chunk_index
        .parse::<i64>()
        .map_err(|_| format!("Invalid chunk_index in entity_id: {entity_id}"))?;

    let row: Option<(String, Option<String>, String)> = conn
        .query_row(
            "SELECT content, heading, file_path FROM doc_chunks WHERE file_path = ?1 AND chunk_index = ?2",
            rusqlite::params![file_path, chunk_index],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                ))
            },
        )
        .ok();

    Ok(row)
}

pub fn query_docs(
    config: &LocalModelConfig,
    conn: &Connection,
    diff_text: &str,
    top_n: usize,
) -> Result<Vec<RetrievedChunk>, String> {
    if config.base_url.is_empty() || diff_text.is_empty() {
        return Ok(Vec::new());
    }

    let query_vec = embed_long_text(config, diff_text)?;
    retrieve_top_k(
        conn,
        &query_vec,
        "doc_chunk",
        &config.embedding_model,
        top_n,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::migrations::get_migrations;

    fn setup_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();
        conn
    }

    #[test]
    fn retrieve_top_k_empty_db_returns_empty() {
        let conn = setup_db();
        let query = vec![1.0_f32, 0.0, 0.0];
        let results = retrieve_top_k(&conn, &query, "doc_chunk", "test-model", 3).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn retrieve_top_k_zero_k_returns_empty() {
        let conn = setup_db();
        let query = vec![1.0_f32, 0.0, 0.0];
        let results = retrieve_top_k(&conn, &query, "doc_chunk", "test-model", 0).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn retrieve_top_k_returns_sorted_by_similarity() {
        let conn = setup_db();

        // Insert doc_chunks
        conn.execute(
            "INSERT INTO doc_chunks (file_path, chunk_index, heading, content, token_count) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["docs/a.md", 0_i64, "Intro", "some content about getting started", 10_i64],
        ).unwrap();
        conn.execute(
            "INSERT INTO doc_chunks (file_path, chunk_index, heading, content, token_count) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["docs/b.md", 0_i64, "API Reference", "api endpoint definition here", 10_i64],
        ).unwrap();
        conn.execute(
            "INSERT INTO doc_chunks (file_path, chunk_index, heading, content, token_count) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["docs/c.md", 0_i64, "Testing", "test framework and runners", 10_i64],
        ).unwrap();

        // Insert embeddings with known similarity profile
        // Query: [1.0, 0.0, 0.0] — closer to vectors with high first component
        let store_embedding = |entity_id: &str, vec: Vec<f32>| {
            let blob: Vec<u8> = vec.iter().flat_map(|f| f.to_le_bytes()).collect();
            conn.execute(
                "INSERT INTO embeddings (entity_type, entity_id, content_hash, model_name, dimensions, vector) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    "doc_chunk",
                    entity_id,
                    format!("hash-{entity_id}"),
                    "test-model",
                    3_i64,
                    blob,
                ],
            ).unwrap();
        };

        store_embedding("docs/a.md::0", vec![0.9_f32, 0.1, 0.1]); // High similarity
        store_embedding("docs/b.md::0", vec![0.5, 0.5, 0.5]); // Medium
        store_embedding("docs/c.md::0", vec![0.1, 0.9, 0.0]); // Low

        let query = vec![1.0_f32, 0.0, 0.0];
        let results = retrieve_top_k(&conn, &query, "doc_chunk", "test-model", 3).unwrap();

        assert_eq!(results.len(), 3);
        assert!(results[0].similarity > results[1].similarity);
        assert!(results[1].similarity > results[2].similarity);

        // Highest similarity should be "docs/a.md::0" (most aligned with [1,0,0])
        assert_eq!(results[0].entity_id, "docs/a.md::0");
        assert_eq!(results[0].file_path, "docs/a.md");
        assert_eq!(results[0].heading, Some("Intro".to_string()));
        assert!(!results[0].content.is_empty());
    }

    #[test]
    fn retrieve_top_k_overfetches_for_reranker() {
        let conn = setup_db();

        // Insert 5 chunks + embeddings
        for i in 0..5 {
            conn.execute(
                "INSERT INTO doc_chunks (file_path, chunk_index, heading, content, token_count) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![format!("docs/{i}.md"), 0_i64, format!("H{i}"), format!("content {i}"), 10_i64],
            ).unwrap();

            let entity_id = format!("docs/{i}.md::0");
            let val = 1.0 - (i as f32 * 0.15);
            let blob: Vec<u8> = [val, 0.1_f32, 0.1]
                .iter()
                .flat_map(|f| f.to_le_bytes())
                .collect();
            conn.execute(
                "INSERT INTO embeddings (entity_type, entity_id, content_hash, model_name, dimensions, vector) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params!["doc_chunk", &entity_id, format!("hash-{i}"), "test-model", 3_i64, blob],
            ).unwrap();
        }

        let query = vec![1.0_f32, 0.0, 0.0];
        let results = retrieve_top_k(&conn, &query, "doc_chunk", "test-model", 2).unwrap();

        // k=2, overfetch = 6 or min(6,5)=5. Returns all 5 sorted.
        assert_eq!(results.len(), 5);
        assert!(results[0].similarity >= results[1].similarity);
    }

    #[test]
    fn retrieved_chunk_ord_sorts_by_similarity_desc() {
        let a = RetrievedChunk {
            entity_id: "a".to_string(),
            similarity: 0.9,
            content: String::new(),
            heading: None,
            file_path: String::new(),
        };
        let b = RetrievedChunk {
            entity_id: "b".to_string(),
            similarity: 0.5,
            content: String::new(),
            heading: None,
            file_path: String::new(),
        };
        assert!(a > b);
    }

    #[test]
    fn retrieved_chunk_ord_tiebreaks_on_entity_id() {
        let a = RetrievedChunk {
            entity_id: "a".to_string(),
            similarity: 0.5,
            content: String::new(),
            heading: None,
            file_path: String::new(),
        };
        let b = RetrievedChunk {
            entity_id: "b".to_string(),
            similarity: 0.5,
            content: String::new(),
            heading: None,
            file_path: String::new(),
        };
        assert!(a < b); // entity_id a < b when similarity tied
    }
}
