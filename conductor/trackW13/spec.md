# Track W13 Spec: Security Boundary, Authz, and Policy Graph

## Background

Security boundary tracking currently scores 7/10. ChangeGuard has protected paths, redaction, signing, process policy, and commit validators, but endpoint authz, permissions, roles, scopes, policy files, and security review requirements are not first-class graph concepts.

## Objective

Raise security boundary tracking to 9/10 by modeling auth, authorization, policy, roles, scopes, protected resources, secrets, external process execution, and security review ownership.

## Proposed Design

1. Add auth/authz graph nodes for middleware, route requirements, roles, scopes, policies, secret dependencies, external process boundaries, and protected resources.
2. Parse common auth patterns, IAM/policy files, and framework-specific permission annotations where feasible.
3. Link security boundaries to endpoints, services, config keys, deploy manifests, ADRs, tests, dependencies, and ledger transactions.
4. Add risk rules for auth bypass, policy broadening, secret exposure, protected path edits, unreviewed external process execution, and missing security review.
5. Add `changeguard security impact --changed` and `changeguard security boundaries`.

## Critical Files

| File | Expected work |
|---|---|
| `src/impact/redact.rs` | Keep redaction guarantees intact |
| `src/platform/process_policy.rs` | Link process policy to security graph |
| `src/policy/protected_paths.rs` | Link protected paths to owners and review rules |
| `src/index/` | Add authz and policy extraction hooks |
| `src/impact/enrichment/` | Add security boundary risk provider |
| `src/commands/` and `src/cli.rs` | Add security review commands |

## Definition of Done

- Security boundaries are queryable by endpoint, service, policy, role, scope, secret, protected path, and external process surface where known.
- Security-related impact output distinguishes inference, configured facts, and unknowns.
- Secret values are never emitted in human, JSON, prompt, bridge, or ledger output.
- Target score after completion: 9/10.
