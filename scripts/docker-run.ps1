# Docker Build and Run Script for Rust-Spray (PowerShell Version)
# This script helps build and run all the available Docker configurations

param(
    [Parameter(Position=0)]
    [string]$Command = "help"
)

# Colors for output
function Write-Status { 
    param($Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue 
}

function Write-Success { 
    param($Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green 
}

function Write-Error { 
    param($Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red 
}

function Write-Warning { 
    param($Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow 
}

# Check if Docker is running
function Test-Docker {
    try {
        $null = docker info 2>$null
        Write-Success "Docker is running"
        return $true
    } catch {
        Write-Error "Docker is not running. Please start Docker Desktop and try again."
        return $false
    }
}

# Function to build Docker image
function Build-DockerImage {
    param(
        [string]$Dockerfile,
        [string]$Tag,
        [string]$Context = "."
    )
    
    Write-Status "Building Docker image: $Tag"
    Write-Status "Using Dockerfile: $Dockerfile"
    
    try {
        docker build -f $Dockerfile -t $Tag $Context
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Successfully built $Tag"
            return $true
        } else {
            Write-Error "Failed to build $Tag"
            return $false
        }
    } catch {
        Write-Error "Failed to build $Tag : $_"
        return $false
    }
}

# Function to run Docker container
function Start-DockerContainer {
    param(
        [string]$Image,
        [string]$Name,
        [string]$AdditionalArgs = ""
    )
    
    Write-Status "Running container: $Name from image: $Image"
    
    # Stop and remove existing container if it exists
    $existingContainer = docker ps -a --filter "name=$Name" --format "{{.Names}}" 2>$null
    if ($existingContainer -eq $Name) {
        Write-Warning "Stopping and removing existing container: $Name"
        docker stop $Name 2>$null | Out-Null
        docker rm $Name 2>$null | Out-Null
    }
    
    try {
        if ($AdditionalArgs) {
            Invoke-Expression "docker run --name $Name $AdditionalArgs $Image"
        } else {
            docker run --name $Name $Image
        }
        
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Successfully ran container: $Name"
            return $true
        } else {
            Write-Error "Failed to run container: $Name"
            return $false
        }
    } catch {
        Write-Error "Failed to run container: $Name : $_"
        return $false
    }
}

# Show available Docker configurations
function Show-Available {
    Write-Host "🐳 Available Docker Configurations:" -ForegroundColor Cyan
    Write-Host "==================================="
    Write-Host ""
    Write-Host "1. Cross-compilation Dockerfiles (Root directory):"
    Write-Host "   • Dockerfile.cross-aarch64     - Cross-compile for ARM64 with OpenCV"
    Write-Host "   • Dockerfile.armv7-opencv      - Cross-compile for ARMv7 with OpenCV"
    Write-Host "   • Dockerfile.pi-opencv         - Raspberry Pi optimized build"
    Write-Host "   • Dockerfile.pi-opencv-armv7   - Raspberry Pi ARMv7 build"
    Write-Host ""
    Write-Host "2. Native ARM Dockerfiles (docker/ directory):"
    Write-Host "   • docker/Dockerfile.aarch64    - Native ARM64 build"
    Write-Host "   • docker/Dockerfile.armv7      - Native ARMv7 build"
    Write-Host "   • docker/aarch64-opencv.dockerfile - ARM64 with OpenCV"
    Write-Host ""
    Write-Host "3. PowerShell commands:"
    Write-Host "   .\scripts\docker-run.ps1 build-cross-aarch64    - Build cross-compilation image for ARM64"
    Write-Host "   .\scripts\docker-run.ps1 build-native-aarch64   - Build native ARM64 image"
    Write-Host "   .\scripts\docker-run.ps1 build-all              - Build all available images"
    Write-Host "   .\scripts\docker-run.ps1 run-aarch64            - Run ARM64 container"
    Write-Host "   .\scripts\docker-run.ps1 clean                  - Clean up all Rust-Spray images"
}

# Build cross-compilation image for aarch64
function Build-CrossAarch64 {
    Write-Status "Building cross-compilation image for aarch64..."
    return Build-DockerImage "Dockerfile.cross-aarch64" "rust-spray:cross-aarch64" "."
}

# Build native aarch64 image
function Build-NativeAarch64 {
    Write-Status "Building native aarch64 image..."
    return Build-DockerImage "docker/Dockerfile.aarch64" "rust-spray:native-aarch64" "."
}

# Build aarch64 with OpenCV
function Build-Aarch64OpenCV {
    Write-Status "Building aarch64 image with OpenCV..."
    return Build-DockerImage "docker/aarch64-opencv.dockerfile" "rust-spray:aarch64-opencv" "."
}

# Build ARMv7 images
function Build-Armv7 {
    Write-Status "Building ARMv7 images..."
    $success1 = Build-DockerImage "docker/Dockerfile.armv7" "rust-spray:native-armv7" "."
    $success2 = Build-DockerImage "Dockerfile.armv7-opencv" "rust-spray:cross-armv7-opencv" "."
    return ($success1 -and $success2)
}

# Build Raspberry Pi optimized images
function Build-RaspberryPi {
    Write-Status "Building Raspberry Pi optimized images..."
    $success1 = Build-DockerImage "Dockerfile.pi-opencv" "rust-spray:pi-opencv" "."
    $success2 = Build-DockerImage "Dockerfile.pi-opencv-armv7" "rust-spray:pi-opencv-armv7" "."
    return ($success1 -and $success2)
}

# Build all images
function Build-All {
    Write-Status "Building all Docker images..."
    $failedBuilds = @()
    
    if (-not (Build-CrossAarch64)) {
        $failedBuilds += "cross-aarch64"
    }
    
    if (-not (Build-NativeAarch64)) {
        $failedBuilds += "native-aarch64"
    }
    
    if (-not (Build-Aarch64OpenCV)) {
        $failedBuilds += "aarch64-opencv"
    }
    
    if (-not (Build-Armv7)) {
        $failedBuilds += "armv7"
    }
    
    if (-not (Build-RaspberryPi)) {
        $failedBuilds += "raspberry-pi"
    }
    
    Write-Host ""
    Write-Status "Build Summary:"
    if ($failedBuilds.Count -eq 0) {
        Write-Success "All images built successfully!"
    } else {
        Write-Warning "Some builds failed: $($failedBuilds -join ', ')"
    }
    
    # Show built images
    Write-Host ""
    Write-Status "Available Rust-Spray images:"
    $images = docker images --filter "reference=rust-spray*" --format "table {{.Repository}}:{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}"
    if ($images) {
        Write-Host $images
    } else {
        Write-Warning "No Rust-Spray images found"
    }
}

# Run aarch64 container
function Start-Aarch64 {
    $image = "rust-spray:cross-aarch64"
    $existingImages = docker images --filter "reference=$image" --format "{{.Repository}}:{{.Tag}}"
    
    if (-not $existingImages) {
        Write-Warning "Image $image not found. Building it first..."
        if (-not (Build-CrossAarch64)) {
            return $false
        }
    }
    
    return Start-DockerContainer $image "rust-spray-aarch64" "-it --rm"
}

# Clean up all Rust-Spray images
function Remove-RustSprayImages {
    Write-Status "Cleaning up Rust-Spray Docker images..."
    
    # Stop and remove containers
    $containers = docker ps -a --filter "name=rust-spray*" --format "{{.ID}}"
    if ($containers) {
        Write-Status "Stopping and removing containers..."
        $containers | ForEach-Object { 
            docker stop $_ 2>$null | Out-Null
            docker rm $_ 2>$null | Out-Null
        }
    }
    
    # Remove images
    $images = docker images --filter "reference=rust-spray*" --format "{{.ID}}"
    if ($images) {
        Write-Status "Removing images..."
        $images | ForEach-Object { docker rmi -f $_ 2>$null | Out-Null }
    }
    
    Write-Success "Cleanup completed"
}

# Development mode - mount source code
function Start-DevContainer {
    $image = "rust-spray:cross-aarch64"
    Write-Status "Running development container with source code mounted..."
    
    $existingImages = docker images --filter "reference=$image" --format "{{.Repository}}:{{.Tag}}"
    if (-not $existingImages) {
        Write-Warning "Image $image not found. Building it first..."
        if (-not (Build-CrossAarch64)) {
            return $false
        }
    }
    
    $currentDir = (Get-Location).Path
    return Start-DockerContainer $image "rust-spray-dev" "-it --rm -v ${currentDir}:/workspace"
}

# List current images
function Show-Images {
    Write-Status "Current Rust-Spray images:"
    $images = docker images --filter "reference=rust-spray*" --format "table {{.Repository}}:{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}"
    if ($images) {
        Write-Host $images
    } else {
        Write-Warning "No Rust-Spray images found"
    }
}

# Main logic
if (-not (Test-Docker)) {
    exit 1
}

switch ($Command.ToLower()) {
    "build-cross-aarch64" {
        Build-CrossAarch64
    }
    "build-native-aarch64" {
        Build-NativeAarch64
    }
    "build-aarch64-opencv" {
        Build-Aarch64OpenCV
    }
    "build-armv7" {
        Build-Armv7
    }
    "build-raspberry-pi" {
        Build-RaspberryPi
    }
    "build-all" {
        Build-All
    }
    "run-aarch64" {
        Start-Aarch64
    }
    "dev-run" {
        Start-DevContainer
    }
    "clean" {
        Remove-RustSprayImages
    }
    { $_ -in @("list", "images") } {
        Show-Images
    }
    default {
        Show-Available
    }
}
