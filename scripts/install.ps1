# Install shrug — static CLI for Atlassian Cloud
# Usage: powershell -ExecutionPolicy ByPass -c "irm https://github.com/mfassaie/shrug/releases/latest/download/install.ps1 | iex"

$ErrorActionPreference = "Stop"

$Repo = "mfassaie/shrug"
$Target = "x86_64-pc-windows-msvc"
$InstallDir = if ($env:SHRUG_INSTALL_DIR) { $env:SHRUG_INSTALL_DIR } else { "$env:LOCALAPPDATA\shrug\bin" }

function Get-LatestVersion {
    $response = Invoke-WebRequest -Uri "https://github.com/$Repo/releases/latest" -MaximumRedirection 0 -ErrorAction SilentlyContinue -SkipHttpErrorCheck
    if ($response.Headers.Location) {
        $location = $response.Headers.Location
        if ($location -is [array]) { $location = $location[0] }
        return $location.Split("/")[-1]
    }
    # Fallback: use the API
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    return $release.tag_name
}

$Version = Get-LatestVersion
if (-not $Version) {
    Write-Error "Could not determine latest version"
    exit 1
}

Write-Host "Installing shrug $Version ($Target)..."

$Url = "https://github.com/$Repo/releases/download/$Version/shrug-$Target.zip"
$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("shrug-install-" + [System.Guid]::NewGuid().ToString("N").Substring(0, 8))
New-Item -ItemType Directory -Path $TempDir -Force | Out-Null

try {
    $ZipPath = Join-Path $TempDir "shrug.zip"
    Invoke-WebRequest -Uri $Url -OutFile $ZipPath

    Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force

    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    Copy-Item (Join-Path $TempDir "shrug.exe") (Join-Path $InstallDir "shrug.exe") -Force

    Write-Host "Installed shrug $Version to $InstallDir\shrug.exe"

    # Add to user PATH if not already present
    $UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($UserPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("PATH", "$InstallDir;$UserPath", "User")
        Write-Host ""
        Write-Host "Added $InstallDir to your user PATH."
        Write-Host "Restart your terminal for the change to take effect."
    }
}
finally {
    Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue
}
