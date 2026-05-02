use crate::state::storage::StorageManager;
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CIGateStats {
    pub total_gates: usize,
    pub github_actions_gates: usize,
    pub gitlab_ci_gates: usize,
    pub circleci_gates: usize,
    pub makefile_gates: usize,
    pub files_processed: usize,
}

struct CIGateRow {
    ci_file_id: i64,
    platform: String,
    job_name: String,
    trigger: Option<String>,
    steps: Option<String>,
}

const CI_GATE_BATCH_SIZE: usize = 500;

pub struct CIGateExtractor<'a> {
    storage: &'a StorageManager,
    repo_path: PathBuf,
}

impl<'a> CIGateExtractor<'a> {
    pub fn new(storage: &'a StorageManager, repo_path: PathBuf) -> Self {
        Self { storage, repo_path }
    }

    pub fn extract(&self) -> Result<CIGateStats> {
        // 1. Ensure all CI config files are registered in project_files
        let ci_files = self.discover_ci_files();

        if ci_files.is_empty() {
            info!("CI gates extraction: no CI config files found");
            return Ok(CIGateStats {
                total_gates: 0,
                github_actions_gates: 0,
                gitlab_ci_gates: 0,
                circleci_gates: 0,
                makefile_gates: 0,
                files_processed: 0,
            });
        }

        // 2. Clear existing ci_gates data before re-indexing
        {
            let conn = self.storage.get_connection();
            conn.execute("DELETE FROM ci_gates", []).into_diagnostic()?;
        }

        let now = chrono::Utc::now().to_rfc3339();
        let mut total_gates = 0usize;
        let mut github_actions_gates = 0usize;
        let mut gitlab_ci_gates = 0usize;
        let mut circleci_gates = 0usize;
        let mut makefile_gates = 0usize;
        let mut files_processed = 0usize;
        let mut batch: Vec<CIGateRow> = Vec::new();

        for (relative_path, platform) in &ci_files {
            let full_path = self.repo_path.join(relative_path);
            let content = match std::fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(e) => {
                    warn!("Failed to read CI config file {}: {}", relative_path, e);
                    continue;
                }
            };

            // Get or create the project_files entry
            let file_id = self.ensure_file_entry(relative_path, &content, &now)?;

            // Parse based on platform
            let gates = match platform.as_str() {
                "github_actions" => parse_github_actions(&content),
                "gitlab_ci" => parse_gitlab_ci(&content),
                "circleci" => parse_circleci(&content),
                "makefile" => parse_makefile(&content),
                _ => Vec::new(),
            };

            for gate in &gates {
                batch.push(CIGateRow {
                    ci_file_id: file_id,
                    platform: platform.clone(),
                    job_name: gate.job_name.clone(),
                    trigger: gate.trigger.clone(),
                    steps: gate.steps.clone(),
                });

                match platform.as_str() {
                    "github_actions" => github_actions_gates += 1,
                    "gitlab_ci" => gitlab_ci_gates += 1,
                    "circleci" => circleci_gates += 1,
                    "makefile" => makefile_gates += 1,
                    _ => {}
                }

                total_gates += 1;

                if batch.len() >= CI_GATE_BATCH_SIZE {
                    self.insert_batch(&batch)?;
                    batch.clear();
                }
            }

            files_processed += 1;
        }

        // Flush remaining
        if !batch.is_empty() {
            self.insert_batch(&batch)?;
        }

        info!(
            "CI gates extraction complete: {} gates from {} files ({} GitHub Actions, {} GitLab CI, {} CircleCI, {} Makefile)",
            total_gates,
            files_processed,
            github_actions_gates,
            gitlab_ci_gates,
            circleci_gates,
            makefile_gates
        );

