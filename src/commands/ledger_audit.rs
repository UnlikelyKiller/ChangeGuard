use camino::Utf8PathBuf;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Table};
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::Serialize;
use std::env;

use crate::config::load::load_config;
use crate::config::model::Config;
use crate::impact::hotspots::calculate_hotspots;
use crate::impact::packet::Hotspot;
use crate::impact::temporal::GixHistoryProvider;
use crate::ledger::db::LedgerDb;
use crate::ledger::transaction::TransactionManager;
use crate::ledger::types::LedgerEntry;
use crate::ledger::ui::{LedgerStatus, get_change_type_icon, get_status_icon};
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use crate::verify::results::{VERIFY_HISTORY, VerifyHistoryRecord};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAuditReport {
    pub velocity: VelocitySummary,
    pub churn: Vec<ChurnEntry>,
    pub unaudited_drift: Vec<DriftEntry>,
    pub hotspots: Vec<Hotspot>,
    pub ci_trend: Vec<bool>,
    pub recent_entries: Vec<AuditEntry>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VelocitySummary {
    pub last_7_days: i64,
    pub last_30_days: i64,
    pub total: i64,
    pub pending: i64,
    pub federated: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChurnEntry {
    pub entity: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriftEntry {
    pub file_path: String,
    pub change_type: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub id: i64,
    pub tx_id: String,
    pub entity: String,
    pub trace_id: Option<String>,
    pub origin: String,
    pub summary: String,
    pub reason: String,
    pub change_type: crate::ledger::ChangeType,
    pub committed_at: String,
    pub is_breaking: bool,
    pub provenance: Vec<ProvenanceEntry>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvenanceEntry {
    pub entity: String,
    pub symbol_name: String,
    pub symbol_type: String,
    pub action: crate::ledger::provenance::ProvenanceAction,
}

fn get_repo_root() -> Result<Utf8PathBuf> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let discovered = gix::discover(&current_dir).into_diagnostic()?;
    let root = discovered
        .workdir()
        .ok_or_else(|| miette::miette!("Failed to find work directory for repository"))?;

    Utf8PathBuf::from_path_buf(root.to_path_buf())
        .map_err(|_| miette::miette!("Repository root is not valid UTF-8"))
}

fn get_layout() -> Result<Layout> {
    let root = get_repo_root()?;
    Ok(Layout::new(root))
}

fn load_ledger_config(layout: &Layout) -> Config {
    load_config(layout).unwrap_or_else(|e| {
        tracing::warn!("Failed to load config: {e}. Using defaults.");
        Config::default()
    })
}

pub fn execute_ledger_audit(
    entity: Option<String>,
    include_unaudited: bool,
    limit: usize,
    offset: usize,
    json: bool,
) -> Result<()> {
    let layout = get_layout()?;
    let mut storage = StorageManager::open_read_only(&layout.root)?;
    let config = load_ledger_config(&layout);
    let manager = TransactionManager::new(
        storage.get_connection_mut(),
        layout.root.clone().into(),
        config,
    );

    if !json {
        println!("{}", "ChangeGuard Project Audit".bold().underline());
    }

    if let Some(path) = entity {
        audit_entity(&manager, &path, limit, offset, json)?;
    } else {
        audit_global(&manager, include_unaudited, limit, offset, json)?;
    }

    Ok(())
}

fn gather_audit_data(
    manager: &TransactionManager,
    layout: &Layout,
    include_unaudited: bool,
    limit: usize,
    offset: usize,
) -> Result<ProjectAuditReport> {
    let db = LedgerDb::new(manager.get_connection());

    // 1. Velocity
    let v_7 = db
        .get_transaction_velocity(7)
        .map_err(|e| miette::miette!("{}", e))?;
    let v_30 = db
        .get_transaction_velocity(30)
        .map_err(|e| miette::miette!("{}", e))?;
    let total = db
        .get_transaction_velocity(36500) // ~100 years
        .map_err(|e| miette::miette!("{}", e))?;
    let pending_count = manager
        .get_all_pending()
        .map_err(|e| miette::miette!("{}", e))?
        .len() as i64;

    // We don't have a specific get_total_federated_commit_count, so we'll query for them
    let federated_count = db
        .get_federated_entries_by_entity("%", "%", 1000000)
        .map(|entries| entries.len() as i64)
        .unwrap_or(0);

    let velocity = VelocitySummary {
        last_7_days: v_7 as i64,
        last_30_days: v_30 as i64,
        total: total as i64,
        pending: pending_count,
        federated: federated_count,
    };

    // 2. Churn (respecting limit)
    let churn_data = db
        .get_top_churned_entities(limit)
        .map_err(|e| miette::miette!("{}", e))?;
    let churn = churn_data
        .into_iter()
        .map(|(e, c)| ChurnEntry {
            entity: e,
            count: c as i64,
        })
        .collect();

    // 3. Unaudited Drift
    let unaudited_drift = if include_unaudited {
        let unaudited = manager
            .get_all_unaudited()
            .map_err(|e| miette::miette!("{}", e))?;
        // Use the entity field since get_all_unaudited returns Transaction objects
        unaudited
            .into_iter()
            .map(|u| DriftEntry {
                file_path: u.entity,
                change_type: format!("{:?}", u.category), // Stale transactions don't have a ChangeType, category is close enough for drift
            })
            .collect()
    } else {
        vec![]
    };

    // 4. Hotspots (respecting limit)
    let storage = StorageManager::open_read_only(&layout.root)?;
    let discovered = gix::discover(&layout.root).into_diagnostic()?;
    let history_provider = GixHistoryProvider::new(&discovered);
    let hotspots = calculate_hotspots(
        &storage,
        &history_provider,
        &crate::impact::hotspots::HotspotQuery {
            commits: 500,
            limit,
            decay_half_life: 100, // Default decay half-life for audit summary
            ..Default::default()
        },
    )
    .unwrap_or_default();

    // 5. CI Trend
    let history_path = layout.reports_dir().join(VERIFY_HISTORY);
    let ci_trend = if history_path.exists() {
        let content = std::fs::read_to_string(&history_path).into_diagnostic()?;
        let history: Vec<VerifyHistoryRecord> = serde_json::from_str(&content).unwrap_or_default();
        history
            .into_iter()
            .rev()
            .take(limit)
            .map(|h| h.passed)
            .collect()
    } else {
        vec![]
    };

    // 6. Recent Entries
    let recent = db
        .get_recent_ledger_entries_paginated(limit, offset)
        .map_err(|e| miette::miette!("{}", e))?;

    let recent_entries = audit_entries_from_ledger_entries(&db, recent)?;

    Ok(ProjectAuditReport {
        velocity,
        churn,
        unaudited_drift,
        hotspots,
        ci_trend,
        recent_entries,
    })
}

fn audit_entries_from_ledger_entries(
    db: &LedgerDb<'_>,
    entries: Vec<LedgerEntry>,
) -> Result<Vec<AuditEntry>> {
    let mut audit_entries = Vec::new();
    for entry in entries {
        let provenance_data = db
            .get_token_provenance_for_tx(&entry.tx_id)
            .map_err(|e| miette::miette!("{}", e))?;

        let provenance = provenance_data
            .into_iter()
            .map(|p| ProvenanceEntry {
                entity: p.entity,
                symbol_name: p.symbol_name,
                symbol_type: p.symbol_type,
                action: p.action,
            })
            .collect();

        audit_entries.push(AuditEntry {
            id: entry.id,
            tx_id: entry.tx_id,
            entity: entry.entity,
            trace_id: entry.trace_id,
            origin: entry.origin,
            summary: entry.summary,
            reason: entry.reason,
            change_type: entry.change_type,
            committed_at: entry.committed_at,
            is_breaking: entry.is_breaking,
            provenance,
        });
    }

    Ok(audit_entries)
}

fn audit_global(
    manager: &TransactionManager,
    include_unaudited: bool,
    limit: usize,
    offset: usize,
    json: bool,
) -> Result<()> {
    let layout = get_layout()?;
    let report = gather_audit_data(manager, &layout, include_unaudited, limit, offset)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).into_diagnostic()?
        );
        return Ok(());
    }

    render_project_audit_human(&report, include_unaudited, limit, offset);
    Ok(())
}

