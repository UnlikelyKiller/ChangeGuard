use miette::Result;
use crate::git::repo::{open_repo, get_head_info};
use crate::git::status::get_repo_status;
use crate::git::{RepoSnapshot, ChangeType};
use crate::impact::packet::{ImpactPacket, ChangedFile, RiskLevel};
use crate::state::layout::Layout;
use crate::index::languages::parse_symbols;
use crate::state::reports::write_impact_report;
use std::env;
use std::fs;
use std::path::Path;
use owo_colors::OwoColorize;

pub fn execute_impact() -> Result<()> {
    let current_dir = env::current_dir().map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;
    
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
    let mut packet = map_snapshot_to_packet(snapshot, &current_dir);

    // Load rules and perform risk analysis
    if let Ok(rules) = crate::policy::load::load_rules(&layout) {
        let _ = crate::impact::analysis::analyze_risk(&mut packet, &rules);
    }
    
    packet.finalize();

    write_impact_report(&layout, &packet)?;

    println!("{} Wrote impact report to {}", "SUCCESS".green().bold(), ".changeguard/reports/latest-impact.json".cyan());

    Ok(())
}

fn map_snapshot_to_packet(snapshot: RepoSnapshot, base_dir: &Path) -> ImpactPacket {
    let mut packet = ImpactPacket::default();
    packet.head_hash = snapshot.head_hash;
    packet.branch_name = snapshot.branch_name;
    
    if snapshot.is_clean {
        packet.risk_level = RiskLevel::Low;
        packet.risk_reasons = vec!["No changes detected".to_string()];
    } else {
        packet.risk_level = RiskLevel::Medium;
        packet.risk_reasons = vec!["Provisional baseline risk".to_string()];
    }

    packet.changes = snapshot.changes.into_iter().map(|c| {
        let status = match c.change_type {
            ChangeType::Added => "Added".to_string(),
            ChangeType::Modified => "Modified".to_string(),
            ChangeType::Deleted => "Deleted".to_string(),
            ChangeType::Renamed { .. } => "Renamed".to_string(),
        };

        let symbols = if matches!(c.change_type, ChangeType::Added | ChangeType::Modified) {
            let full_path = base_dir.join(&c.path);
            if let Ok(content) = fs::read_to_string(&full_path) {
                parse_symbols(&c.path, &content).ok().flatten()
            } else {
                None
            }
        } else {
            None
        };

        ChangedFile {
            path: c.path,
            status,
            is_staged: c.is_staged,
            symbols,
        }
    }).collect();

    packet
}
