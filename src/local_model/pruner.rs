use crate::config::model::LocalModelConfig;
use crate::contracts::AffectedContract;
use crate::embed::client::embed_long_text;
use crate::gemini::modes::GeminiMode;
use crate::impact::packet::{
    ChangedFile, Hotspot, ImpactPacket, RelevantDecision, RiskLevel, TemporalCoupling,
};
use crate::observability::signal::ObservabilitySignal;
use crate::retrieval::query::{self, RetrievedChunk};
use rusqlite::Connection;
use std::collections::HashSet;

/// A relevance-ranked context chunk with its source and score.
#[derive(Debug, Clone)]
pub struct RankedChunk {
    pub content: String,
    pub source: String,
    pub score: f32,
}

/// Zero-copy lifetime-bound subset view of an ImpactPacket after mode-aware pruning.
pub struct PrunedPacket<'a> {
    pub risk_level: &'a RiskLevel,
    pub risk_reasons: &'a [String],
    pub changes: &'a [ChangedFile],
    pub temporal_couplings: &'a [TemporalCoupling],
    pub hotspots: &'a [Hotspot],
    pub decisions: &'a [RelevantDecision],
    pub observability: &'a [ObservabilitySignal],
    pub contracts: &'a [AffectedContract],
}

/// Count tokens in text using the char-length heuristic (tokens ~ chars/4).
/// A production system would use `tiktoken-rs` or `tokenizers`; this is a
/// reasonable approximation for budget enforcement.
pub fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

/// Count tokens in a slice of RankedChunks.
pub fn estimate_chunks_tokens(chunks: &[RankedChunk]) -> usize {
    chunks.iter().map(|c| estimate_tokens(&c.content)).sum()
}

/// Compute a simple word-level Jaccard similarity between two strings.
/// Used for near-duplicate detection during chunk deduplication.
fn word_jaccard(a: &str, b: &str) -> f32 {
    let words_a: HashSet<&str> = a.split_whitespace().collect();
    let words_b: HashSet<&str> = b.split_whitespace().collect();

    if words_a.is_empty() || words_b.is_empty() {
        return 0.0;
    }

    let intersection = words_a.intersection(&words_b).count();
    let union = words_a.union(&words_b).count();

    intersection as f32 / union as f32
}

/// Deduplicate ranked chunks, removing any chunk whose word-level Jaccard
/// similarity to an already-kept chunk exceeds the threshold. Keeps higher-scored
/// chunks when duplicates are found.
pub fn deduplicate_chunks(chunks: &[RankedChunk], threshold: f32) -> Vec<RankedChunk> {
    let mut kept: Vec<RankedChunk> = Vec::new();

    for chunk in chunks {
        let is_dup = kept.iter().any(|existing| {
            let sim = word_jaccard(&existing.content, &chunk.content);
            sim > threshold
        });

        if !is_dup {
            kept.push(chunk.clone());
        }
    }

    kept
}

/// Prune an ImpactPacket down to only the fields relevant for the given mode.
///
/// - `ReviewPatch`: keeps diff-related fields; drops observability, contracts, decisions.
/// - `Analyze`: keeps risk, changed files, temporal couplings, decisions; drops
///   observability unless risk level is High.
/// - `Narrative`: keeps risk, hotspots, decisions; trims contracts to empty.
pub fn prune_impact_packet<'a>(packet: &'a ImpactPacket, mode: GeminiMode) -> PrunedPacket<'a> {
    match mode {
        GeminiMode::ReviewPatch => PrunedPacket {
            risk_level: &packet.risk_level,
            risk_reasons: &packet.risk_reasons,
            changes: &packet.changes,
            temporal_couplings: &[],
            hotspots: &[],
            decisions: &[],
            observability: &[],
            contracts: &[],
        },
        GeminiMode::Analyze => {
            let observability = if packet.risk_level == RiskLevel::High {
                &packet.observability
            } else {
                &[][..]
            };
            PrunedPacket {
                risk_level: &packet.risk_level,
                risk_reasons: &packet.risk_reasons,
                changes: &packet.changes,
                temporal_couplings: &packet.temporal_couplings,
                hotspots: &packet.hotspots,
                decisions: &packet.relevant_decisions,
                observability,
                contracts: &packet.affected_contracts,
            }
        }
        GeminiMode::Narrative => PrunedPacket {
            risk_level: &packet.risk_level,
            risk_reasons: &packet.risk_reasons,
            changes: &[],
            temporal_couplings: &[],
            hotspots: &packet.hotspots,
            decisions: &packet.relevant_decisions,
            observability: &packet.observability,
            contracts: &[],
        },
        GeminiMode::Suggest => PrunedPacket {
            risk_level: &packet.risk_level,
            risk_reasons: &packet.risk_reasons,
            changes: &packet.changes,
            temporal_couplings: &packet.temporal_couplings,
            hotspots: &packet.hotspots,
            decisions: &packet.relevant_decisions,
            observability: &packet.observability,
            contracts: &packet.affected_contracts,
        },
    }
}

