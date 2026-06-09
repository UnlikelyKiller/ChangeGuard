# Track Z1 Plan: Command Audit Remediation and Ollama Cloud Hardening

## Phase 0: Baseline and Safety

- [x] Confirm ledger state and drift with `changeguard ledger status --compact`.
- [x] Run `changeguard scan --impact` and inspect `.changeguard/reports/latest-impact.json`.
- [x] Record a redacted Ollama config summary: field names present, base URL shape, model, and secret source state.
- [x] Run manual Ollama smoke without printing credentials:
  - [x] `POST https://ollama.com/api/chat` with `model = "minimax-m3:cloud"`.
  - [x] `POST https://ollama.com/v1/chat/completions` with `model = "minimax-m3:cloud"`.
  - [x] Confirm `https://api.ollama.com/v1/chat/completions` behavior is either unsupported or intentionally handled.

Definition of done: Baseline evidence proves whether failures are local implementation issues, config issues, provider issues, or plan-limit issues.

## Phase 1: Secret Redaction

- [x] Add config redaction helper that recursively redacts secret-like field names.
- [x] Treat `ollama_key` as secret.
- [x] Apply redaction to `config view` human output.
- [x] Apply redaction to `config view --json`.
- [x] Audit `config verify`, `config schema`, `config diff`, impact output, and bridge export for raw secret leakage.
- [x] Add regression tests using a unique sentinel secret string and assert it never appears.

Definition of done: No CLI output path exercised by tests emits the sentinel secret.

## Phase 2: Ollama Config Resolution

- [x] Add `ollama_key` as a serde alias or explicit compatibility field for `ollama_cloud_api_key`.
- [x] Resolve `OLLAMA_API_KEY` in addition to `OLLAMA_CLOUD_API_KEY`.
- [x] Set documented native cloud default to `https://ollama.com/api`.
- [x] Preserve OpenAI-compatible support with base URL `https://ollama.com`.
- [x] Add config validation hints for unsupported or suspicious base URL shapes.
- [x] Add tests for TOML, env, and dotenv precedence.

Definition of done: A config containing only `[local_model] ollama_key = "..."; ollama_cloud_model = "minimax-m3:cloud"` is sufficient for `has_ollama_cloud_fallback`.

## Phase 3: Ollama Completion Client

- [x] Introduce endpoint kind detection: local OpenAI-compatible, Ollama native, Ollama cloud native, or explicit OpenAI-compatible cloud.
- [x] Route `/api` bases to native `POST /chat`.
- [x] Route non-`/api` bases to OpenAI-compatible `POST /v1/chat/completions`.
- [x] Parse native `message.content` responses.
- [x] Preserve OpenAI-compatible `choices[0].message.content` parsing.
- [x] Add clear diagnostics for 401, 404, 429, 503, timeout, empty content, and reasoning-only content.
- [x] Keep retries bounded and avoid unbounded provider probes.

Definition of done: Mocked tests cover native and OpenAI-compatible Ollama paths, and the live smoke succeeds when a valid key is configured.

## Phase 4: `ask` UX and Provider Diagnostics

- [x] Ensure `ask --backend local` does not fail the local probe when a valid cloud fallback is configured.
- [x] Print provider selection in human mode without exposing secrets.
- [x] Keep progress on stderr.
- [x] Add actionable failure text for invalid key, unsupported base URL, exhausted usage, concurrency queue rejection, and model unavailable.
- [x] Confirm Gemini fallback/selection remains unchanged unless explicitly requested.

Definition of done: `changeguard ask --backend local --timeout 20` succeeds with Ollama Cloud or returns a one-screen actionable error.

## Phase 5: `verify --health`

- [x] Identify why current `verify --health` exceeds 15 seconds.
- [x] Add per-probe progress lines in human mode.
- [x] Add timeout to every probe.
- [x] Ensure health mode never executes full test or verification commands.
- [x] Add regression tests for slow and missing probes.

Definition of done: `verify --health` returns within 5 seconds on the ChangeGuard repo, or reports one bounded timeout and exits normally.

## Phase 6: `verify --dry-run` Compression

- [x] Group predicted impacts by source.
- [x] Show counts and top examples per group.
- [x] Add verbose full-expansion mode.
- [x] Add tests for grouped output.

Definition of done: Dry-run output remains useful on high-risk dirty trees without flooding the terminal.

## Phase 7: Structured Output and Bridge Ordering

- [x] Re-run all `--json` command smokes and parse stdout.
- [x] Move human/log diagnostics to stderr where needed.
- [x] Document NDJSON output surfaces explicitly.
- [x] Update bridge query output ordering and tests.

Definition of done: JSON and NDJSON command outputs are script-safe and deterministic.

## Phase 8: Non-Empty Surface Fixtures

- [x] Add endpoint fixture producing at least one route.
- [x] Add service fixture producing at least one service/topology row.
- [x] Add deploy fixture producing at least one manifest impact row.
- [x] Add observability fixture producing at least one coverage row.
- [x] Add test-mapping fixture producing at least one test row.
- [x] Add dependency fixture producing at least one dependency row without requiring network.

Definition of done: Each W-surface has both non-empty and empty-state coverage.

## Phase 9: Federation and Temp-Repo UX

- [x] Improve `federate scan` fresh-repo message.
- [x] Improve ledger pending-conflict message for `ledger atomic` and `ledger start`.
- [x] Add temp-repo tests for `init`, `federate scan`, `federate export`, and pending conflict guidance.

Definition of done: Fresh-repo users receive exact next commands instead of generic missing-index errors.

## Phase 10: Final Verification and Install

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo nextest run --lib --bins --workspace`
- [x] `cargo nextest run --test integration`
- [x] `changeguard verify`
- [x] `cargo install --path .`
- [x] `changeguard ledger status --compact`

Definition of done: All final gates pass, installed binary matches source behavior, no secrets are printed in verification logs, and ledger state is reported explicitly.
