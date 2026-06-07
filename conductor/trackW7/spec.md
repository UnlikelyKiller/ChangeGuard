# Track W7 Spec: CI/CD and Deployment Surface Ownership

## Background

CI/CD and deployment surface tracking currently scores 7/10. ChangeGuard detects workflow/config changes and deployment manifests, but environment metadata, release gates, rollback plans, deployed artifact identity, and ownership are shallow.

## Objective

Raise CI/CD and deployment tracking to 9/10 by modeling workflows, jobs, environments, artifacts, release gates, deploy manifests, owners, secrets, and service links.

## Proposed Design

1. Add deployment graph nodes for environment, artifact, service, manifest, workflow, job, secret, release gate, rollback plan, and owner.
2. Parse CI workflows into jobs, dependencies, triggers, required checks, artifacts, and deployment targets.
3. Link deploy manifests to services, runtime config, endpoints, dependencies, and observability signals.
4. Add risk rules for changed release gates, removed tests/checks, changed deployment strategy, changed base images, and unowned infrastructure.
5. Add `changeguard deploy impact --changed` and `changeguard ci diff`.

## Critical Files

| File | Expected work |
|---|---|
| `src/coverage/deploy.rs` | Extend manifest classification and service links |
| `src/impact/enrichment/deploy.rs` | Add environment and ownership risk |
| `src/index/ci_gates.rs` | Parse workflow jobs, checks, and triggers |
| `src/commands/` and `src/cli.rs` | Add CI/deploy review commands |
| `src/config/model.rs` | Add optional deploy ownership overlays |

## Definition of Done

- CI jobs and deploy manifests can be linked to services, environments, release gates, and owners.
- Risk output identifies which gate, workflow, environment, or service is affected.
- Local analysis remains useful without live deployment API access.
- Target score after completion: 9/10.
