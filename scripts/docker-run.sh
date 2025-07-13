#!/bin/bash
# Docker Build and Run Script for Rust-Spray
# This script helps build and run all the available Docker configurations

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check if Docker is running
check_docker() {
    if ! docker info >/dev/null 2>&1; then
        print_error "Docker is not running. Please start Docker Desktop and try again."
        exit 1
    fi
    print_success "Docker is running"
}

# Function to build Docker image
build_image() {
    local dockerfile="$1"
    local tag="$2"
    local context="${3:-.}"
    
    print_status "Building Docker image: $tag"
    print_status "Using Dockerfile: $dockerfile"
    
    if docker build -f "$dockerfile" -t "$tag" "$context"; then
        print_success "Successfully built $tag"
        return 0
    else
        print_error "Failed to build $tag"
        return 1
    fi
}

# Function to run Docker container
run_container() {
    local image="$1"
    local name="$2"
    local additional_args="${3:-}"
    
    print_status "Running container: $name from image: $image"
    
    # Stop and remove existing container if it exists
    if docker ps -a | grep -q "$name"; then
        print_warning "Stopping and removing existing container: $name"
        docker stop "$name" >/dev/null 2>&1 || true
        docker rm "$name" >/dev/null 2>&1 || true
    fi
    
    if docker run --name "$name" $additional_args "$image"; then
        print_success "Successfully ran container: $name"
        return 0
    else
        print_error "Failed to run container: $name"
        return 1
    fi
}

# Show available Docker configurations
show_available() {
    echo "🐳 Available Docker Configurations:"
    echo "==================================="
    echo ""
    echo "1. Cross-compilation Dockerfiles (Root directory):"
    echo "   • Dockerfile.cross-aarch64     - Cross-compile for ARM64 with OpenCV"
    echo "   • Dockerfile.armv7-opencv      - Cross-compile for ARMv7 with OpenCV"
    echo "   • Dockerfile.pi-opencv         - Raspberry Pi optimized build"
    echo "   • Dockerfile.pi-opencv-armv7   - Raspberry Pi ARMv7 build"
    echo ""
    echo "2. Native ARM Dockerfiles (docker/ directory):"
    echo "   • docker/Dockerfile.aarch64    - Native ARM64 build"
    echo "   • docker/Dockerfile.armv7      - Native ARMv7 build"
    echo "   • docker/aarch64-opencv.dockerfile - ARM64 with OpenCV"
    echo ""
    echo "3. Build commands:"
    echo "   $0 build-cross-aarch64    - Build cross-compilation image for ARM64"
    echo "   $0 build-native-aarch64   - Build native ARM64 image"
    echo "   $0 build-all              - Build all available images"
    echo "   $0 run-aarch64            - Run ARM64 container"
    echo "   $0 clean                  - Clean up all Rust-Spray images"
}

# Build cross-compilation image for aarch64
build_cross_aarch64() {
    print_status "Building cross-compilation image for aarch64..."
    build_image "Dockerfile.cross-aarch64" "rust-spray:cross-aarch64" "."
}

# Build native aarch64 image
build_native_aarch64() {
    print_status "Building native aarch64 image..."
    build_image "docker/Dockerfile.aarch64" "rust-spray:native-aarch64" "."
}

# Build aarch64 with OpenCV
build_aarch64_opencv() {
    print_status "Building aarch64 image with OpenCV..."
    build_image "docker/aarch64-opencv.dockerfile" "rust-spray:aarch64-opencv" "."
}

# Build ARMv7 images
build_armv7() {
    print_status "Building ARMv7 images..."
    build_image "docker/Dockerfile.armv7" "rust-spray:native-armv7" "."
    build_image "Dockerfile.armv7-opencv" "rust-spray:cross-armv7-opencv" "."
}

# Build Raspberry Pi optimized images
build_raspberry_pi() {
    print_status "Building Raspberry Pi optimized images..."
    build_image "Dockerfile.pi-opencv" "rust-spray:pi-opencv" "."
    build_image "Dockerfile.pi-opencv-armv7" "rust-spray:pi-opencv-armv7" "."
}

# Build all images
build_all() {
    print_status "Building all Docker images..."
    local failed_builds=()
    
    if ! build_cross_aarch64; then
        failed_builds+=("cross-aarch64")
    fi
    
    if ! build_native_aarch64; then
        failed_builds+=("native-aarch64")
    fi
    
    if ! build_aarch64_opencv; then
        failed_builds+=("aarch64-opencv")
    fi
    
    if ! build_armv7; then
        failed_builds+=("armv7")
    fi
    
    if ! build_raspberry_pi; then
        failed_builds+=("raspberry-pi")
    fi
    
    echo ""
    print_status "Build Summary:"
    if [ ${#failed_builds[@]} -eq 0 ]; then
        print_success "All images built successfully!"
    else
        print_warning "Some builds failed: ${failed_builds[*]}"
    fi
    
    # Show built images
    echo ""
    print_status "Available Rust-Spray images:"
    docker images | grep rust-spray || print_warning "No Rust-Spray images found"
}

# Run aarch64 container
run_aarch64() {
    local image="rust-spray:cross-aarch64"
    if ! docker images | grep -q "$image"; then
        print_warning "Image $image not found. Building it first..."
        build_cross_aarch64
    fi
    
    run_container "$image" "rust-spray-aarch64" "-it --rm"
}

# Clean up all Rust-Spray images
clean_images() {
    print_status "Cleaning up Rust-Spray Docker images..."
    
    # Stop and remove containers
    local containers=$(docker ps -a | grep rust-spray | awk '{print $1}')
    if [ -n "$containers" ]; then
        print_status "Stopping and removing containers..."
        echo "$containers" | xargs docker stop >/dev/null 2>&1 || true
        echo "$containers" | xargs docker rm >/dev/null 2>&1 || true
    fi
    
    # Remove images
    local images=$(docker images | grep rust-spray | awk '{print $3}')
    if [ -n "$images" ]; then
        print_status "Removing images..."
        echo "$images" | xargs docker rmi -f >/dev/null 2>&1 || true
    fi
    
    print_success "Cleanup completed"
}

# Development mode - mount source code
dev_run() {
    local image="rust-spray:cross-aarch64"
    print_status "Running development container with source code mounted..."
    
    if ! docker images | grep -q "$image"; then
        print_warning "Image $image not found. Building it first..."
        build_cross_aarch64
    fi
    
    run_container "$image" "rust-spray-dev" "-it --rm -v $(pwd):/workspace"
}

# Main function
main() {
    case "${1:-help}" in
        "build-cross-aarch64")
            check_docker
            build_cross_aarch64
            ;;
        "build-native-aarch64")
            check_docker
            build_native_aarch64
            ;;
        "build-aarch64-opencv")
            check_docker
            build_aarch64_opencv
            ;;
        "build-armv7")
            check_docker
            build_armv7
            ;;
        "build-raspberry-pi")
            check_docker
            build_raspberry_pi
            ;;
        "build-all")
            check_docker
            build_all
            ;;
        "run-aarch64")
            check_docker
            run_aarch64
            ;;
        "dev-run")
            check_docker
            dev_run
            ;;
        "clean")
            check_docker
            clean_images
            ;;
        "list"|"images")
            check_docker
            print_status "Current Rust-Spray images:"
            docker images | grep rust-spray || print_warning "No Rust-Spray images found"
            ;;
        "help"|*)
            show_available
            ;;
    esac
}

main "$@"
