<#
.SYNOPSIS
Installs GitHunter for the current Windows user.

.DESCRIPTION
Builds GitHunter from the main branch, copies githunter.exe to a user-owned
installation directory, and adds that directory to the persistent user PATH.
New PowerShell, Command Prompt, and Windows Terminal sessions can then run
`githunter` from any directory.
#>
[CmdletBinding()]
param(
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA "GitHunter\bin")
)

$ErrorActionPreference = "Stop"
$repositoryUrl = "https://github.com/SecurityTalent/GitHunter.git"
$temporaryDir = Join-Path ([System.IO.Path]::GetTempPath()) ("githunter-" + [guid]::NewGuid())
$locationPushed = $false

function Require-Command([string]$Name, [string]$InstallHint) {
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "$Name is required. $InstallHint"
    }
}

Require-Command "git" "Install Git for Windows, then run this script again."
Require-Command "cargo" "Install Rust from https://rustup.rs/, then run this script again."

try {
    Write-Host "Installing GitHunter globally for the current user..."
    New-Item -ItemType Directory -Path $temporaryDir | Out-Null

    Write-Host "Cloning GitHunter..."
    git clone --depth 1 $repositoryUrl $temporaryDir

    Push-Location $temporaryDir
    $locationPushed = $true
    Write-Host "Building release binary..."
    cargo build --release
    Pop-Location
    $locationPushed = $false

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Copy-Item (Join-Path $temporaryDir "target\release\githunter.exe") `
        (Join-Path $InstallDir "githunter.exe") -Force

    $separator = [System.IO.Path]::PathSeparator
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $normalizedInstallDir = $InstallDir.TrimEnd('\')
    $pathEntries = @($userPath -split [regex]::Escape($separator) | Where-Object {
        $_.TrimEnd('\') -ieq $normalizedInstallDir
    })
    if ($pathEntries.Count -eq 0) {
        $newUserPath = if ([string]::IsNullOrWhiteSpace($userPath)) {
            $InstallDir
        } else {
            "$userPath$separator$InstallDir"
        }
        [Environment]::SetEnvironmentVariable("Path", $newUserPath, "User")
        Write-Host "Added $InstallDir to your user PATH."
    }

    if (-not (($env:Path -split [regex]::Escape($separator)) | Where-Object {
        $_.TrimEnd('\') -ieq $normalizedInstallDir
    })) {
        $env:Path = "$InstallDir$separator$env:Path"
    }

    Write-Host ""
    Write-Host "GitHunter installed successfully."
    Write-Host "This terminal is ready now: githunter --help"
    Write-Host "Open a new terminal to use githunter from any directory."
}
finally {
    if ($locationPushed) {
        Pop-Location
    }
    if (Test-Path -LiteralPath $temporaryDir) {
        Remove-Item -LiteralPath $temporaryDir -Recurse -Force
    }
}