        Ok(CIGateStats {
            total_gates,
            github_actions_gates,
            gitlab_ci_gates,
            circleci_gates,
            makefile_gates,
            files_processed,
        })
    }

    fn discover_ci_files(&self) -> Vec<(String, String)> {
        let mut ci_files = Vec::new();

        // GitHub Actions: .github/workflows/*.yml and *.yaml
        let workflows_dir = self.repo_path.join(".github").join("workflows");
        if workflows_dir.exists()
            && let Ok(entries) = std::fs::read_dir(&workflows_dir)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str())
                    && (ext == "yml" || ext == "yaml")
                    && let Ok(relative) = path.strip_prefix(&self.repo_path)
                {
                    ci_files.push((
                        relative.to_string_lossy().replace('\\', "/"),
                        "github_actions".to_string(),
                    ));
                }
            }
        }

        // GitLab CI: .gitlab-ci.yml
        let gitlab_ci_path = self.repo_path.join(".gitlab-ci.yml");
        if gitlab_ci_path.exists() {
            ci_files.push((".gitlab-ci.yml".to_string(), "gitlab_ci".to_string()));
        }

        // CircleCI: .circleci/config.yml
        let circleci_path = self.repo_path.join(".circleci").join("config.yml");
        if circleci_path.exists() {
            ci_files.push((".circleci/config.yml".to_string(), "circleci".to_string()));
        }

        // Makefiles
        for makefile_name in &["Makefile", "makefile", "GNUmakefile"] {
            let makefile_path = self.repo_path.join(makefile_name);
            if makefile_path.exists() {
                ci_files.push((makefile_name.to_string(), "makefile".to_string()));
            }
        }

        ci_files
    }

    fn ensure_file_entry(&self, relative_path: &str, content: &str, now: &str) -> Result<i64> {
        let content_hash = blake3::hash(content.as_bytes()).to_hex().to_string();

        // Check if file already exists
        let conn = self.storage.get_connection();
        let existing_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM project_files WHERE file_path = ?1",
                [relative_path],
                |row| row.get(0),
            )
            .ok();

        if let Some(id) = existing_id {
            return Ok(id);
        }

        // Insert new file entry using a transaction
        let conn = self.storage.get_connection();
        let tx = conn.unchecked_transaction().into_diagnostic()?;
        tx.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, parser_version, parse_status, last_indexed_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                relative_path,
                "YAML", // Most CI configs are YAML
                content_hash,
                content.len() as i64,
                "1",
                "OK",
                now,
            ],
        )
        .into_diagnostic()?;

        let id = tx.last_insert_rowid();
        tx.commit().into_diagnostic()?;
        Ok(id)
    }

    fn insert_batch(&self, rows: &[CIGateRow]) -> Result<()> {
        let conn = self.storage.get_connection();
        let tx = conn.unchecked_transaction().into_diagnostic()?;
        let now = chrono::Utc::now().to_rfc3339();

        for row in rows {
            tx.execute(
                "INSERT INTO ci_gates (ci_file_id, platform, job_name, trigger, steps, last_indexed_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    row.ci_file_id,
                    row.platform,
                    row.job_name,
                    row.trigger,
                    row.steps,
                    now,
                ],
            )
            .into_diagnostic()?;
        }

        tx.commit().into_diagnostic()?;
        Ok(())
    }
}

// --- Parsed CI gate struct (used internally during extraction) ---

struct ParsedCIGate {
    job_name: String,
    trigger: Option<String>,
    steps: Option<String>,
}

// --- GitHub Actions Parser ---

