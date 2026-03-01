# OpenZax Installer for Windows
# Usage: iwr -useb https://openzax.dev/install.ps1 | iex

$ErrorActionPreference = "Stop"

$REPO = "zAxCoder/OpenZax"
$BINARY_NAME = "openzax"
$INSTALL_DIR = "$env:USERPROFILE\.openzax\bin"
$GITHUB_API = "https://api.github.com/repos/$REPO/releases/latest"

# Colors
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

function Write-Success { Write-ColorOutput Green $args }
function Write-Info { Write-ColorOutput Cyan $args }
function Write-Warning { Write-ColorOutput Yellow $args }
function Write-Error { Write-ColorOutput Red $args }

# Print banner
function Print-Banner {
    Write-Info ""
    Write-Info "╔════════════════════════════════════════════════════════╗"
    Write-Info "║                                                        ║"
    Write-Info "║  OpenZax Installer                                     ║"
    Write-Info "║  Secure AI Development Assistant                       ║"
    Write-Info "║                                                        ║"
    Write-Info "╚════════════════════════════════════════════════════════╝"
    Write-Info ""
}

# Check if Rust is installed
function Test-Rust {
    try {
        $null = Get-Command cargo -ErrorAction Stop
        Write-Success "✓ Rust is installed"
        return $true
    }
    catch {
        Write-Warning "⚠ Rust is not installed"
        return $false
    }
}

# Install Rust
function Install-Rust {
    Write-Info "Installing Rust..."
    
    $rustupUrl = "https://win.rustup.rs/x86_64"
    $rustupPath = "$env:TEMP\rustup-init.exe"
    
    Write-Info "Downloading rustup-init.exe..."
    Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath
    
    Write-Info "Running Rust installer..."
    Start-Process -FilePath $rustupPath -ArgumentList "-y" -Wait -NoNewWindow
    
    # Refresh PATH
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
    
    Write-Success "✓ Rust installed successfully"
}

# Install from source
function Install-FromSource {
    Write-Info "Installing OpenZax from source..."
    
    # Create temp directory
    $tempDir = New-Item -ItemType Directory -Path "$env:TEMP\openzax-install-$(Get-Random)"
    Push-Location $tempDir
    
    try {
        # Clone repository
        Write-Info "Cloning repository..."
        git clone "https://github.com/$REPO.git" openzax
        Set-Location openzax
        
        # Build release
        Write-Info "Building OpenZax (this may take a few minutes)..."
        cargo build --release --manifest-path crates/cli/Cargo.toml
        
        # Create install directory
        New-Item -ItemType Directory -Force -Path $INSTALL_DIR | Out-Null
        
        # Copy binary
        Copy-Item "target\release\$BINARY_NAME.exe" "$INSTALL_DIR\" -Force
        
        Write-Success "✓ OpenZax installed to $INSTALL_DIR\$BINARY_NAME.exe"
    }
    finally {
        # Cleanup
        Pop-Location
        Remove-Item -Recurse -Force $tempDir -ErrorAction SilentlyContinue
    }
}

# Download pre-built binary
function Install-Binary {
    Write-Info "Checking for pre-built binaries..."
    
    try {
        # Get latest release info
        $releaseInfo = Invoke-RestMethod -Uri $GITHUB_API
        $downloadUrl = $releaseInfo.assets | Where-Object { $_.name -like "*windows*x86_64*" } | Select-Object -First 1 -ExpandProperty browser_download_url
        
        if (-not $downloadUrl) {
            Write-Warning "⚠ No pre-built binary found for Windows x86_64"
            return $false
        }
        
        Write-Info "Downloading OpenZax..."
        New-Item -ItemType Directory -Force -Path $INSTALL_DIR | Out-Null
        
        $outputPath = "$INSTALL_DIR\$BINARY_NAME.exe"
        Invoke-WebRequest -Uri $downloadUrl -OutFile $outputPath
        
        Write-Success "✓ OpenZax downloaded successfully"
        return $true
    }
    catch {
        Write-Warning "⚠ Failed to download binary: $_"
        return $false
    }
}

# Add to PATH
function Add-ToPath {
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    
    if ($currentPath -like "*$INSTALL_DIR*") {
        Write-Success "✓ $INSTALL_DIR is already in PATH"
        return
    }
    
    $newPath = "$currentPath;$INSTALL_DIR"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    
    # Update current session
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
    
    Write-Success "✓ Added $INSTALL_DIR to PATH"
}

# Create config directory
function Setup-Config {
    $configDir = "$env:USERPROFILE\.openzax"
    New-Item -ItemType Directory -Force -Path "$configDir\skills" | Out-Null
    New-Item -ItemType Directory -Force -Path "$configDir\models" | Out-Null
    
    Write-Success "✓ Created config directory at $configDir"
}

# Print success message
function Print-Success {
    Write-Host ""
    Write-Success "╔════════════════════════════════════════════════════════╗"
    Write-Success "║                                                        ║"
    Write-Success "║  OpenZax installed successfully!                       ║"
    Write-Success "║                                                        ║"
    Write-Success "╚════════════════════════════════════════════════════════╝"
    Write-Host ""
    Write-Info "Quick Start:"
    Write-Host "  1. Restart your terminal"
    Write-Host "  2. Run: " -NoNewline
    Write-Info "openzax"
    Write-Host "  3. Or try: " -NoNewline
    Write-Info "openzax --help"
    Write-Host ""
    Write-Host "Documentation: " -NoNewline
    Write-Info "https://github.com/$REPO"
    Write-Host ""
}

# Main installation flow
function Main {
    Print-Banner
    
    # Check for Rust
    if (-not (Test-Rust)) {
        $response = Read-Host "Install Rust? [Y/n]"
        if ($response -eq "" -or $response -eq "Y" -or $response -eq "y") {
            Install-Rust
        }
        else {
            Write-Error "✗ Rust is required to build OpenZax"
            exit 1
        }
    }
    
    # Try to download binary, fallback to source
    if (-not (Install-Binary)) {
        Write-Info "Building from source..."
        Install-FromSource
    }
    
    # Setup
    Setup-Config
    Add-ToPath
    
    # Success
    Print-Success
}

# Run installer
Main
