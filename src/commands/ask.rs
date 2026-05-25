use crate::commands::helpers::{get_layout, load_ledger_config};
use crate::config::model::Config;
use crate::gemini::modes::{GeminiMode, build_system_prompt};
use crate::gemini::wrapper::run_query;
use crate::index::warn_if_stale;
use crate::local_model::pruner;
use crate::state::storage::StorageManager;
use miette::Result;
use owo_colors::OwoColorize;
use std::env;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, clap::ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum Backend {
    Local,
    Gemini,
}

pub fn execute_ask(
    query: Option<String>,
    semantic: bool,
    limit: usize,
    mode: GeminiMode,
    narrative: bool,
    backend: Option<Backend>,
    auto_index: bool,
) -> Result<()> {
    let layout = get_layout()?;
    let config = load_ledger_config(&layout)?;

    layout.ensure_state_dir()?;
    let storage_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(storage_path.as_std_path())?;

    // --- Staleness check ---
    let threshold = config.index.stale_threshold_days;
    let storage = if auto_index {
        crate::index::staleness::try_auto_index(storage, threshold)?
    } else {
        let _ = warn_if_stale(&storage, threshold);
        storage
    };

    let (mut latest_packet, mut is_global) = match storage.get_latest_packet()? {
        Some(pkt) => (pkt, false),
        None => {
            tracing::info!(
                "No impact report found — falling back to global knowledge retrieval mode."
            );
            (crate::impact::packet::ImpactPacket::default(), true)
        }
    };

    // If the latest packet exists but contains no changes, it's effectively a global query context
    if !is_global && latest_packet.changes.is_empty() {
        tracing::debug!("Latest impact packet is clean (no changes) — defaulting to global mode.");
        is_global = true;
    }

    // 1. Integrate external AI-Brains context
    if let Some(ref q) = query
        && let Ok(bridge_records) = crate::bridge::client::query_unified(q)
    {
        for record in bridge_records {
            if let crate::bridge::model::BridgePayload::Insight {
                memory_id,
                relevance,
                content,
            } = record.payload
            {
                latest_packet
                    .ai_insights
                    .push(crate::impact::packet::AiInsight {
                        memory_id,
                        relevance,
                        content,
                    });
            }
        }
    }

    let resolved_backend = resolve_backend(&config, backend);
    let query_string = query.unwrap_or_else(|| {
        if is_global {
            "Give me an overview of this codebase and its key components.".to_string()
        } else {
            "Analyze the current impact and risk.".to_string()
        }
    });

    // In global mode, always use semantic retrieval to pull relevant context
    let semantic = semantic || is_global;

    if is_global {
        println!(
            "{}",
            "[Global Mode] No pending changes found — querying the full Knowledge Graph for context."
                .cyan()
        );
    }

    match resolved_backend {
        Backend::Local => {
            let max_tokens = config.local_model.context_window;

            // Phase 1: Probe local model completions endpoint for fail-fast
            let mut probe_config = config.local_model.clone();
            probe_config.timeout_secs = 5;
            if let Err(e) = crate::local_model::client::ping_completions(&probe_config) {
                return Err(miette::miette!(
                    "Local completion model is unreachable ({}). Check your server or use --backend gemini.",
                    e
                ));
            }

            let mut relevant_chunks = gather_semantic_chunks(
                &storage,
                &query_string,
                limit,
                &config.local_model,
                is_global,
            );

            if relevant_chunks.is_empty() {
                relevant_chunks = pruner::query_relevant_chunks(
                    &query_string,
                    &config.local_model,
                    storage.get_connection(),
                    limit,
                    config.local_model.chunk_min_similarity,
                    config.local_model.chunk_dedup_threshold,
                )
                .unwrap_or_else(|e| {
                    tracing::warn!("Chunk retrieval failed: {e}, proceeding without chunks");
                    Vec::new()
                });
                // CR7: Apply KG neighborhood to pruner fallback chunks as well.
                // Pruner chunks only have file paths as source, so extract the file
                // stem (module name) as a candidate symbol for KG lookups.
                if is_global
                    && !relevant_chunks.is_empty()
                    && let Some(cozo) = &storage.cozo
                {
                    let syms = relevant_chunks.iter().filter_map(|c| {
                        let path = std::path::Path::new(&c.source);
                        path.file_stem()?.to_str()
                    });
                    if let Some(kg_ctx) = fetch_kg_neighborhood(cozo, syms) {
                        relevant_chunks.push(crate::local_model::pruner::RankedChunk {
                            source: "Knowledge Graph".to_string(),
                            content: kg_ctx,
                            score: 1.0,
                        });
                    }
                }
            }
            if is_global && relevant_chunks.is_empty() {
                return Err(miette::miette!(
                    "Global Ask requires codebase context, but semantic search returned no results. \
                     Run 'changeguard index --semantic' to build the index."
                ));
            }

            // 4. Assemble context with budget enforcement
            let system_prompt = if is_global {
                "You are ChangeGuard, an expert software engineering assistant. You act as a codebase oracle answering architectural and implementation questions based on retrieved knowledge graph and semantic context snippets. Provide direct, technical, and accurate answers citing the retrieved snippets where relevant.".to_string()
            } else {
                crate::local_model::context::get_system_prompt(&mode.to_string())
            };
            let user_prompt = if is_global {
                format!(
                    "Answer the following codebase query:\n\nQuery: {}",
                    query_string
                )
            } else if narrative {
                crate::gemini::prompt::build_architect_prompt(&latest_packet, &query_string)
            } else {
                crate::gemini::prompt::build_suggest_prompt(&latest_packet, &query_string)
            };

            let adaptive_mode = if semantic {
                crate::local_model::context::AdaptiveMode::CodebaseFocus
            } else {
                crate::local_model::context::AdaptiveMode::ChangesFocus
            };

            let messages = crate::local_model::context::assemble_context(
                &system_prompt,
                &user_prompt,
                &relevant_chunks,
                max_tokens,
                adaptive_mode,
            );

            match crate::local_model::client::complete(
                &config.local_model,
                &messages,
                &crate::local_model::client::CompletionOptions::default(),
            ) {
                Ok(response) => {
                    println!("\n{}", "Local Model Response:".bold().green());
                    println!("{response}");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("{}", e.to_string().red());
                    Err(miette::miette!("Local model failed: {e}"))
                }
            }
        }
        Backend::Gemini => {
            let budget_tokens = config.gemini.context_window;
            let char_limit = (budget_tokens as f64 * 0.8 * 4.0) as usize;

            let user_prompt = if is_global {
                format!(
                    "Answer the following codebase query:\n\nQuery: {}",
                    query_string
                )
            } else if narrative {
                crate::gemini::prompt::build_architect_prompt(&latest_packet, &query_string)
            } else {
                crate::gemini::prompt::build_suggest_prompt(&latest_packet, &query_string)
            };

            let final_user_prompt = if semantic {
                let mut relevant_chunks = gather_semantic_chunks(
                    &storage,
                    &query_string,
                    limit,
                    &config.local_model,
                    is_global,
                );

                if relevant_chunks.is_empty() {
                    relevant_chunks = pruner::query_relevant_chunks(
                        &query_string,
                        &config.local_model,
                        storage.get_connection(),
                        limit,
                        config.local_model.chunk_min_similarity,
                        config.local_model.chunk_dedup_threshold,
                    )
                    .unwrap_or_default();
                }

                if is_global && relevant_chunks.is_empty() {
                    return Err(miette::miette!(
                        "Global Ask requires codebase context, but semantic search returned no results. \
                         Run 'changeguard index --semantic' to build the index."
                    ));
                }

                // Build a combined prompt for Gemini that includes semantic snippets
                let codebase_context = relevant_chunks
                    .iter()
                    .map(|c| format!("[{}] {}", c.source, c.content))
                    .collect::<Vec<_>>()
                    .join("\n\n");

                format!(
                    "{}\n\n## Codebase Context Chunks\n\n{}\n\nUser Question: {}",
                    user_prompt, codebase_context, query_string
                )
            } else {
                user_prompt
            };

            // The system prompt is static application text. Sanitize context.
            let sanitize_result = crate::gemini::sanitize::sanitize_for_gemini(&final_user_prompt);
            let mut sanitized_user_prompt = sanitize_result.sanitized;

            if sanitized_user_prompt.len() > char_limit {
                tracing::warn!("Prompt exceeds Gemini budget, truncating...");
                sanitized_user_prompt.truncate(char_limit);
                sanitized_user_prompt.push_str("\n\n[Prompt truncated for Gemini budget]");
            }

            let system_prompt = if is_global {
                "You are ChangeGuard, an expert software engineering assistant. You act as a codebase oracle answering architectural and implementation questions based on retrieved knowledge graph and semantic context snippets. Provide direct, technical, and accurate answers citing the retrieved snippets where relevant.".to_string()
            } else {
                build_system_prompt(mode)
            };

            run_query(
                &system_prompt,
                &sanitized_user_prompt,
                Some(config.gemini.timeout_secs.unwrap_or(120)),
                &crate::gemini::wrapper::select_gemini_model(&config.gemini, mode, &latest_packet),
                config.gemini.api_key.as_deref(),
            )
        }
    }
}

