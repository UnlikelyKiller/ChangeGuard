# Track Z1: Command Audit Remediation and Ollama Cloud Hardening

## Objective

Close every remaining "Doesn't Work / Risks" and friction item from the 2026-06-09 ChangeGuard command audit, while preserving the CLI-first, local-first contract. The track must turn each audit finding into either a fixed behavior, an explicit non-goal with rationale, or a tested documented limitation.

This track is high priority because one finding is a secret exposure: `changeguard config view --json` emitted an Ollama API key from `.changeguard/config.toml`.

## Evidence

Local audit evidence:

- `cargo build --all-features` passed on the dirty checkout.
- `cargo fmt --all -- --check` passed.
- `cargo clippy --all-targets --all-features -- -D warnings` passed.
- `cargo nextest run --test integration` passed: 224 passed, 3 skipped.
- `target\debug\changeguard.exe scan --impact` reported `riskLevel = high`.
- `target\debug\changeguard.exe ledger status --compact` reported `1 pending, 0 unaudited drift`.
- Manual Ollama Cloud smoke using `.changeguard\config.toml` `ollama_key` succeeded against `https://ollama.com/api/chat` with `model = "minimax-m3:cloud"`.
- Manual OpenAI-compatible smoke authenticated at `https://ollama.com/v1/chat/completions`.
- `https://api.ollama.com/v1/chat/completions` returned 401 with the same key.
- `changeguard ask --backend local` still returned `Ollama Cloud fallback returned 401: unauthorized`.

External research:

- Ollama API introduction documents local base URL `http://localhost:11434/api` and cloud base URL `https://ollama.com/api`.
- Ollama authentication docs say direct access to `https://ollama.com/api` requires a bearer API key.
- Ollama chat docs define native `POST /api/chat` with `messages`, default streaming, and response content under `message.content`.
- Ollama OpenAI compatibility docs show `/v1/chat/completions` compatibility for OpenAI-style clients.
- MiniMax M3 is cloud-enabled as `minimax-m3:cloud`, high usage, with a 512K context window on the model page.
- Pricing docs describe cloud concurrency limits: Free 1, Pro 3, Max 10, with usage based primarily on GPU time and reset windows.

Primary sources:

- https://docs.ollama.com/api/introduction
- https://docs.ollama.com/api/authentication
- https://docs.ollama.com/api/chat
- https://docs.ollama.com/api/openai-compatibility
- https://ollama.com/library/minimax-m3
- https://ollama.com/pricing

## Scope

### Z1.1 Secret-safe configuration output

Fix all config display paths so secret values never print in human or JSON mode.

Required behavior:

- `config view --json` redacts every field whose name implies a secret, including `api_key`, `apikey`, `key`, `token`, `secret`, `password`, `credential`, and known aliases such as `ollama_key`.
- Redaction applies recursively to nested config structs, maps, arrays, and future provider-specific sections.
- Redacted values retain useful state: absent, empty, set, defaulted, and source where available.
- Config schema output may disclose variable names and secret metadata, but never raw values.
- Tests include `ollama_key`, `ollama_cloud_api_key`, `GEMINI_API_KEY`, and generic nested keys.

Definition of done:

- A regression test fails if `config view --json` contains the configured secret literal.
- Human `config view` also redacts the same fields.
- `config verify --json`, `config schema --json`, `config diff --json`, `scan --impact --json`, and bridge export are checked for the same literal in focused tests or smokes.

### Z1.2 Ollama Cloud config compatibility and endpoint routing

Make ChangeGuard able to use the key currently present in `.changeguard\config.toml` and align defaults with official Ollama docs.

Required behavior:

- Accept `ollama_key` as a backward-compatible alias for `ollama_cloud_api_key`.
- Accept `OLLAMA_API_KEY` as an env/dotenv fallback in addition to `OLLAMA_CLOUD_API_KEY`.
- Prefer documented cloud native base `https://ollama.com/api` for native Ollama requests.
- Support OpenAI-compatible base `https://ollama.com` for `/v1/chat/completions`.
- Do not append `/v1/chat/completions` to a base URL that already ends in `/api`.
- Validate base URL shape and show a clear warning when `https://api.ollama.com` is configured for an OpenAI-compatible call.
- Preserve local Ollama behavior at `http://localhost:11434/api` and local OpenAI-compatible behavior at `http://localhost:11434/v1`.

Definition of done:

- `changeguard ask --backend local --timeout 20 "..."` succeeds with a valid `ollama_key` and `minimax-m3:cloud`, without requiring env vars.
- Invalid key returns a concise 401 message that names which credential source was used, without printing the key.
- Misconfigured base URL returns a targeted hint: use `https://ollama.com/api` for native Ollama or `https://ollama.com` for OpenAI-compatible mode.
- Unit tests cover alias parsing, env precedence, native `/api/chat`, OpenAI `/v1/chat/completions`, and the `api.ollama.com` failure mode.

### Z1.3 Ollama native response parsing and streaming contract

Add a native Ollama completion path rather than treating Ollama Cloud only as OpenAI-compatible.

Required behavior:

