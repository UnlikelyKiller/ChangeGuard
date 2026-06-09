# Track W7 Plan: CI/CD and Deployment Surface Ownership

- [x] Task W7.1: Build workflow and manifest fixtures for GitHub Actions, Dockerfile, Compose, Kubernetes, Terraform, and Helm.
- [x] Task W7.2: Write tests for job dependency parsing, trigger detection, required checks, and deploy target linking.
- [x] Task W7.3: Add deployment graph nodes and owner/environment overlays.
- [x] Task W7.4: Link CI jobs and deploy manifests to services, config keys, endpoints, dependencies, and observability signals.
- [x] Task W7.5: Add impact rules for release gate, strategy, base image, and required check changes.
- [x] Task W7.6: Implement `changeguard deploy impact --changed` and `changeguard ci diff`.
- [x] Task W7.7: Add docs for local-only versus optional live deployment metadata.
- [x] Task W7.8: Run deploy, CI, impact, and full verification gates; reinstall.

## Definition of Done Checklist

- [x] CI and deploy surfaces include owners and environments when available.
- [x] Removed or weakened release gates raise risk.
- [x] Manifest changes map to services and verification hints.
- [x] Full verification gate passes.
