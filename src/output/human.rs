use crate::exec::boundary::ExecutionResult;
use crate::git::{ChangeType, RepoSnapshot};
use crate::impact::packet::{AnalysisStatus, ImpactPacket, RiskLevel};
use crate::observability::signal::SignalSeverity;
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

pub fn print_impact_summary(packet: &ImpactPacket, config: &crate::config::model::Config) {
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

    if !packet.observability.is_empty() {
        println!("\n{}", "Production Signals:".bold());
        let mut table = build_table(["Signal Type", "Severity", "Excerpt"]);
        for signal in &packet.observability {
            let severity = match signal.severity {
                SignalSeverity::Critical => "CRITICAL".red().bold().to_string(),
                SignalSeverity::Warning => "WARNING".yellow().to_string(),
                SignalSeverity::Normal => "NORMAL".dimmed().to_string(),
            };
            let excerpt = signal.excerpt.lines().next().unwrap_or("");
            table.add_row(vec![
                signal.signal_type.clone(),
                severity,
                excerpt.to_string(),
            ]);
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

    if !packet.affected_contracts.is_empty() {
        println!("\n{}", "Affected API Contracts:".bold());
        let mut table = build_table(["Method", "Path", "Spec", "Similarity"]);
        for contract in &packet.affected_contracts {
            table.add_row(vec![
                contract.method.clone(),
                contract.path.clone(),
                contract.spec_file.clone(),
                format!("{:.0}%", contract.similarity * 100.0),
            ]);
        }
        println!("{table}");
    }

    if !packet.trace_config_drift.is_empty() || !packet.trace_env_vars.is_empty() {
        println!("\n{}", "Observability Trace Drift:".bold());
        let mut table = build_table(["Type", "Entity", "Details"]);
        for drift in &packet.trace_config_drift {
            table.add_row(vec![
                "Config File".to_string(),
                drift.file.display().to_string(),
                format!("{:?}{}", drift.config_type, if drift.is_deleted { " (DELETED)" } else { "" }),
            ]);
        }
        for env in &packet.trace_env_vars {
            table.add_row(vec![
                "Env Var".to_string(),
                env.var_name.clone(),
                format!("Pattern: {}", env.pattern),
            ]);
        }
        println!("{table}");
    }

    if let Some(sdk_delta) = &packet.sdk_dependencies_delta {
        if !sdk_delta.added.is_empty()
            || !sdk_delta.removed.is_empty()
            || !sdk_delta.modified.is_empty()
        {
            println!("\n{}", "Third-party SDK Changes:".bold());
            let mut table = build_table(["State", "SDK Name", "Pattern Match"]);
            for sdk in &sdk_delta.added {
                table.add_row(vec![
                    "Added".green().to_string(),
                    sdk.sdk_name.clone(),
                    sdk.import_statement.clone(),
                ]);
            }
            for sdk in &sdk_delta.modified {
                table.add_row(vec![
                    "Modified".yellow().to_string(),
                    sdk.sdk_name.clone(),
                    sdk.import_statement.clone(),
                ]);
            }
            for sdk in &sdk_delta.removed {
                table.add_row(vec![
                    "Removed".red().to_string(),
                    sdk.sdk_name.clone(),
                    sdk.import_statement.clone(),
                ]);
            }
            println!("{table}");
        }
    }

    if let Some(delta) = &packet.service_map_delta {
        if !delta.affected_services.is_empty() {
            println!("\n{}", "Service Map Impact:".bold());
            println!(
                "{:<20} {}",
                "Affected Services:".bold().cyan(),
                delta.affected_services.join(", ")
            );
            if !delta.cross_service_edges.is_empty() {
                println!("\n{}", "Cross-Service Dependencies:".bold().dimmed());
                let mut table = build_table(["Caller Service", "Callee Service", "Edge Count"]);
                for (caller, callee, count) in &delta.cross_service_edges {
                    table.add_row(vec![caller.clone(), callee.clone(), count.to_string()]);
                }
                println!("{table}");
            }
        }
    }

    if !packet.data_flow_matches.is_empty() {
        println!("\n{}", "Data-Flow Coupling (Route -> Model):".bold());
        let mut table = build_table(["Chain Pattern (Route -> Data Model)", "Chain Depth", "Co-change %"]);
        for m in &packet.data_flow_matches {
            table.add_row(vec![
                m.chain_label.clone(),
                m.total_nodes.to_string(),
                format!("{:.0}%", m.change_pct * 100.0),
            ]);
        }
        println!("{table}");
    }

    if !packet.deploy_manifest_changes.is_empty() {
        println!("\n{}", "Deployment Manifest Changes:".bold());
        let mut table = build_table(["Type", "File Path", "Status"]);
        for change in &packet.deploy_manifest_changes {
            table.add_row(vec![
                format!("{:?}", change.manifest_type),
                change.file.display().to_string(),
                if change.is_deleted { "Deleted".red().to_string() } else { "Modified".yellow().to_string() },
            ]);
        }
        println!("{table}");
    }

    if !packet.relevant_decisions.is_empty() {
        println!("\n{}", "Relevant Architectural Decisions:".bold());
        let mut table = build_table(["Decision File", "Staleness", "Similarity"]);
        for decision in &packet.relevant_decisions {
            let threshold = config.coverage.adr_staleness.threshold_days;
            let staleness = match decision.staleness_days {
                Some(days) if days > threshold => format!("{} days (STALE)", days).red().bold().to_string(),
                Some(days) => format!("{} days", days).dimmed().to_string(),
                None => "Unknown".dimmed().to_string(),
            };
            table.add_row(vec![
                decision.file_path.display().to_string(),
                staleness,
                format!("{:.0}%", decision.similarity * 100.0),
            ]);
        }
        println!("{table}");
    }

    let mut partial_files: Vec<_> = packet
        .changes
        .iter()
        .filter(|file| !file.analysis_warnings.is_empty())
        .collect();

    if !partial_files.is_empty() {
        partial_files.sort_unstable_by_key(|f| &f.path);

        println!(
            "\n{} {} file(s) had partial or unsupported analysis:",
            "Warning:".yellow().bold(),
            partial_files.len()
        );

        let mut table = build_table(["File", "Issue", "Resolution"]);

        for file in &partial_files {
            let resolution = match file.analysis_status.symbols {
                AnalysisStatus::Unsupported => "Unsupported language",
                AnalysisStatus::ReadFailed => "Check file exists and is readable",
                AnalysisStatus::ExtractionFailed => "Check syntax or report bug",
                AnalysisStatus::Ok => "Partial — check warnings",
                AnalysisStatus::NotRun => "Analysis not yet run",
            };

            let warning_summary = file.analysis_warnings.join("; ");
            table.add_row(vec![
                file.path.display().to_string(),
                warning_summary,
                resolution.to_string(),
            ]);
        }

        println!("{table}");
    }
}

pub fn print_impact_brief(packet: &ImpactPacket) {
    let risk_color = match packet.risk_level {
        RiskLevel::Low => "LOW".green().bold().to_string(),
        RiskLevel::Medium => "MEDIUM".yellow().bold().to_string(),
        RiskLevel::High => "HIGH".red().bold().to_string(),
    };

    let change_count = packet.changes.len();
    let coupling_count = packet.temporal_couplings.len();
    let partial_count = packet
        .changes
        .iter()
        .filter(|f| !f.analysis_warnings.is_empty())
        .count();

    println!(
        "{} risk | {} changed | {} couplings{}",
        risk_color,
        change_count,
        coupling_count,
        if partial_count > 0 {
            format!(" | {} partial", partial_count)
        } else {
            String::new()
        },
    );
}

pub fn print_doctor_report(
    platform: &str,
    shell: &str,
    tools: &[(String, crate::platform::ExecutableStatus)],
    path_display: &str,
    path_kind: &str,
    is_wsl_mounted: bool,
    local_model_status: &str,
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

    println!("\n{:<20} {}", "Local Model:".bold(), local_model_status);

    println!(
        "\n{}",
        "Invocation: use `changeguard <command>` — not npx, cargo run, or any wrapper.".dimmed()
    );
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

pub fn print_hotspots_table_with_centrality(hotspots: &[crate::impact::packet::Hotspot]) {
    print_header("Codebase Hotspots (Risk Density)");

    if hotspots.is_empty() {
        println!("No hotspots identified.");
        return;
    }

    let mut table = build_table(["Rank", "Score", "Freq", "Comp", "Centrality", "File Path"]);

    for (i, hotspot) in hotspots.iter().enumerate() {
        let score_color = if hotspot.score > 0.7 {
            hotspot.score.to_string().red().bold().to_string()
        } else if hotspot.score > 0.4 {
            hotspot.score.to_string().yellow().to_string()
        } else {
            hotspot.score.to_string().green().to_string()
        };

        let centrality_str = hotspot
            .centrality
            .map(|c| c.to_string())
            .unwrap_or_else(|| "N/A".to_string());

        table.add_row(vec![
            (i + 1).to_string(),
            score_color,
            hotspot.frequency.to_string(),
            hotspot.complexity.to_string(),
            centrality_str,
            hotspot.path.display().to_string().cyan().to_string(),
        ]);
    }

    println!("{table}");
    println!(
        "\n{} High frequency + high complexity = high risk density. Centrality = entry points reachable.",
        "Note:".bold().dimmed()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::AffectedContract;
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
        print_impact_summary(&packet, &crate::config::model::Config::default());
    }

    #[test]
    fn test_print_impact_summary_with_contracts() {
        let packet = ImpactPacket {
            risk_level: RiskLevel::Medium,
            risk_reasons: vec!["Public contract potentially affected: POST /pets".to_string()],
            affected_contracts: vec![AffectedContract {
                endpoint_id: "api/openapi.json::GET::/pets".to_string(),
                path: "/pets".to_string(),
                method: "GET".to_string(),
                summary: "List all pets".to_string(),
                similarity: 0.85,
                spec_file: "api/openapi.json".to_string(),
            }],
            ..Default::default()
        };
        print_impact_summary(&packet, &crate::config::model::Config::default());
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
