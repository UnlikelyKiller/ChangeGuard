# Track X5: Security Child Node Orphan Pruning

**Status:** Completed  
**Milestone:** X — Command Surface Correctness  
**Priority:** Medium

## Objective

When Cedar policy files are deleted, `principal`, `action`, and `resource` nodes linked to those policies persist in CozoDB as orphans. The W13 fix added policy-level pruning (pruning `category: 'policy'` nodes) but did not extend to policy-child node categories (`principal`, `action`, `resource`). These show up in `security boundaries` output as dangling entries.

## Problem Statement

After deleting `policies/test.cedar`, the graph_loader now correctly removes the `policy` node. However, nodes with `category: 'principal'`, `category: 'action'`, and `category: 'resource'` that were derived from `test.cedar` remain. The pruning logic checks the `id` field of `policy` nodes against current Cedar filenames, but does not cascade to child nodes whose IDs embed the policy filename.

Example orphan: `urn:changeguard:principal:C:\dev\ChangeGuard\policies\test.cedar:User`

## Acceptance Criteria

1. After `changeguard index --analyze-graph` when the `policies/` directory is absent or empty:
   - All `category: 'principal'` nodes are removed.
   - All `category: 'action'` nodes are removed.
   - All `category: 'resource'` nodes are removed.
2. When policies exist, child nodes for *deleted* policy files are pruned while child nodes for *existing* policy files are preserved.
3. The `security boundaries` command shows no orphaned entries after policy deletion.
4. Pruning is unconditional (runs before the `policy_dir.exists()` guard), matching the pattern from the W13 Cedar fix.

## API Contracts

CozoDB pruning scripts to run unconditionally:
```datalog
-- When valid_cedar_filenames is empty:
?[id] := *node{id, category: 'principal'} :rm node {id}
?[id] := *node{id, category: 'action'} :rm node {id}
?[id] := *node{id, category: 'resource'} :rm node {id}

-- When valid filenames exist:
?[id] := *node{id, category: 'principal'}, not valid_id(id) :rm node {id}
```

## Key Files

- `src/index/graph_loader.rs` — Section 9 cedar orphan pruning (extend to child categories)
- `src/commands/security.rs` — `security boundaries` (validation target)

## Definition of Done

- `changeguard index --analyze-graph` followed by `changeguard security boundaries` shows 0 entries when no Cedar policies exist.
- Existing policy entries are preserved when policies are valid.
- `cargo nextest run --lib --bins --workspace` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.
