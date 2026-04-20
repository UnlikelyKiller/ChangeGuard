use crate::git::repo::open_repo;
use crate::impact::temporal::{GixHistoryProvider, HistoryProvider};
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use camino::Utf8PathBuf;
use miette::{IntoDiagnostic, Result};
use std::collections::HashMap;
use std::env;

pub struct Hotspot {
    pub path: Utf8PathBuf,
    pub score: f32,
    pub complexity: i32,
    pub frequency: usize,
}

pub fn execute_hotspots(limit: usize, commits: usize) -> Result<()> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let repo = open_repo(&current_dir)?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    
    let db_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(db_path.as_std_path())?;

    println!("Analyzing {} commits for temporal hotspots...", commits);
    
    let provider = GixHistoryProvider::new(&repo);
    let history = provider.get_history(commits).map_err(|e| miette::miette!("Git history error: {e}"))?;

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
        return Err(miette::miette!("No eligible commits found in history window."));
    }

    // Fetch max complexity for each file from SQLite
    // We'll use a raw query since we don't have a high-level one for this yet
    let mut stmt = storage.get_connection().prepare(
        "SELECT file_path, MAX(IFNULL(cognitive_complexity, 0), IFNULL(cyclomatic_complexity, 0)) as max_comp 
         FROM symbols 
         GROUP BY file_path"
    ).into_diagnostic()?;

    let file_complexities: HashMap<String, i32> = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).into_diagnostic()?
    .filter_map(|res| res.ok())
    .collect();

    let mut hotspots = Vec::new();

    for (path, freq) in frequency_map {
        let path_str = path.to_string();
        let complexity = file_complexities.get(&path_str).cloned().unwrap_or(0);
        
        // Scoring: 
        // Normalized Frequency (0-1) + Normalized Complexity (0-1)
        // For simplicity: frequency / total_commits + complexity / 50 (capped at 1.0)
        let f_score = freq as f32 / total_eligible_commits as f32;
        let c_score = (complexity as f32 / 50.0).min(1.0);
        
        let score = (f_score * 0.5) + (c_score * 0.5);

        hotspots.push(Hotspot {
            path,
            score,
            complexity,
            frequency: freq,
        });
    }

    hotspots.sort_unstable_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    hotspots.truncate(limit);

    crate::output::human::print_hotspots_table(&hotspots);

    Ok(())
}
