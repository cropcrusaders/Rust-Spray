# PowerShell script to deploy Rust-Spray to Raspberry Pi
# Usage: .\deploy-to-pi.ps1 -PiAddress "192.168.1.100" -PiUser "pi"

param(
    [Parameter(Mandatory=$true)]
    [string]$PiAddress,
    
    [Parameter(Mandatory=$false)]
    [string]$PiUser = "pi"
)

Write-Host "🚀 Deploying Rust-Spray to Raspberry Pi..." -ForegroundColor Green

# Export Docker image
Write-Host "📦 Exporting Docker image..." -ForegroundColor Yellow
docker save rust-spray:cross-aarch64 -o rust-spray-cross-aarch64.tar

if (-not (Test-Path "rust-spray-cross-aarch64.tar")) {
    Write-Error "Failed to export Docker image"
    exit 1
}

Write-Host "✅ Docker image exported ($(Get-ChildItem rust-spray-cross-aarch64.tar | Select-Object -ExpandProperty Length | ForEach-Object {[math]::Round($_/1MB, 2)}) MB)" -ForegroundColor Green

# Check if we can reach the Pi
Write-Host "🔍 Testing connection to Pi at $PiAddress..." -ForegroundColor Yellow
if (-not (Test-Connection -ComputerName $PiAddress -Count 1 -Quiet)) {
    Write-Error "Cannot reach Raspberry Pi at $PiAddress"
    Write-Host "💡 Make sure:" -ForegroundColor Cyan
    Write-Host "  - Pi is powered on and connected to network" -ForegroundColor White
    Write-Host "  - SSH is enabled (sudo systemctl enable ssh)" -ForegroundColor White
    Write-Host "  - IP address is correct" -ForegroundColor White
    exit 1
}

Write-Host "✅ Pi is reachable" -ForegroundColor Green

# Transfer files using SCP (requires WSL or Git Bash with SSH)
Write-Host "📤 Transferring files to Pi..." -ForegroundColor Yellow

# Check if we have scp available
try {
    # Try transferring the setup script
    scp raspi-setup.sh "${PiUser}@${PiAddress}:~/"
    Write-Host "✅ Setup script transferred" -ForegroundColor Green
    
    # Transfer the Docker image (this will take a while)
    Write-Host "⏳ Transferring Docker image (this may take several minutes)..." -ForegroundColor Yellow
    scp rust-spray-cross-aarch64.tar "${PiUser}@${PiAddress}:~/"
    Write-Host "✅ Docker image transferred" -ForegroundColor Green
    
} catch {
    Write-Error "SCP transfer failed. You may need to:"
    Write-Host "💡 Options to transfer files:" -ForegroundColor Cyan
    Write-Host "  1. Use WinSCP GUI application" -ForegroundColor White
    Write-Host "  2. Use WSL: wsl scp files..." -ForegroundColor White
    Write-Host "  3. Copy files to USB drive and transfer manually" -ForegroundColor White
    exit 1
}

# Execute setup script on Pi
Write-Host "🔧 Running setup script on Pi..." -ForegroundColor Yellow
try {
    ssh "${PiUser}@${PiAddress}" "chmod +x raspi-setup.sh && ./raspi-setup.sh"
    Write-Host "✅ Setup script completed" -ForegroundColor Green
} catch {
    Write-Warning "Setup script execution failed. You can run it manually:"
    Write-Host "ssh ${PiUser}@${PiAddress}" -ForegroundColor White
    Write-Host "chmod +x raspi-setup.sh && ./raspi-setup.sh" -ForegroundColor White
}

# Load Docker image on Pi
Write-Host "🐳 Loading Docker image on Pi..." -ForegroundColor Yellow
try {
    ssh "${PiUser}@${PiAddress}" "docker load -i rust-spray-cross-aarch64.tar"
    Write-Host "✅ Docker image loaded successfully" -ForegroundColor Green
} catch {
    Write-Warning "Docker image loading failed. You can load it manually:"
    Write-Host "ssh ${PiUser}@${PiAddress}" -ForegroundColor White
    Write-Host "docker load -i rust-spray-cross-aarch64.tar" -ForegroundColor White
}

# Clean up local tar file
Remove-Item rust-spray-cross-aarch64.tar -Force
Write-Host "🧹 Cleaned up local files" -ForegroundColor Green

Write-Host ""
Write-Host "🎉 Deployment complete!" -ForegroundColor Green
Write-Host ""
Write-Host "🔗 To connect to your Pi:" -ForegroundColor Cyan
Write-Host "ssh ${PiUser}@${PiAddress}" -ForegroundColor White
Write-Host ""
Write-Host "🚀 To start Rust-Spray:" -ForegroundColor Cyan
Write-Host "cd ~/rust-spray" -ForegroundColor White
Write-Host "docker-compose up -d" -ForegroundColor White
Write-Host ""
Write-Host "📊 To view logs:" -ForegroundColor Cyan
Write-Host "docker-compose logs -f" -ForegroundColor White
