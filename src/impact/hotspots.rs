use crate::impact::packet::Hotspot;
use crate::impact::temporal::HistoryProvider;
use crate::state::storage::StorageManager;
use camino::Utf8PathBuf;
use miette::{IntoDiagnostic, Result};
use std::collections::HashMap;

pub fn calculate_hotspots(
    storage: &StorageManager,
    history_provider: &dyn HistoryProvider,
    commits: usize,
    limit: usize,
    all_parents: bool,
    dir_filter: Option<&str>,
    lang_filter: Option<&str>,
) -> Result<Vec<Hotspot>> {
    let history = history_provider
        .get_history(commits, all_parents)
        .map_err(|e| miette::miette!("Git history error: {e}"))?;

    let mut frequency_map: HashMap<Utf8PathBuf, usize> = HashMap::new();
    let mut total_eligible_commits = 0;

    for commit_set in &history {
        if commit_set.is_merge || commit_set.files.is_empty() {
            continue;
        }
        total_eligible_commits += 1;
        for file in &commit_set.files {
            // Apply filtering during crawl
            let path_str = file.as_str();

            if dir_filter.is_some_and(|dir| !path_str.starts_with(dir)) {
                continue;
            }

            if lang_filter.is_some_and(|lang| !path_str.ends_with(&format!(".{lang}"))) {
                continue;
            }

            *frequency_map.entry(file.clone()).or_default() += 1;
        }
    }

    if total_eligible_commits == 0 {
        return Ok(Vec::new());
    }

    // Primary: query the impact-time symbols table for complexity data.
    // Fallback: per-file gap-fill from project_symbols (populated by `changeguard index`)
    // when the symbols table has no data for that file.
    let mut file_complexities: HashMap<String, i32>;

    // Load primary complexity data from the impact-time symbols table
    let primary_result: HashMap<String, i32> = {
        let mut stmt = storage.get_connection().prepare(
            "SELECT file_path, MAX(IFNULL(cognitive_complexity, 0), IFNULL(cyclomatic_complexity, 0)) as max_comp
             FROM symbols
             GROUP BY file_path"
        ).into_diagnostic()?;

        stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
        })
        .into_diagnostic()?
        .collect::<rusqlite::Result<HashMap<String, i32>>>()
        .into_diagnostic()?
    };

    file_complexities = primary_result;

    // Per-file gap-fill: for files not in the primary result, try project_symbols
    let needs_fallback: Vec<String> = frequency_map
        .keys()
        .map(|p| p.to_string())
        .filter(|path| !file_complexities.contains_key(path) || file_complexities[path] == 0)
        .collect();

    if !needs_fallback.is_empty() {
        let fallback_result = {
            let conn = storage.get_connection();
            let mut fallback = HashMap::new();
            for path in &needs_fallback {
                match conn.query_row(
                    "SELECT IFNULL(AVG(IFNULL(ps.cognitive_complexity, 0)), 0) as avg_comp
                     FROM project_symbols ps
                     JOIN project_files pf ON ps.file_id = pf.id
                     WHERE pf.file_path = ?1",
                    [path],
                    |row| row.get::<_, f64>(0),
                ) {
                    Ok(avg) => {
                        fallback.insert(path.clone(), avg as i32);
                    }
                    Err(_) => {
                        // Table doesn't exist or no data for this file
                        tracing::debug!("No project_symbols data for file: {}", path);
                    }
                }
            }
            fallback
        };

        for (path, complexity) in fallback_result {
            if !file_complexities.contains_key(&path) || file_complexities[&path] == 0 {
                file_complexities.insert(path, complexity);
            }
        }
    }

    let mut hotspots = Vec::new();

    // Find max frequency for normalization
    let max_freq = frequency_map.values().max().cloned().unwrap_or(1) as f32;
    // Find max complexity for normalization; clamp to 1 to avoid division by zero when all files have 0 complexity
    let max_comp = file_complexities.values().max().cloned().unwrap_or(0).max(1) as f32;

    for (path, freq) in frequency_map {
        let path_str = path.to_string();
        let complexity = file_complexities.get(&path_str).cloned().unwrap_or(0);

        // Scoring:
        // Normalized Frequency (0-1) * Normalized Complexity (0-1)
        // Multiplication surfaces the "worst of both worlds" more effectively than addition.
        let f_norm = freq as f32 / max_freq;
        let c_norm = complexity as f32 / max_comp;

        let score = f_norm * c_norm;

        hotspots.push(Hotspot {
            path: path.into(),
            score,
            complexity,
            frequency: freq,
            centrality: None,
        });
    }

    // Deterministic sorting: Score (desc) then Path (asc)
    hotspots.sort_unstable_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
    });
    hotspots.truncate(limit);

    Ok(hotspots)
}
