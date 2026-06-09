# Track X14: `impact` and `scan` Surface Actionable Risk Summaries

**Status:** Planned  
**Milestone:** X — Command Surface Correctness  
**Priority:** Low

## Objective

`changeguard scan --impact` and `changeguard impact` produce detailed structured output, but on a clean tree (no staged changes) the output is either empty or shows a low-information "no changes detected" message. Additionally, the overall risk percentage sometimes conflicts with the individual risk items (e.g., "0 HIGH risks" but "Overall Risk: HIGH"). A summary reconciliation pass would improve confidence.

## Problem Statement

Two related issues:
1. On a clean working tree, `scan --impact` outputs nothing — no "tree is clean" confirmation.
2. The `Overall Risk` level is computed by a heuristic that can show `HIGH` even when no individual items are HIGH, due to count-based escalation logic in the risk scorer.

## Acceptance Criteria

1. On a clean working tree, `scan --impact` prints:
   ```
   Working tree is clean — no staged or modified files detected.
   Run 'git add <files>' before scanning for impact.
   ```
2. `Overall Risk` displayed matches the highest single-item risk level (or is "NONE" when items is 0).
3. Risk reconciliation: if `Overall Risk` would escalate beyond the highest line-item, a note `"(escalated due to N changed files)"` is appended.
4. `--json` output includes `"tree_clean": true/false`.

## Key Files

- `src/commands/impact.rs` — `execute_impact`
- `src/impact/packet.rs` — `ImpactPacket.overall_risk` computation
- `src/output/human.rs` — risk display

## Definition of Done

- `changeguard scan --impact` on clean tree shows the clean-tree message.
- `Overall Risk` level matches or is explained by escalation note.
- `--json` includes `tree_clean`.
- `cargo nextest run --lib --bins --workspace` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.
