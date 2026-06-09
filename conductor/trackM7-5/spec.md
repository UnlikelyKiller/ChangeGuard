# Specification: Track M7-5 — CI Pipeline Self-Awareness

## Objective
Surface risk when CI configuration itself changes in a diff — detecting CI-only changes, CI+source co-changes, and pre-commit hook modifications.

## Components

### 1. CI Config Detection (`src/index/ci_gates.rs` extend)

```rust
pub fn is_ci_config_changed(changed_files: &[ChangedFile]) -> Option<CiConfigChange>
```

Check if any changed file matches known CI config patterns:
- `.github/workflows/*.yml`, `.github/workflows/*.yaml`
- `.gitlab-ci.yml`
- `Jenkinsfile`, `Jenkinsfile.*`
- `.circleci/config.yml`
- `.travis.yml`
- `azure-pipelines.yml`
- `Makefile` (if in root and contains CI-like targets: `test`, `build`, `deploy`)

Also detect non-standard CI files:
- `.github/**` directory changes without matching a known CI config
- `.ci/**`, `ci/**` directory changes

### 2. Pre-Commit Hook Detection

```rust
pub fn detect_pre_commit_changes(changed_files: &[ChangedFile]) -> Vec<String>
```

Detect changes to:
- `.pre-commit-config.yaml`
- `lefthook.yml`
- `.husky/**`

Reason: "Pre-commit hooks modified — local checks may change"

### 3. Risk Impact

| Scenario | Elevation | Risk weight |
|---|---|---|
| CI config changed alone | Low | 3 |
| CI config + source changed | Medium | 5 |
| CI + deploy manifests changed | +1 tier | 5 |
| Pre-commit hooks changed | Low | 2 |
| Unknown CI-like file changed | Low | 1 |

### 4. Generated CI File Detection

```rust
pub fn is_generated_ci_file(content: &str) -> bool
```

Check for:
- `# auto-generated` or `# generated` headers
- `@generated` annotations
- `.github/workflows/generated-*.yml` patterns

Generated files are informational only (no risk elevation).

### 5. Impact Enrichment

No new `ImpactPacket` fields. Append risk reasons directly with appropriate weight. Weight contributes to `analyze_risk` total through the CI gate weight pathway.

## Test Specifications

| Test | Assertion |
|---|---|
| `.github/workflows/ci.yml` changed | CI risk reason added |
| CI config + `src/main.rs` changed | Medium elevation |
| CI + `Dockerfile` changed | Escalated by one tier |
| `.pre-commit-config.yaml` changed | Pre-commit hook reason added |
| `generated-ci.yml` with `# auto-generated` | Informational only, no risk elevation |
| Non-standard CI path `ci/deploy.sh` | Informational reason added |
| `Makefile` with CI targets | CI risk reason when targets include `test` |

## Constraints & Guidelines

- **TDD**: Tests written before implementation.
- **No new packet fields**: Uses existing `risk_reasons` only.
- **Config-driven**: `[coverage.ci_self_awareness].enabled = false` → no detection.
- **Determinism**: Risk reasons added in alphabetical order of file paths.

## Hardening Additions (in plan)

| Addition | Reason |
|---|---|
| Non-standard CI detection | Not all repos use GitHub Actions |
| CI + deploy compounding | CI config AND deploy manifest changes compound risk |
| Generated CI file skip | Auto-generated workflows should not flag |
| Pre-commit hook awareness | Local checks changing is a distinct risk category |
