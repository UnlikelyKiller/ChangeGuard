# Specification: Track 9 - Change Risk Analysis Engine

## Overview
Implement a heuristic-based risk scoring engine to evaluate the potential impact of repository changes. This engine will populate the `risk_level` and `risk_reasons` fields in the `ImpactPacket`.

## Data Models

### RiskLevel (Existing in src/impact/packet.rs)
- `Low`: Minimal risk (doc changes, minor internal logic).
- `Medium`: Standard risk (public symbol changes, multiple file edits).
- `High`: Significant risk (protected path hits, major public API shifts).

### RiskFactor (New Internal Model)
- `Source`: The origin of the risk (e.g., "ProtectedPath", "ChangeVolume", "PublicSymbol").
- `Weight`: Integer score (0-100).
- `Description`: Human-readable reason.

## Heuristics

1. **Protected Paths**:
    - Any hit on a protected path (e.g., `Cargo.toml`, `.github/*`) triggers an automatic `High` risk or adds significant weight.
    - Reason: "Protected path hit: {path}"

2. **Change Volume**:
    - Number of files > 5: +20 weight.
    - Total symbols changed > 20: +20 weight.
    - Reason: "High volume of changes ({count} files)"

3. **Symbol Visibility**:
    - Modification of a `public` symbol (Rust `pub`, TS `export`): +30 weight.
    - Deletion of a `public` symbol: +50 weight.
    - Reason: "Public symbol modified: {name}"

4. **Rule Overrides**:
    - Policy rules can explicitly set a risk level for certain paths.

## Scoring Thresholds
- `0-20`: Low
- `21-60`: Medium
- `>60`: High

## Components

### Analysis Engine (`src/impact/analysis.rs`)
- `pub fn analyze_risk(packet: &mut ImpactPacket, rules: &Rules) -> Result<()>`
- Aggregates heuristics and updates the packet.

### Policy Integration
- Update `Rules` model to support risk weights if necessary (YAGNI for now, stick to hardcoded defaults).

## Verification
- Unit tests for each heuristic.
- Integration tests in `tests/risk_analysis.rs` ensuring correct level assignment for complex scenarios.
