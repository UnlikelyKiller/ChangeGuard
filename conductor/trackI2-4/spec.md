# Track I2-4: Doctor Completions Endpoint Ping

**Milestone:** I — Issue Remediation  
**Phase:** 2 — Reliability  
**Issue:** CG-10  
**Status:** In Planning

## Objective

`changeguard doctor` reports "Local Model: reachable" by testing only the `/v1/embeddings` endpoint via `embed::client::ping`. It does not test `/v1/chat/completions`, which is required for `ask --backend local`. This creates false confidence — the doctor reports green while `ask --backend local` fails.

## Requirements

### Completions Liveness Probe
Add a `ping_completions(config: &LocalModelConfig) -> Result<String, String>` function (in `src/local_model/client.rs` or a new `src/local_model/health.rs`) that:
1. Sends a minimal `POST /v1/chat/completions` with a 1-token prompt and `max_tokens: 1`.
2. Returns `Ok(model_name)` on HTTP 200 (even if the response body is truncated).
3. Returns `Err(reason)` on any transport error or non-200 status.
4. Uses a short timeout (5 seconds) — this is a health check, not a real completion.

### Doctor Output: Two Separate Lines
Replace the single "Local Model: reachable (1024 dims, model: bge-m3)" line with two:
```
Embedding model:   ✓ bge-m3        @ http://127.0.0.1:8081  (1024 dims)
Completion model:  ✓ qwen3.5-9b    @ http://127.0.0.1:8081
```

Or, when completions are unreachable but embeddings work:
```
Embedding model:   ✓ bge-m3        @ http://127.0.0.1:8081  (1024 dims)
Completion model:  ⚠ unreachable   @ http://127.0.0.1:8081  — Connection refused
```

The `⚠` line should be yellow-colored (same palette as existing doctor warnings).

### No Failure on Unreachable Completions
`doctor` should not exit with a non-zero code solely because completions are unreachable. It is informational. Only emit `Err` from `doctor` if storage is corrupt or the binary itself cannot initialize.

## API Contract

```rust
// src/local_model/client.rs (or health.rs)
pub fn ping_completions(config: &LocalModelConfig) -> Result<String, String>;
// Returns Ok(model_id) or Err(human-readable reason)
```

## Testing Strategy

- Unit test using `httpmock`:
  - `completions_ping_success`: mock server returns HTTP 200 with `{"choices":[{"message":{"content":"hi"}}]}`; assert `Ok("test-model")`.
  - `completions_ping_transport_failure`: mock server closes connection; assert `Err` containing a non-empty reason.
  - `completions_ping_non_200`: mock server returns HTTP 503; assert `Err` containing "503".
- No integration test for the doctor output formatting.

## Out of Scope

- No change to the embeddings ping logic.
- Model name detection from the completions response is best-effort (read `model` field from response body if present, else report URL).
