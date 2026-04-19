use crate::git::RepoSnapshot;
use crate::git::diff::get_diff_summary;
use crate::git::repo::{get_head_info, open_repo};
use crate::git::status::get_repo_status;
use crate::output::human::print_scan_summary;
use crate::state::layout::Layout;
use crate::state::reports::{ScanDiffSummary, ScanReport, write_scan_report};
use miette::Result;
use std::env;

pub fn execute_scan() -> Result<()> {
    let current_dir = env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;

    let repo = open_repo(&current_dir)?;
    let (head_hash, branch_name) = get_head_info(&repo)?;
    let changes = get_repo_status(&repo)?;

    let is_clean = changes.is_empty();

    let snapshot = RepoSnapshot {
        head_hash,
        branch_name,
        is_clean,
        changes,
    };
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());

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

    Ok(())
}
