---
name: changeguard
description: Use ChangeGuard for local-first change intelligence and transactional provenance. Trigger this skill whenever a repository contains `.changeguard/`, or the user mentions "risk," "impact analysis," "blast radius," "hotspots," "temporal coupling," "drift," "verification planning," "architectural transactions," or wants safer changes with evidence from `scan`, `impact`, `verify`, or `ledger`.
---

# ChangeGuard

Use this skill to perform risk analysis, impact assessment, and record transactional provenance for code changes. ChangeGuard provides a safety and planning layer to understand what changed, what is affected, and what must be verified.

## When NOT to use

Avoid triggering ChangeGuard for:
- **Trivial Formatting**: Pure whitespace changes or `cargo fmt` runs.
- **Dependency Bumps**: Simple version updates in lockfiles with no API changes.
- **Explicit Bypass**: When the user explicitly says "just make the edit" or "bypass ChangeGuard."
- **Non-Code Assets**: Edits to binary files, media, or temporary scratch files.

## Availability & Fallback

ChangeGuard is a **native binary** — invoke it as `changeguard <command>`. Do **not** use `npx changeguard`, `cargo run --`, `./changeguard`, or any other wrapper.

Check if it is available and initialized in the current repository:

```bash
changeguard doctor
```

If the command is missing, see [install.md](./references/install.md). If you cannot install it, continue with standard repository tools but inform the user that ChangeGuard signals are unavailable.

## Core Workflow

Before making a meaningful edit, assess the risk:

```bash
changeguard scan --impact
```

Read the generated report at `.changeguard/reports/latest-impact.json` to identify risk level, affected symbols, temporal couplings, runtime dependencies (environment variables, config keys), and — when configured — relevant documentation decisions, production observability signals, and affected API contracts.

After making edits, verify the change:

```bash
changeguard verify
```

Evidence of successful validation is stored in `.changeguard/reports/latest-verify.json`. For full command details, see [commands.md](./references/commands.md).

## Indexing (Observability & Intelligence)

ChangeGuard can index documentation and API specs for richer impact analysis:

```bash
changeguard index --docs      # index markdown/text docs from configured paths
changeguard index --contracts # index OpenAPI 3.x / Swagger 2.0 specs
```

These populate the embedding store used by semantic retrieval, contract matching, and test prediction. Re-indexing skips unchanged files via content-addressed hashing.

## AI Backend (Ask Command)

The `ask` command supports two AI backends:

```bash
changeguard ask --backend local "review this change"   # local LLM (llama-server)
changeguard ask --backend gemini "analyze the impact"  # Gemini API (default)
```

Auto-selection: if `prefer_local = true` and a local base URL is configured, Local is used; otherwise Gemini. Set `GEMINI_API_KEY` for Gemini. The local backend uses an OpenAI-compatible `/v1/chat/completions` endpoint.

## Configuration (New Sections)

In `changeguard.toml`:

```toml
[local_model]
base_url = "http://localhost:8081"         # or CHANGEGUARD_LOCAL_MODEL_URL
embedding_model = "bge-m3"                 # or CHANGEGUARD_EMBEDDING_MODEL
generation_model = "qwen3.5-9b"            # or CHANGEGUARD_GENERATION_MODEL
dimensions = 0                             # 0 = auto-detect; or CHANGEGUARD_EMBEDDING_DIMENSIONS
timeout_secs = 30
prefer_local = true

[docs]
include = [".changeguard/docs/*.md", "README.md"]
chunk_tokens = 512
chunk_overlap = 64
retrieval_top_k = 5

[observability]
prometheus_url = ""                        # Prometheus API base URL
log_paths = []                             # log file paths to scan
error_rate_threshold = 0.05
log_lookback_secs = 3600
risk_threshold = 0.6

[contracts]
spec_paths = []                            # OpenAPI/Swagger spec file globs
match_threshold = 0.5

[coverage]
enabled = true                             # Master toggle for M7 features
[coverage.traces]
enabled = true
[coverage.sdk]
enabled = true
patterns = ["stripe", "auth0", "aws-sdk"] # SDK patterns to detect
[coverage.services]
enabled = true
[coverage.data_flow]
enabled = true
[coverage.deploy]
enabled = true
[coverage.ci_self_awareness]
enabled = true
[coverage.adr_staleness]
enabled = true
threshold_days = 365
```

## Impact Packet Enrichment

When configured, impact reports include these enrichment sections:

