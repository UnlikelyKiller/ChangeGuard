use crate::state::StateError;
use crate::state::layout::Layout;
use chrono::Utc;
use miette::Result;
use serde::{Deserialize, Serialize};
use std::fs;

use super::plan::VerificationPlan;

pub const LATEST_VERIFY_REPORT: &str = "latest-verify.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VerificationResult {
    pub command: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub stdout_summary: String,
    pub stderr_summary: String,
    pub truncated: bool,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VerificationReport {
    pub plan: Option<VerificationPlan>,
    pub results: Vec<VerificationResult>,
    #[serde(default)]
    pub prediction_warnings: Vec<String>,
    pub overall_pass: bool,
    pub timestamp: String,
}

impl VerificationReport {
    pub fn new(plan: Option<VerificationPlan>, results: Vec<VerificationResult>) -> Self {
        let overall_pass = results.iter().all(|result| result.exit_code == 0);
        Self {
            plan,
            results,
            prediction_warnings: Vec::new(),
            overall_pass,
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.prediction_warnings = warnings;
        self
    }
}

pub fn write_verify_report(layout: &Layout, report: &VerificationReport) -> Result<()> {
    layout.ensure_state_dir()?;
    let report_path = layout.reports_dir().join(LATEST_VERIFY_REPORT);
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
    use camino::Utf8Path;
    use tempfile::tempdir;

    #[test]
    fn test_write_verify_report() {
        let tmp = tempdir().unwrap();
        let root = Utf8Path::from_path(tmp.path()).unwrap();
        let layout = Layout::new(root);
        let report = VerificationReport::new(
            None,
            vec![VerificationResult {
                command: "cargo test".to_string(),
                exit_code: 0,
                duration_ms: 123,
                stdout_summary: "ok".to_string(),
                stderr_summary: String::new(),
                truncated: false,
                timestamp: "2026-01-01T00:00:00Z".to_string(),
            }],
        );

        write_verify_report(&layout, &report).unwrap();
        let saved = fs::read_to_string(layout.reports_dir().join(LATEST_VERIFY_REPORT)).unwrap();
        let loaded: VerificationReport = serde_json::from_str(&saved).unwrap();
        assert!(loaded.overall_pass);
        assert_eq!(loaded.results[0].command, "cargo test");
    }
}
