# Track I1-3 Plan: Log Verbosity Default Filter

## Phase 1 — Red (Failing Tests)

- [ ] Add unit test `default_env_filter_parses` (in `src/main.rs` or a dedicated `tests/log_filter.rs`):
  ```rust
  #[test]
  fn default_env_filter_parses() {
      tracing_subscriber::EnvFilter::new("info,graph_builder=warn,tantivy=warn,sled=warn");
  }
  ```
- [ ] Add test asserting the Clap CLI struct accepts `--verbose` without panicking (use `try_parse_from`).
- [ ] Commit: `test(log): red — default env filter string is valid and --verbose flag is accepted`

## Phase 2 — Green (Implementation)

- [ ] In the top-level Clap struct (wherever `--help` is defined), add:
  ```rust
  #[arg(long, short = 'v', global = true)]
  verbose: bool,
  ```
- [ ] In `src/main.rs` tracing init, replace the existing subscriber setup with:
  ```rust
  let filter = if args.verbose {
      tracing_subscriber::EnvFilter::new("debug")
  } else {
      tracing_subscriber::EnvFilter::try_from_default_env()
          .unwrap_or_else(|_| {
              tracing_subscriber::EnvFilter::new(
                  "info,graph_builder=warn,tantivy=warn,sled=warn"
              )
          })
  };
  tracing_subscriber::fmt().with_env_filter(filter).init();
  ```
- [ ] Run CI gate: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`.
- [ ] Commit: `fix(log): default filter silences graph_builder/tantivy/sled; add --verbose flag (CG-5)`

## Verification

- [ ] `changeguard index --analyze-graph` → ≤50 lines of output.
- [ ] `changeguard index --analyze-graph --verbose` → full graph_builder output visible.
- [ ] `RUST_LOG=debug changeguard doctor` → full debug output (env var respected).