fn parse_github_actions(content: &str) -> Vec<ParsedCIGate> {
    let mut gates = Vec::new();
    let mut triggers = Vec::new();

    // Extract triggers from the "on:" section
    // Look for: on: push:, on: pull_request:, on: [push, pull_request], etc.
    let mut in_on_section = false;
    let mut on_indent = 0;
    let mut brace_depth = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('#') {
            continue;
        }

        // Detect "on:" key at top level
        let line_indent = line.len() - line.trim_start().len();
        if trimmed == "on:" || trimmed.starts_with("on:") {
            in_on_section = true;
            on_indent = line_indent;
            brace_depth = 0;
            // Check for inline triggers like: on: push
            if trimmed != "on:" {
                let rest = trimmed.strip_prefix("on:").unwrap_or("").trim();
                if !rest.is_empty() && !rest.contains('{') && !rest.contains(':') {
                    triggers.push(rest.to_string());
                }
            }
            continue;
        }

        if in_on_section {
            // Check if we've left the "on:" section
            if !trimmed.is_empty() && line_indent <= on_indent && !trimmed.starts_with('-') {
                // We've hit a new top-level key
                in_on_section = false;
            } else {
                // Parse trigger types within the "on:" section
                if trimmed.contains('{') {
                    brace_depth += trimmed.matches('{').count();
                }
                if trimmed.contains('}') {
                    brace_depth = brace_depth.saturating_sub(trimmed.matches('}').count());
                }

                if brace_depth == 0 {
                    // Extract trigger type from lines like "push:", "pull_request:", etc.
                    if let Some(key) = trimmed.strip_suffix(':')
                        && !key.starts_with('#')
                        && !key.contains(' ')
                        && key != "branches"
                        && key != "paths"
                        && key != "tags"
                    {
                        triggers.push(key.to_string());
                    }
                    // Also handle array-style: on: [push, pull_request]
                    if trimmed.starts_with('-') {
                        let item = trimmed.strip_prefix('-').unwrap_or(trimmed).trim();
                        let item = item.strip_suffix(',').unwrap_or(item).trim();
                        if !item.is_empty() && !item.starts_with('#') {
                            triggers.push(item.to_string());
                        }
                    }
                }
            }
        }
    }

    let trigger_str = if triggers.is_empty() {
        None
    } else {
        Some(triggers.join(", "))
    };

    // Extract jobs from the "jobs:" section
    let mut jobs = HashMap::new();
    let mut in_jobs_section = false;
    let mut current_job: Option<String> = None;
    let mut current_job_steps: Vec<String> = Vec::new();
    let mut in_steps = false;
    let mut jobs_indent = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }

        let line_indent = line.len() - line.trim_start().len();

        if trimmed == "jobs:" || trimmed.starts_with("jobs:") {
            in_jobs_section = true;
            jobs_indent = line_indent;
            continue;
        }

        if in_jobs_section {
            // Check if we've left the jobs section
            if !trimmed.is_empty() && line_indent <= jobs_indent {
                in_jobs_section = false;
                if let Some(job_name) = current_job.take() {
                    jobs.insert(job_name, current_job_steps.clone());
                    current_job_steps.clear();
                }
                continue;
            }

            // Detect job name (2-level indent under jobs:)
            if line_indent == jobs_indent + 2 && trimmed.ends_with(':') && !trimmed.starts_with('-')
            {
                // Save previous job
                if let Some(job_name) = current_job.take() {
                    jobs.insert(job_name, current_job_steps.clone());
                    current_job_steps.clear();
                }
                let name = trimmed.strip_suffix(':').unwrap_or(trimmed).to_string();
                current_job = Some(name);
                in_steps = false;
                continue;
            }

            // Detect "steps:" section
            if trimmed == "steps:" || trimmed.starts_with("steps:") {
                in_steps = true;
                continue;
            }

            // If we hit another key at the same level as steps, end steps section
            if in_steps && line_indent > 0 {
                // Check for step name or run command
                if trimmed.starts_with("- name:") || trimmed.starts_with("- name :") {
                    let name = trimmed
                        .strip_prefix("-")
                        .unwrap_or(trimmed)
                        .trim()
                        .strip_prefix("name")
                        .unwrap_or(trimmed)
                        .trim()
                        .strip_prefix(":")
                        .unwrap_or(trimmed)
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                    if !name.is_empty() {
                        current_job_steps.push(name);
                    }
                } else if trimmed.starts_with("- run:") || trimmed.starts_with("- run :") {
                    let cmd = trimmed
                        .strip_prefix("-")
                        .unwrap_or(trimmed)
                        .trim()
                        .strip_prefix("run")
                        .unwrap_or(trimmed)
                        .trim()
                        .strip_prefix(":")
                        .unwrap_or(trimmed)
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                    if !cmd.is_empty() {
                        current_job_steps.push(cmd);
                    }
                }
            }
        }
    }

    // Save last job
    if let Some(job_name) = current_job.take() {
        jobs.insert(job_name, current_job_steps.clone());
    }

    for (job_name, steps) in &jobs {
        let steps_str = if steps.is_empty() {
            None
        } else {
            Some(steps.join("; "))
        };
        gates.push(ParsedCIGate {
            job_name: job_name.clone(),
            trigger: trigger_str.clone(),
            steps: steps_str,
        });
    }

    gates
}

