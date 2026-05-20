# Track I3-3: Local Model Windows Preflight Check

**Milestone:** I — Issue Remediation  
**Phase:** 3 — Feature Depth  
**Issue:** CG-1c  
**Status:** In Planning

## Objective

When `ask --backend local` fails due to the `localhost` → IPv6 resolution bug (CG-1a, fixed in Track I1-1), the error message is now more informative (from CG-1b). However, even with a good error message, users may not know to change their config. This track adds a proactive preflight check: detect `localhost` in `base_url` on Windows and either warn or automatically retry on `127.0.0.1`.

## Requirements

### `check_url_for_windows_localhost(url: &str) -> Option<String>`
Returns `Some(suggested_url)` if:
- The platform is Windows (`#[cfg(target_os = "windows")]`)
- The `url` contains `//localhost` (case-insensitive)

Returns `None` on non-Windows or if URL does not contain `localhost`.

### Integration Points

**In `complete()` (local model client):**
- Before attempting the request, call `check_url_for_windows_localhost`.
- If `Some(fixed_url)` is returned, log a `warn!` message:
  ```
  WARN 'localhost' in base_url resolves to IPv6 on Windows. Retrying with 127.0.0.1. Update your config to avoid this.
  ```
- Retry the request using `fixed_url` transparently (no user action required).
- If the retry also fails, surface the error with both the original and retried URL.

**In `changeguard doctor`:**
- After calling `ping_completions` (Track I2-4), if the configured `base_url` contains `localhost` on Windows, emit a yellow advisory:
  ```
  ⚠  base_url contains 'localhost' — on Windows this resolves to ::1 (IPv6). Use 127.0.0.1 instead.
  ```

### Platform Guard
All `localhost` detection logic must be wrapped in `#[cfg(target_os = "windows")]` or a runtime `cfg!(target_os = "windows")` check so Linux/macOS builds are unaffected.

## API Contract

```rust
// src/local_model/client.rs or src/local_model/health.rs
#[cfg(target_os = "windows")]
pub fn check_url_for_windows_localhost(url: &str) -> Option<String>;
```

## Testing Strategy

- Unit test `localhost_detected_on_windows`: on Windows target, call with `"http://localhost:8081"`; assert `Some("http://127.0.0.1:8081")`.
- Unit test `non_localhost_ignored`: call with `"http://127.0.0.1:8081"`; assert `None`.
- Unit test `non_localhost_host_ignored`: call with `"http://myserver:8081"`; assert `None`.
- `#[cfg(target_os = "windows")]` gate ensures these tests only compile/run on Windows.

## Out of Scope

- Automatic config file rewriting (warn only, not write).
- IPv6 support or dual-stack fallback (out of scope; `127.0.0.1` is the correct default).
