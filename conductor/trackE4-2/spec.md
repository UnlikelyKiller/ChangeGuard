# Specification: Track E4-2 CI/CD Workflow Awareness

## Overview

Implement the second track of Phase E4 (Safety Context) from `docs/expansion-plan.md`. This track parses CI/CD configuration files (GitHub Actions, GitLab CI, CircleCI, Makefile), adds a `ci_gates` table (Migration M18), and integrates with the `impact` command (+30 risk weight for CI changes) and `verify` command (suggest CI-matching verification commands).

## Motivation

ChangeGuard already includes `.github/workflows/` in its default protected paths, but it has no understanding of what those workflows do. A change to a CI configuration file can alter what gets tested, built, or deployed, making it one of the highest-risk change categories. This track makes CI/CD workflows a first-class tracked artifact, elevates their risk weight, and suggests verification commands that match CI pipeline steps.

## Components

### 1. CI Config Parsing (`src/index/ci_gates.rs`)

New module that parses CI/CD configuration files and extracts pipeline structure.

**GitHub Actions parsing** (`.github/workflows/*.yml` and `.github/workflows/*.yaml`):
- Parse YAML to extract: workflow name, `on` triggers (push, pull_request, schedule, workflow_dispatch), jobs (name, runs-on, steps), and step commands.
- Store each job as a `ci_gates` row with `platform = 'github_actions'`.
- Extract step commands for the `steps` JSON field (command name, run command if present).

**GitLab CI parsing** (`.gitlab-ci.yml`):
- Parse YAML to extract: stages, jobs (name, script, stage, only/except rules), and job dependencies.
- Store each job as a `ci_gates` row with `platform = 'gitlab_ci'`.

**CircleCI parsing** (`.circleci/config.yml`):
- Parse YAML to extract: jobs (name, docker executor, steps), workflows (name, jobs, filters), and step commands.
- Store each job as a `ci_gates` row with `platform = 'circleci'`.

**Makefile parsing** (`Makefile`, `makefile`, `GNUmakefile`):
- Parse line-by-line to extract: targets and their dependencies.
- Store each target as a `ci_gates` row with `platform = 'makefile'`.
- Extract target commands for the `steps` JSON field.

### 2. Database Schema (`src/state/migrations.rs`)

Add migration M18 to create the `ci_gates` table (alongside the `test_mapping` and `env_schema` tables from Tracks E4-1 and E4-3):

```sql
CREATE TABLE IF NOT EXISTS ci_gates (
    id INTEGER PRIMARY KEY,
    ci_file_id INTEGER NOT NULL REFERENCES project_files(id),
    platform TEXT NOT NULL,
    job_name TEXT NOT NULL,
    trigger TEXT,
    steps JSON,
    last_indexed_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ci_gates_file ON ci_gates(ci_file_id);
CREATE INDEX IF NOT EXISTS idx_ci_gates_platform ON ci_gates(platform);
```

The `steps` column stores a JSON array of step objects, each with `name` and optionally `command` fields. This allows downstream features to understand what each CI job does without re-parsing the config file.

### 3. Index Integration (`src/commands/index.rs`)

Wire CI config parsing into the `changeguard index` command:

- Detect CI config files by path pattern: `.github/workflows/*.yml`, `.gitlab-ci.yml`, `.circleci/config.yml`, `Makefile`, `makefile`, `GNUmakefile`.
- Parse each detected file using the appropriate parser.
- Insert results into `ci_gates`.
- On re-index, delete existing `ci_gates` rows for a file before inserting new ones.

### 4. Impact Integration (`src/impact/analysis.rs`)

Add CI-specific risk behavior:

- When a changed file matches a `ci_file_id` in the `ci_gates` table, add risk weight in the **Verification Gap** category (max 30 points per expansion plan Section 4.2).
- Add "CI/CD pipeline change: X" to `risk_reasons` where X is the job name(s) affected.
- This is in addition to any protected-path elevation that may already apply.

### 5. Verify Integration (`src/verify/predict.rs`)

Add CI-matching verification command suggestions:

- When `verify` produces a plan, check if the changed files include CI config files or if `ci_gates` entries exist for the repo.
- For each `ci_gates` entry whose `steps` contain a `test`, `check`, `lint`, or `build` command, suggest running that command.
- Example: if `Makefile` has a `test` target, suggest `make test`. If `.github/workflows/ci.yml` has a `test` job with `cargo test`, suggest `cargo test`.
- Suggestions are appended to the verification plan with reason: "CI gate suggests: X".
- **Confidence tiers for verification suggestions:**
  - `HIGH`: command exists in PATH and config declares it (e.g., `make test` where `make` is found and the Makefile has a `test` target)
  - `MEDIUM`: config declares it but executable not found in PATH (e.g., Makefile has `test` target but `make` is not on PATH)
  - `LOW`: inferred from ecosystem (e.g., Rust project suggests `cargo test` without a CI config explicitly declaring it)

## Constraints & Guidelines

- **Graceful degradation**: If a CI config file is malformed or uses unsupported features, extract what is possible and mark the remainder as `PARSE_FAILED`. Never crash on invalid YAML or Makefiles.
- **Security**: Never extract or store secret values from CI config files. If a step references `${{ secrets.* }}`, store only the reference (`secrets.ENV_VAR_NAME`), not the value.
- **No false confidence**: If a CI config cannot be fully parsed, store what was extracted and mark the job as partially parsed. Do not guess at missing information.
- **TDD Requirement**: Write or update tests for each CI parser, the database schema, risk weight application, and verification command suggestions.
- **No performance regression**: CI config parsing must complete in under 1 second for repos with up to 20 workflow files.
- **Backward-compatible schema**: The `ci_gates` table is new and additive. No existing tables are modified.

## Edge Cases

- **No CI config in the repo**: Skip CI parsing entirely. No warnings, no errors. The `ci_gates` table remains empty.
- **Malformed YAML**: Extract what is possible. Store partially parsed jobs. Mark unparseable sections as `PARSE_FAILED` in the `steps` JSON.
- **CI config referencing external secrets**: Store the secret reference (`secrets.DEPLOY_KEY`) but never the secret value. Add a note in `steps` that the job uses secrets.
- **Multiple CI platforms in one repo**: Index all of them. Each gets its own `ci_gates` rows with the appropriate `platform` value.
- **Makefile with implicit rules**: Only extract explicit targets that have recipe lines. Do not attempt to resolve implicit rules.
- **Dynamic CI config** (e.g., GitHub Actions with `workflow_call` or reusable workflows): Store the top-level job name and mark `trigger` as the workflow call reference. Do not recursively resolve reusable workflows in this phase.

## Acceptance Criteria

- `changeguard index` populates `ci_gates` when CI config files exist in the repo.
- `changeguard impact` flags CI config changes with +30 risk weight and "CI/CD pipeline change" risk reason.
- `changeguard verify` suggests CI-matching verification commands (e.g., `make test`, `cargo test`) based on CI config.
- Malformed CI config files produce warnings, not crashes.
- Repos without CI config continue to function normally with empty `ci_gates`.

## Definition of Done

- [ ] All acceptance criteria pass
- [ ] All unit tests pass
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test` passes with no regressions
- [ ] No deviations from this spec without documented justification
- [ ] Migration M18 applied cleanly to existing ledger.db
- [ ] `changeguard index` populates E4 tables for fixture repos