- Native requests call `POST {base}/chat` when the configured base URL ends in `/api`.
- Native request body includes `model`, `messages`, `stream = false`, and options mapped from `CompletionOptions`.
- Native response parses `message.content`; if empty but `message.thinking` or OpenAI-compatible `reasoning` exists, report a concise diagnostic rather than silently treating it as success.
- Timeout handling and 429/503 retry behavior remain bounded.
- Tool-call fields do not break parsing.

Definition of done:

- Mock tests prove native `/api/chat` parses `message.content`.
- Mock tests prove OpenAI-compatible `/v1/chat/completions` parses `choices[0].message.content`.
- Empty content with reasoning-only output produces an actionable error or fallback policy.
- `ask` continues to pass existing local-server tests.

### Z1.4 Verify health boundedness

Fix `changeguard verify --health` hanging or running too long without output.

Required behavior:

- Print each probe before it starts in human mode.
- Bound every probe with an explicit timeout.
- Never run the full verification suite as part of health mode.
- Summarize executable availability, configured steps, runner selection, local model availability, and predictor/index readiness.
- JSON mode, if added, must be clean stdout JSON.

Definition of done:

- `target\debug\changeguard.exe verify --health` returns within 5 seconds on this machine or reports which bounded probe exceeded its timeout.
- A test covers a deliberately slow/missing dependency and proves health mode exits.
- Health output includes nextest vs cargo-test runner selection without executing tests.

### Z1.5 Dry-run plan compression

Keep `verify --dry-run` useful without printing hundreds of graph-node entries inline.

Required behavior:

- Group predicted impacts by source: rules, call graph, temporal, test mapping, semantic.
- Show counts and top N examples per group.
- Add `--verbose` or equivalent to display the full expanded list.
- Preserve machine-readable output if JSON mode is introduced later.

Definition of done:

- On the current dirty tree, `verify --dry-run` fits in a normal terminal page while still showing the selected runner and required commands.
- A test verifies grouped counts and top-example output.

### Z1.6 Structured output contract hardening

Re-audit JSON and NDJSON behavior after current Y-track fixes.

Required behavior:

- `--json` means parseable JSON on stdout and diagnostics on stderr.
- NDJSON bridge/search output is explicitly documented and tested as NDJSON, not a JSON array.
- Human output is never mixed into JSON stdout.
- Logs from indexing/search do not corrupt structured output.

Definition of done:

- Command-level tests parse JSON for each JSON surface.
- NDJSON tests parse each non-empty line independently.
- A smoke script can run `changeguard <surface> --json | ConvertFrom-Json` for array/object outputs.

### Z1.7 Empty-state fixture depth for W-surfaces

Move beyond "empty array means command did not crash" for key surfaces.

Required behavior:

- Add fixtures or temp-repo setups that produce non-empty output for endpoints, services, deploy, observability coverage, tests, and dependencies.
- Preserve empty-state hints in human mode.
- JSON empty states remain schema-stable.

Definition of done:

- At least one test per listed surface asserts a meaningful non-empty row and key fields.
- Empty-state tests still cover no-data behavior.

### Z1.8 Bridge query output ordering

Make `bridge query` output predictable.

Required behavior:

- Status/progress messages go to stderr.
- Retrieved records go to stdout in either human format or structured format.
- If IPC falls back to CLI, the fallback reason is visible but not interleaved with records.

Definition of done:

- `bridge query "command audit"` has deterministic stdout/stderr separation in tests.
- Existing bridge query tests continue to pass.

### Z1.9 Federation and temp-repo onboarding UX

Clarify what `init`, `scan`, `index`, and `federate scan` require in fresh repositories.

Required behavior:

- `federate scan` in an initialized but unindexed repo explains whether `scan`, `index`, or `federate export` is required.
- `ledger atomic` blocked by an existing pending transaction explains the conflict and suggests commit/rollback/status.
- `init` docs state which commands create config, ledger state, graph state, and schema export.

Definition of done:

- Temp-repo CLI tests cover `init -> federate scan`, `init -> federate export`, and pending-conflict messaging.
- Error messages name exact next commands.

## Non-Goals

- Do not change `.changeguard` state file formats except through normal migrations if needed.
- Do not require Ollama Cloud for baseline ChangeGuard operation.
- Do not make network access mandatory for tests; live Ollama smokes remain manual or opt-in.
- Do not remove Gemini support.
- Do not expose or commit API keys.

## Verification Strategy

Targeted first:

- `cargo test local_model::client`
- `cargo test config::model`
- `cargo test commands::config`
- `cargo nextest run --test integration cli_config cli_ask cli_verify cli_surfaces bridge_query_tests cli_federate`
- Manual live smoke with a valid Ollama key, redacting all output:
  - Native: `POST https://ollama.com/api/chat`
  - OpenAI-compatible: `POST https://ollama.com/v1/chat/completions`
  - CLI: `changeguard ask --backend local --timeout 20 "Reply with exactly: ChangeGuard Ollama smoke ok"`

Final gates:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run --lib --bins --workspace`
- `cargo nextest run --test integration`
- `changeguard verify`
- `cargo install --path .`

## Risks

- Live Ollama behavior may vary by plan, concurrency, usage limits, and model availability.
- MiniMax M3 can emit reasoning-only content when constrained too tightly; tests should avoid low `max_tokens` values for success assertions.
- Secret redaction must be generic enough for future provider config fields.
- Changing completion endpoint selection can regress local OpenAI-compatible servers if URL normalization is too aggressive.
