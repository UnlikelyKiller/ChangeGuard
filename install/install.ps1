param(
    [string]$Repo = "UnlikelyKiller/ChangeGuard",
    [string]$Version = "latest",
    [string]$InstallDir = "$HOME\.changeguard",
    [switch]$NoPathUpdate,
    [switch]$BuildFromSource,
    [switch]$Daemon
)

$ErrorActionPreference = "Stop"

function Write-Step($Message) {
    Write-Host "==> $Message"
}

function Test-Command($Name) {
    $null -ne (Get-Command $Name -ErrorAction SilentlyContinue)
}

function Add-UserPath($PathToAdd) {
    $current = [Environment]::GetEnvironmentVariable("Path", "User")
    $parts = @()
    if ($current) {
        $parts = $current -split ';' | Where-Object { $_ }
    }

    if ($parts -notcontains $PathToAdd) {
        $newPath = if ($current) { "$current;$PathToAdd" } else { $PathToAdd }
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        $env:Path = "$env:Path;$PathToAdd"
        Write-Step "Added $PathToAdd to the user PATH. Open a new terminal for other sessions."
    }
}

function Install-FromRelease {
    $binDir = Join-Path $InstallDir "bin"
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null

    $asset = "changeguard-x86_64-pc-windows-msvc.zip"
    $tagPath = if ($Version -eq "latest") { "latest/download" } else { "download/$Version" }
    $url = "https://github.com/$Repo/releases/$tagPath/$asset"
    $tmp = Join-Path ([System.IO.Path]::GetTempPath()) "changeguard-$([System.Guid]::NewGuid()).zip"
    $extractDir = Join-Path ([System.IO.Path]::GetTempPath()) "changeguard-$([System.Guid]::NewGuid())"

    Write-Step "Downloading $url"
    Invoke-WebRequest -Uri $url -OutFile $tmp -UseBasicParsing
    Expand-Archive -Path $tmp -DestinationPath $extractDir -Force

    $exe = Get-ChildItem -Path $extractDir -Recurse -Filter "changeguard.exe" | Select-Object -First 1
    if (-not $exe) {
        throw "Release archive did not contain changeguard.exe"
    }

    Copy-Item -Path $exe.FullName -Destination (Join-Path $binDir "changeguard.exe") -Force
    Remove-Item -Path $tmp -Force -ErrorAction SilentlyContinue
    Remove-Item -Path $extractDir -Recurse -Force -ErrorAction SilentlyContinue
}

function Install-FromCargo {
    if (-not (Test-Command "cargo")) {
        throw "Rust cargo was not found. Install Rust from https://rustup.rs or publish a ChangeGuard release asset, then rerun this installer."
    }

    $features = @()
    if ($Daemon) {
        $features = @("--features", "daemon")
    }

    if ((Test-Path "Cargo.toml") -and ((Get-Content "Cargo.toml" -Raw) -match 'name\s*=\s*"changeguard"')) {
        Write-Step "Installing ChangeGuard from the current checkout"
        cargo install --path . --locked --root $InstallDir @features
    } else {
        Write-Step "Installing ChangeGuard from https://github.com/$Repo"
        cargo install --git "https://github.com/$Repo" --branch main --locked --root $InstallDir @features
    }
}

$binDir = Join-Path $InstallDir "bin"
New-Item -ItemType Directory -Force -Path $binDir | Out-Null

if ($BuildFromSource) {
    Install-FromCargo
} else {
    try {
        Install-FromRelease
    } catch {
        Write-Step "Release install failed: $($_.Exception.Message)"
        Write-Step "Falling back to cargo install"
        Install-FromCargo
    }
}

if (-not $NoPathUpdate) {
    Add-UserPath $binDir
}

$changeguard = Join-Path $binDir "changeguard.exe"
if (-not (Test-Path $changeguard)) {
    $changeguard = "changeguard"
}

Write-Step "Verifying installation"
& $changeguard --help | Select-Object -First 5

Write-Host ""
Write-Host "ChangeGuard installed. AI agents can now run: changeguard doctor"
