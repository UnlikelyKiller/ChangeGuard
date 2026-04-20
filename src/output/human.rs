use crate::exec::boundary::ExecutionResult;
use crate::git::{ChangeType, RepoSnapshot};
use crate::impact::packet::{ImpactPacket, RiskLevel};
use crate::output::diagnostics::print_header;
use crate::output::table::build_table;
use crate::verify::plan::VerificationPlan;
use owo_colors::OwoColorize;

pub fn print_scan_summary(snapshot: &RepoSnapshot) {
    print_header("ChangeGuard Git Scan Summary");

    let branch = snapshot.branch_name.as_deref().unwrap_or("DETACHED");
    let head = snapshot.head_hash.as_deref().unwrap_or("None");

    println!("{:<15} {}", "Branch:".bold().cyan(), branch);
    println!("{:<15} {}", "HEAD:".bold().cyan(), head);
    println!(
        "{:<15} {}",
        "State:".bold().cyan(),
        if snapshot.is_clean {
            "CLEAN".green().bold().to_string()
        } else {
            "DIRTY".yellow().bold().to_string()
        }
    );

    if !snapshot.is_clean {
        println!("\n{}", "Changes:".bold());

        let mut table = build_table(["State", "Action", "File Path"]);

        for change in &snapshot.changes {
            let status_indicator = if change.is_staged {
                "Staged".green().to_string()
            } else {
                "Unstaged".dimmed().to_string()
            };
            let (change_label, color_path) = match &change.change_type {
                ChangeType::Added => (
                    "Added".green().to_string(),
                    change.path.display().to_string().green().to_string(),
                ),
                ChangeType::Modified => (
                    "Modified".yellow().to_string(),
                    change.path.display().to_string().yellow().to_string(),
                ),
                ChangeType::Deleted => (
                    "Deleted".red().to_string(),
                    change.path.display().to_string().red().to_string(),
                ),
                ChangeType::Renamed { old_path } => (
                    "Renamed".blue().to_string(),
                    format!("{} -> {}", old_path.display(), change.path.display())
                        .blue()
                        .to_string(),
                ),
            };

            table.add_row(vec![status_indicator, change_label, color_path]);
        }

        println!("{table}");
    }
}

pub fn print_impact_summary(packet: &ImpactPacket) {
    print_header("ChangeGuard Impact Analysis");

    let risk_color = match packet.risk_level {
        RiskLevel::Low => "LOW".green().bold().to_string(),
        RiskLevel::Medium => "MEDIUM".yellow().bold().to_string(),
        RiskLevel::High => "HIGH".red().bold().to_string(),
    };

    println!("{:<15} {}", "Risk Level:".bold().cyan(), risk_color);

    if !packet.risk_reasons.is_empty() {
        println!("\n{}", "Risk Reasons:".bold());
        let mut table = build_table(["#", "Reason"]);
        for (i, reason) in packet.risk_reasons.iter().enumerate() {
            table.add_row(vec![(i + 1).to_string(), reason.to_string()]);
        }
        println!("{table}");
    }

    if !packet.temporal_couplings.is_empty() {
        println!("\n{}", "Temporal Couplings (Historical Co-changes):".bold());
        let mut table = build_table(["File A", "File B", "Affinity"]);
        for coupling in &packet.temporal_couplings {
            table.add_row(vec![
                coupling.file_a.display().to_string(),
                coupling.file_b.display().to_string(),
                format!("{:.0}%", coupling.score * 100.0),
            ]);
        }
        println!("{table}");
    }

    let partial_analysis_count = packet
        .changes
        .iter()
        .filter(|file| !file.analysis_warnings.is_empty())
        .count();
    if partial_analysis_count > 0 {
        println!(
            "\n{} {} file(s) had partial or unsupported analysis. Inspect latest-impact.json for details.",
            "Warning:".yellow().bold(),
            partial_analysis_count
        );
    }
}

pub fn print_doctor_report(
    platform: &str,
    shell: &str,
    tools: &[(String, crate::platform::ExecutableStatus)],
    path_display: &str,
    path_kind: &str,
    is_wsl_mounted: bool,
) {
    println!(
        "\n{}",
        "ChangeGuard Doctor - Environment Health Check"
            .bold()
            .bright_cyan()
    );
    println!("{}", "=".repeat(50).cyan());

    println!("{:<20} {}", "Environment:".bold(), platform);
    println!("{:<20} {}", "Active Shell:".bold(), shell);

    println!("\n{}", "Tools:".bold().bright_cyan());
    for (name, status) in tools {
        match status {
            crate::platform::ExecutableStatus::Found(path) => {
                println!(
                    "  {:<18} {} ({})",
                    name.bold(),
                    "Found".green(),
                    path.display().to_string().dimmed()
                );
            }
            crate::platform::ExecutableStatus::NotFound => {
                println!("  {:<18} {}", name.bold(), "Not Found".red());
            }
        }
    }

    println!("\n{:<20} {}", "Current Path:".bold(), path_display);
    println!("{:<20} {}", "Path Type:".bold(), path_kind);

    if is_wsl_mounted {
        println!(
            "\n{}",
            "Warning: Running on a WSL mounted drive may be slower due to cross-filesystem overhead."
                .yellow()
                .italic()
        );
    }

    println!("\n{}", "Doctor check complete.".bright_cyan());
}