fn gather_semantic_chunks(
    storage: &StorageManager,
    query_string: &str,
    limit: usize,
    config: &crate::config::model::LocalModelConfig,
    is_global: bool,
) -> Vec<crate::local_model::pruner::RankedChunk> {
    let mut relevant_chunks = Vec::new();
    let mut semantic_symbols = std::collections::HashSet::new();

    if let Some(cozo) = &storage.cozo
        && let Ok(vector_store) = crate::semantic::vector_store::VectorStore::new(
            cozo,
            config.dimensions,
            config.disable_hnsw,
        )
        && let Ok(embedder) =
            crate::semantic::embedder::SemanticEmbedder::new(config.clone()).embed(query_string)
        && let Ok(results) = vector_store.query(embedder, limit)
    {
        for (file_path, name, _offset, dist) in results {
            let score = 1.0 - (dist / 2.0);
            if score >= config.chunk_min_similarity {
                semantic_symbols.insert(name.clone());
                if let Ok(content) =
                    crate::util::fs::read_to_string_with_encoding(std::path::Path::new(&file_path))
                {
                    let snippet = content.chars().take(1000).collect::<String>();
                    relevant_chunks.push(crate::local_model::pruner::RankedChunk {
                        source: format!("{}:: {}", file_path, name),
                        content: snippet,
                        score,
                    });
                }
            }
        }

        if is_global
            && !semantic_symbols.is_empty()
            && let Some(cozo) = &storage.cozo
            && let Some(kg_ctx) =
                fetch_kg_neighborhood(cozo, semantic_symbols.iter().map(|s| s.as_str()))
        {
            relevant_chunks.push(crate::local_model::pruner::RankedChunk {
                source: "Knowledge Graph".to_string(),
                content: kg_ctx,
                score: 1.0,
            });
        }
    }

    relevant_chunks
}

