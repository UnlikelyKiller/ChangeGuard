# Track I3-3 Plan: Local Model Windows Preflight Check

## Phase 1 — Red (Failing Tests)

- [ ] Add `check_url_for_windows_localhost` stub (returns `None` always) behind `#[cfg(target_os = "windows")]`.
- [ ] Write unit tests (Windows-only via `#[cfg(target_os = "windows")]`):
  - `localhost_detected`: input `"http://localhost:8081"` → `Some("http://127.0.0.1:8081")`.
  - `non_localhost_ignored`: input `"http://127.0.0.1:8081"` → `None`.
  - `other_host_ignored`: input `"http://myserver:8081"` → `None`.
  - `uppercase_localhost_detected`: input `"http://LOCALHOST:8081"` → `Some("http://127.0.0.1:8081")`.
- [ ] Commit: `test(local-model): red — Windows localhost detection`

## Phase 2 — Green (Implementation)

- [ ] Implement `check_url_for_windows_localhost`:
  ```rust
  #[cfg(target_os = "windows")]
  pub fn check_url_for_windows_localhost(url: &str) -> Option<String> {
      if url.to_lowercase().contains("//localhost") {
          Some(url.to_lowercase().replacen("//localhost", "//127.0.0.1", 1))
      } else {
          None
      }
  }
  ```
- [ ] In `complete()` in `src/local_model/client.rs`, before building the `ureq` agent:
  ```rust
  #[cfg(target_os = "windows")]
  let url = if let Some(fixed) = check_url_for_windows_localhost(&url) {
      tracing::warn!("'localhost' resolves to IPv6 on Windows; retrying with 127.0.0.1. Update config to suppress this.");
      fixed
  } else {
      url
  };
  ```
- [ ] In `src/commands/doctor.rs`, after the completions ping, add a `#[cfg(target_os = "windows")]` advisory check on `config.base_url`.
- [ ] Run CI gate: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`.
- [ ] Commit: `feat(local-model): Windows localhost preflight — auto-retry with 127.0.0.1 and doctor advisory (CG-1c)`

## Verification

- [ ] With `base_url = "http://localhost:8081"` and router running: `changeguard ask "test" --backend local` succeeds (auto-retried on 127.0.0.1) and emits a WARN about the config.
- [ ] `changeguard doctor` shows the yellow advisory about `localhost`.
- [ ] With `base_url = "http://127.0.0.1:8081"`: no warning emitted.
