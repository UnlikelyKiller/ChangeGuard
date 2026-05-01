## Plan: Track E4-2 CI/CD Workflow Awareness

### Phase 1: Database Schema
- [ ] Task 1.1: Add `ci_gates` table creation to migration M18 in `src/state/migrations.rs` with columns `id`, `ci_file`, `platform`, `job_name`, `trigger`, `steps`, `last_indexed_at` and indices on `ci_file` and `platform`.
- [ ] Task 1.2: Write a test verifying the `ci_gates` table is created and supports insert/query operations, including querying by `ci_file` and `platform`.
- [ ] Task 1.3: Write a test verifying `steps` JSON column stores and retrieves step arrays correctly.

### Phase 2: CI Config Parsers
- [ ] Task 2.1: Create `src/index/ci_gates.rs` module with a `CiGate` struct holding `ci_file`, `platform`, `job_name`, `trigger`, `steps`, `last_indexed_at`.
- [ ] Task 2.2: Implement GitHub Actions parser: parse `.github/workflows/*.yml` YAML files to extract workflow name, `on` triggers, jobs (name, runs-on, steps), and step commands.
- [ ] Task 2.3: Implement GitLab CI parser: parse `.gitlab-ci.yml` YAML files to extract stages, jobs (name, script, stage), and job dependencies.
- [ ] Task 2.4: Implement CircleCI parser: parse `.circleci/config.yml` YAML files to extract jobs (name, steps) and workflows (name, jobs, filters).
- [ ] Task 2.5: Implement Makefile parser: parse `Makefile`, `makefile`, `GNUmakefile` files line-by-line to extract targets and their recipe commands.
- [ ] Task 2.6: Write unit tests for each CI parser using fixture config files with known structure.
- [ ] Task 2.7: Write unit tests for malformed YAML handling: verify that partial parsing succeeds and unparseable sections are marked as `PARSE_FAILED`.

### Phase 3: Security and Edge Case Handling
- [ ] Task 3.1: Implement secret value redaction: when parsing CI config, detect `${{ secrets.* }}` and similar secret references, store only the reference name, never the value.
- [ ] Task 3.2: Write test: a GitHub Actions workflow with `${{ secrets.DEPLOY_KEY }}` stores `secrets.DEPLOY_KEY` in the steps JSON, not the actual value.
- [ ] Task 3.3: Implement multi-platform support: detect all CI config files in a repo and parse each with the appropriate parser.
- [ ] Task 3.4: Write test: a repo with both `.github/workflows/ci.yml` and `Makefile` produces `ci_gates` rows for both platforms.

### Phase 4: Index Integration
- [ ] Task 4.1: Add `extract_ci_gates` function to `src/index/ci_gates.rs` that detects CI config files by path pattern and dispatches to the appropriate parser.
- [ ] Task 4.2: Wire CI config parsing into `src/commands/index.rs`: detect and parse CI config files, insert results into `ci_gates`.
- [ ] Task 4.3: Implement upsert logic: on re-index, delete existing `ci_gates` rows for a file before inserting new ones.
- [ ] Task 4.4: Write integration test: run `changeguard index` on a fixture repo with GitHub Actions and Makefile, and verify `ci_gates` rows are populated correctly.

### Phase 5: Impact Integration
- [ ] Task 5.1: Modify `analyze_risk()` in `src/impact/analysis.rs` to detect when a changed file matches a `ci_file` in the `ci_gates` table.
- [ ] Task 5.2: Apply +30 risk weight to CI config file changes. Add "CI/CD pipeline change: X" to `risk_reasons` where X is the job name(s) affected.
- [ ] Task 5.3: Write test: changing `.github/workflows/ci.yml` produces +30 risk weight and a CI-related risk reason.
- [ ] Task 5.4: Write test: changing `Makefile` produces +30 risk weight and a CI-related risk reason.
- [ ] Task 5.5: Write test: changing a non-CI file does NOT produce the CI risk weight.

### Phase 6: Verify Integration
- [ ] Task 6.1: Modify `src/verify/predict.rs` to check `ci_gates` entries for the repo when producing verification plans.
- [ ] Task 6.2: For each `ci_gates` entry whose `steps` contain a `test`, `check`, `lint`, or `build` command, suggest running that command as a verification step.
- [ ] Task 6.3: Append CI-suggested commands to the verification plan with reason "CI gate suggests: X".
- [ ] Task 6.4: Write test: `verify` on a repo with a `Makefile` containing a `test` target suggests `make test`.
- [ ] Task 6.5: Write test: `verify` on a repo with `.github/workflows/ci.yml` containing a `test` job suggests running the `cargo test` command from that job's steps.

### Phase 7: Final Validation
- [ ] Task 7.1: Run full test suite (`cargo test`) and verify no regressions in existing `impact`, `hotspots`, `verify`, or `ledger` tests.
- [ ] Task 7.2: Run `changeguard index` on a fixture repo with CI config files and verify `ci_gates` rows are created.
- [ ] Task 7.3: Run `changeguard impact` on a fixture repo with CI config changes and verify +30 risk weight and CI-related risk reason.
- [ ] Task 7.4: Run `changeguard verify` on a fixture repo and verify CI-matching verification commands appear in the plan.