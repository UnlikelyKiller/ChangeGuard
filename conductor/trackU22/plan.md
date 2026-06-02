# Track U22 Plan: ChangeGuard LLM Query Timeout Guardrails

## Context

When `changeguard ask` connects to an LLM backend (local llama-server, Ollama Cloud fallback, or Gemini), a slow or unresponsive server can hang the client indefinitely. The existing timeouts in `src/local_model/client.rs` and `src/gemini/wrapper.rs` (using `ureq::AgentBuilder` timeouts) are *connect* and *read* timeouts on the agent — they apply to the configured value (`config.local_model.timeout_secs` or `config.gemini.timeout_secs`), but there is no CLI-level override and no clear contract for end-users to control them. A user running `changeguard ask` in CI cannot easily say "give up after 2 seconds, log a warning, and exit." This track adds:

1. A `--timeout <seconds>` CLI flag on `changeguard ask` (default 15s).
2. The same value respected by both backends (Local + Gemini).
3. A clear stderr message when the timeout fires, with the request aborting cleanly.
4. A regression test that proves the timeout actually terminates the call (using `httpmock` delays).

The implementation is **purely client-side**: ureq already supports `timeout_read`, so the only new surface area is wiring the flag through the call chain.

## Key Files

- `src/cli.rs` — add `--timeout` to `Commands::Ask`; thread into `run_with`.
- `src/commands/ask.rs` — accept `timeout_secs: u64`; pass to `complete()` and `run_query()`.
- `src/local_model/client.rs` — `complete()` now accepts `timeout_secs_override: Option<u64>`; new `transport_is_timeout` helper walks ureq's `io::ErrorKind::TimedOut` source chain.
- `src/gemini/wrapper.rs` — `run_query` already accepts `timeout_secs: Option<u64>` (no signature change); call site uses the CLI value.
- `src/commands/ask.rs` (Backend::Gemini branch) — uses `timeout_secs` directly instead of `config.gemini.timeout_secs.unwrap_or(120)`.
- `src/commands/config_verify.rs` — new `AskSection` reports `cli_default_timeout_secs`, `local_model.timeout_secs`, and `gemini.timeout_secs`.
- `tests/integration/cli_ask.rs` — regression test using `httpmock` delay that proves the override terminates within 4s.
- `src/local_model/client.rs` tests — `complete_timeout_override_fires` and `complete_timeout_override_none_falls_back_to_config`.

## Design Decisions

- **Default = 15s** matches the spec and is aggressive enough for interactive use; CI users can override with `--timeout 60` or `--timeout 2`.
- **Override precedence**: CLI `--timeout` > config file (`local_model.timeout_secs` / `gemini.timeout_secs`).
- **Signature change**: `complete()` now takes `timeout_secs_override: Option<u64>`; `Some(n)` overrides, `None` falls back to `config.timeout_secs`. All five existing callers (`ask.rs`, `explanation.rs`, `intent_drafter.rs`, `semantic_extractor.rs`, plus the in-file unit tests) updated to pass `None` or `Some(timeout_secs)`.
- **Probe is unchanged**: the 5s `ping_completions` probe in `execute_ask` is fail-fast and intentionally not subject to the CLI override. The override applies to the actual `complete()` call.
- **Timeout detection**: ureq 2.12 normalizes both `WouldBlock` and `TimedOut` to `io::ErrorKind::TimedOut` internally, but only the inner `io::Error` carries the kind — the outer `Transport::Display` string is the OS-level error message (e.g. "Error encountered in the status line"). A new `transport_is_timeout` helper walks the `std::error::Error::source()` chain and `downcast_ref`s to `io::Error` to detect the kind. The error message becomes `"<label> timed out after Ns"`.

## Implementation Tasks (TDD: red → green)

- [x] **U22.1 (red)**: Add `complete_timeout_override_fires` and `complete_timeout_override_none_falls_back_to_config` tests in `src/local_model/client.rs`.
- [x] **U22.2 (green)**: Refactor `complete_with_endpoint` and `complete` in `src/local_model/client.rs` to accept an `Option<u64>` timeout override; add `transport_is_timeout` helper.
- [x] **U22.3**: Add `--timeout <seconds>` flag to `Commands::Ask` in `src/cli.rs` with `default_value_t = 15`; thread into `run_with` and `execute_ask`.
- [x] **U22.4**: Update `execute_ask` signature to accept `timeout_secs: u64`; pass `Some(timeout_secs)` to `complete()` and use `timeout_secs` directly for `run_query()`.
- [x] **U22.5**: Add `test_ask_respects_cli_timeout_override` integration test in `tests/integration/cli_ask.rs` using `httpmock` delay.
- [x] **U22.6**: Add `AskSection` in `src/commands/config_verify.rs`; update section count test; add `test_ask_section_shows_timeout` and `test_ask_section_marks_overridden_values_explicit`.
- [x] **U22.7**: Run the CI gate: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo nextest run --lib --bins --workspace`.
- [x] **U22.8**: `cargo install --path .` and smoke-test the binary (`changeguard ask --help` shows `--timeout 15`).
- [x] **U22.9**: Ledger provenance (start/commit) and push.

## Reused Functions / Patterns

- `ureq::AgentBuilder::timeout_read(Duration)` — already in use at `src/local_model/client.rs:79, 226` and `src/gemini/wrapper.rs:97`. No new deps.
- `ureq::Transport` source chain + `std::io::Error` downcast — new helper, but pattern matches `connection_closed()` in ureq 2.12's own error.rs.
- `httpmock::MockServer` + `.delay()` — already used in `src/local_model/client.rs` tests.
- `ConfigSection` trait in `src/commands/config_verify.rs` (U19) — new `AskSection` plugs in.

## Verification

1. `cargo nextest run --lib --bins --workspace` — 796 tests pass.
2. `cargo install --path . && changeguard ask --help` — `--timeout <TIMEOUT> [default: 15]` visible.
3. Manual: `changeguard ask --timeout 1 "test" --backend local` against a non-responsive server exits within ~2s with a clear message.
4. `changeguard config verify --verbose` — `Ask` section shows all three timeout rows.
