# OpenZax Windows Installer
# Usage (one-line install): powershell -ExecutionPolicy Bypass -c "irm https://raw.githubusercontent.com/zAxCoder/OpenZax/master/install.ps1 | iex"

$REPO = "zAxCoder/OpenZax"
$INSTALL_DIR = "$env:USERPROFILE\.openzax\bin"
$dst = "$INSTALL_DIR\openzax.exe"

Write-Host ""
Write-Host "  OPENZAX INSTALLER" -ForegroundColor White
Write-Host "  =================" -ForegroundColor DarkGray
Write-Host ""

New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null

$downloaded = $false

# Step 1: Try to download prebuilt binary from GitHub Releases
try {
    Write-Host "  [..] Fetching latest release info..." -ForegroundColor Yellow
    $headers = @{ "User-Agent" = "openzax-installer" }
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest" -Headers $headers -UseBasicParsing -ErrorAction Stop
    $tag = $release.tag_name

    $asset = $release.assets | Where-Object {
        $_.name -match "windows" -or $_.name -match "x86_64-pc-windows"
    } | Select-Object -First 1

    if ($asset) {
        Write-Host "  [..] Downloading openzax $tag..." -ForegroundColor Yellow
        Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $dst -UseBasicParsing -ErrorAction Stop
        Write-Host "  [OK] Downloaded $tag" -ForegroundColor Green
        $downloaded = $true
    } else {
        Write-Host "  [..] No prebuilt binary in release, will build from source." -ForegroundColor Yellow
    }
} catch {
    Write-Host "  [..] GitHub Releases not available, building from source..." -ForegroundColor Yellow
}

# Step 2: Build from source if download failed
if (-not $downloaded) {
    # Find cargo
    $cargoPath = (Get-Command cargo -ErrorAction SilentlyContinue).Source
    if (-not $cargoPath) { $cargoPath = "$env:USERPROFILE\.cargo\bin\cargo.exe" }
    if (-not (Test-Path $cargoPath)) {
        Write-Host "  [!] Rust/Cargo not found." -ForegroundColor Red
        Write-Host "  Install Rust first: https://rustup.rs" -ForegroundColor Yellow
        Write-Host "  Then re-run this installer." -ForegroundColor Yellow
        exit 1
    }
    Write-Host "  [OK] Cargo found." -ForegroundColor Green

    # Check if we're running from project directory
    $projectDir = $null
    $scriptPath = $MyInvocation.MyCommand.Path
    if ($scriptPath -and (Test-Path "$scriptPath")) {
        $projectDir = Split-Path -Parent $scriptPath
    }

    # If not in project dir (e.g., running via irm|iex), clone the repo
    if (-not $projectDir -or -not (Test-Path "$projectDir\Cargo.toml")) {
        Write-Host "  [..] Cloning repository..." -ForegroundColor Yellow
        $tempDir = "$env:TEMP\openzax-build-$(Get-Random)"
        $gitResult = git clone "https://github.com/$REPO.git" $tempDir 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Host "  [!] Failed to clone repository." -ForegroundColor Red
            Write-Host "  Make sure the repo is public: https://github.com/$REPO" -ForegroundColor Yellow
            exit 1
        }
        $projectDir = $tempDir
        $cleanup = $true
    } else {
        $cleanup = $false
    }

    Write-Host "  [..] Building openzax (this takes 2-5 minutes)..." -ForegroundColor Yellow
    Push-Location $projectDir
    $buildOutput = & $cargoPath build -p openzax-cli --release 2>&1
    $buildExit = $LASTEXITCODE
    Pop-Location

    if ($buildExit -ne 0) {
        Write-Host "  [!] Build failed:" -ForegroundColor Red
        $buildOutput | Select-Object -Last 10 | ForEach-Object { Write-Host "    $_" -ForegroundColor DarkGray }
        if ($cleanup) { Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue }
        exit 1
    }

    $src = "$projectDir\target\release\openzax.exe"
    if (-not (Test-Path $src)) {
        Write-Host "  [!] Built binary not found at $src" -ForegroundColor Red
        exit 1
    }

    Copy-Item -Path $src -Destination $dst -Force
    if ($cleanup) { Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue }
    Write-Host "  [OK] Build successful" -ForegroundColor Green
}

Write-Host "  [OK] Installed to $dst" -ForegroundColor Green

# Add to PATH
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$INSTALL_DIR*") {
    [Environment]::SetEnvironmentVariable("PATH", "$userPath;$INSTALL_DIR", "User")
    Write-Host "  [OK] Added to PATH" -ForegroundColor Green
    Write-Host ""
    Write-Host "  IMPORTANT: Close and reopen your terminal for PATH to take effect." -ForegroundColor Yellow
} else {
    Write-Host "  [OK] Already in PATH" -ForegroundColor Green
}

# Create data directories
$dataDir = "$env:USERPROFILE\.openzax"
@("bin", "skills", "models") | ForEach-Object {
    New-Item -ItemType Directory -Path "$dataDir\$_" -Force | Out-Null
}

Write-Host ""
Write-Host "  Installation complete!" -ForegroundColor Green
Write-Host ""
Write-Host "  Run: openzax" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Optional: Set your own API key (free):" -ForegroundColor White
Write-Host '  [System.Environment]::SetEnvironmentVariable("OPENROUTER_API_KEY","sk-or-...","User")' -ForegroundColor DarkGray
Write-Host "  Get key: https://openrouter.ai/keys" -ForegroundColor DarkGray
Write-Host ""