// --- GitLab CI Parser ---

fn parse_gitlab_ci(content: &str) -> Vec<ParsedCIGate> {
    let mut gates = Vec::new();
    let mut stages: Vec<String> = Vec::new();

    // Extract stages
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("stages:") {
            // Read array-style stages on following lines
            continue;
        }
        if trimmed.starts_with('-') && !stages.is_empty() {
            // Could be a stage entry
            let item = trimmed.strip_prefix('-').unwrap_or(trimmed).trim();
            let item = item.trim_matches('"').trim_matches('\'');
            if !item.is_empty() && !item.contains(':') && !item.contains('{') {
                // Might be a stage, but we don't know if we're in the stages section
            }
        }
    }

    // Parse simple stages list
    let mut in_stages = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("stages:") {
            in_stages = true;
            // Check for inline array: stages: [build, test]
            if let Some(rest) = trimmed.strip_prefix("stages:") {
                let rest = rest.trim();
                if rest.starts_with('[') {
                    // Inline array
                    let inner = rest.trim_start_matches('[').trim_end_matches(']');
                    for stage in inner.split(',') {
                        let s = stage.trim().trim_matches('"').trim_matches('\'');
                        if !s.is_empty() {
                            stages.push(s.to_string());
                        }
                    }
                    in_stages = false;
                }
            }
            continue;
        }
        if in_stages {
            if trimmed.starts_with('-') {
                let item = trimmed.strip_prefix('-').unwrap_or(trimmed).trim();
                let item = item.trim_matches('"').trim_matches('\'');
                if !item.is_empty() && !item.contains(':') {
                    stages.push(item.to_string());
                }
            } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                in_stages = false;
            }
        }
    }

    // Extract job definitions (top-level keys with script/stage/extends)
    let mut current_job: Option<String> = None;
    let mut current_steps: Vec<String> = Vec::new();
    let mut current_trigger: Option<String> = None;

    let top_level_keys = [
        "image",
        "services",
        "stages",
        "variables",
        "before_script",
        "after_script",
        "cache",
        "default",
        "include",
        "workflow",
    ];
    let top_level_set: std::collections::HashSet<&str> = top_level_keys.iter().copied().collect();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        let line_indent = line.len() - line.trim_start().len();

        // Top-level key
        if line_indent == 0 && trimmed.contains(':') && !trimmed.starts_with('-') {
            // Save previous job
            if let Some(job_name) = current_job.take() {
                if !top_level_set.contains(job_name.as_str()) {
                    gates.push(ParsedCIGate {
                        job_name: job_name.clone(),
                        trigger: current_trigger.take(),
                        steps: if current_steps.is_empty() {
                            None
                        } else {
                            Some(current_steps.join("; "))
                        },
                    });
                }
                current_steps.clear();
            }

            let colon_pos = trimmed.find(':').unwrap_or(trimmed.len());
            let key = &trimmed[..colon_pos].trim();

            current_job = Some(key.to_string());
            current_trigger = None;

            // Check if this is a non-job key
            if top_level_set.contains(key) {
                current_job = None;
            }
        }

        // Extract script lines within a job
        if current_job.is_some() {
            if trimmed.starts_with("- ") || trimmed.starts_with("-") {
                // Could be a script step
                let item = trimmed.strip_prefix('-').unwrap_or(trimmed).trim();
                let item = item.trim_matches('"').trim_matches('\'');
                if !item.is_empty() && !item.contains("${") && item.len() < 200 {
                    current_steps.push(item.to_string());
                }
            }

            // Extract trigger info (only/except rules)
            if (trimmed.starts_with("only:") || trimmed.starts_with("except:"))
                && let Some(trigger_type) = trimmed.strip_suffix(':')
            {
                current_trigger = Some(trigger_type.to_string());
            }
        }
    }

    // Save last job
    if let Some(job_name) = current_job.take()
        && !top_level_set.contains(job_name.as_str())
    {
        gates.push(ParsedCIGate {
            job_name: job_name.clone(),
            trigger: current_trigger,
            steps: if current_steps.is_empty() {
                None
            } else {
                Some(current_steps.join("; "))
            },
        });
    }

    // If no trigger was detected, use stages as triggers
    if !stages.is_empty() {
        let trigger_str = stages.join(", ");
        for gate in &mut gates {
            if gate.trigger.is_none() {
                gate.trigger = Some(format!("stages: {}", trigger_str));
            }
        }
    }

    gates
}

