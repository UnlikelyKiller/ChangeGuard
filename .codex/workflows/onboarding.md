# Project Onboarding Workflow

This workflow synchronizes the local context with the AI-Brains memory vault and ChangeGuard safety signals.

// turbo-all
1. **Initialize AI-Brains Context**
   ```powershell
   ai-brains safety sync
   ai-brains preflight --max-words 1000
   ```

2. **Verify Tooling**
   ```powershell
   changeguard ledger status --compact
   ```

3. **Establish Bearings**
   - Identify the active track in `conductor/conductor.md`.
   - Check `changeguard doctor` if needed.
