# Track 5: Basic Impact Packet Shell - Technical Specification

## Overview

This track implements Phase 6 of the Changeguard implementation plan. The goal is to create the initial foundational structure for the "Impact Packet"—a JSON artifact containing repository metadata, file changes, and a provisional risk assessment. This establishes the end-to-end flow from git scan through report generation before any complex language-specific indexing is added.

## 1. Directory Structure Additions
- `src/impact/mod.rs`: Defines the `impact` module.
- `src/impact/packet.rs`: Defines the `ImpactPacket` schema and `RiskLevel` enum.
- `src/state/reports.rs`: Handles writing generated reports to disk.
- `src/commands/impact.rs`: The CLI command runner for `changeguard impact`.
- `tests/cli_impact.rs`: Integration tests for the `impact` command.

## 2. Packet Schema Requirements (`src/impact/packet.rs`)
The structure of the `ImpactPacket` needs to be stable and predictable as it will be used to pass context to Gemini later.

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangedFile {
    pub path: PathBuf,
    pub status: String, // e.g., "Added", "Modified", "Deleted", "Renamed"
    pub is_staged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactPacket {
    pub schema_version: String,
    pub timestamp_utc: String, // ISO 8601 string
    pub head_hash: Option<String>,
    pub branch_name: Option<String>,
    pub risk_level: RiskLevel,
    pub risk_reasons: Vec<String>,
    pub changes: Vec<ChangedFile>,
}

impl Default for ImpactPacket {
    fn default() -> Self {
        Self {
            schema_version: "v1".to_string(),
            timestamp_utc: "".to_string(),
            head_hash: None,
            branch_name: None,
            risk_level: RiskLevel::Medium,
            risk_reasons: vec!["Provisional baseline risk".to_string()],
            changes: Vec::new(),
        }
    }
}
```

## 3. Persistence Requirements (`src/state/reports.rs`)
Reports must be persisted to the `.changeguard/reports/` directory.

- Provide a function `pub fn write_impact_report(layout: &Layout, packet: &ImpactPacket) -> Result<()>`
- Use `serde_json::to_string_pretty` for inspectable output.
- Write to `.changeguard/reports/latest-impact.json`.
- Fail cleanly using `miette` errors if IO fails (e.g., lack of permissions).

## 4. Command Requirements (`src/commands/impact.rs`)
The `execute_impact()` function must orchestrate the flow:
1. Discover current git repo.
2. Generate `RepoSnapshot` using existing `git` module functions.
3. Map `RepoSnapshot` to `ImpactPacket`:
   - Copy `head_hash` and `branch_name`.
   - Map `FileChange` structs to `ChangedFile` structs in the packet.
   - For now, leave risk as `Medium` unless the repo is completely clean (in which case it can be `Low`).
4. Initialize the state `Layout` to write the report via `reports.rs`.
5. Print a user-friendly console summary (e.g., "Wrote impact report to .changeguard/reports/latest-impact.json").

## 5. Error Handling
All errors should bubble up through `miette::Result`.
Add specific variants to `CommandError` or create an `ImpactError` for serialization or report writing failures.

## 6. Testing Strategy
- **Unit Tests:** Serialization golden snapshot tests in `src/impact/packet.rs`.
- **Integration Tests:** `tests/cli_impact.rs` should mock a git repo with a few changed files, invoke `Commands::Impact` (or binary via `assert_cmd`), and assert that `latest-impact.json` is generated with the expected schema version and files. Use `cargo test -j 1`.
