# Install Rust on Windows
Write-Host "Installing Rust..." -ForegroundColor Cyan

# Download rustup-init.exe
$rustupUrl = "https://win.rustup.rs/x86_64"
$rustupPath = "$env:TEMP\rustup-init.exe"

Write-Host "Downloading rustup-init.exe..." -ForegroundColor Yellow
Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath

Write-Host "Running rustup installer..." -ForegroundColor Yellow
Write-Host "Please follow the installer prompts (press 1 for default installation)" -ForegroundColor Green

# Run the installer
Start-Process -FilePath $rustupPath -ArgumentList "-y" -Wait

Write-Host ""
Write-Host "Rust installation complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Please restart your terminal or run:" -ForegroundColor Yellow
Write-Host '  $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")' -ForegroundColor Cyan
Write-Host ""
Write-Host "Then verify installation with:" -ForegroundColor Yellow
Write-Host "  rustc --version" -ForegroundColor Cyan
Write-Host "  cargo --version" -ForegroundColor Cyan
