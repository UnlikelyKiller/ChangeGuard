# Specification: Track M7-4 — Deployment Manifest Awareness

## Objective
Classify Dockerfile, docker-compose, Kubernetes, Terraform, and Helm changes with tiered risk weighting, surface deployment risk in the impact packet.

## Components

### 1. Manifest Classification (`src/coverage/deploy.rs`)

```rust
pub fn classify_deploy_manifest(file_path: &Path) -> Option<ManifestType>
```

Match files against deployment patterns:

| Pattern | ManifestType |
|---|---|
| `Dockerfile`, `Dockerfile.*` | `Dockerfile` |
| `docker-compose*.yml`, `docker-compose*.yaml` | `DockerCompose` |
| `*.tf`, `*.tfvars` | `Terraform` |
| `k8s/**/*.yaml`, `kubernetes/**/*.yaml` | `Kubernetes` |
| `helm/**/*.yaml`, `Chart.yaml` | `Helm` |
| `.github/workflows/*.yml` | `CiWorkflow` |

Validate `.yaml`/`.yml` files are parseable (not binary) before classifying.

### 2. Dockerfile Instruction Scanning

```rust
pub fn scan_dockerfile_directives(content: &str, changed_source_files: &[&str]) -> Vec<String>
```

Detect `COPY`, `ADD`, and `FROM` directives. If `COPY src/ ./src/` in Dockerfile and `src/` changed, escalate risk by one tier.

### 3. Terraform Resource Type Detection

```rust
pub fn scan_terraform_resources(content: &str) -> Vec<(String, bool)>
```

Flag resource types with high blast radius: `aws_rds_cluster`, `kubernetes_deployment`, `google_compute_instance`, `azurerm_kubernetes_cluster`.

### 4. Types

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ManifestType { Dockerfile, DockerCompose, Kubernetes, Terraform, Helm, CiWorkflow }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeployManifestChange {
    pub file: PathBuf,
    pub manifest_type: ManifestType,
    pub risk_tier: u8,
    pub coupled_files: Vec<String>,  // docker-compose service → file coupling
    pub high_blast_resources: Vec<String>,  // terraform resource types
}
```

### 5. Risk Impact

| Change | Elevation | Risk weight |
|---|---|---|
| Dockerfile changed | Low | 3 |
| Docker Compose / K8s / Helm changed | Medium | 5 |
| Terraform changed | Medium | 5 |
| Dockerfile COPY src/ and src/ changed | +1 tier | 5 |
| docker-compose build context + Dockerfile co-change | +1 tier | 5 |
| 2+ manifest types in same diff | High | 8 |
| High blast radius terraform resource changed | +1 tier | 6 |

Risk weight per manifest: 3, cumulated cap at 15.

### 6. Impact Enrichment

`deploy_manifest_changes: Vec<DeployManifestChange>` on `ImpactPacket`. Sorted by `risk_tier` descending, `file` ascending in `finalize()`. Cleared in `truncate_for_context()`.

## Test Specifications

| Test | Assertion |
|---|---|
| Dockerfile classified | `ManifestType::Dockerfile` returned |
| `docker-compose.yml` classified | `ManifestType::DockerCompose` returned |
| `main.tf` classified | `ManifestType::Terraform` returned |
| `k8s/deployment.yaml` classified | `ManifestType::Kubernetes` returned |
| `Chart.yaml` classified | `ManifestType::Helm` returned |
| Non-manifest file skipped | `classify_deploy_manifest` returns `None` |
| Binary `.yaml` skipped | File with non-UTF8 content not classified |
| Dockerfile COPY detection | `COPY src/` detected when `src/` changed |
| Terraform RDS cluster flagged | `aws_rds_cluster` resource returned in `high_blast_resources` |
| Multi-manifest dedup | `Dockerfile` + `Dockerfile.prod` count as 1 Dockerfile type |
| docker-compose→Dockerfile coupling | `build: ./api` in compose + `./api/Dockerfile` changed → coupling |

## Constraints & Guidelines

- **TDD**: Tests written before implementation.
- **No hot-path embedding**: Pure file classification; no embedding calls.
- **Config-driven**: `[coverage.deploy].enabled = false` → no manifest detection.
- **YAML validation**: Files must parse as valid YAML to be classified as k8s/helm manifests.
- **Determinism**: `DeployManifestChange` implements `Ord` by risk_tier descending.

## Hardening Additions (in plan)

| Addition | Reason |
|---|---|
| Dockerfile COPY/ADD scanning | Context-sensitive risk when build context changes |
| docker-compose service→file coupling | Compose service build context changes compound risk |
| Terraform resource-type risk | Some resource types have higher blast radius |
| Helm values coupling | `Chart.yaml` + `values.yaml` co-change is a specific pattern |
| Binary-in-yaml skip | Prevent false classification of non-UTF8 files |
| Multi-manifest dedup | Prevent duplicate risk reasons for same manifest type |
