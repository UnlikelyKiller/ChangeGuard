# Specification: Track M7-1 — Trace Config & SDK Dependency Detection

## Objective
Detect observability pipeline configuration changes and third-party SDK import modifications during `scan`, enriching the impact packet with trace config drift and SDK dependency deltas.

## Components

### 1. Trace Config File Detection (`src/coverage/traces.rs`)

```rust
pub fn detect_trace_config_changes(changed_files: &[ChangedFile], patterns: &[String]) -> Vec<TraceConfigChange>
```

Match changed files against configurable glob patterns:
- `*otel-collector*.yaml`, `*otel-collector*.yml` → `TraceConfigType::OpenTelemetryCollector`
- `*jaeger-agent*.yaml`, `*jaeger-agent*.yml` → `TraceConfigType::JaegerAgent`
- `*datadog-agent*.yaml`, `*datadog.yaml` → `TraceConfigType::DataDogAgent`
- `*grafana-agent*.yaml` → `TraceConfigType::GrafanaAgent`
- `*tempo*.yaml` → `TraceConfigType::GrafanaTempo`

Risk reason: "Observability pipeline configuration modified: {file} ({config_type})"

### 2. Trace Env-Var Detection (`src/coverage/traces.rs`)

```rust
pub fn detect_trace_env_vars(env_deps: &[EnvVarDep], patterns: &[String]) -> Vec<TraceEnvVarChange>
```

Flag env vars matching patterns: `OTEL_*`, `JAEGER_*`, `DD_*`, `OTLP_*`, `HONEYCOMB_*`.

Risk reason: "Observability environment variable changed: {var_name} ({pattern})"

### 3. SDK Dependency Detection (`src/coverage/sdk.rs`)

```rust
pub fn detect_sdk_changes(
    changed_files: &[ChangedFile],
    patterns: &[String],
    project_root: &Path,
) -> SdkDependencyDelta
```

For each changed source file, scan import statements against the configured SDK patterns:
- `stripe`, `auth0`, `twilio`, `aws-sdk`, `@aws-sdk`, `google-cloud`, `sendgrid`, `firebase`, `supabase`, `openai`, `anthropic`

Extract the import path (language-aware: Rust `use`, Python `from`/`import`, JS/TS `import`/`require`, Go `import`).

- **New SDK added**: "New third-party SDK introduced: {sdk} in {file}"
- **SDK modified**: "Third-party SDK integration modified: {sdk} in {file}"
- **SDK removed**: Informational only (no risk elevation)

### 4. Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TraceConfigType { OpenTelemetryCollector, JaegerAgent, DataDogAgent, GrafanaAgent, GrafanaTempo }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceConfigChange {
    pub file: PathBuf,
    pub config_type: TraceConfigType,
    pub risk_weight: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEnvVarChange {
    pub var_name: String,
    pub pattern: String,
    pub risk_weight: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SdkDependencyDelta {
    pub added: Vec<SdkDependency>,
    pub removed: Vec<SdkDependency>,
    pub modified: Vec<SdkDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SdkDependency {
    pub sdk_name: String,
    pub file_path: PathBuf,
    pub import_statement: String,
}
```

## Test Specifications

| Test | Assertion |
|---|---|
| otel-collector yaml detected | `TraceConfigChange` returned with type `OpenTelemetryCollector` |
| non-trace yaml skipped | No `TraceConfigChange` for `app-config.yaml` |
| glob-safe invalid pattern | Invalid pattern logs WARN, does not abort |
| OTEL_* env var flagged | `TraceEnvVarChange` returned with `var_name` populated |
| OTEL_SDK_DISABLED excluded | Excluded by `exclude_env_patterns` |
| new Stripe import detected | `SdkDependencyDelta.added` contains Stripe entry |
| modified Stripe import detected | `SdkDependencyDelta.modified` contains Stripe entry |
| Python `from stripe import Charge` | Stripe detected |
| Go `import "github.com/stripe/stripe-go"` | Stripe detected |
| case-insensitive SDK matching | `STRIPE`, `Stripe`, `stripe` all match |
| trace file deleted since last scan | Recorded as removed `TraceConfigChange` |

## Constraints & Guidelines

- **TDD**: Tests written before implementation.
- **Config-driven**: When `[coverage.traces].enabled = false`, both trace functions return empty vecs.
- **Glob-safe**: Invalid glob patterns log `WARN` and skip; never abort `scan`.
- **Case-insensitive SDK matching**: Import text `.to_lowercase()` compared against pattern `.to_lowercase()`.
- **Language-aware**: Import extraction must handle Rust, Python, TypeScript, JavaScript, and Go import syntax.

## Hardening Additions (in plan)

| Addition | Reason |
|---|---|
| Case-insensitive SDK matching | Import casing varies across languages |
| Language-aware import extraction | Parsing imports correctly per language avoids false positives |
| Glob-safe config pattern validation | Malformed user config must never abort the tool |
| Double-extension trace files | `.yaml.tmpl`, `.yml.dist` should still match |
| Stale trace config detection | Deleted trace configs should be recorded as removed |
| `exclude_env_patterns` for env-var exclusion | Users need to suppress noisy false positives |