/// CR8: Escape a symbol name for safe interpolation inside a Cozo Datalog string literal.
/// Cozo uses single-quoted string literals; a single quote must be doubled to escape it.
/// Backslashes are also escaped to prevent unintended Datalog escaping sequences.
pub fn escape_cozo_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "''")
}

/// CR7: Run the KG neighborhood edge query for a set of symbol names and return a
/// formatted context string, or `None` if no relevant edges are found.
fn fetch_kg_neighborhood(
    cozo: &crate::state::storage_cozo::CozoStorage,
    symbols: impl Iterator<Item = impl AsRef<str>>,
) -> Option<String> {
    let symbols_array = symbols
        .map(|s| format!("'{}'", escape_cozo_string(s.as_ref())))
        .collect::<Vec<_>>()
        .join(", ");
    if symbols_array.is_empty() {
        return None;
    }
    let script = format!(
        r#"
        ?[caller, callee, relation] := *edge{{source: caller, target: callee, relation: relation}}, 
                                       caller in [{symbols_array}] or callee in [{symbols_array}]
        :limit 50
    "#
    );
    let res = cozo.run_script(&script).ok()?;
    let mut kg_context = String::from("Knowledge Graph Relationships:\n");
    for row in res.rows {
        if let (
            Some(cozo::DataValue::Str(caller)),
            Some(cozo::DataValue::Str(callee)),
            Some(cozo::DataValue::Str(rel)),
        ) = (row.first(), row.get(1), row.get(2))
        {
            kg_context.push_str(&format!("- {} {} {}\n", caller, rel, callee));
        }
    }
    if kg_context.len() > 30 {
        Some(kg_context)
    } else {
        None
    }
}

