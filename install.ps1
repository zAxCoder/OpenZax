# OpenZax Installer for Windows
# Run: powershell -ExecutionPolicy Bypass -File install.ps1

Write-Host ""
Write-Host "  OPENZAX INSTALLER" -ForegroundColor Cyan
Write-Host "  =================" -ForegroundColor DarkGray
Write-Host ""

# Check for cargo
$cargoPath = "$env:USERPROFILE\.cargo\bin\cargo.exe"
if (-not (Test-Path $cargoPath)) {
    $cargoPath = (Get-Command cargo -ErrorAction SilentlyContinue).Source
}
if (-not $cargoPath -or -not (Test-Path $cargoPath)) {
    Write-Host "  [!] Rust/Cargo not found. Install from https://rustup.rs" -ForegroundColor Red
    exit 1
}
Write-Host "  [OK] Cargo found: $cargoPath" -ForegroundColor Green

# Build release binary
$projectDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Write-Host "  [..] Building openzax (release mode, this may take a few minutes)..." -ForegroundColor Yellow
Set-Location $projectDir
$buildOutput = & $cargoPath build -p openzax-cli --release 2>&1
$buildExit = $LASTEXITCODE
if ($buildExit -ne 0) {
    Write-Host "  [!] Build failed:" -ForegroundColor Red
    $buildOutput | ForEach-Object { Write-Host "       $_" -ForegroundColor DarkGray }
    exit 1
}
Write-Host "  [OK] Build successful" -ForegroundColor Green

# Copy to user bin directory
$installDir = "$env:USERPROFILE\.openzax\bin"
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

$src = Join-Path $projectDir "target\release\openzax.exe"
if (-not (Test-Path $src)) {
    $src = Join-Path $projectDir "target\fresh\release\openzax.exe"
}
if (-not (Test-Path $src)) {
    $src = Join-Path $projectDir "target\fresh\debug\openzax.exe"
    Write-Host "  [!] Release build not found, using debug build" -ForegroundColor Yellow
}
$dst = Join-Path $installDir "openzax.exe"
Copy-Item -Path $src -Destination $dst -Force
Write-Host "  [OK] Installed to $dst" -ForegroundColor Green

# Add to PATH if not already there
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$userPath;$installDir", "User")
    Write-Host "  [OK] Added $installDir to user PATH" -ForegroundColor Green
    Write-Host ""
    Write-Host "  IMPORTANT: Close and reopen your terminal for PATH changes to take effect." -ForegroundColor Yellow
} else {
    Write-Host "  [OK] Already in PATH" -ForegroundColor Green
}

# Create data directory
$dataDir = "$env:USERPROFILE\.openzax"
if (-not (Test-Path "$dataDir\skills")) { New-Item -ItemType Directory -Path "$dataDir\skills" -Force | Out-Null }
if (-not (Test-Path "$dataDir\models")) { New-Item -ItemType Directory -Path "$dataDir\models" -Force | Out-Null }

Write-Host ""
Write-Host "  Installation complete!" -ForegroundColor Green
Write-Host ""
Write-Host "  Usage:" -ForegroundColor Cyan
Write-Host "    openzax shell                          Start the TUI" -ForegroundColor White
Write-Host "    openzax shell --api-key sk-or-...      Start with API key" -ForegroundColor White
Write-Host "    openzax doctor                         Check system health" -ForegroundColor White
Write-Host "    openzax --help                         Show all commands" -ForegroundColor White
Write-Host ""
Write-Host "  Free API keys (no credit card):" -ForegroundColor Cyan
Write-Host "    OpenRouter:  https://openrouter.ai/keys" -ForegroundColor White
Write-Host "    Groq:        https://console.groq.com" -ForegroundColor White
Write-Host "    Cerebras:    https://cloud.cerebras.ai" -ForegroundColor White
Write-Host ""
Write-Host "  Set your key:" -ForegroundColor Cyan
Write-Host "    set OPENROUTER_API_KEY=sk-or-v1-..." -ForegroundColor White
Write-Host "    openzax shell" -ForegroundColor White
Write-Host ""
