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

pub fn print_impact_summary(packet: &ImpactPacket, _config: &crate::config::model::Config) {
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
                format!(
                    "{:?}{}",
                    drift.config_type,
                    if drift.is_deleted { " (DELETED)" } else { "" }
                ),
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

    if let Some(sdk_delta) = &packet.sdk_dependencies_delta
        && (!sdk_delta.added.is_empty()
            || !sdk_delta.removed.is_empty()
            || !sdk_delta.modified.is_empty())
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

    if let Some(delta) = &packet.service_map_delta
        && !delta.affected_services.is_empty()
    {
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

    if !packet.data_flow_matches.is_empty() {
        println!("\n{}", "Data-Flow Coupling (Route -> Model):".bold());
        let mut table = build_table([
            "Chain Pattern (Route -> Data Model)",
            "Chain Depth",
            "Co-change %",
        ]);
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
        let mut table = build_table(["Type", "File Path", "Tier", "High-Blast Resources"]);
        for change in &packet.deploy_manifest_changes {
            table.add_row(vec![
                format!("{:?}", change.manifest_type),
                change.file.display().to_string(),
                change.risk_tier.to_string(),
                if change.high_blast_resources.is_empty() {
                    "-".to_string()
                } else {
                    change.high_blast_resources.join(", ")
                },
            ]);
        }
        println!("{table}");
    }

    if let Some(ref ci) = packet.ci_config_change {
        println!("\n{}", "CI Pipeline Impact:".bold());
        if !ci.known_ci_files.is_empty() {
            println!(
                "  {} {}",
                "Known CI files changed:".bold().cyan(),
                ci.known_ci_files.join(", ")
            );
        }
        if !ci.pre_commit_files.is_empty() {
            println!(
                "  {} {}",
                "Pre-commit hooks changed:".bold().cyan(),
                ci.pre_commit_files.join(", ")
            );
        }
        if !ci.unknown_ci_files.is_empty() {
            println!(
                "  {} {}",
                "Unknown CI-like files:".bold().cyan(),
                ci.unknown_ci_files.join(", ")
            );
        }
        if !ci.generated_ci_files.is_empty() {
            println!(
                "  {} {}",
                "Generated CI files changed:".bold().cyan(),
                ci.generated_ci_files.join(", ")
            );
        }
        let mut flags = Vec::new();
        if ci.source_changed {
            flags.push("source files co-changed".yellow().to_string());
        }
        if ci.deploy_changed {
            flags.push("deploy manifests co-changed".yellow().to_string());
        }
        if !flags.is_empty() {
            println!("  {} {}", "Flags:".bold().cyan(), flags.join(" | "));
        }
    }

    if !packet.ci_predictions.is_empty() {
        println!("\n{}", "Predicted CI Failures:".bold());
        let mut table = build_table(["Job Name", "Platform", "Probability"]);
        for pred in &packet.ci_predictions {
            let prob_color = if pred.failure_probability > 0.7 {
                format!("{:.0}%", pred.failure_probability * 100.0)
                    .red()
                    .bold()
                    .to_string()
            } else if pred.failure_probability > 0.4 {
                format!("{:.0}%", pred.failure_probability * 100.0)
                    .yellow()
                    .to_string()
            } else {
                format!("{:.0}%", pred.failure_probability * 100.0)
                    .green()
                    .to_string()
            };
            table.add_row(vec![
                pred.job_name.clone(),
                pred.platform.clone(),
                prob_color,
            ]);
        }
        println!("{table}");
    }

    if !packet.dead_code_findings.is_empty() {
        println!("\n{}", "Dead Code Findings:".bold());
        let mut table = build_table(["Symbol", "File", "Confidence", "Factors"]);
        for finding in &packet.dead_code_findings {
            let confidence_color = if finding.confidence > 0.9 {
                format!("{:.0}%", finding.confidence * 100.0)
                    .red()
                    .bold()
                    .to_string()
            } else if finding.confidence > 0.75 {
                format!("{:.0}%", finding.confidence * 100.0)
                    .yellow()
                    .to_string()
            } else {
                format!("{:.0}%", finding.confidence * 100.0)
                    .green()
                    .to_string()
            };

            let factors: Vec<String> = finding
                .factors
                .iter()
                .map(|f| match f {
                    crate::impact::packet::ConfidenceFactor::UnreachableFromEntrypoints => {
                        "Unreachable".to_string()
                    }
                    crate::impact::packet::ConfidenceFactor::GitInactive {
                        days_since_last_commit,
                    } => format!("Inactive ({}d)", days_since_last_commit),
                    crate::impact::packet::ConfidenceFactor::NoTestCoverage => {
                        "No Tests".to_string()
                    }
                })
                .collect();

            table.add_row(vec![
                finding.symbol_name.clone(),
                finding.file_path.display().to_string(),
                confidence_color,
                factors.join(", "),
            ]);
        }
        println!("{table}");
    }

    if !packet.relevant_decisions.is_empty() {
        println!("\n{}", "Relevant Architectural Decisions:".bold());
        let mut table = build_table(["Decision File", "Staleness", "Similarity"]);
        for decision in &packet.relevant_decisions {
            let staleness = match (decision.staleness_days, decision.staleness_tier) {
                (Some(days), Some(crate::impact::packet::StalenessTier::Critical)) => {
                    format!("[STALE: {} days — Critical]", days)
                        .red()
                        .bold()
                        .to_string()
                }
                (Some(days), Some(crate::impact::packet::StalenessTier::Warning)) => {
                    format!("[STALE: {} days — Warning]", days)
                        .yellow()
                        .to_string()
                }
                (Some(days), None) => format!("{} days", days).dimmed().to_string(),
                (None, _) => "Unknown".dimmed().to_string(),
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

pub fn print_dead_code_summary(
    findings: &[crate::impact::packet::DeadCodeFinding],
    threshold: f64,
) {
    print_header("Dead Code Analysis");

    if findings.is_empty() {
        println!(
            "No dead code findings above the {:.0}% confidence threshold.",
            threshold * 100.0
        );
        return;
    }

    println!(
        "Found {} symbol(s) with confidence >= {:.0}%:\n",
        findings.len(),
        threshold * 100.0
    );

    let mut table = build_table(["Symbol", "File", "Confidence", "Factors"]);
    for finding in findings {
        let confidence_color = if finding.confidence > 0.9 {
            format!("{:.0}%", finding.confidence * 100.0)
                .red()
                .bold()
                .to_string()
        } else if finding.confidence > 0.75 {
            format!("{:.0}%", finding.confidence * 100.0)
                .yellow()
                .to_string()
        } else {
            format!("{:.0}%", finding.confidence * 100.0)
                .green()
                .to_string()
        };

        let factors: Vec<String> = finding
            .factors
            .iter()
            .map(|f| match f {
                crate::impact::packet::ConfidenceFactor::UnreachableFromEntrypoints => {
                    "Unreachable".to_string()
                }
                crate::impact::packet::ConfidenceFactor::GitInactive {
                    days_since_last_commit,
                } => format!("Inactive ({}d)", days_since_last_commit),
                crate::impact::packet::ConfidenceFactor::NoTestCoverage => "No Tests".to_string(),
            })
            .collect();

        table.add_row(vec![
            finding.symbol_name.clone(),
            finding.file_path.display().to_string(),
            confidence_color,
            factors.join(", "),
        ]);
    }

    println!("{table}");
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

pub struct DoctorReport<'a> {
    pub platform: &'a str,
    pub shell: &'a str,
    pub tools: &'a [(String, crate::platform::ExecutableStatus)],
    pub path_display: &'a str,
    pub path_kind: &'a str,
    pub is_wsl_mounted: bool,
    pub embedding_model_status: &'a str,
    pub completion_model_status: &'a str,
    pub native_graph_status: &'a str,
}

pub fn print_doctor_report(report: &DoctorReport) {
    println!(
        "\n{}",
        "ChangeGuard Doctor - Environment Health Check"
            .bold()
            .bright_cyan()
    );
    println!("{}", "=".repeat(50).cyan());

    println!("{:<20} {}", "Environment:".bold(), report.platform);
    println!("{:<20} {}", "Active Shell:".bold(), report.shell);

    println!("\n{}", "Tools:".bold().bright_cyan());
    for (name, status) in report.tools {
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

    println!("\n{:<20} {}", "Current Path:".bold(), report.path_display);
    println!("{:<20} {}", "Path Type:".bold(), report.path_kind);

    if report.is_wsl_mounted {
        println!(
            "\n{}",
            "Warning: Running on a WSL mounted drive may be slower due to cross-filesystem overhead."
                .yellow()
                .italic()
        );
    }

    println!(
        "\n{:<20} {}",
        "Embedding Model:".bold(),
        report.embedding_model_status
    );
    println!(
        "{:<20} {}",
        "Completion Model:".bold(),
        report.completion_model_status
    );
    println!(
        "{:<20} {}",
        "Native Graph:".bold(),
        report.native_graph_status
    );

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
        let score_display = format!("{:.3}", hotspot.display_score);
        let score_color = if hotspot.score > 0.7 {
            score_display.red().bold().to_string()
        } else if hotspot.score > 0.4 {
            score_display.yellow().to_string()
        } else {
            score_display.dimmed().to_string()
        };

        table.add_row(vec![
            (i + 1).to_string(),
            score_color,
            format!("{:.1}", hotspot.frequency),
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
        let score_display = format!("{:.3}", hotspot.display_score);
        let score_color = if hotspot.score > 0.7 {
            score_display.red().bold().to_string()
        } else if hotspot.score > 0.4 {
            score_display.yellow().to_string()
        } else {
            score_display.dimmed().to_string()
        };

        let centrality_str = hotspot
            .centrality
            .map(|c| c.to_string())
            .unwrap_or_else(|| "N/A".to_string());

        table.add_row(vec![
            (i + 1).to_string(),
            score_color,
            format!("{:.1}", hotspot.frequency),
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

    #[test]
    fn test_print_impact_summary_shows_critical_staleness() {
        use crate::impact::packet::{RelevantDecision, StalenessTier};
        let packet = ImpactPacket {
            relevant_decisions: vec![RelevantDecision {
                file_path: PathBuf::from("docs/adr.md"),
                heading: Some("Architecture".to_string()),
                excerpt: "ADR content".to_string(),
                similarity: 0.9,
                rerank_score: None,
                staleness_days: Some(800),
                staleness_tier: Some(StalenessTier::Critical),
            }],
            ..Default::default()
        };
        // Just verify no panic and the function accepts the new fields
        print_impact_summary(&packet, &crate::config::model::Config::default());
    }
}
