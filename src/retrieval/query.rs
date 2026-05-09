use crate::config::model::LocalModelConfig;
use crate::embed::client::embed_long_text;
use crate::embed::similarity::pairwise_cosine;
use crate::embed::storage::load_candidates;
use crate::impact::packet::StalenessTier;
use chrono::NaiveDate;
use rusqlite::Connection;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

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

/// Compute the staleness (age in days) of an ADR file.
///
/// Uses multi-source age detection in priority order, taking the **most recent** date found.
/// Sources: file mtime → YAML frontmatter `date:` → `created:` metadata line → git log.
///
/// Recently-updated exemption: if file mtime is within 30 days, returns `None`.
pub fn compute_staleness(file_path: &Path, _threshold_days: u32) -> Option<u32> {
    const EXEMPTION_DAYS: u64 = 30;
    const SECS_PER_DAY: u64 = 86400;

    let now = SystemTime::now();

    // 1. Check mtime first for exemption
    let mtime_opt = fs::metadata(file_path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|m| now.duration_since(m).ok());

    if let Some(elapsed) = mtime_opt
        && elapsed.as_secs() / SECS_PER_DAY < EXEMPTION_DAYS
    {
        return None;
    }

    let mut most_recent: Option<SystemTime> = mtime_opt.map(|_| {
        fs::metadata(file_path)
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(SystemTime::UNIX_EPOCH)
    });

    // 2. Parse YAML frontmatter `date:` field
    if let Ok(content) = fs::read_to_string(file_path) {
        if let Some(stripped) = content.strip_prefix("---")
            && let Some(end) = stripped.find("---")
        {
            let frontmatter = &stripped[..end];
            for line in frontmatter.lines() {
                let line = line.trim();
                if let Some(val) = line.strip_prefix("date:") {
                    let val = val.trim();
                    if let Ok(naive) = NaiveDate::parse_from_str(val, "%Y-%m-%d") {
                        let date_time = naive.and_hms_opt(0, 0, 0).unwrap_or_default();
                        let ts = SystemTime::UNIX_EPOCH
                            + std::time::Duration::from_secs(date_time.and_utc().timestamp() as u64);
                        most_recent = most_recent.map(|m| m.max(ts)).or(Some(ts));
                    }
                    break;
                }
            }
        }

        // 3. Parse `created:` metadata line in body
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("created:") {
                let val = rest.trim();
                if let Ok(naive) = NaiveDate::parse_from_str(val, "%Y-%m-%d") {
                    let date_time = naive.and_hms_opt(0, 0, 0).unwrap_or_default();
                    let ts = SystemTime::UNIX_EPOCH
                        + std::time::Duration::from_secs(date_time.and_utc().timestamp() as u64);
                    most_recent = most_recent.map(|m| m.max(ts)).or(Some(ts));
                }
                break;
            }
        }
    }

    // 4. Git-based fallback
    let git_ts = git_last_commit_timestamp(file_path);
    if let Some(ts) = git_ts {
        most_recent = most_recent.map(|m| m.max(ts)).or(Some(ts));
    }

    let most_recent = most_recent?;
    let age = now.duration_since(most_recent).ok()?;
    let days = age.as_secs() / SECS_PER_DAY;
    Some(days as u32)
}

fn git_last_commit_timestamp(file_path: &Path) -> Option<SystemTime> {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%ct", "--", file_path.to_str()?])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let timestamp_secs: u64 = stdout.trim().parse().ok()?;

    Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp_secs))
}

pub fn compute_staleness_tier(days: u32, threshold_days: u32) -> Option<StalenessTier> {
    if days < threshold_days {
        None
    } else if days <= threshold_days.saturating_mul(2) {
        Some(StalenessTier::Warning)
    } else {
        Some(StalenessTier::Critical)
    }
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

    #[test]
    fn compute_staleness_exempt_when_mtime_within_30_days() {
        use std::fs::OpenOptions;
        use std::io::Write;
        use std::time::{Duration, SystemTime};

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("adr.md");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .unwrap();
        file.write_all(b"content").unwrap();
        let recent = SystemTime::now() - Duration::from_secs(5 * 86400);
        file.set_modified(recent).unwrap();
        let result = compute_staleness(&path, 365);
        assert!(result.is_none());
    }

    #[test]
    fn compute_staleness_populated_when_mtime_older_than_threshold() {
        use std::fs::OpenOptions;
        use std::io::Write;
        use std::time::{Duration, SystemTime};

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("adr.md");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .unwrap();
        file.write_all(b"content").unwrap();
        let old = SystemTime::now() - Duration::from_secs(400 * 86400);
        file.set_modified(old).unwrap();
        let result = compute_staleness(&path, 365);
        assert!(result.is_some());
        assert!(result.unwrap() >= 399);
    }

    #[test]
    fn compute_staleness_uses_frontmatter_date() {
        use std::fs::OpenOptions;
        use std::io::Write;
        use std::time::{Duration, SystemTime};

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("adr.md");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .unwrap();
        let frontmatter = b"---\ndate: 2024-01-01\n---\nBody content\n";
        file.write_all(frontmatter).unwrap();
        let frontmatter_ts = SystemTime::UNIX_EPOCH + Duration::from_secs(1704067200);
        file.set_modified(frontmatter_ts).unwrap();
        let result = compute_staleness(&path, 365);
        assert!(result.is_some());
        let days = result.unwrap();
        assert!(days >= 850);
    }

    #[test]
    fn compute_staleness_uses_created_metadata_line() {
        use std::fs::OpenOptions;
        use std::io::Write;
        use std::time::{Duration, SystemTime};

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("adr.md");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .unwrap();
        let content = b"created: 2024-01-01\n\nBody content\n";
        file.write_all(content).unwrap();
        let created_ts = SystemTime::UNIX_EPOCH + Duration::from_secs(1704067200);
        file.set_modified(created_ts).unwrap();
        let result = compute_staleness(&path, 365);
        assert!(result.is_some());
        assert!(result.unwrap() >= 850);
    }

    #[test]
    fn compute_staleness_tier_none_when_below_threshold() {
        let tier = compute_staleness_tier(100, 365);
        assert_eq!(tier, None);
    }

    #[test]
    fn compute_staleness_tier_warning_when_within_double_threshold() {
        let tier = compute_staleness_tier(500, 365);
        assert_eq!(tier, Some(StalenessTier::Warning));
    }

    #[test]
    fn compute_staleness_tier_critical_when_exceeds_double_threshold() {
        let tier = compute_staleness_tier(800, 365);
        assert_eq!(tier, Some(StalenessTier::Critical));
    }
}
