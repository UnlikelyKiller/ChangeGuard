@ENV: POWERSHELL_WIN32
@FORBID: &&, [[, ]], then, fi, done, echo -e
@ENFORCE: ;, { }, $_, Get-*, Test-Path, Join-Path
@SCOPE: run_shell_command, run_command, terminal

1. **NO BASH-ISMS**: Never use Bash-specific syntax (if/then/fi, while/do/done) on this repository.
2. **NATIVE CMDLETS**: Always use native PowerShell cmdlets (e.g., `Get-ChildItem` instead of `ls`, `Test-Path` instead of `[ -f ]`).
3. **PATH SEPARATORS**: Use backslashes (`\`) for shell-level path operations to ensure compatibility with Windows-native tools.
4. **TOOL SELECTION**: This environment is Windows 11. If a tool is labeled 'Bash', it is likely a cross-platform shim—avoid it for complex logic and prefer native PowerShell execution tools.
5. **PIPELINE OBJECTS**: Use the PowerShell pipeline (`$_`) and object properties (`.FullName`, `.Lines`) for structured data processing.