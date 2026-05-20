# Track I1-1 Plan: Local Model URL Hardening & Error Transparency

## Phase 1 — Red (Failing Tests)

- [ ] In `src/local_model/client.rs` `mod tests`, add test `transport_error_includes_cause`:
  - Start an httpmock server that refuses the connection (or returns an invalid response to force a transport error).
  - Assert the returned `Err` string contains `"not reachable at"` and a non-empty substring after ` — `.
- [ ] Add test `default_base_url_is_127` in `src/config/model.rs` or inline: assert `LocalModelConfig::default().base_url` equals `"http://127.0.0.1:8081"`.
- [ ] Commit: `test(local-model): red — transport error exposes cause and default URL is 127.0.0.1`

## Phase 2 — Green (Implementation)

- [ ] **CG-1a:** Locate the `Default` impl or config template that sets `base_url = "http://localhost:8081"` and change it to `"http://127.0.0.1:8081"`.
  - Add the comment about IPv6 resolution.
- [ ] **CG-1b:** In `src/local_model/client.rs` `complete()`, change the `Transport` arm to use `inner` (drop the leading `_`), and include it in the `Err` string with ` — {}`.
- [ ] Run CI gate: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`.
- [ ] Commit: `fix(local-model): use 127.0.0.1 default and surface transport error cause (CG-1a, CG-1b)`

## Verification

- [ ] `changeguard doctor` still shows local model status.
- [ ] Temporarily set `base_url = "http://localhost:9999"` (nothing listening) and run `changeguard ask "test" --backend local` — confirm the error message contains a meaningful cause string.
- [ ] Restore config.
