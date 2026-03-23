# Record shrug CLI demo with VHS
# Run from project root: .\record-demo.ps1

$env:PATH = "$env:USERPROFILE\bin;$env:PATH"

# Check dependencies
$missing = @()
if (-not (Get-Command vhs -ErrorAction SilentlyContinue)) { $missing += "vhs" }
if (-not (Get-Command ttyd -ErrorAction SilentlyContinue)) { $missing += "ttyd" }
if (-not (Get-Command ffmpeg -ErrorAction SilentlyContinue)) { $missing += "ffmpeg" }
if (-not (Get-Command shrug -ErrorAction SilentlyContinue)) { $missing += "shrug" }

if ($missing.Count -gt 0) {
    Write-Host "Missing: $($missing -join ', ')" -ForegroundColor Red
    Write-Host "All should be in $env:USERPROFILE\bin or on PATH"
    exit 1
}

Write-Host "shrug version: $(shrug --version)" -ForegroundColor Cyan
Write-Host "vhs version: $(vhs --version)" -ForegroundColor Cyan
Write-Host ""
Write-Host "Recording demo..." -ForegroundColor Yellow
Write-Host ""

vhs demo.tape

if (Test-Path "assets\demo.gif") {
    $size = (Get-Item "assets\demo.gif").Length / 1MB
    Write-Host ""
    Write-Host "Done! assets\demo.gif ($([math]::Round($size, 1)) MB)" -ForegroundColor Green
} else {
    Write-Host "Failed - no demo.gif created" -ForegroundColor Red
    exit 1
}
