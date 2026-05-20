use crate::config::load::load_config;
use crate::git::RepoSnapshot;
use crate::git::diff::get_diff_summary;
use crate::git::repo::{get_head_info, open_repo};
use crate::git::status::get_repo_status;
use crate::output::human::print_scan_summary;
use crate::state::layout::Layout;
use crate::state::reports::{ScanDiffSummary, ScanReport, write_scan_report};
use globset::{Glob, GlobSetBuilder};
use miette::Result;
use std::env;

pub fn execute_scan(run_impact: bool) -> Result<()> {
    let current_dir = env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;

    let repo = open_repo(&current_dir)?;
    let (head_hash, branch_name) = get_head_info(&repo)?;
    let all_changes = get_repo_status(&repo)?;

    // Filter changes against config ignore_patterns
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let changes = if let Ok(config) = load_config(&layout) {
        let mut builder = GlobSetBuilder::new();
        for pattern in &config.watch.ignore_patterns {
            builder.add(
                Glob::new(pattern)
                    .map_err(|e| miette::miette!("Invalid glob pattern '{}': {}", pattern, e))?,
            );
        }
        let ignore_set = builder
            .build()
            .map_err(|e| miette::miette!("Failed to build glob set: {}", e))?;
        all_changes
            .into_iter()
            .filter(|change| {
                let path_str = change.path.to_string_lossy();
                !ignore_set.is_match(path_str.as_ref())
            })
            .collect()
    } else {
        all_changes
    };

    let is_clean = changes.is_empty();

    let snapshot = RepoSnapshot {
        head_hash,
        branch_name,
        is_clean,
        changes,
    };
    let mut diff_summaries = snapshot
        .changes
        .iter()
        .filter_map(|change| {
            get_diff_summary(&repo, &change.path).map(|summary| ScanDiffSummary {
                path: change.path.to_string_lossy().to_string(),
                summary,
            })
        })
        .collect::<Vec<_>>();
    diff_summaries.sort_by(|a, b| a.path.cmp(&b.path));

    let scan_report = ScanReport::from_snapshot(&snapshot, diff_summaries);
    write_scan_report(&layout, &scan_report)?;

    print_scan_summary(&snapshot);

    if run_impact {
        crate::commands::impact::execute_impact(false, false, false, false)?;
    }

    Ok(())
}
