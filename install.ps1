# OpenZax Installer for Windows
# Run directly:  powershell -ExecutionPolicy Bypass -File install.ps1
# Run via pipe:  irm https://raw.githubusercontent.com/zAxCoder/OpenZax/master/install.ps1 | iex

Write-Host ""
Write-Host "  OPENZAX INSTALLER" -ForegroundColor Cyan
Write-Host "  =================" -ForegroundColor DarkGray
Write-Host ""

# Get latest release info
Write-Host "  [..] Checking latest version..." -ForegroundColor Yellow
try {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/zAxCoder/OpenZax/releases/latest"
    $version = $release.tag_name
    $asset = $release.assets | Where-Object { $_.name -eq "openzax-windows-x86_64.zip" }
    
    if (-not $asset) {
        throw "Windows binary not found in release"
    }
    
    Write-Host "  [OK] Latest version: $version" -ForegroundColor Green
} catch {
    Write-Host "  [!] Failed to fetch release info. Falling back to build from source..." -ForegroundColor Yellow
    
    # Fallback to original build-from-source logic
    $cargoPath = "$env:USERPROFILE\.cargo\bin\cargo.exe"
    if (-not (Test-Path $cargoPath)) {
        $cargoPath = (Get-Command cargo -ErrorAction SilentlyContinue).Source
    }
    if (-not $cargoPath -or -not (Test-Path $cargoPath)) {
        Write-Host "  [!] Rust/Cargo not found. Install from https://rustup.rs" -ForegroundColor Red
        exit 1
    }
    Write-Host "  [OK] Cargo found: $cargoPath" -ForegroundColor Green

    $gitCmd = (Get-Command git -ErrorAction SilentlyContinue)
    if (-not $gitCmd) {
        Write-Host "  [!] Git not found. Please install git from https://git-scm.com" -ForegroundColor Red
        exit 1
    }
    
    $tempDir = Join-Path $env:TEMP "openzax-install-$(Get-Random)"
    Write-Host "  [..] Cloning OpenZax from GitHub..." -ForegroundColor Yellow
    git clone --depth 1 https://github.com/zAxCoder/OpenZax.git $tempDir 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  [!] Git clone failed. Check your internet connection." -ForegroundColor Red
        exit 1
    }
    Write-Host "  [OK] Source cloned" -ForegroundColor Green

    Write-Host "  [..] Building openzax (release mode, this may take a few minutes)..." -ForegroundColor Yellow
    Push-Location $tempDir
    $buildOutput = & $cargoPath build -p openzax-cli --release 2>&1
    $buildExit = $LASTEXITCODE
    Pop-Location

    if ($buildExit -ne 0) {
        Write-Host "  [!] Build failed:" -ForegroundColor Red
        $buildOutput | ForEach-Object { Write-Host "       $_" -ForegroundColor DarkGray }
        Remove-Item -Recurse -Force $tempDir -ErrorAction SilentlyContinue
        exit 1
    }
    Write-Host "  [OK] Build successful" -ForegroundColor Green

    $installDir = "$env:USERPROFILE\.openzax\bin"
    if (-not (Test-Path $installDir)) {
        New-Item -ItemType Directory -Path $installDir -Force | Out-Null
    }

    $src = Join-Path $tempDir "target\release\openzax.exe"
    $dst = Join-Path $installDir "openzax.exe"
    Copy-Item -Path $src -Destination $dst -Force
    Write-Host "  [OK] Installed to $dst" -ForegroundColor Green

    Remove-Item -Recurse -Force $tempDir -ErrorAction SilentlyContinue
    
    # Continue to PATH setup below
    $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($userPath -notlike "*$installDir*") {
        [Environment]::SetEnvironmentVariable("PATH", "$userPath;$installDir", "User")
        Write-Host "  [OK] Added $installDir to user PATH" -ForegroundColor Green
        Write-Host ""
        Write-Host "  IMPORTANT: Close and reopen your terminal for PATH changes to take effect." -ForegroundColor Yellow
    } else {
        Write-Host "  [OK] Already in PATH" -ForegroundColor Green
    }
    
    $dataDir = "$env:USERPROFILE\.openzax"
    if (-not (Test-Path "$dataDir\skills")) { New-Item -ItemType Directory -Path "$dataDir\skills" -Force | Out-Null }
    if (-not (Test-Path "$dataDir\models")) { New-Item -ItemType Directory -Path "$dataDir\models" -Force | Out-Null }

    Write-Host ""
    Write-Host "  Installation complete!" -ForegroundColor Green
    Write-Host ""
    exit 0
}

# Download binary from release
$installDir = "$env:USERPROFILE\.openzax\bin"
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

$tempZip = Join-Path $env:TEMP "openzax-windows.zip"
Write-Host "  [..] Downloading binary..." -ForegroundColor Yellow

try {
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $tempZip
    Write-Host "  [OK] Downloaded" -ForegroundColor Green
} catch {
    Write-Host "  [!] Download failed: $_" -ForegroundColor Red
    exit 1
}

# Extract binary
Write-Host "  [..] Extracting..." -ForegroundColor Yellow
try {
    Expand-Archive -Path $tempZip -DestinationPath $installDir -Force
    Remove-Item $tempZip -Force
    Write-Host "  [OK] Installed to $installDir\openzax.exe" -ForegroundColor Green
} catch {
    Write-Host "  [!] Extraction failed: $_" -ForegroundColor Red
    exit 1
}

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

# Create data directories
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
