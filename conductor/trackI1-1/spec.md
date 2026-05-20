# Track I1-1: Local Model URL Hardening & Error Transparency

**Milestone:** I — Issue Remediation  
**Phase:** 1 — Hotfixes  
**Issues:** CG-1a, CG-1b  
**Status:** In Planning

## Objective

Fix two related defects in `src/local_model/client.rs` that make the `--backend local` path unreliable on Windows:

1. **CG-1a:** The default `local_model.base_url` resolves to `::1` (IPv6) on Windows because `localhost` is mapped to IPv6 first. The llama-server router binds to `0.0.0.0` (IPv4 only). Change the default to `http://127.0.0.1:8081`.

2. **CG-1b:** `Err(ureq::Error::Transport(_inner))` in `complete()` silently discards the root cause and returns a generic string. Surface `_inner` in the error message so users see the actual OS-level failure (e.g., `Connection refused`, `Network unreachable`).

## Requirements

### CG-1a: Default URL
- Locate where `LocalModelConfig::base_url` receives its default (either `src/config/model.rs` `Default` impl or the `DEFAULT_CONFIG` string in `src/config/defaults.rs`).
- Change the default from `"http://localhost:8081"` to `"http://127.0.0.1:8081"`.
- Add an inline comment: `# Use 127.0.0.1 — 'localhost' resolves to ::1 (IPv6) on Windows, which breaks IPv4-only servers`.

### CG-1b: Transport Error Transparency
- In `src/local_model/client.rs`, the `Transport` arm currently ignores `_inner`:
  ```rust
  Err(ureq::Error::Transport(_inner)) => {
      return Err(format!("Local model server not reachable at {}", config.base_url));
  }
  ```
- Change to surface the inner error:
  ```rust
  Err(ureq::Error::Transport(inner)) => {
      return Err(format!(
          "Local model server not reachable at {} — {}",
          config.base_url, inner
      ));
  }
  ```
- `ureq::Transport` implements `Display`, so `{}` formatting is sufficient; no `{:?}` needed.

## API Contract

`complete()` return type is unchanged (`Result<String, String>`). The `Err` string for transport failures is extended with ` — <inner>`.

## Testing Strategy

- Unit test in `src/local_model/client.rs` (existing `mod tests`): use `httpmock` server that immediately closes the connection; assert the returned `Err` string contains both the URL and a non-empty cause substring.
- No integration test required — verifying the error string format is sufficient.

## Out of Scope

- CG-1c (preflight connectivity check with Windows-specific localhost warning) is Track I3-3.
- No changes to embed client URL defaulting.
