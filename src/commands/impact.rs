use miette::Result;
use crate::git::repo::{open_repo, get_head_info};
use crate::git::status::get_repo_status;
use crate::git::{RepoSnapshot, ChangeType};
use crate::impact::packet::{ImpactPacket, ChangedFile, RiskLevel};
use crate::state::layout::Layout;
use crate::index::languages::parse_symbols;
use crate::state::reports::write_impact_report;
use crate::ui::{print_header, success_marker};
use std::env;
use std::fs;
use std::path::Path;
use owo_colors::OwoColorize;
use comfy_table::Table;
use indicatif::{ProgressBar, ProgressStyle};

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
    let mut packet = map_snapshot_to_packet(snapshot, &current_dir)?;

    // Load rules and perform risk analysis
    if let Ok(rules) = crate::policy::load::load_rules(&layout) {
        let _ = crate::impact::analysis::analyze_risk(&mut packet, &rules);
    }
    
    packet.finalize();

    write_impact_report(&layout, &packet)?;

    print_impact_summary(&packet);

    println!("\n{} Wrote impact report to {}", success_marker(), ".changeguard/reports/latest-impact.json".cyan());

    // Persist to SQLite
    let db_path = layout.state_subdir().join("ledger.db");
    if let Ok(storage) = crate::state::storage::StorageManager::init(db_path.as_std_path()) {
        let _ = storage.save_packet(&packet);
    }

    Ok(())
}

fn print_impact_summary(packet: &ImpactPacket) {
    print_header("ChangeGuard Impact Analysis");
    
    let risk_color = match packet.risk_level {
        RiskLevel::Low => "LOW".green().bold().to_string(),
        RiskLevel::Medium => "MEDIUM".yellow().bold().to_string(),
        RiskLevel::High => "HIGH".red().bold().to_string(),
    };
    
    println!("{:<15} {}", "Risk Level:".bold().cyan(), risk_color);
    
    if !packet.risk_reasons.is_empty() {
        println!("\n{}", "Risk Reasons:".bold());
        let mut table = Table::new();
        table.set_header(vec!["#", "Reason"]);
        for (i, reason) in packet.risk_reasons.iter().enumerate() {
            table.add_row(vec![(i + 1).to_string(), reason.to_string()]);
        }
        println!("{table}");
    }
}

fn map_snapshot_to_packet(snapshot: RepoSnapshot, base_dir: &Path) -> Result<ImpactPacket> {
    let mut packet = ImpactPacket::default();
    packet.head_hash = snapshot.head_hash;
    packet.branch_name = snapshot.branch_name;
    
    let pb = ProgressBar::new(snapshot.changes.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
        .expect("Failed to set progress bar style"));
    pb.set_message("Extracting symbols...");

    packet.changes = snapshot.changes.into_iter().map(|c| {
        pb.set_message(format!("Extracting symbols from {}", c.path.display()));
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

        pb.inc(1);
        ChangedFile {
            path: c.path,
            status,
            is_staged: c.is_staged,
            symbols,
        }
    }).collect();

    pb.finish_with_message("Symbol extraction complete.");
    Ok(packet)
}
