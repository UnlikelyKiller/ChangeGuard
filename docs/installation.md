# Installing ChangeGuard

ChangeGuard is meant to be available as a normal CLI command, similar to `gh`.
Once installed, AI agents and developers can run:

```bash
changeguard doctor
changeguard scan
changeguard impact
changeguard verify
```

## One-Line Install

Windows PowerShell:

```powershell
iwr https://raw.githubusercontent.com/UnlikelyKiller/ChangeGuard/main/install/install.ps1 -UseB | iex
```

macOS or Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/UnlikelyKiller/ChangeGuard/main/install/install.sh | sh
```

The installer tries to download a prebuilt GitHub release binary first. If no release asset exists for the platform, it falls back to `cargo install --git`.

## Requirements

For release binaries:

- `git` should be installed for normal ChangeGuard operation.
- `gemini` is optional and only needed for `changeguard ask`.

For source fallback:

- Rust/Cargo must be installed from <https://rustup.rs>.

## Install Location

Windows default:

```text
%USERPROFILE%\.changeguard\bin
```

macOS/Linux default:

```text
~/.local/bin
```

The installer updates the user PATH when possible. Open a new terminal after installation if `changeguard` is not immediately found.

## Options

Windows:

```powershell
.\install\install.ps1 -BuildFromSource
.\install\install.ps1 -InstallDir "$HOME\.local"
.\install\install.ps1 -Daemon
.\install\install.ps1 -NoPathUpdate
```

macOS/Linux:

```bash
CHANGEGUARD_BUILD_FROM_SOURCE=1 ./install/install.sh
CHANGEGUARD_INSTALL_DIR="$HOME/.changeguard" ./install/install.sh
CHANGEGUARD_DAEMON=1 ./install/install.sh
CHANGEGUARD_NO_PATH_UPDATE=1 ./install/install.sh
```

## Agent Bootstrap

If an AI agent is asked to use ChangeGuard in a repository, it should:

1. Check availability:

   ```bash
   changeguard doctor
   ```

2. If unavailable and installation is allowed, run the platform installer above.
3. Re-run:

   ```bash
   changeguard doctor
   ```

4. Initialize the repository only when the user wants ChangeGuard state in that repo:

   ```bash
   changeguard init
   ```

5. Run the normal workflow:

   ```bash
   changeguard scan
   changeguard impact
   changeguard verify
   ```

## Release Assets

Tagged releases publish these binary assets:

- `changeguard-x86_64-pc-windows-msvc.zip`
- `changeguard-x86_64-unknown-linux-gnu.tar.gz`
- `changeguard-x86_64-apple-darwin.tar.gz`
- `changeguard-aarch64-apple-darwin.tar.gz`

Create a release by pushing a tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```