fn render_project_audit_human(
    report: &ProjectAuditReport,
    include_unaudited: bool,
    limit: usize,
    offset: usize,
) {
    // Print Human Readable Report
    println!("\n{}", "PROJECT VELOCITY".blue().bold());
    println!(
        "  Last 7 Days:   {}",
        report.velocity.last_7_days.to_string().yellow()
    );
    println!(
        "  Last 30 Days:  {}",
        report.velocity.last_30_days.to_string().yellow()
    );
    println!(
        "  Total Commits: {}",
        report.velocity.total.to_string().cyan()
    );
    println!(
        "  Pending:       {}",
        report.velocity.pending.to_string().magenta()
    );
    println!(
        "  Federated:     {}",
        report.velocity.federated.to_string().magenta()
    );

    println!(
        "\n{}",
        format!("TOP CHURNED FILES (Limit: {})", limit)
            .blue()
            .bold()
    );
    if report.churn.is_empty() {
        println!("  None.");
    } else {
        for c in &report.churn {
            println!(
                "  {:<40} {} commits",
                c.entity.cyan(),
                c.count.to_string().yellow()
            );
        }
    }

    if include_unaudited {
        println!("\n{}", "UNAUDITED DRIFT".red().bold());
        if report.unaudited_drift.is_empty() {
            println!("  None.");
        } else {
            for d in &report.unaudited_drift {
                println!("  {:<40} {}", d.file_path.cyan(), d.change_type.yellow());
            }
        }
    }

    println!(
        "\n{}",
        format!("TOP HOTSPOTS (Limit: {})", limit).yellow().bold()
    );
    if report.hotspots.is_empty() {
        println!("  None.");
    } else {
        for h in &report.hotspots {
            println!(
                "  {:<40} score: {:.2}",
                h.path.display().to_string().cyan(),
                h.display_score.yellow()
            );
        }
    }

    println!("\n{}", format!("CI TREND (Last {})", limit).yellow().bold());
    if report.ci_trend.is_empty() {
        println!("  No history yet.");
    } else {
        let mut trend_str = String::new();
        // The gather function gives them newest first, so we rev() to print oldest to newest left to right
        for passed in report.ci_trend.iter().rev() {
            if *passed {
                trend_str.push_str(&"PASS".green().to_string());
            } else {
                trend_str.push_str(&"FAIL".red().to_string());
            }
            trend_str.push(' ');
        }
        println!("  {}", trend_str);
    }

    println!(
        "\n{}",
        format!(
            "RECENT COMMITTED ENTRIES (Limit: {}, Offset: {})",
            limit, offset
        )
        .green()
        .bold()
    );
    if report.recent_entries.is_empty() {
        println!("  None.");
    } else {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec![
                Cell::new("ID").fg(Color::Cyan),
                Cell::new("TX ID").fg(Color::Cyan),
                Cell::new("Entity").fg(Color::Cyan),
                Cell::new("Change").fg(Color::Cyan),
                Cell::new("Summary").fg(Color::Cyan),
                Cell::new("Committed").fg(Color::Cyan),
            ]);

        for entry in &report.recent_entries {
            // Pick an entity to display in the table, or "Multiple"
            let entity_display = if entry.provenance.is_empty() {
                entry.entity.clone()
            } else if entry.provenance.len() == 1 {
                entry.provenance[0].entity.clone()
            } else {
                let first = &entry.provenance[0].entity;
                if entry.provenance.iter().all(|p| p.entity == *first) {
                    first.clone()
                } else {
                    format!("{} (+{})", first, entry.provenance.len() - 1)
                }
            };

            table.add_row(vec![
                Cell::new(entry.id.to_string()),
                Cell::new(&entry.tx_id[..8]).fg(Color::Yellow),
                Cell::new(&entity_display).fg(Color::Cyan),
                Cell::new(format!(
                    "{} {:?}",
                    get_change_type_icon(&entry.change_type),
                    entry.change_type
                )),
                Cell::new(&entry.summary),
                Cell::new(&entry.committed_at),
            ]);
        }
        println!("{table}");
    }
}

