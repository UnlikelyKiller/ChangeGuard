# Track I2-4 Plan: Doctor Completions Endpoint Ping

## Phase 1 — Red (Failing Tests)

- [ ] Add `ping_completions` stub (returns `Err("not implemented".to_string())`).
- [ ] Write unit tests (in `src/local_model/client.rs` or `health.rs` `mod tests`):
  - `completions_ping_success`: httpmock returns 200 with valid choices body; assert `Ok`.
  - `completions_ping_transport_failure`: httpmock drops connection; assert `Err` with non-empty string.
  - `completions_ping_non_200`: httpmock returns 503; assert `Err` containing "503".
- [ ] Commit: `test(doctor): red — completions ping validates 200, surfaces transport and status errors`

## Phase 2 — Green (Implementation)

- [ ] Implement `ping_completions`:
  - Build a minimal chat completions request body (`max_tokens: 1`, single system message).
  - Use `ureq::AgentBuilder` with `timeout_read(Duration::from_secs(5))`.
  - On `Ok`, parse `model` from response body (best-effort, fall back to URL).
  - On `Transport` error, return `Err(format!("unreachable — {inner}"))`.
  - On non-200 status, return `Err(format!("{code} from server"))`.
- [ ] In `src/commands/doctor.rs`:
  - Call `ping_completions` after the existing embeddings ping.
  - Print two separate lines (embeddings + completions) with `✓` / `⚠` icons.
  - Use yellow ANSI color for the `⚠` line (match existing doctor color palette).
- [ ] Run CI gate: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`.
- [ ] Commit: `feat(doctor): split embedding/completion model status; add completions ping (CG-10)`

## Verification

- [ ] With router running: `changeguard doctor` shows two lines — both green.
- [ ] With router stopped: `changeguard doctor` shows embeddings green (or red), completions yellow `⚠` with cause.
- [ ] With `base_url = "http://localhost:8081"` (pre-fix URL): completions line shows meaningful error, not generic "not reachable".
