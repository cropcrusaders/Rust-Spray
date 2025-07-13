#!/bin/bash
# Raspberry Pi Setup Script for Rust-Spray
# Run this script on your Raspberry Pi to set up the environment

set -e

echo "🔧 Setting up Raspberry Pi for Rust-Spray..."

# Update system
echo "📦 Updating system packages..."
sudo apt update && sudo apt upgrade -y

# Install Docker if not present
if ! command -v docker &> /dev/null; then
    echo "🐳 Installing Docker..."
    curl -fsSL https://get.docker.com -o get-docker.sh
    sudo sh get-docker.sh
    sudo usermod -aG docker $USER
    rm get-docker.sh
    echo "⚠️  Please log out and back in for Docker group changes to take effect"
else
    echo "✅ Docker already installed"
fi

# Install additional packages for camera support
echo "📷 Installing camera support packages..."
sudo apt install -y \
    v4l-utils \
    libcamera-tools \
    python3-picamera2 \
    raspi-config

# Enable camera interface
echo "📷 Enabling camera interface..."
sudo raspi-config nonint do_camera 0

# Create directories for Rust-Spray
echo "📁 Creating application directories..."
mkdir -p ~/rust-spray/{config,logs,data}

# Create docker-compose file for easy deployment
cat > ~/rust-spray/docker-compose.yml << 'EOF'
version: '3.8'
services:
  rust-spray:
    image: rust-spray:cross-aarch64
    container_name: rust-spray
    restart: unless-stopped
    privileged: true  # For camera access
    devices:
      - /dev/video0:/dev/video0  # USB camera
      - /dev/vchiq:/dev/vchiq    # Pi camera
    volumes:
      - ./config:/config
      - ./logs:/logs
      - ./data:/data
    environment:
      - RUST_LOG=info
    ports:
      - "8080:8080"  # If your app has a web interface
    networks:
      - rust-spray-net

networks:
  rust-spray-net:
    driver: bridge
EOF

# Create sample config file
cat > ~/rust-spray/config/Config.toml << 'EOF'
# Rust-Spray Configuration
[camera]
device_id = 0
width = 1280
height = 720
fps = 30

[detection]
hsv_h_min = 40
hsv_h_max = 80
hsv_s_min = 40
hsv_s_max = 255
hsv_v_min = 40
hsv_v_max = 255

[spray]
enabled = true
delay_ms = 100
EOF

echo "✅ Setup complete!"
echo ""
echo "📋 Next steps:"
echo "1. Copy your Docker image to this Pi"
echo "2. Load the image: docker load -i rust-spray-cross-aarch64.tar"
echo "3. Start the application: cd ~/rust-spray && docker-compose up -d"
echo ""
echo "🔍 Useful commands:"
echo "  Check camera: v4l2-ctl --list-devices"
echo "  Test Pi camera: libcamera-hello"
echo "  View logs: docker-compose logs -f"
echo "  Stop service: docker-compose down"
