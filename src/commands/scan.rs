use crate::config::load::load_config;
use crate::git::RepoSnapshot;
use crate::git::diff::get_diff_summary;
use crate::git::repo::{get_head_info, open_repo};
use crate::git::status::get_repo_status;
use crate::output::human::print_scan_summary;
use crate::state::layout::Layout;
use crate::state::reports::{ScanDiffSummary, ScanReport, write_scan_report};
use miette::{IntoDiagnostic, Result};
use std::env;
use std::path::PathBuf;

pub fn execute_scan(
    run_impact: bool,
    summary: bool,
    json: bool,
    out: Option<PathBuf>,
) -> Result<()> {
    if !run_impact && (summary || json || out.is_some()) {
        return Err(miette::miette!(
            "--summary, --json and --out require --impact"
        ));
    }

    let current_dir = env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;

    let repo = open_repo(&current_dir)?;
    let (head_hash, branch_name) = get_head_info(&repo)?;
    let all_changes = get_repo_status(&repo)?;

    // Filter changes against config ignore_patterns
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let config = load_config(&layout).unwrap_or_default();
    let changes =
        crate::git::ignore::filter_ignored_changes(all_changes, &config.watch.ignore_patterns)?;

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

    let write_impact_json = json || out.is_some();

    if !write_impact_json {
        print_scan_summary(&snapshot);
    }

    if run_impact {
        if write_impact_json {
            let impact_packet = crate::commands::impact::execute_impact_silent()?;
            let json_output = serde_json::to_string_pretty(&impact_packet).into_diagnostic()?;

            if let Some(path) = out {
                std::fs::write(&path, json_output).into_diagnostic()?;
            } else {
                println!("{}", json_output);
            }
        } else {
            crate::commands::impact::execute_impact(false, summary, false, false)?;
        }
    }

    Ok(())
}
