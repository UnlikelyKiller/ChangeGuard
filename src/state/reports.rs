use crate::git::{ChangeType, FileChange, RepoSnapshot};
use crate::impact::packet::ImpactPacket;
use crate::state::StateError;
use crate::state::layout::Layout;
use miette::Result;
use serde::Serialize;
use std::fs;

pub const LATEST_IMPACT_REPORT: &str = "latest-impact.json";
pub const LATEST_SCAN_REPORT: &str = "latest-scan.json";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScanDiffSummary {
    pub path: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScanChange {
    pub path: String,
    pub change_type: String,
    pub is_staged: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScanReport {
    pub head_hash: Option<String>,
    pub branch_name: Option<String>,
    pub is_clean: bool,
    pub changes: Vec<ScanChange>,
    pub diff_summaries: Vec<ScanDiffSummary>,
}

impl ScanReport {
    pub fn from_snapshot(snapshot: &RepoSnapshot, diff_summaries: Vec<ScanDiffSummary>) -> Self {
        Self {
            head_hash: snapshot.head_hash.clone(),
            branch_name: snapshot.branch_name.clone(),
            is_clean: snapshot.is_clean,
            changes: snapshot.changes.iter().map(ScanChange::from).collect(),
            diff_summaries,
        }
    }
}

impl From<&FileChange> for ScanChange {
    fn from(change: &FileChange) -> Self {
        let change_type = match &change.change_type {
            ChangeType::Added => "Added".to_string(),
            ChangeType::Modified => "Modified".to_string(),
            ChangeType::Deleted => "Deleted".to_string(),
            ChangeType::Renamed { old_path } => {
                format!(
                    "Renamed: {} -> {}",
                    old_path.display(),
                    change.path.display()
                )
            }
        };

        Self {
            path: change.path.to_string_lossy().to_string(),
            change_type,
            is_staged: change.is_staged,
        }
    }
}

/// Writes the generated impact report to disk in the reports directory.
pub fn write_impact_report(layout: &Layout, packet: &ImpactPacket) -> Result<()> {
    // Ensure the reports directory exists
    layout.ensure_state_dir()?;

    let report_path = layout.reports_dir().join(LATEST_IMPACT_REPORT);
    let json = serde_json::to_string_pretty(packet)
        .map_err(std::io::Error::other)
        .map_err(|e| StateError::WriteReportFailed {
            path: report_path.to_string(),
            source: e,
        })?;

    fs::write(&report_path, json).map_err(|e| StateError::WriteReportFailed {
        path: report_path.to_string(),
        source: e,
    })?;

    Ok(())
}

pub fn write_scan_report(layout: &Layout, report: &ScanReport) -> Result<()> {
    layout.ensure_state_dir()?;

    let report_path = layout.reports_dir().join(LATEST_SCAN_REPORT);
    let json = serde_json::to_string_pretty(report)
        .map_err(std::io::Error::other)
        .map_err(|e| StateError::WriteReportFailed {
            path: report_path.to_string(),
            source: e,
        })?;

    fs::write(&report_path, json).map_err(|e| StateError::WriteReportFailed {
        path: report_path.to_string(),
        source: e,
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::layout::Layout;
    use camino::Utf8Path;
    use tempfile::tempdir;

    #[test]
    fn test_write_impact_report() {
        let tmp = tempdir().unwrap();
        let root = Utf8Path::from_path(tmp.path()).unwrap();
        let layout = Layout::new(root);
        let packet = ImpactPacket::default();

        write_impact_report(&layout, &packet).unwrap();

        let report_path = layout.reports_dir().join(LATEST_IMPACT_REPORT);
        assert!(report_path.exists());

        let content = fs::read_to_string(report_path).unwrap();
        let deserialized: ImpactPacket = serde_json::from_str(&content).unwrap();
        assert_eq!(deserialized.schema_version, packet.schema_version);
    }
}