fn audit_entity(
    manager: &TransactionManager,
    entity: &str,
    limit: usize,
    offset: usize,
    json: bool,
) -> Result<()> {
    let db = LedgerDb::new(manager.get_connection());
    let entries = manager
        .get_ledger_entries_paginated(entity, limit, offset)
        .map_err(|e| miette::miette!("{}", e))?;

    if json {
        let audit_entries = audit_entries_from_ledger_entries(&db, entries)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&audit_entries).into_diagnostic()?
        );
        return Ok(());
    }

    println!("\nAudit History for {}:", entity.cyan());

    if entries.is_empty() {
        println!("  No committed entries found.");
    } else {
        for entry in entries {
            let prefix = if entry.origin == "LOCAL" {
                format!(
                    "{} [{:04}]",
                    get_status_icon(LedgerStatus::Committed),
                    entry.id
                )
                .yellow()
                .to_string()
            } else {
                format!(
                    "{} [FEDERATED: {}]",
                    get_status_icon(LedgerStatus::Federated),
                    entry.trace_id.as_deref().unwrap_or("UNKNOWN")
                )
                .magenta()
                .bold()
                .to_string()
            };

            println!("\n{} committed on {}", prefix, entry.committed_at.dimmed());
            println!("  Summary: {}", entry.summary.bold());
            println!(
                "  Change:  {} {:?}",
                get_change_type_icon(&entry.change_type),
                entry.change_type
            );
            println!("  Reason:  {}", entry.reason);

            // Show token provenance
            let provenance = db
                .get_token_provenance_for_tx(&entry.tx_id)
                .map_err(|e| miette::miette!("{}", e))?;

            // Filter provenance by this entity (since a TX could span multiple, though usually one per start)
            let entity_prov: Vec<_> = provenance
                .into_iter()
                .filter(|p| p.entity == entity)
                .collect();

            if !entity_prov.is_empty() {
                println!("  Symbols:");
                for p in entity_prov {
                    let action_str = p.action.to_string();
                    let formatted = match p.action {
                        crate::ledger::provenance::ProvenanceAction::Added => {
                            action_str.green().to_string()
                        }
                        crate::ledger::provenance::ProvenanceAction::Modified => {
                            action_str.blue().to_string()
                        }
                        crate::ledger::provenance::ProvenanceAction::Deleted => {
                            action_str.red().to_string()
                        }
                    };
                    println!(
                        "    {} {:<10} {} ({})",
                        "•".dimmed(),
                        formatted,
                        p.symbol_name,
                        p.symbol_type.dimmed()
                    );
                }
            }
        }
    }

    Ok(())
}
