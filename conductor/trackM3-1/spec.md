# Specification: Track M3-1 — Local Model Client & Context Assembly

## Objective
Build the OpenAI-compatible completions client for llama-server and the context assembly pipeline that constructs prompts from the impact packet, similar past history, and user query. This is the local model infrastructure for `changeguard ask`.

## Components

### 1. Completions Client (`src/local_model/client.rs`)

```rust
pub fn complete(
    base_url: &str,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    timeout_secs: u64,
) -> Result<String>
```

- POST to `{base_url}/v1/chat/completions`
- Set `Content-Type: application/json`
- Body: `{"model": "<model>", "messages": [{"role":"system","content":"..."},{"role":"user","content":"..."}], "stream": false}`
- Parse `choices[0].message.content` as response string
- On `503`, retry once after 2s delay; on `ureq::Error::Status(429, _)`, return `Err` with "rate limited" message; on any other `Status` error, return `Err` with server response body
- When server is unreachable (connection refused, DNS failure), return `Err` with: "Local model server not reachable at {base_url}. Start llama-server or use --backend gemini."
- Use `ureq::AgentBuilder` with `.timeout_read(Duration::from_secs(timeout_secs)).timeout_write(Duration::from_secs(30))` matching `src/gemini/wrapper.rs:60`

### 2. Context Assembly (`src/local_model/context.rs`)

```rust
pub fn assemble_context(
    config: &LocalModelConfig,
    packet: &ImpactPacket,
    mode: GeminiMode,
    query: &str,
    diff: Option<&str>,
) -> String
```

Component order (descending priority for budget retention):
1. User query (never truncated)
2. Impact packet summary: risk_level, risk_reasons, top-3 changed file paths (~500 token target)
3. Retrieved `relevant_decisions` from packet (formatted as a fenced markdown block)
4. Top-5 temporal couplings summary
5. Top-5 hotspots summary
6. Full diff (only for `ReviewPatch` mode, if available)

Budget enforcement:
- Apply `enforce_budget` to components 2–6 against `config.context_window - 500` (reserve 500 tokens for generation headroom)
- When budget overflows, components are trimmed from lowest priority (diff first, then hotspots, then couplings, then decisions)
- Log `WARN` with component name when any component is trimmed

### 3. Shared Rerank Client Alias (`src/local_model/rerank.rs`)

Re-export from `src/retrieval/rerank.rs`. The real implementation lives in `src/retrieval/` (built by Track M2-2). This module provides a convenient access point for callers in the `local_model` namespace.

### 4. Module Declaration

- Create `src/local_model/mod.rs` exporting `client`, `context`, `rerank` submodules
- Add `pub mod local_model;` to `src/lib.rs`

## Test Specifications

| Test | Assertion |
|---|---|
| `complete` mock valid response | Returns content string from `choices[0].message.content` |
| `complete` mock 503 retry | Retries once, succeeds on second attempt |
| `complete` unreachable | Returns `Err` with descriptive message |
| `assemble_context` all under budget | All components present in output |
| `assemble_context` budget overflow | Components trimmed from lowest priority; query always present |
| `assemble_context` ReviewPatch mode | Diff included at end |
| `assemble_context` non-ReviewPatch mode | Diff not included |
| `assemble_context` no relevant_decisions | Decisions block absent, other components adjusted accordingly |

## Constraints & Guidelines

- **TDD**: All tests written before implementation.
- **No panics**: All `Result` propagation via `?`.
- **Mock tests**: Use `httpmock` for all HTTP-calling tests.
- **System prompt reuse**: The system prompts from `src/gemini/modes.rs` are reused; do not duplicate.
- **CI safety**: All tests pass with `base_url = ""` (no network calls in CI).

## Hardening Additions (not in original plan)

| Addition | Reason |
|---|---|
| `complete()` uses `ureq::AgentBuilder` pattern from `src/gemini/wrapper.rs:60` | Consistency with existing HTTP client patterns. Configurable `timeout_read`/`timeout_write`. |
| `Content-Type: application/json` header set on all requests | Required by OpenAI-compatible API spec; prevents silent 400s from default Content-Type. |
