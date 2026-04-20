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
) -> Result<Vec<Hotspot>> {
    let history = history_provider
        .get_history(commits)
        .map_err(|e| miette::miette!("Git history error: {e}"))?;

    let mut frequency_map: HashMap<Utf8PathBuf, usize> = HashMap::new();
    let mut total_eligible_commits = 0;

    for commit_set in &history {
        if commit_set.is_merge || commit_set.files.is_empty() {
            continue;
        }
        total_eligible_commits += 1;
        for file in &commit_set.files {
            *frequency_map.entry(file.clone()).or_default() += 1;
        }
    }

    if total_eligible_commits == 0 {
        return Ok(Vec::new());
    }

    let mut stmt = storage.get_connection().prepare(
        "SELECT file_path, MAX(IFNULL(cognitive_complexity, 0), IFNULL(cyclomatic_complexity, 0)) as max_comp 
         FROM symbols 
         GROUP BY file_path"
    ).into_diagnostic()?;

    let file_complexities: HashMap<String, i32> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .into_diagnostic()?
        .filter_map(|res| res.ok())
        .collect();

    let mut hotspots = Vec::new();

    for (path, freq) in frequency_map {
        let path_str = path.to_string();
        let complexity = file_complexities.get(&path_str).cloned().unwrap_or(0);

        let f_score = freq as f32 / total_eligible_commits as f32;
        let c_score = (complexity as f32 / 50.0).min(1.0);

        let score = (f_score * 0.5) + (c_score * 0.5);

        hotspots.push(Hotspot {
            path: path.into(),
            score,
            complexity,
            frequency: freq,
        });
    }

    hotspots.sort_unstable_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
    });
    hotspots.truncate(limit);

    Ok(hotspots)
}
