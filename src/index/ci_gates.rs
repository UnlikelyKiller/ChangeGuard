use crate::impact::packet::{ChangedFile, CiConfigChange};
use crate::state::storage::StorageManager;
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
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
    workflow_name: Option<String>,
    environment: Option<String>,
    artifacts: Option<String>,
    release_gates: Option<String>,
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
                    workflow_name: gate.workflow_name.clone(),
                    environment: gate.environment.clone(),
                    artifacts: gate
                        .artifacts
                        .as_ref()
                        .map(|a| serde_json::to_string(a).unwrap_or_default()),
                    release_gates: gate
                        .release_gates
                        .as_ref()
                        .map(|a| serde_json::to_string(a).unwrap_or_default()),
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
                "INSERT INTO ci_gates (ci_file_id, platform, job_name, trigger, steps, workflow_name, environment, artifacts, release_gates, last_indexed_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    row.ci_file_id,
                    row.platform,
                    row.job_name,
                    row.trigger,
                    row.steps,
                    row.workflow_name,
                    row.environment,
                    row.artifacts,
                    row.release_gates,
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
    workflow_name: Option<String>,
    environment: Option<String>,
    artifacts: Option<Vec<String>>,
    release_gates: Option<Vec<String>>,
}

// --- GitHub Actions Parser ---

fn parse_github_actions(content: &str) -> Vec<ParsedCIGate> {
    let mut gates = Vec::new();
    let mut triggers = Vec::new();
    let mut workflow_name = None;

    // Extract triggers from the "on:" section
    let mut in_on_section = false;
    let mut on_indent = 0;
    let mut brace_depth = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('#') {
            continue;
        }

        // Detect workflow name
        if trimmed.starts_with("name:") {
            workflow_name = Some(
                trimmed
                    .strip_prefix("name:")
                    .unwrap()
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string(),
            );
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
                in_on_section = false;
            } else {
                if trimmed.contains('{') {
                    brace_depth += trimmed.matches('{').count();
                }
                if trimmed.contains('}') {
                    brace_depth = brace_depth.saturating_sub(trimmed.matches('}').count());
                }

                if brace_depth == 0 {
                    if let Some(key) = trimmed.strip_suffix(':')
                        && !key.starts_with('#')
                        && !key.contains(' ')
                        && key != "branches"
                        && key != "paths"
                        && key != "tags"
                    {
                        triggers.push(key.to_string());
                    }
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

    // Extract jobs
    let mut in_jobs_section = false;
    let mut current_job: Option<String> = None;
    let mut current_job_steps: Vec<String> = Vec::new();
    let mut current_job_env: Option<String> = None;
    let mut current_job_artifacts: Vec<String> = Vec::new();
    let mut current_job_releases: Vec<String> = Vec::new();
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
            if !trimmed.is_empty() && line_indent <= jobs_indent {
                in_jobs_section = false;
                if let Some(job_name) = current_job.take() {
                    gates.push(ParsedCIGate {
                        job_name,
                        trigger: trigger_str.clone(),
                        steps: if current_job_steps.is_empty() {
                            None
                        } else {
                            Some(current_job_steps.join("; "))
                        },
                        workflow_name: workflow_name.clone(),
                        environment: current_job_env.take(),
                        artifacts: if current_job_artifacts.is_empty() {
                            None
                        } else {
                            Some(current_job_artifacts.clone())
                        },
                        release_gates: if current_job_releases.is_empty() {
                            None
                        } else {
                            Some(current_job_releases.clone())
                        },
                    });
                    current_job_steps.clear();
                    current_job_artifacts.clear();
                    current_job_releases.clear();
                }
                continue;
            }

            if line_indent == jobs_indent + 2 && trimmed.ends_with(':') && !trimmed.starts_with('-')
            {
                if let Some(job_name) = current_job.take() {
                    gates.push(ParsedCIGate {
                        job_name,
                        trigger: trigger_str.clone(),
                        steps: if current_job_steps.is_empty() {
                            None
                        } else {
                            Some(current_job_steps.join("; "))
                        },
                        workflow_name: workflow_name.clone(),
                        environment: current_job_env.take(),
                        artifacts: if current_job_artifacts.is_empty() {
                            None
                        } else {
                            Some(current_job_artifacts.clone())
                        },
                        release_gates: if current_job_releases.is_empty() {
                            None
                        } else {
                            Some(current_job_releases.clone())
                        },
                    });
                    current_job_steps.clear();
                    current_job_artifacts.clear();
                    current_job_releases.clear();
                }
                current_job = Some(trimmed.strip_suffix(':').unwrap_or(trimmed).to_string());
                in_steps = false;
                continue;
            }

            if current_job.is_some() {
                if trimmed.starts_with("environment:") {
                    let env = trimmed.strip_prefix("environment:").unwrap().trim();
                    if !env.is_empty() {
                        current_job_env =
                            Some(env.trim_matches('"').trim_matches('\'').to_string());
                    }
                }

                if trimmed == "steps:" || trimmed.starts_with("steps:") {
                    in_steps = true;
                    continue;
                }

                if in_steps && line_indent > 0 {
                    if trimmed.starts_with("- name:") {
                        let name = trimmed
                            .strip_prefix("- name:")
                            .unwrap()
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string();
                        if !name.is_empty() {
                            current_job_steps.push(name);
                        }
                    } else if trimmed.starts_with("- run:") {
                        let cmd = trimmed
                            .strip_prefix("- run:")
                            .unwrap()
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string();
                        if !cmd.is_empty() {
                            current_job_steps.push(cmd);
                        }
                    } else if trimmed.contains("uses: actions/upload-artifact") {
                        current_job_artifacts.push("upload-artifact".to_string());
                    } else if trimmed.contains("uses: softprops/action-gh-release") {
                        current_job_releases.push("gh-release".to_string());
                    }
                }
            }
        }
    }

    if let Some(job_name) = current_job.take() {
        gates.push(ParsedCIGate {
            job_name,
            trigger: trigger_str.clone(),
            steps: if current_job_steps.is_empty() {
                None
            } else {
                Some(current_job_steps.join("; "))
            },
            workflow_name: workflow_name.clone(),
            environment: current_job_env,
            artifacts: if current_job_artifacts.is_empty() {
                None
            } else {
                Some(current_job_artifacts)
            },
            release_gates: if current_job_releases.is_empty() {
                None
            } else {
                Some(current_job_releases)
            },
        });
    }

    gates
}

