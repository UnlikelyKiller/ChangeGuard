# Track I1-3: Log Verbosity Default Filter

**Milestone:** I — Issue Remediation  
**Phase:** 1 — Hotfixes  
**Issue:** CG-5  
**Status:** In Planning

## Objective

`index --analyze-graph`, `viz`, and `scan --impact` produce thousands of `INFO` lines from `graph_builder::graph::csr` (the graph-algo layer inside cozo-redux) before printing any meaningful output. The project already depends on `tracing-subscriber = { version = "0.3.20", features = ["fmt", "env-filter"] }`, so the fix is purely in `src/main.rs` initialization.

## Requirements

### Default Filter
Set a structured default `EnvFilter` that:
- Keeps `changeguard` crate output at `info` (all user-relevant messages preserved).
- Demotes `graph_builder`, `tantivy`, and `sled` to `warn` (eliminates the noisy internal logs).
- Falls back to the user's `RUST_LOG` environment variable when set.

Implementation:
```rust
let filter = tracing_subscriber::EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| {
        tracing_subscriber::EnvFilter::new(
            "info,graph_builder=warn,tantivy=warn,sled=warn"
        )
    });
```

### `--verbose` / `-v` Global Flag
- Add a global `--verbose` boolean flag to the top-level CLI (`src/main.rs` or the Clap root struct).
- When `--verbose` is set, ignore the default filter and use `EnvFilter::new("debug")` (or respect `RUST_LOG` at its full level).
- This allows developers to restore full `graph_builder::INFO` output when diagnosing KG issues without editing env vars.

## API Contract

No public API changes. The global `--verbose` flag appears in `changeguard --help`.

## Testing Strategy

- Testing tracing init in unit tests is impractical (global subscriber). Verify manually:
  - `changeguard index --analyze-graph` → output should be under 50 lines (was >2000).
  - `changeguard index --analyze-graph --verbose` → full graph_builder output restored.
- Optionally add a compile-time test asserting the `EnvFilter` string is valid:
  ```rust
  #[test]
  fn default_env_filter_parses() {
      tracing_subscriber::EnvFilter::new(
          "info,graph_builder=warn,tantivy=warn,sled=warn"
      ); // panics if invalid
  }
  ```

## Out of Scope

- No change to `tracing` call sites or log levels inside ChangeGuard's own code.
- `RUST_LOG` override is honored as-is; no documentation change needed.