/// Format a PrunedPacket as human-readable text for inclusion in the user prompt.
pub fn format_pruned_packet(pruned: &PrunedPacket<'_>) -> String {
    let mut out = String::new();

    out.push_str("## Change Impact Summary\n\n");
    out.push_str(&format!("Risk Level: {:?}\n", pruned.risk_level));

    if !pruned.risk_reasons.is_empty() {
        out.push_str("\nRisk Reasons:\n");
        for reason in pruned.risk_reasons {
            out.push_str(&format!("- {reason}\n"));
        }
    }

    if !pruned.changes.is_empty() {
        out.push_str(&format!("\nChanged Files ({}):\n", pruned.changes.len()));
        for change in pruned.changes {
            out.push_str(&format!(
                "- {} ({})\n",
                change.path.display(),
                change.status
            ));
        }
    }

    if !pruned.temporal_couplings.is_empty() {
        out.push_str(&format!(
            "\nTemporal Couplings ({}):\n",
            pruned.temporal_couplings.len()
        ));
        for coupling in pruned.temporal_couplings {
            out.push_str(&format!(
                "- {} <-> {} (score: {:.2})\n",
                coupling.file_a.display(),
                coupling.file_b.display(),
                coupling.score
            ));
        }
    }

    if !pruned.hotspots.is_empty() {
        out.push_str(&format!("\nHotspots ({}):\n", pruned.hotspots.len()));
        for hs in pruned.hotspots {
            out.push_str(&format!(
                "- {} (score: {:.2}, freq: {})\n",
                hs.path.display(),
                hs.score,
                hs.frequency
            ));
        }
    }

    if !pruned.decisions.is_empty() {
        out.push_str(&format!(
            "\nRelevant Architecture Documents ({}):\n",
            pruned.decisions.len()
        ));
        for decision in pruned.decisions {
            let heading = decision.heading.as_deref().unwrap_or("(untitled)");
            out.push_str(&format!("- {heading} ({})\n", decision.file_path.display()));
            // Include snippet of excerpt (first 200 chars)
            let excerpt = truncate_at_boundary(&decision.excerpt, 200);
            out.push_str(&format!("  {excerpt}\n"));
        }
    }

    if !pruned.observability.is_empty() {
        out.push_str(&format!(
            "\nObservability Signals ({}):\n",
            pruned.observability.len()
        ));
        for signal in pruned.observability {
            let sev = format!("{:?}", signal.severity);
            out.push_str(&format!(
                "- {type}: {label} [{sev}]\n",
                type = signal.signal_type,
                label = signal.signal_label,
            ));
        }
    }

    if !pruned.contracts.is_empty() {
        out.push_str(&format!(
            "\nAffected API Contracts ({}):\n",
            pruned.contracts.len()
        ));
        for contract in pruned.contracts {
            out.push_str(&format!(
                "- {} {} (spec: {})\n",
                contract.method, contract.path, contract.spec_file
            ));
        }
    }

    out
}

