# Track W13 Spec: Security Boundary, Authz, and Policy Graph

## Background

Security boundary tracking currently scores 7/10. ChangeGuard has protected paths, redaction, signing, process policy, and commit validators, but endpoint authz, permissions, roles, scopes, policy files, and security review requirements are not first-class graph concepts.

## Objective

Raise security boundary tracking to 9/10 by modeling auth, authorization, Cedar policies, roles, scopes, protected resources, secrets, external process execution, and security review ownership.

## Proposed Design

1. Add auth/authz graph nodes for middleware, route requirements, roles, scopes, Cedar policy statements, principal constraints, action constraints, resource constraints, secret dependencies, external process boundaries, and protected resources.
2. Make Cedar policy parsing the primary structured authorization ingestion path using the public `cedar-policy` PST where available.
3. Parse common IAM/policy files, framework middleware, and permission annotations as secondary best-effort evidence for repos that do not use Cedar.
4. Link security boundaries to endpoints, services, config keys, deploy manifests, ADRs, tests, dependencies, and ledger transactions.
5. Add risk rules for auth bypass, policy broadening, Cedar permit/forbid scope changes, secret exposure, protected path edits, unreviewed external process execution, and missing security review.
6. Add `changeguard security impact --changed` and `changeguard security boundaries`.

## Critical Files

| File | Expected work |
|---|---|
| `src/impact/redact.rs` | Keep redaction guarantees intact |
| `src/platform/process_policy.rs` | Link process policy to security graph |
| `src/policy/protected_paths.rs` | Link protected paths to owners and review rules |
| Cedar policy importer | Parse Cedar policies into principal/action/resource graph edges |
| `src/index/` | Add authz and policy extraction hooks |
| `src/impact/enrichment/` | Add security boundary risk provider |
| `src/commands/` and `src/cli.rs` | Add security review commands |

## Definition of Done

- Security boundaries are queryable by endpoint, service, policy, role, scope, secret, protected path, and external process surface where known.
- Cedar policies are parsed through the public PST into principal/action/resource edges, including conservative handling for wildcard and non-exhaustive expression cases.
- Non-Cedar auth extraction remains best-effort and clearly labeled as inferred or configured evidence.
- Security-related impact output distinguishes inference, configured facts, and unknowns.
- Secret values are never emitted in human, JSON, prompt, bridge, or ledger output.
- Target score after completion: 9/10.