pub fn print_verify_result(cmd: &str, timeout_secs: u64, result: &ExecutionResult) {
    println!("\n{}", "ChangeGuard Verification".bold().bright_cyan());
    println!("{}", "=".repeat(50).cyan());
    println!("{:<15} {}", "Command:".bold(), cmd.yellow());
    println!("{:<15} {}s", "Timeout:".bold(), timeout_secs);
    println!();

    println!("{}", "Output:".bold());
    println!("{}", result.stdout);

    if !result.stderr.is_empty() {
        println!("\n{}", "Errors:".bold().red());
        println!("{}", result.stderr.red());
    }

    println!("\n{}", "=".repeat(50).cyan());
    println!(
        "{:<15} {}",
        "Exit Code:".bold(),
        if result.exit_code == 0 {
            result.exit_code.green().to_string()
        } else {
            result.exit_code.red().to_string()
        }
    );
    println!("{:<15} {:?}", "Duration:".bold(), result.duration);

    if result.truncated {
        println!(
            "{}",
            "Warning: Output was truncated due to size limits."
                .yellow()
                .italic()
        );
    }

    if result.exit_code == 0 {
        println!("\n{}", "Verification PASSED".green().bold());
    } else {
        println!("\n{}", "Verification FAILED".red().bold());
    }
}

pub fn print_verify_plan(plan: &VerificationPlan) {
    println!("\n{}", "Verification Plan".bold().bright_cyan());
    println!("{}", "=".repeat(50).cyan());
    for (i, step) in plan.steps.iter().enumerate() {
        println!(
            "  {}. {} ({})",
            i + 1,
            step.command.yellow(),
            step.description.dimmed()
        );
    }
    println!("{}", "=".repeat(50).cyan());
}

pub fn print_hotspots_table(hotspots: &[crate::impact::packet::Hotspot]) {
    print_header("Codebase Hotspots (Risk Density)");

    if hotspots.is_empty() {
        println!("No hotspots identified.");
        return;
    }

    let mut table = build_table(["Rank", "Score", "Freq", "Comp", "File Path"]);

    for (i, hotspot) in hotspots.iter().enumerate() {
        let score_color = if hotspot.score > 0.7 {
            hotspot.score.to_string().red().bold().to_string()
        } else if hotspot.score > 0.4 {
            hotspot.score.to_string().yellow().to_string()
        } else {
            hotspot.score.to_string().green().to_string()
        };

        table.add_row(vec![
            (i + 1).to_string(),
            score_color,
            hotspot.frequency.to_string(),
            hotspot.complexity.to_string(),
            hotspot.path.display().to_string().cyan().to_string(),
        ]);
    }

    println!("{table}");
    println!(
        "\n{} High frequency + high complexity = high risk density.",
        "Note:".bold().dimmed()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::{ChangeType, FileChange};
    use crate::impact::packet::{ImpactPacket, RiskLevel};
    use crate::verify::plan::{VerificationPlan, VerificationStep};
    use std::path::PathBuf;

    #[test]
    fn test_print_scan_summary_clean() {
        let snapshot = RepoSnapshot {
            head_hash: Some("abc".to_string()),
            branch_name: Some("main".to_string()),
            is_clean: true,
            changes: vec![],
        };
        // Just verify no panic
        print_scan_summary(&snapshot);
    }

    #[test]
    fn test_print_scan_summary_dirty() {
        let snapshot = RepoSnapshot {
            head_hash: Some("abc".to_string()),
            branch_name: Some("main".to_string()),
            is_clean: false,
            changes: vec![
                FileChange {
                    path: PathBuf::from("src/main.rs"),
                    change_type: ChangeType::Modified,
                    is_staged: true,
                },
                FileChange {
                    path: PathBuf::from("new.rs"),
                    change_type: ChangeType::Added,
                    is_staged: false,
                },
            ],
        };
        print_scan_summary(&snapshot);
    }

    #[test]
    fn test_print_impact_summary() {
        let packet = ImpactPacket {
            risk_level: RiskLevel::High,
            risk_reasons: vec!["Test reason".to_string()],
            ..Default::default()
        };
        print_impact_summary(&packet);
    }

    #[test]
    fn test_print_verify_plan() {
        let plan = VerificationPlan {
            steps: vec![VerificationStep {
                command: "cargo test".to_string(),
                timeout_secs: 300,
                description: "Run tests".to_string(),
            }],
        };
        print_verify_plan(&plan);
    }
}