// --- GitLab CI Parser ---

fn parse_gitlab_ci(content: &str) -> Vec<ParsedCIGate> {
    let mut gates = Vec::new();
    let mut stages: Vec<String> = Vec::new();

    // Parse stages
    let mut in_stages = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("stages:") {
            in_stages = true;
            if let Some(rest) = trimmed.strip_prefix("stages:") {
                let rest = rest.trim();
                if rest.starts_with('[') {
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
                let item = trimmed
                    .strip_prefix('-')
                    .unwrap()
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if !item.is_empty() && !item.contains(':') {
                    stages.push(item.to_string());
                }
            } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                in_stages = false;
            }
        }
    }

    let mut current_job: Option<String> = None;
    let mut current_steps: Vec<String> = Vec::new();
    let mut current_trigger: Option<String> = None;
    let top_level_keys: std::collections::HashSet<&str> = [
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
    ]
    .iter()
    .copied()
    .collect();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        let line_indent = line.len() - line.trim_start().len();

        if line_indent == 0 && trimmed.contains(':') && !trimmed.starts_with('-') {
            if let Some(job_name) = current_job.take() {
                if !top_level_keys.contains(job_name.as_str()) {
                    gates.push(ParsedCIGate {
                        job_name,
                        trigger: current_trigger.take(),
                        steps: if current_steps.is_empty() {
                            None
                        } else {
                            Some(current_steps.join("; "))
                        },
                        workflow_name: None,
                        environment: None,
                        artifacts: None,
                        release_gates: None,
                    });
                }
                current_steps.clear();
            }
            let key = trimmed.split(':').next().unwrap().trim();
            current_job = Some(key.to_string());
            if top_level_keys.contains(key) {
                current_job = None;
            }
        }

        if let Some(ref _job) = current_job {
            if trimmed.starts_with("- ") {
                let item = trimmed
                    .strip_prefix("- ")
                    .unwrap()
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if !item.is_empty() && item.len() < 200 {
                    current_steps.push(item.to_string());
                }
            }
            if (trimmed.starts_with("only:") || trimmed.starts_with("except:"))
                && let Some(trigger_type) = trimmed.strip_suffix(':')
            {
                current_trigger = Some(trigger_type.to_string());
            }
        }
    }

    if let Some(job_name) = current_job.take()
        && !top_level_keys.contains(job_name.as_str())
    {
        gates.push(ParsedCIGate {
            job_name,
            trigger: current_trigger,
            steps: if current_steps.is_empty() {
                None
            } else {
                Some(current_steps.join("; "))
            },
            workflow_name: None,
            environment: None,
            artifacts: None,
            release_gates: None,
        });
    }

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
    let mut in_jobs = false;
    let mut current_job: Option<String> = None;
    let mut current_steps: Vec<String> = Vec::new();
    let mut jobs_indent = 0;

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
                        job_name,
                        trigger: Some("push".to_string()),
                        steps: if current_steps.is_empty() {
                            None
                        } else {
                            Some(current_steps.join("; "))
                        },
                        workflow_name: None,
                        environment: None,
                        artifacts: None,
                        release_gates: None,
                    });
                    current_steps.clear();
                }
                continue;
            }

            if line_indent == jobs_indent + 2 && trimmed.ends_with(':') && !trimmed.starts_with('-')
            {
                if let Some(job_name) = current_job.take() {
                    gates.push(ParsedCIGate {
                        job_name,
                        trigger: Some("push".to_string()),
                        steps: if current_steps.is_empty() {
                            None
                        } else {
                            Some(current_steps.join("; "))
                        },
                        workflow_name: None,
                        environment: None,
                        artifacts: None,
                        release_gates: None,
                    });
                    current_steps.clear();
                }
                current_job = Some(trimmed.strip_suffix(':').unwrap().to_string());
                continue;
            }

            if current_job.is_some()
                && (trimmed.starts_with("- run:") || trimmed.starts_with("- name:"))
            {
                let cmd = trimmed
                    .split_once(':')
                    .map(|x| x.1)
                    .unwrap_or("")
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if !cmd.is_empty() {
                    current_steps.push(cmd.to_string());
                }
            }
        }
    }

    if let Some(job_name) = current_job.take() {
        gates.push(ParsedCIGate {
            job_name,
            trigger: Some("push".to_string()),
            steps: if current_steps.is_empty() {
                None
            } else {
                Some(current_steps.join("; "))
            },
            workflow_name: None,
            environment: None,
            artifacts: None,
            release_gates: None,
        });
    }
    gates
}

