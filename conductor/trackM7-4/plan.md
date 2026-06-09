## Plan: Track M7-4 — Deployment Manifest Awareness

### Phase 1: Manifest Classification
- [ ] Task 1.1: Implement `classify_deploy_manifest()` in `src/coverage/deploy.rs`.
- [ ] Task 1.2: Implement YAML parseability validation (skip non-parseable files).
- [ ] Task 1.3: Implement multi-manifest dedup (same type counted once).
- [ ] Task 1.4: Write test: Dockerfile classified as `ManifestType::Dockerfile`.
- [ ] Task 1.5: Write test: `docker-compose.yml` classified as `DockerCompose`.
- [ ] Task 1.6: Write test: `main.tf` classified as `Terraform`.
- [ ] Task 1.7: Write test: `k8s/deployment.yaml` classified as `Kubernetes`.
- [ ] Task 1.8: Write test: `Chart.yaml` classified as `Helm`.
- [ ] Task 1.9: Write test: non-manifest file returns `None`.
- [ ] Task 1.10: Write test: binary `.yaml` skipped.

### Phase 2: Dockerfile Scanning
- [ ] Task 2.1: Implement `scan_dockerfile_directives()`.
- [ ] Task 2.2: Detect `COPY`, `ADD`, `FROM` directives.
- [ ] Task 2.3: Cross-reference changed source files against COPY/ADD paths.
- [ ] Task 2.4: Write test: `COPY src/` detected when `src/` changed.
- [ ] Task 2.5: Write test: no COPY match when `src/` unchanged.

### Phase 3: Terraform Resource Scanning
- [ ] Task 3.1: Implement `scan_terraform_resources()`.
- [ ] Task 3.2: Flag high blast-radius resource types.
- [ ] Task 3.3: Write test: `aws_rds_cluster` resource flagged.
- [ ] Task 3.4: Write test: `aws_s3_bucket` not flagged (not in high-blast list).

### Phase 4: Types
- [ ] Task 4.1: Define `ManifestType` enum with `Ord` derive.
- [ ] Task 4.2: Define `DeployManifestChange` with `Ord` (by risk_tier descending).
- [ ] Task 4.3: Add `deploy_manifest_changes: Vec<DeployManifestChange>` to `ImpactPacket`.
- [ ] Task 4.4: Wire `finalize()` sort and `truncate_for_context()` clear.
- [ ] Task 4.5: Write test: serialization roundtrip.
- [ ] Task 4.6: Write test: finalize sorts by risk_tier descending.

### Phase 5: Risk Enrichment
- [ ] Task 5.1: Implement deployment risk enrichment in `execute_impact()`.
- [ ] Task 5.2: Implement tiered risk weighting per spec.
- [ ] Task 5.3: Implement docker-compose→Dockerfile coupling escalation.
- [ ] Task 5.4: Write test: Dockerfile change → risk weight 3.
- [ ] Task 5.5: Write test: k8s + terraform co-change → High elevation.
- [ ] Task 5.6: Write test: `[coverage.deploy].enabled = false` → no enrichment.
- [ ] Task 5.7: Write test: weight cap at 15 (6 manifests → weight 15).

### Phase 6: Final Validation
- [ ] Task 6.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Task 6.2: Run `cargo test coverage::deploy` — all tests pass.
- [ ] Task 6.3: Run full `cargo test` — no regressions.