// --- CircleCI Parser ---

fn parse_circleci(content: &str) -> Vec<ParsedCIGate> {
    let mut gates = Vec::new();

    // CircleCI uses version: 2.1 (or 2) and has jobs: and workflows: sections
    let mut in_jobs = false;
    let mut in_workflows = false;
    let mut current_job: Option<String> = None;
    let mut current_steps: Vec<String> = Vec::new();
    let job_triggers: HashMap<String, String> = HashMap::new();
    let mut current_workflow: Option<String> = None;
    let mut jobs_indent = 0;
    let mut workflows_indent = 0;

    // First pass: extract workflow triggers
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        let line_indent = line.len() - line.trim_start().len();

        if trimmed == "workflows:" || trimmed.starts_with("workflows:") {
            in_workflows = true;
            workflows_indent = line_indent;
            continue;
        }

        if in_workflows {
            if !trimmed.is_empty() && line_indent <= workflows_indent {
                in_workflows = false;
                current_workflow = None;
                continue;
            }

            // Workflow name
            if line_indent == workflows_indent + 2
                && trimmed.ends_with(':')
                && !trimmed.starts_with('-')
            {
                let name = trimmed.strip_suffix(':').unwrap_or(trimmed).to_string();
                current_workflow = Some(name);
                continue;
            }

            // Look for triggers
            if current_workflow.is_some()
                && (trimmed.contains("filters:") || trimmed.starts_with("-"))
            {
                // Check for branch filters
                let item = trimmed.strip_prefix('-').unwrap_or(trimmed).trim();
                if item.contains(':') {
                    // It's a mapping, could be a trigger
                }
            }
        }
    }

    // Second pass: extract jobs
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        let line_indent = line.len() - line.trim_start().len();

        if trimmed == "jobs:" || trimmed.starts_with("jobs:") {
            in_jobs = true;
            jobs_indent = line_indent;
            continue;
        }

        if in_jobs {
            if !trimmed.is_empty() && line_indent <= jobs_indent {
                in_jobs = false;
                if let Some(job_name) = current_job.take() {
                    gates.push(ParsedCIGate {
                        job_name: job_name.clone(),
                        trigger: job_triggers.get(&job_name).cloned(),
                        steps: if current_steps.is_empty() {
                            None
                        } else {
                            Some(current_steps.join("; "))
                        },
                    });
                    current_steps.clear();
                }
                continue;
            }

            // Job name at 2 spaces indent
            if line_indent == jobs_indent + 2 && trimmed.ends_with(':') && !trimmed.starts_with('-')
            {
                // Save previous job
                if let Some(job_name) = current_job.take() {
                    gates.push(ParsedCIGate {
                        job_name: job_name.clone(),
                        trigger: job_triggers.get(&job_name).cloned(),
                        steps: if current_steps.is_empty() {
                            None
                        } else {
                            Some(current_steps.join("; "))
                        },
                    });
                    current_steps.clear();
                }
                let name = trimmed.strip_suffix(':').unwrap_or(trimmed).to_string();
                current_job = Some(name);
                continue;
            }

            // Extract steps
            if let Some(ref _job) = current_job {
                if trimmed.starts_with("- run:") || trimmed.starts_with("- run :") {
                    let cmd = trimmed
                        .strip_prefix("-")
                        .unwrap_or(trimmed)
                        .trim()
                        .strip_prefix("run")
                        .unwrap_or(trimmed)
                        .trim()
                        .strip_prefix(":")
                        .unwrap_or(trimmed)
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                    if !cmd.is_empty() {
                        current_steps.push(cmd);
                    }
                } else if trimmed.starts_with("- name:") || trimmed.starts_with("- name :") {
                    let name = trimmed
                        .strip_prefix("-")
                        .unwrap_or(trimmed)
                        .trim()
                        .strip_prefix("name")
                        .unwrap_or(trimmed)
                        .trim()
                        .strip_prefix(":")
                        .unwrap_or(trimmed)
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                    if !name.is_empty() {
                        current_steps.push(name);
                    }
                }
            }
        }
    }

    // Save last job
    if let Some(job_name) = current_job.take() {
        gates.push(ParsedCIGate {
            job_name: job_name.clone(),
            trigger: job_triggers.get(&job_name).cloned(),
            steps: if current_steps.is_empty() {
                None
            } else {
                Some(current_steps.join("; "))
            },
        });
    }

    // If no trigger detected, use "push" as default for CircleCI
    for gate in &mut gates {
        if gate.trigger.is_none() {
            gate.trigger = Some("push".to_string());
        }
    }

    gates
}

