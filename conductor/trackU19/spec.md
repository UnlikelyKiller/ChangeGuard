# Track U19 Spec: Data-Driven `config verify` Section Table

## Background

`changeguard config verify` in `src/commands/config.rs` reports resolved settings via three hand-wired `println!` calls:
- `format_backend_line` (line 119)
- `format_semantic_line` (line 128, added in U13)
- And a static `Verifying ChangeGuard configuration...` line

Each new section (U15 will add split-fields output, U16 will add the cap, K-tracks add storage, temporal risk, etc.) requires:
- A new `format_X_line` function
- A new `println!` call in the right place
- No way to reorder, group, or filter sections
- No machine-readable output

The function-per-section pattern doesn't scale. As ChangeGuard grows, this list will become 10+ lines of `println!` calls and matching functions.

## Objective

Replace the hand-wired `println!` calls in `execute_config_verify` with a data-driven section table. Each section is a `Section` value with:
- A name and ordering key
- A human-formatter (for terminal output via `comfy-table`)
- An optional JSON-formatter (for `--json` output)
- A condition (skip the section entirely if false, e.g. when the relevant config is `Default`)

The new structure also enables `--section=<name>` filtering (already supported on `config view`, but `config verify` doesn't have it) and a `--verbose` flag that shows the *raw* defaults vs. resolved values.

## Proposed Design

### Section trait

```rust
pub trait ConfigSection {
    fn name(&self) -> &'static str;
    fn order(&self) -> u8;  // sort key
    fn is_applicable(&self, config: &Config) -> bool { true }  // default
    fn render_rows(&self, config: &Config) -> Vec<ConfigRow>;
}

#[derive(Serialize)]
pub struct ConfigRow {
    pub label: String,
    pub value: String,
    pub source: ValueSource,  // Explicit, Default, Auto, Inherited
}

#[derive(Serialize)]
pub enum ValueSource {
    Explicit,   // user set in TOML
    Default,    // field default
    Auto,       // auto-derived
    Inherited,  // from a different section (legacy compat)
}
```

### Section registry

In `src/commands/config_verify.rs` (new module):

```rust
#[derive(Serialize)]
pub struct SectionReport {
    pub section: String,
    pub rows: Vec<ConfigRow>,
}

pub fn all_sections() -> Vec<Box<dyn ConfigSection>> {
    vec![
        Box::new(BackendSection),
        Box::new(SemanticSection),
        // future: Box::new(StorageSection), Box::new(TemporalSection), ...
    ]
}

pub fn render_verify_report(config: &Config, json: bool) -> String {
    let sections: Vec<_> = all_sections()
        .into_iter()
        .filter(|s| s.is_applicable(config))
        .collect();

    let reports: Vec<SectionReport> = sections.iter().map(|s| SectionReport {
        section: s.name().to_string(),
        rows: s.render_rows(config),
    }).collect();

    if json {
        serde_json::to_string_pretty(&reports).unwrap_or_default()
    } else {
        let mut table = Table::new();
        table.set_header(vec!["Section", "Key", "Value", "Source"]);
        for report in &reports {
            for row in &report.rows {
                table.add_row([report.section.as_str(), row.label.as_str(), row.value.as_str(), row.source.to_string().as_str()]);
            }
        }
        table.to_string()
    }
}
```

### Migrate existing sections

- `BackendSection` wraps the current `format_backend_line_with` logic and exposes it as `ConfigRow`s
- `SemanticSection` wraps `format_semantic_line` and the U15/U16 split-field output

### Add `--json` and `--section=<name>` flags

- `--json`: serializes the section table as a structured array
- `--section=<name>`: filters to a single section
- `--verbose`: includes "Default" rows that would normally be hidden (e.g. "hnsw_rebuild_threshold = 500 (default)")

### Source tracking

The `ValueSource` enum makes it possible to surface *why* a value is what it is. The current code conflates "you set 4" with "the default is 4" with "auto-derived to 4" — three different things, currently all displayed identically.

## Critical files

| File | Change |
|---|---|
| `src/commands/config_verify.rs` (new) | `ConfigSection` trait, `ConfigRow`, `ValueSource`, `all_sections()`, `render_verify_report()` |
| `src/commands/config.rs` | `execute_config_verify` uses the new renderer; remove hand-wired `println!` calls and the `format_backend_line` / `format_semantic_line` private functions (or convert them to `ConfigSection` impls) |
| `src/cli.rs` | Add `--json` and `--section` flags to `config verify` subcommand |
| `src/commands/mod.rs` | Add `pub mod config_verify;` |

## Existing utilities to reuse

- `comfy_table` 7.2.2 (already in `Cargo.lock`)
- `serde_json` 1.0 (already in `Cargo.lock`)
- The current `format_backend_line_with` / `format_semantic_line` functions — wrap them in `ConfigSection` impls, don't rewrite from scratch
- `format_backend_line_with`'s env/dotenv reader pattern — keep it as a function on the `BackendSection` impl

## TDD plan (Red → Green)

1. `src/commands/config_verify.rs`:
   - `sections_returns_all_implementations`
   - `is_applicable_filters_correctly`
   - `render_human_includes_explicit_value_with_source`
   - `render_json_serializes_section_array`
   - `value_source_distinguishes_explicit_from_default`
2. `src/commands/config.rs`: refactor `execute_config_verify` to use the new module; keep behavior identical for the default case
3. `src/cli.rs`: add flags; ensure backward-compat (default = human, no section filter)

## Verification

1. CI gate.
2. Manual: `changeguard config verify` output is byte-identical to the U14 baseline for the default case.
3. Manual: `changeguard config verify --json` produces a parseable JSON document.
4. Manual: `changeguard config verify --section=semantic` shows only the semantic rows.
5. Add a new section in a one-line change to prove the pattern scales.

## Why this scope

The current `println!` chain is already showing its age. U15/U16 will add 2-3 more sections (split fields, cap, dry-run). K-tracks (storage, temporal risk, etc.) will add more. Doing the refactor *now* — when there's only 2 sections — is the cheapest time. Doing it later, when there are 10+ sections, is 3x the work.

## Out of scope

- Plugin system for third-party sections (would need a `inventory` or `linkme` based registry)
- Live-updating the table as config changes (this is a one-shot command, not a daemon)

## References

- comfy-table: https://github.com/Nukesor/comfy-table
- Current `format_backend_line_with` at `src/commands/config.rs:125`
- U13 `format_semantic_line` at `src/commands/config.rs:128`
- Tracing best practices (background): stderr for human, stdout for machine contract