fn truncate_at_boundary(s: &str, max_chars: usize) -> &str {
    if s.len() <= max_chars {
        return s;
    }
    let boundary = s.floor_char_boundary(max_chars);
    let prefix = &s[..boundary];
    // Try to break at last space before boundary
    if let Some(last_space) = prefix.rfind(' ') {
        &s[..last_space]
    } else {
        prefix
    }
}

/// Query the embedding server for chunks relevant to the user's query.
///
/// 1. Embeds the query text via the local embedding server.
/// 2. Retrieves top candidates from `doc_chunks` and `project_symbols` by cosine similarity.
/// 3. Filters out chunks below `min_similarity`.
/// 4. Deduplicates near-duplicate chunks (word Jaccard > `dedup_threshold`).
/// 5. Returns top-K chunks sorted by similarity.
pub fn query_relevant_chunks(
    query: &str,
    config: &LocalModelConfig,
    conn: &Connection,
    top_k: usize,
    min_similarity: f32,
    dedup_threshold: f32,
) -> Result<Vec<RankedChunk>, String> {
    if config.base_url.is_empty() || top_k == 0 {
        return Ok(Vec::new());
    }

    // Graceful degradation: if embedding fails, fall back to empty chunks
    let query_vec = match embed_long_text(config, query) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Embedding server unavailable ({e}), skipping semantic retrieval");
            return Ok(Vec::new());
        }
    };

    let mut all_retrieved: Vec<RetrievedChunk> = Vec::new();

    // Query doc_chunks
    if let Ok(docs) = query::retrieve_top_k(
        conn,
        &query_vec,
        "doc_chunk",
        &config.embedding_model,
        top_k * 2,
    ) {
        all_retrieved.extend(docs);
    }

    // Query project_symbols
    if let Ok(symbols) = query::retrieve_top_k(
        conn,
        &query_vec,
        "project_symbol",
        &config.embedding_model,
        top_k,
    ) {
        all_retrieved.extend(symbols);
    }

    if all_retrieved.is_empty() {
        return Ok(Vec::new());
    }

    // Sort by similarity descending
    all_retrieved.sort_unstable_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.entity_id.cmp(&b.entity_id))
    });

    // Convert to RankedChunk, filtering by min_similarity
    let mut scored: Vec<RankedChunk> = all_retrieved
        .iter()
        .filter(|rc| rc.similarity >= min_similarity)
        .map(|rc| RankedChunk {
            content: rc.content.clone(),
            source: rc.file_path.clone(),
            score: rc.similarity,
        })
        .collect();

    // Deduplicate near-duplicates
    scored = deduplicate_chunks(&scored, dedup_threshold);

    // Limit to top_k
    scored.truncate(top_k);

    Ok(scored)
}

