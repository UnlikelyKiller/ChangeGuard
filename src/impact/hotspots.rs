use crate::impact::packet::Hotspot;
use crate::impact::temporal::HistoryProvider;
use crate::state::storage::StorageManager;
use camino::Utf8PathBuf;
use miette::{IntoDiagnostic, Result};
use std::collections::HashMap;

pub fn normalize_score(raw_score: f64) -> f64 {
    (raw_score * 1000.0).ln_1p()
}

#[derive(Debug, Clone, Default)]
pub struct HotspotQuery {
    pub commits: usize,
    pub days: Option<u64>,
    pub since_commit: Option<String>,
    pub limit: usize,
    pub all_parents: bool,
    pub decay_half_life: usize,
    pub dir_filter: Option<String>,
    pub lang_filter: Option<String>,
}

pub fn calculate_hotspots(
    storage: &StorageManager,
    history_provider: &dyn HistoryProvider,
    query: &HotspotQuery,
) -> Result<Vec<Hotspot>> {
    let history = history_provider
        .get_history(
            query.commits,
            query.days,
            query.since_commit.clone(),
            query.all_parents,
        )
        .map_err(|e| miette::miette!("Git history error: {e}"))?;

    let mut frequency_map: HashMap<Utf8PathBuf, f64> = HashMap::new();
    let mut total_eligible_commits = 0;

    let half_life = query.decay_half_life as f64;

    for (idx, commit_set) in history.iter().enumerate() {
        if commit_set.is_merge || commit_set.files.is_empty() {
            continue;
        }
        total_eligible_commits += 1;

        // Exponential decay: most recent commit (idx 0) gets weight 1.0
        let weight = if half_life > 0.0 {
            (2.0_f64).powf(-(idx as f64) / half_life)
        } else {
            1.0
        };

        for file in &commit_set.files {
            // Apply filtering during crawl
            let path_str = file.as_str();

            if query
                .dir_filter
                .as_ref()
                .is_some_and(|dir| !path_str.starts_with(dir))
            {
                continue;
            }

            if query
                .lang_filter
                .as_ref()
                .is_some_and(|lang| !path_str.ends_with(&format!(".{lang}")))
            {
                continue;
            }

            *frequency_map.entry(file.clone()).or_default() += weight;
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
    let max_freq = frequency_map
        .values()
        .cloned()
        .fold(f64::MIN, f64::max)
        .max(1.0) as f32;
    // Find max complexity for normalization; clamp to 1 to avoid division by zero when all files have 0 complexity
    let max_comp = file_complexities
        .values()
        .max()
        .cloned()
        .unwrap_or(0)
        .max(1) as f32;

    for (path, freq) in frequency_map {
        let path_str = path.to_string();
        let complexity = file_complexities.get(&path_str).cloned().unwrap_or(0);

        // Scoring:
        // Normalized Frequency (0-1) * Normalized Complexity (0-1)
        // Multiplication surfaces the "worst of both worlds" more effectively than addition.
        let f_norm = freq as f32 / max_freq;
        let c_norm = complexity as f32 / max_comp;

        let score = f_norm * c_norm;

        let display_score = normalize_score(score as f64) as f32;
        hotspots.push(Hotspot {
            path: path.into(),
            score,
            display_score,
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
    hotspots.truncate(query.limit);

    Ok(hotspots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_scale_zero() {
        assert_eq!(normalize_score(0.0), 0.0);
    }

    #[test]
    fn log_scale_positive() {
        assert!(normalize_score(1.0) > 0.0);
    }

    #[test]
    fn log_scale_compresses_outlier() {
        let high = normalize_score(0.135);
        let low = normalize_score(0.006);
        // The raw ratio is 22.5x; after log normalization should be < 5x
        assert!(
            high / low < 5.0,
            "expected log normalization to compress 22x gap; got ratio {:.2}",
            high / low
        );
    }

    #[test]
    fn sort_order_preserved() {
        // ln_1p is monotonic — sort by raw == sort by display
        let scores = vec![0.135_f64, 0.006, 0.05, 0.0];
        let mut by_raw = scores.clone();
        by_raw.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let mut by_display = scores.clone();
        by_display.sort_by(|a, b| {
            normalize_score(*b)
                .partial_cmp(&normalize_score(*a))
                .unwrap()
        });
        assert_eq!(by_raw, by_display, "sort order must be identical");
    }
}