pub fn resolve_backend(config: &Config, explicit: Option<Backend>) -> Backend {
    resolve_backend_with(config, explicit, &|name| env::var(name).ok(), &|name| {
        crate::config::model::read_env_key(name)
    })
}

pub fn resolve_backend_with(
    config: &Config,
    explicit: Option<Backend>,
    env_reader: &dyn Fn(&str) -> Option<String>,
    dotenv_reader: &dyn Fn(&str) -> Option<String>,
) -> Backend {
    if let Some(b) = explicit {
        return b;
    }
    if config.local_model.prefer_local
        && (!config.local_model.base_url.is_empty()
            || config.local_model.embedding_url.is_some()
            || config.local_model.generation_url.is_some())
    {
        return Backend::Local;
    }

    let has_gemini_key = config.gemini.api_key.is_some()
        || env_reader("GEMINI_API_KEY").is_some()
        || dotenv_reader("GEMINI_API_KEY").is_some();

    if !has_gemini_key
        && (!config.local_model.base_url.is_empty()
            || config.local_model.embedding_url.is_some()
            || config.local_model.generation_url.is_some())
    {
        return Backend::Local;
    }
    Backend::Gemini
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::GeminiConfig;
    use crate::impact::packet::ImpactPacket;

    #[test]
    fn test_select_gemini_model_logic() {
        let packet = ImpactPacket::default();

        // 1. Defaults
        unsafe {
            std::env::remove_var("GEMINI_FAST_MODEL");
            std::env::remove_var("GEMINI_DEEP_MODEL");
        }
        let config = GeminiConfig {
            fast_model: Some("fast".to_string()),
            deep_model: Some("deep".to_string()),
            ..GeminiConfig::default()
        };
        let fast_model =
            crate::gemini::wrapper::select_gemini_model(&config, GeminiMode::Suggest, &packet);
        assert_eq!(fast_model, "fast");

        let deep_model =
            crate::gemini::wrapper::select_gemini_model(&config, GeminiMode::ReviewPatch, &packet);
        assert_eq!(deep_model, "deep");

        // 2. Config Overrides
        let config_custom = GeminiConfig {
            model: Some("custom".to_string()),
            ..GeminiConfig::default()
        };
        let model = crate::gemini::wrapper::select_gemini_model(
            &config_custom,
            GeminiMode::Suggest,
            &packet,
        );
        assert_eq!(model, "custom");

        // 3. Env Overrides
        unsafe {
            std::env::set_var("GEMINI_FAST_MODEL", "env-fast");
            std::env::set_var("GEMINI_DEEP_MODEL", "env-deep");
        }
        let config_empty = GeminiConfig::default();
        let fast_model_env = crate::gemini::wrapper::select_gemini_model(
            &config_empty,
            GeminiMode::Suggest,
            &packet,
        );
        assert_eq!(fast_model_env, "env-fast");

        let deep_model_env = crate::gemini::wrapper::select_gemini_model(
            &config_empty,
            GeminiMode::ReviewPatch,
            &packet,
        );
        assert_eq!(deep_model_env, "env-deep");

        unsafe {
            std::env::remove_var("GEMINI_FAST_MODEL");
            std::env::remove_var("GEMINI_DEEP_MODEL");
        }
    }
}