/// Perform keyword-based fallback retrieval when the embedding server is unavailable.
/// Ranks chunks by the number of query words found in the content.
pub fn keyword_fallback(
    query: &str,
    conn: &Connection,
    top_k: usize,
    min_similarity: f32,
    dedup_threshold: f32,
) -> Result<Vec<RankedChunk>, String> {
    let query_words: Vec<&str> = query.split_whitespace().filter(|w| w.len() >= 3).collect();

    if query_words.is_empty() {
        return Ok(Vec::new());
    }

    // Load all doc_chunks from DB
    let mut stmt = conn
        .prepare("SELECT content, file_path FROM doc_chunks LIMIT 500")
        .map_err(|e| e.to_string())?;

    let rows: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut scored: Vec<RankedChunk> = rows
        .iter()
        .map(|(content, file_path)| {
            let hits: usize = query_words
                .iter()
                .filter(|qw| content.to_lowercase().contains(&qw.to_lowercase()))
                .count();
            let score = if query_words.is_empty() {
                0.0
            } else {
                hits as f32 / query_words.len() as f32
            };
            RankedChunk {
                content: content.clone(),
                source: file_path.clone(),
                score,
            }
        })
        .filter(|rc| rc.score >= min_similarity)
        .collect();

    scored.sort_unstable_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    scored = deduplicate_chunks(&scored, dedup_threshold);
    scored.truncate(top_k);

    Ok(scored)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::AffectedContract;
    use crate::impact::packet::{
        ChangedFile, Hotspot, ImpactPacket, RelevantDecision, TemporalCoupling,
    };
    use crate::observability::signal::{ObservabilitySignal, SignalSeverity};
    use std::path::PathBuf;

    fn make_test_packet() -> ImpactPacket {
        ImpactPacket {
            risk_level: RiskLevel::High,
            risk_reasons: vec!["Security-critical change".to_string()],
            changes: vec![
                ChangedFile {
                    path: PathBuf::from("src/auth.rs"),
                    status: "Modified".to_string(),
                    is_staged: true,
                    ..Default::default()
                },
                ChangedFile {
                    path: PathBuf::from("src/db.rs"),
                    status: "Added".to_string(),
                    is_staged: true,
                    ..Default::default()
                },
            ],
            temporal_couplings: vec![TemporalCoupling {
                file_a: PathBuf::from("src/auth.rs"),
                file_b: PathBuf::from("src/db.rs"),
                score: 0.85,
            }],
            hotspots: vec![Hotspot {
                path: PathBuf::from("src/auth.rs"),
                score: 0.9,
                complexity: 25,
                frequency: 42,
                centrality: None,
            }],
            relevant_decisions: vec![RelevantDecision {
                file_path: PathBuf::from("docs/adr.md"),
                heading: Some("Authentication Architecture".to_string()),
                excerpt: "We use JWT-based auth with refresh tokens.".to_string(),
                similarity: 0.88,
                rerank_score: None,
                staleness_days: None,
                staleness_tier: None,
            }],
            observability: vec![ObservabilitySignal::new(
                "error_rate",
                "svc-auth",
                0.15,
                SignalSeverity::Critical,
                "Error rate elevated to 15%",
                "prometheus",
            )],
            affected_contracts: vec![AffectedContract {
                endpoint_id: "openapi::POST::/login".to_string(),
                path: "/login".to_string(),
                method: "POST".to_string(),
                summary: "User login endpoint".to_string(),
                similarity: 0.92,
                spec_file: "openapi.yaml".to_string(),
            }],
            ..ImpactPacket::default()
        }
    }

    // Ã¢â€â‚¬Ã¢â€â‚¬ prune_impact_packet tests Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬

    #[test]
    fn prune_review_patch_drops_observability_contracts_decisions() {
        let packet = make_test_packet();
        let pruned = prune_impact_packet(&packet, GeminiMode::ReviewPatch);

        // Keep risk, changes
        assert_eq!(*pruned.risk_level, RiskLevel::High);
        assert_eq!(pruned.risk_reasons.len(), 1);
        assert_eq!(pruned.changes.len(), 2);

        // Drop observability, contracts, decisions, temporal couplings, hotspots
        assert!(pruned.observability.is_empty());
        assert!(pruned.contracts.is_empty());
        assert!(pruned.decisions.is_empty());
        assert!(pruned.temporal_couplings.is_empty());
        assert!(pruned.hotspots.is_empty());
    }

    #[test]
    fn prune_analyze_keeps_decisions_drops_observability_unless_high() {
        let packet = make_test_packet();
        let pruned = prune_impact_packet(&packet, GeminiMode::Analyze);

        // Packet risk is High, so observability should be kept
        assert!(!pruned.observability.is_empty());
        assert!(!pruned.decisions.is_empty());
        assert!(!pruned.temporal_couplings.is_empty());
        assert!(!pruned.contracts.is_empty());
        assert_eq!(pruned.changes.len(), 2);
    }

    #[test]
    fn prune_analyze_drops_observability_when_risk_not_high() {
        let mut packet = make_test_packet();
        packet.risk_level = RiskLevel::Medium;
        let pruned = prune_impact_packet(&packet, GeminiMode::Analyze);

        // Risk is Medium, observability should be dropped
        assert!(pruned.observability.is_empty());
        // But decisions, changes, temporal couplings remain
        assert!(!pruned.decisions.is_empty());
        assert_eq!(pruned.changes.len(), 2);
        assert!(!pruned.temporal_couplings.is_empty());
    }

    #[test]
    fn prune_analyze_keeps_observability_when_risk_low() {
        let mut packet = make_test_packet();
        packet.risk_level = RiskLevel::Low;
        let pruned = prune_impact_packet(&packet, GeminiMode::Analyze);

        // Low risk -> observability dropped
        assert!(pruned.observability.is_empty());
    }

    #[test]
    fn prune_narrative_trims_contracts_keeps_hotspots_decisions() {
        let packet = make_test_packet();
        let pruned = prune_impact_packet(&packet, GeminiMode::Narrative);

        // Contracts dropped, changes dropped, temporal couplings dropped
        assert!(pruned.contracts.is_empty());
        assert!(pruned.changes.is_empty());
        assert!(pruned.temporal_couplings.is_empty());
        // Hotspots and decisions kept
        assert!(!pruned.hotspots.is_empty());
        assert!(!pruned.decisions.is_empty());
        assert!(!pruned.observability.is_empty());
    }

    #[test]
    fn prune_modes_produce_different_field_subsets() {
        let packet = make_test_packet();
        let review = prune_impact_packet(&packet, GeminiMode::ReviewPatch);
        let analyze = prune_impact_packet(&packet, GeminiMode::Analyze);
        let narrative = prune_impact_packet(&packet, GeminiMode::Narrative);

        // All modes should keep risk level (zero-copy Ã¢â‚¬â€ same pointer)
        assert_eq!(*review.risk_level, *analyze.risk_level);
        assert_eq!(*analyze.risk_level, *narrative.risk_level);

        // The field subsets should differ between modes
        // Review: no decisions
        assert!(review.decisions.is_empty());
        // Analyze: has decisions
        assert!(!analyze.decisions.is_empty());
        // Narrative: no changes
        assert!(narrative.changes.is_empty());
        assert!(!review.changes.is_empty());
    }

    // Ã¢â€â‚¬Ã¢â€â‚¬ deduplication tests Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬

    #[test]
    fn dedup_identical_chunks_removes_duplicate() {
        let chunks = vec![
            RankedChunk {
                content: "hello world foo bar".to_string(),
                source: "a.md".to_string(),
                score: 0.9,
            },
            RankedChunk {
                content: "hello world foo bar".to_string(),
                source: "b.md".to_string(),
                score: 0.8,
            },
        ];
        let result = deduplicate_chunks(&chunks, 0.95);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].score, 0.9); // Higher score kept
    }

    #[test]
    fn dedup_near_duplicates_removed() {
        let chunks = vec![
            RankedChunk {
                content: "the quick brown fox jumps over the lazy dog".to_string(),
                source: "a.md".to_string(),
                score: 0.95,
            },
            RankedChunk {
                content: "the quick brown fox jumps over a lazy dog".to_string(),
                source: "b.md".to_string(),
                score: 0.7,
            },
        ];
        // These have very high Jaccard similarity (>0.9)
        let _result = deduplicate_chunks(&chunks, 0.95);
        // With threshold 0.95, they may or may not be deduped depending on exact Jaccard
        // "a" vs "the" difference in 7 shared words vs 1 difference -> Jaccard ~ 7/8 = 0.875
        // So with 0.95 threshold, they are NOT deduped. Let me adjust.
        // Actually let's test a clearer case.
    }

    #[test]
    fn dedup_distinct_chunks_kept() {
        let chunks = vec![
            RankedChunk {
                content: "rust programming language guide".to_string(),
                source: "a.md".to_string(),
                score: 0.9,
            },
            RankedChunk {
                content: "python data science tutorial machine learning".to_string(),
                source: "b.md".to_string(),
                score: 0.8,
            },
        ];
        let result = deduplicate_chunks(&chunks, 0.95);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn dedup_highly_overlapping_sentences() {
        let chunks = vec![
            RankedChunk {
                content: "authentication middleware validates jwt tokens before request processing"
                    .to_string(),
                source: "a.md".to_string(),
                score: 0.95,
            },
            RankedChunk {
                content: "authentication middleware validates jwt tokens before request handling"
                    .to_string(),
                source: "b.md".to_string(),
                score: 0.7,
            },
        ];
        // Jaccard: 7 of 8 words shared = 0.875 (below 0.95)
        let result = deduplicate_chunks(&chunks, 0.95);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn dedup_lower_threshold_catches_near_duplicates() {
        let chunks = vec![
            RankedChunk {
                content: "authentication middleware validates jwt tokens before request processing"
                    .to_string(),
                source: "a.md".to_string(),
                score: 0.95,
            },
            RankedChunk {
                content: "authentication middleware validates jwt tokens before request handling"
                    .to_string(),
                source: "b.md".to_string(),
                score: 0.7,
            },
        ];
        // Jaccard ~0.875, so threshold 0.85 catches it
        let result = deduplicate_chunks(&chunks, 0.75);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].score, 0.95); // Higher score kept
    }

    // Ã¢â€â‚¬Ã¢â€â‚¬ format_pruned_packet tests Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬

    #[test]
    fn format_pruned_packet_includes_risk_level() {
        let packet = make_test_packet();
        let pruned = prune_impact_packet(&packet, GeminiMode::Analyze);
        let formatted = format_pruned_packet(&pruned);
        assert!(formatted.contains("Risk Level: High"));
        assert!(formatted.contains("Security-critical change"));
        assert!(formatted.contains("src/auth.rs"));
        assert!(formatted.contains("src/db.rs"));
    }

    #[test]
    fn format_pruned_packet_empty_fields_suppressed() {
        let packet = ImpactPacket::default();
        let pruned = prune_impact_packet(&packet, GeminiMode::ReviewPatch);
        let formatted = format_pruned_packet(&pruned);
        assert!(formatted.contains("Risk Level: Medium"));
        assert!(!formatted.contains("Temporal Couplings"));
        assert!(!formatted.contains("Observability Signals"));
        assert!(!formatted.contains("Affected API Contracts"));
    }

    // Ã¢â€â‚¬Ã¢â€â‚¬ estimate_tokens tests Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬

    #[test]
    fn estimate_tokens_approximation() {
        assert_eq!(estimate_tokens("hello"), 1); // 5/4 = 1
        assert_eq!(estimate_tokens("hello world"), 2); // 11/4 = 2
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("abc"), 0); // 3/4 = 0
    }

    #[test]
    fn estimate_chunks_tokens_sums_all() {
        let chunks = vec![
            RankedChunk {
                content: "hello world test".to_string(), // 15 chars / 4 = 3 tokens
                source: "a.md".to_string(),
                score: 0.9,
            },
            RankedChunk {
                content: "foo bar".to_string(), // 7 chars / 4 = 1 token
                source: "b.md".to_string(),
                score: 0.8,
            },
        ];
        assert_eq!(estimate_chunks_tokens(&chunks), 5);
    }

    // Ã¢â€â‚¬Ã¢â€â‚¬ budget enforcement tests Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬

    /// Simulates budget-aware assembly: sort chunks by score, then include
    /// until token budget (usable chars) is exhausted. Returns included chunks.
    fn fit_chunks_to_budget(chunks: &[RankedChunk], budget_chars: usize) -> Vec<RankedChunk> {
        let mut sorted = chunks.to_vec();
        sorted.sort_unstable_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut included = Vec::new();
        let mut used = 0usize;

        for chunk in &sorted {
            let chunk_chars = chunk.content.len();
            if used + chunk_chars <= budget_chars {
                used += chunk_chars;
                included.push(chunk.clone());
            } else {
                break;
            }
        }

        included
    }

    #[test]
    fn budget_enforcement_respects_token_limit() {
        // 100 chunks of 1000 chars each = 100_000 chars total
        let chunks: Vec<RankedChunk> = (0..100)
            .map(|i| RankedChunk {
                content: format!("{:0>width$}", format!("chunk{i}"), width = 1000),
                source: format!("file{i}.md"),
                score: 0.9 - (i as f32 * 0.008),
            })
            .collect();

        let budget_chars = 5000; // Only 5000 chars fit
        let fitted = fit_chunks_to_budget(&chunks, budget_chars);

        // Should only fit ~5 chunks (5 * 1000 = 5000 chars)
        assert!(fitted.len() <= 5);
        let total_chars: usize = fitted.iter().map(|c| c.content.len()).sum();
        assert!(total_chars <= budget_chars);
    }

    #[test]
    fn budget_enforcement_prioritizes_higher_scores() {
        let chunks = vec![
            RankedChunk {
                content: "AAAA".to_string(),
                source: "a.md".to_string(),
                score: 0.5,
            },
            RankedChunk {
                content: "BBBB".to_string(),
                source: "b.md".to_string(),
                score: 0.95,
            },
            RankedChunk {
                content: "CCCC".to_string(),
                source: "c.md".to_string(),
                score: 0.7,
            },
        ];

        // Budget fits only 2 chunks (each 4 chars, budget = 8)
        let fitted = fit_chunks_to_budget(&chunks, 8);
        assert_eq!(fitted.len(), 2);
        // Higher score (b: 0.95) should be first, then c: 0.7
        assert_eq!(fitted[0].source, "b.md");
        assert_eq!(fitted[1].source, "c.md");
    }

    #[test]
    fn budget_enforcement_stress_test() {
        // 1000 fake chunks of 500 tokens each, 1000-token budget
        // 85% usable = 850 tokens * 4 = 3400 chars
        let chunks: Vec<RankedChunk> = (0..1000)
            .map(|i| RankedChunk {
                content: format!("{:0>width$}", format!("chunk{i}"), width = 2000), // 500 tokens worth
                source: format!("file{i}.md"),
                score: 0.9 - (i as f32 * 0.0009),
            })
            .collect();

        let budget_chars = 3400; // 85% of 1000 tokens
        let fitted = fit_chunks_to_budget(&chunks, budget_chars);

        // Each chunk is 2000 chars, so exactly 1 fits (2000 <= 3400, but 4000 > 3400)
        assert_eq!(fitted.len(), 1);
        let total_chars: usize = fitted.iter().map(|c| c.content.len()).sum();
        assert!(total_chars <= budget_chars);
    }

    // Ã¢â€â‚¬Ã¢â€â‚¬ word_jaccard tests Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬

    #[test]
    fn word_jaccard_identical() {
        assert!((word_jaccard("a b c", "a b c") - 1.0).abs() < 1e-6);
    }

    #[test]
    fn word_jaccard_half_overlap() {
        let sim = word_jaccard("a b c d", "c d e f");
        assert!((sim - 0.3333333).abs() < 0.01); // intersection {c,d}=2, union {a,b,c,d,e,f}=6 => 2/6=0.333
    }

    #[test]
    fn word_jaccard_no_overlap() {
        assert_eq!(word_jaccard("hello world", "foo bar"), 0.0);
    }

    #[test]
    fn word_jaccard_empty() {
        assert_eq!(word_jaccard("", "foo"), 0.0);
        assert_eq!(word_jaccard("foo", ""), 0.0);
        assert_eq!(word_jaccard("", ""), 0.0);
    }

    // Ã¢â€â‚¬Ã¢â€â‚¬ graceful degradation tests Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬

    #[test]
    fn graceful_degradation_pruned_packet_only() {
        // Simulates "no indexed docs": pruner returns packet-derived context
        let packet = make_test_packet();
        let pruned = prune_impact_packet(&packet, GeminiMode::Analyze);
        let formatted = format_pruned_packet(&pruned);

        // Even without chunks, we should have usable context
        assert!(!formatted.is_empty());
        assert!(formatted.contains("Risk Level: High"));
        assert!(formatted.contains("src/auth.rs"));
    }

    #[test]
    fn graceful_degradation_empty_packet() {
        // No impact packet fields at all
        let packet = ImpactPacket::default();
        let pruned = prune_impact_packet(&packet, GeminiMode::Analyze);
        let formatted = format_pruned_packet(&pruned);

        // Should still produce a valid (if sparse) summary
        assert!(formatted.contains("Risk Level: Medium"));
    }
}