| Field | Source | Description |
|---|---|---|
| `relevant_decisions` | Doc index + embedding similarity | Semantically relevant documentation chunks (includes staleness warnings) |
| `observability` | Prometheus + log scanner | Production signals (latency, error rate, log anomalies) with severity |
| `affected_contracts` | API endpoint index + file embeddings | Public API endpoints potentially affected by the change |
| `trace_config_drift` | Trace config file & env-var detection | Changes to OTEL, Jaeger, Datadog collector configs or env vars |
| `sdk_dependencies_delta` | Third-party SDK import analysis | New or modified integrations with Stripe, AWS, Auth0, etc. |
| `service_map_delta` | Route topology & service map inference | Impact on inferred service boundaries and cross-service edges |
| `data_flow_matches` | Call graph → Data model coupling | Co-changes between API route handlers and the data models they touch |
| `deploy_manifest_changes`| Deployment manifest classification | Changes to Dockerfiles, K8s manifests, Terraform, or Helm charts |

All enrichment degrades gracefully: if the local model is unreachable or configuration is absent, enrichment is a silent no-op. No blocking, no panics. Risk elevation from observability/contract signals escalates `risk_level` (Low→Medium→High) without overwriting rule-based risk reasons.

## Root Cause & Test Prediction

When `semantic_weight > 0`, the `verify` command queries past test outcomes by diff embedding similarity and blends semantic scores with rule-based predictions:

```bash
changeguard verify --explain   # show which past outcomes influenced each prediction
```

Set `semantic_weight = 0` to disable (regression-safe identical output). The predictor shows "warming up" messages until 50 historical outcomes are recorded.

## Ledger Workflow (Provenance)

For tracked changes, record the intent and outcome in the ledger.

**Tracked Edit (Manual):**
1. `changeguard ledger start <path> --category <CAT> --message "Intent"`
2. *Perform edits...*
3. `changeguard ledger commit <tx-id> --summary "Done" --reason "Why"`

**Surgical Edit (Atomic):**
Use this for single-file changes where the start and commit happen together:
```bash
changeguard ledger atomic <path> --category <CAT> --summary "Task" --reason "Goal"
```

**Lightweight Note:**
Use this to add metadata to a file without a formal transaction:
```bash
changeguard ledger note <path> "Metadata note"
```

## Strategic Reasoning

Adjust your coding strategy based on ChangeGuard signals:

1. **Temporal Coupling**: If a changed file has a high affinity (>70%) with an unchanged file, you **MUST** read that unchanged file. Logical dependencies often exist where imports do not.
2. **Hotspots**: Files with high hotspot scores are brittle. Prioritize refactoring or higher test coverage when editing them. **Note**: When entering an unfamiliar codebase, `changeguard hotspots` serves as an orientation map of where complexity is concentrated.
3. **Federated Impact**: If `federated_impact` warnings appear, your change may break a sibling repository. Explain this risk to the user.
4. **Predictive Verification**: Trust the `verify` command's suggestions, even if they seem unrelated; they are often based on historical failure correlations.
5. **Drift Detection**: If `ledger status` shows `UNAUDITED` entries, files were modified outside a transaction. Use `ledger reconcile` or `ledger adopt` before continuing.

## Interpreting Results

Use the `riskLevel` from impact reports to route your effort:
- **Low**: Small/isolated change. Run suggested verification.
- **Medium**: Inspect affected symbols and risk reasons before choosing tests.
- **High**: Slow down. Inspect temporal couplings, public API changes, and cross-repo links before finalizing. If `riskLevel` was elevated by observability signals or contract risk, the elevation is additive — rule-based risk reasons are preserved alongside.

For quick triage, use `changeguard impact --summary`.

## Editing Rules

**Before Edits:**
- Run `changeguard scan --impact`.
- For tracked changes, run `changeguard ledger start`.

**During Edits:**
- Do not edit state under `.changeguard/`.
- Do not commit transient ChangeGuard files or SQLite state.

**After Edits:**
- Run `changeguard verify` and any repo-specific tests.
- For tracked changes, run `changeguard ledger commit`.

## Working on ChangeGuard Itself

When editing ChangeGuard's own source code, the installed binary at `~/.cargo/bin/changeguard.exe` may be stale. Rebuild and reinstall after every source change:

```powershell
cargo build --release
Copy-Item -Force .\target\release\changeguard.exe $env:USERPROFILE\.cargo\bin\changeguard.exe
```

The CI gate for ChangeGuard development is:
```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
cargo deny check
```

## Final Response Template (Optional)

For substantive changes, summarize the evidence:
```text
ChangeGuard:
- impact: <low|medium|high> (risk reasons)
- hotspots/couplings: <findings or "none">
- verification: <commands run and result>
- ledger: <tx_id or "untracked">
```
