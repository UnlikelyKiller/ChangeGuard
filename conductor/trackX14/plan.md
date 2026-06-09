# Track X14 Plan: Impact Clean-Tree Message + Risk Reconciliation

## Phase 1 — Red (Failing Tests)
- [x] 1. Write unit test `test_impact_clean_tree_message`: given `ImpactPacket` with 0 changed files, assert output contains "Working tree is clean".
- [x] 2. Write unit test `test_risk_reconciliation`: given packet with no HIGH items but `overall_risk = HIGH`, assert escalation note is present.

## Phase 2 — Implementation

### Clean tree detection
- [x] 3. In `execute_impact` or `execute_scan_impact`, after building the `ImpactPacket`, check `packet.changes.is_empty()`:
  ```rust
  if packet.changes.is_empty() {
      if !json {
          println!("  {}", "Working tree is clean — no staged or modified files detected.".yellow());
          println!("  Run {} before scanning for impact.", "'git add <files>'".cyan().bold());
      } else {
          println!("{}", serde_json::json!({"tree_clean": true, "changes": [], "overall_risk": "NONE"}));
      }
      return Ok(());
  }
  ```

### Risk reconciliation
- [x] 4. In `src/impact/packet.rs` or `src/output/human.rs`, after computing `overall_risk`:
  - Find the max risk level among individual items.
  - If `overall_risk > max_item_risk`, compute `escalated_by_count = true` and emit a note string.
- [x] 5. In the human output, append `"(escalated due to {} changed files)"` when escalation occurred.
- [x] 6. In JSON output, add `"tree_clean": false` to normal impact packets.

## Phase 3 — Green + Cleanup
- [x] 7. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [x] 8. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [x] 9. Run `cargo fmt --all -- --check` — clean.
- [x] 10. Update `conductor/conductor.md` status to Completed.