// --- Makefile Parser ---

fn parse_makefile(content: &str) -> Vec<ParsedCIGate> {
    let mut gates = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments, empty lines, and variable assignments
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('.') {
            continue;
        }

        // Skip lines that are recipe lines (start with tab or are continuations)
        if line.starts_with('\t') || line.starts_with(' ') {
            continue;
        }

        // Detect target definitions: target: dependencies
        // Handle patterns like "target:", "target: dep1 dep2"
        if let Some(colon_pos) = trimmed.find(':') {
            let target_part = &trimmed[..colon_pos].trim();

            // Skip variable assignments (e.g., CC = gcc) and pattern rules with %
            if target_part.is_empty() || target_part.contains('=') || target_part.contains('%') {
                continue;
            }

            // Skip special targets that start with . (like .PHONY)
            if target_part.starts_with('.') {
                continue;
            }

            // Handle multiple targets (e.g., "all clean:")
            for target in target_part.split_whitespace() {
                // Skip targets with special characters (likely pattern rules or functions)
                if target.contains('%') || target.contains('(') || target.contains(')') {
                    continue;
                }

                // Extract commands for this target (subsequent tab-indented lines)
                let steps_str = extract_makefile_steps(content, target);

                gates.push(ParsedCIGate {
                    job_name: target.to_string(),
                    trigger: Some("manual".to_string()),
                    steps: if steps_str.is_empty() {
                        None
                    } else {
                        Some(steps_str)
                    },
                });
            }
        }
    }

    gates
}

