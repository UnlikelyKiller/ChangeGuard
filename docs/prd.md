# ChangeGuard PRD

ChangeGuard is a local-first CLI that turns repository changes into explicit risk, verification, and Gemini-ready context.

## Primary user outcome

Given a repo with pending edits, a developer should be able to:

1. initialize local state under `.changeguard/`
2. inspect environment readiness with `doctor`
3. summarize current changes with `scan`
4. generate a structured impact packet with `impact`
5. run targeted verification with `verify`
6. ask Gemini for bounded assistance with `ask`

## Product constraints

- local-first by default
- deterministic outputs where practical
- Windows 11 + PowerShell first
- safe rebuildability of generated state
- explicit recovery path through `changeguard reset`