// --- Makefile Parser ---

fn parse_makefile(content: &str) -> Vec<ParsedCIGate> {
    let mut gates = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('.') {
            continue;
        }
        if line.starts_with('\t') || line.starts_with(' ') {
            continue;
        }

        if let Some(colon_pos) = trimmed.find(':') {
            let target_part = trimmed[..colon_pos].trim();
            if target_part.is_empty()
                || target_part.contains('=')
                || target_part.contains('%')
                || target_part.starts_with('.')
            {
                continue;
            }

            for target in target_part.split_whitespace() {
                if target.contains('%') || target.contains('(') || target.contains(')') {
                    continue;
                }
                let steps_str = extract_makefile_steps(content, target);
                gates.push(ParsedCIGate {
                    job_name: target.to_string(),
                    trigger: Some("manual".to_string()),
                    steps: if steps_str.is_empty() {
                        None
                    } else {
                        Some(steps_str)
                    },
                    workflow_name: None,
                    environment: None,
                    artifacts: None,
                    release_gates: None,
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
        if !in_target {
            if let Some(colon_pos) = line.find(':')
                && line[..colon_pos].split_whitespace().any(|t| t == target)
            {
                in_target = true;
            }
        } else {
            if line.starts_with('\t') || (line.starts_with("    ") && !line.trim().is_empty()) {
                let cmd = line.trim();
                if !cmd.is_empty() && !cmd.starts_with('#') {
                    if cmd.len() > 200 {
                        steps.push(format!("{}...", &cmd[..197]));
                    } else {
                        steps.push(cmd.to_string());
                    }
                }
            } else if !line.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                break;
            }
        }
    }
    if steps.len() > 20 {
        steps.truncate(20);
    }
    steps.join("; ")
}

// --- CI Self-Awareness Detection ---

pub fn is_ci_config_changed(changed_files: &[ChangedFile]) -> Option<CiConfigChange> {
    let mut result = CiConfigChange::default();
    for file in changed_files {
        let path_str = file.path.to_string_lossy().replace('\\', "/");
        if is_pre_commit_path(&path_str) {
            result.pre_commit_files.push(path_str.clone());
            continue;
        }
        if is_generated_ci_path(&path_str) {
            result.generated_ci_files.push(path_str.clone());
            continue;
        }
        if is_known_ci_path(&path_str) {
            if is_root_makefile(&path_str) {
                let has_ci_targets = file.ci_gates.iter().any(|g| {
                    g.platform == "makefile"
                        && ["test", "build", "deploy", "lint", "ci"].contains(&g.job_name.as_str())
                });
                if has_ci_targets {
                    result.known_ci_files.push(path_str.clone());
                } else if file.ci_gates.is_empty()
                    && let Ok(content) = std::fs::read_to_string(&file.path)
                    && makefile_has_ci_targets(&content)
                {
                    result.known_ci_files.push(path_str.clone());
                }
            } else {
                result.known_ci_files.push(path_str.clone());
            }
            continue;
        }
        if is_unknown_ci_path(&path_str) {
            result.unknown_ci_files.push(path_str.clone());
        }
    }
    result.source_changed = changed_files
        .iter()
        .any(|c| c.symbols.is_some() || c.imports.is_some());
    result.known_ci_files.sort();
    result.unknown_ci_files.sort();
    result.pre_commit_files.sort();
    result.generated_ci_files.sort();
    if !result.known_ci_files.is_empty()
        || !result.unknown_ci_files.is_empty()
        || !result.pre_commit_files.is_empty()
        || !result.generated_ci_files.is_empty()
    {
        Some(result)
    } else {
        None
    }
}

pub fn detect_pre_commit_changes(changed_files: &[ChangedFile]) -> Vec<String> {
    let mut result = Vec::new();
    for file in changed_files {
        let path_str = file.path.to_string_lossy().replace('\\', "/");
        if is_pre_commit_path(&path_str) {
            result.push(path_str);
        }
    }
    result.sort();
    result
}

pub fn is_generated_ci_file(content: &str) -> bool {
    for line in content.lines().take(10) {
        let trimmed = line.trim();
        if trimmed.starts_with("# auto-generated")
            || trimmed.starts_with("# generated")
            || trimmed.contains("@generated")
        {
            return true;
        }
    }
    false
}

pub fn makefile_has_ci_targets(content: &str) -> bool {
    let ci_targets: &[&str] = &["test", "build", "deploy", "lint", "ci"];
    let gates = parse_makefile(content);
    gates
        .iter()
        .any(|g| ci_targets.contains(&g.job_name.as_str()))
}

// --- Path matching helpers ---

fn is_known_ci_path(path: &str) -> bool {
    if path.starts_with(".github/workflows/") && (path.ends_with(".yml") || path.ends_with(".yaml"))
    {
        return true;
    }
    if path == ".gitlab-ci.yml"
        || path.starts_with("Jenkinsfile")
        || path == ".circleci/config.yml"
        || path == ".travis.yml"
        || path == "azure-pipelines.yml"
        || is_root_makefile(path)
    {
        return true;
    }
    false
}

fn is_root_makefile(path: &str) -> bool {
    path == "Makefile" || path == "makefile" || path == "GNUmakefile"
}
fn is_unknown_ci_path(path: &str) -> bool {
    if path.starts_with(".github/") && !path.starts_with(".github/workflows/") {
        return true;
    }
    path.starts_with(".ci/") || path.starts_with("ci/")
}
fn is_pre_commit_path(path: &str) -> bool {
    path == ".pre-commit-config.yaml" || path == "lefthook.yml" || path.starts_with(".husky/")
}
fn is_generated_ci_path(path: &str) -> bool {
    path.starts_with(".github/workflows/generated-")
        && (path.ends_with(".yml") || path.ends_with(".yaml"))
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
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Build
        run: cargo build
"#;
        let gates = parse_github_actions(content);
        assert!(!gates.is_empty());
        assert_eq!(gates[0].job_name, "build");
        assert_eq!(gates[0].workflow_name, Some("CI".to_string()));
    }

    #[test]
    fn test_parse_gitlab_ci_basic() {
        let content = r#"
stages:
  - build
build_job:
  stage: build
  script:
    - cargo build
"#;
        let gates = parse_gitlab_ci(content);
        assert!(!gates.is_empty());
        assert_eq!(gates[0].job_name, "build_job");
    }

    #[test]
    fn test_parse_circleci_basic() {
        let content = r#"
version: 2.1
jobs:
  build:
    steps:
      - run: cargo build
"#;
        let gates = parse_circleci(content);
        assert!(!gates.is_empty());
        assert_eq!(gates[0].job_name, "build");
    }

    #[test]
    fn test_parse_makefile_basic() {
        let content = "test:\n\tcargo test\n";
        let gates = parse_makefile(content);
        assert!(!gates.is_empty());
        assert_eq!(gates[0].job_name, "test");
    }
}
