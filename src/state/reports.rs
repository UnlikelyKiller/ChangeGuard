use crate::impact::packet::ImpactPacket;
use crate::state::StateError;
use crate::state::layout::Layout;
use miette::Result;
use std::fs;

pub const LATEST_IMPACT_REPORT: &str = "latest-impact.json";

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