fn extract_makefile_steps(content: &str, target: &str) -> String {
    let mut steps = Vec::new();
    let mut in_target = false;

    for line in content.lines() {
        // Detect target line
        if !in_target {
            if let Some(colon_pos) = line.find(':') {
                let target_part = line[..colon_pos].trim();
                if target_part.split_whitespace().any(|t| t == target) {
                    in_target = true;
                }
            }
        } else {
            // We're inside a target's recipe
            if line.starts_with('\t') || (line.starts_with("    ") && !line.trim().is_empty()) {
                let cmd = line.trim();
                if !cmd.is_empty() && !cmd.starts_with('#') {
                    // Truncate long commands
                    if cmd.len() > 200 {
                        steps.push(format!("{}...", &cmd[..197]));
                    } else {
                        steps.push(cmd.to_string());
                    }
                }
            } else if !line.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                // End of target recipe
                break;
            }
        }
    }

    // Truncate steps list to avoid excessive data
    if steps.len() > 20 {
        steps.truncate(20);
    }

    steps.join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_actions_basic() {
        let content = r#"
name: CI
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v3
      - name: Build
        run: cargo build
      - name: Test
        run: cargo test
  lint:
    runs-on: ubuntu-latest
    steps:
      - name: Lint
        run: cargo clippy
"#;
        let gates = parse_github_actions(content);
        assert!(
            gates.len() >= 2,
            "Should have at least 2 jobs, got {}",
            gates.len()
        );
        assert!(
            gates.iter().any(|g| g.job_name == "build"),
            "Should have build job"
        );
        assert!(
            gates.iter().any(|g| g.job_name == "lint"),
            "Should have lint job"
        );
        assert!(gates[0].trigger.is_some(), "Should have trigger info");
    }

    #[test]
    fn test_parse_gitlab_ci_basic() {
        let content = r#"
stages:
  - build
  - test

build_job:
  stage: build
  script:
    - cargo build

test_job:
  stage: test
  script:
    - cargo test
"#;
        let gates = parse_gitlab_ci(content);
        assert!(
            gates.len() >= 2,
            "Should have at least 2 jobs, got {}",
            gates.len()
        );
        assert!(
            gates.iter().any(|g| g.job_name == "build_job"),
            "Should have build_job"
        );
        assert!(
            gates.iter().any(|g| g.job_name == "test_job"),
            "Should have test_job"
        );
    }

    #[test]
    fn test_parse_circleci_basic() {
        let content = r#"
version: 2.1

jobs:
  build:
    docker:
      - image: rust:latest
    steps:
      - run: cargo build
      - run: cargo test

  deploy:
    docker:
      - image: rust:latest
    steps:
      - run: cargo deploy
"#;
        let gates = parse_circleci(content);
        assert!(
            gates.len() >= 2,
            "Should have at least 2 jobs, got {}",
            gates.len()
        );
        assert!(
            gates.iter().any(|g| g.job_name == "build"),
            "Should have build job"
        );
        assert!(
            gates.iter().any(|g| g.job_name == "deploy"),
            "Should have deploy job"
        );
    }

    #[test]
    fn test_parse_makefile_basic() {
        let content = r#"
.PHONY: all build test clean

all: build test

build:
	cargo build --release

test:
	cargo test

clean:
	cargo clean
"#;
        let gates = parse_makefile(content);
        // .PHONY is skipped, "all" is a target but has no steps of its own (it's a meta-target)
        // build, test, and clean should be detected
        assert!(
            gates.len() >= 2,
            "Should have at least 2 targets, got {}",
            gates.len()
        );
        assert!(
            gates.iter().any(|g| g.job_name == "build"),
            "Should have build target"
        );
        assert!(
            gates.iter().any(|g| g.job_name == "test"),
            "Should have test target"
        );
    }

    #[test]
    fn test_ci_gate_stats_serialization() {
        let stats = CIGateStats {
            total_gates: 10,
            github_actions_gates: 5,
            gitlab_ci_gates: 3,
            circleci_gates: 1,
            makefile_gates: 1,
            files_processed: 4,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("totalGates"));
        assert!(json.contains("githubActionsGates"));
        assert!(json.contains("gitlabCiGates"));
        assert!(json.contains("circleciGates"));
        assert!(json.contains("makefileGates"));
        assert!(json.contains("filesProcessed"));
    }
}
