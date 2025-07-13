# 🔧 Raspberry Pi SSH Setup Guide

## Prerequisites
- Raspberry Pi 4/5 with Raspberry Pi OS (64-bit)
- MicroSD card (32GB+ recommended)
- Network connection (Ethernet or WiFi)

## 1. Enable SSH on Raspberry Pi

### Method A: Using Raspberry Pi Imager (Recommended)
1. Download [Raspberry Pi Imager](https://www.raspberrypi.com/software/)
2. Flash Raspberry Pi OS (64-bit) to SD card
3. Before writing, click the gear icon (⚙️) for advanced options:
   - ✅ Enable SSH
   - ✅ Set username/password (e.g., user: `pi`, password: `raspberry`)
   - ✅ Configure WiFi if needed
   - ✅ Set locale settings

### Method B: Manual Setup (if already installed)
1. Connect Pi to monitor/keyboard
2. Open terminal and run:
   ```bash
   sudo systemctl enable ssh
   sudo systemctl start ssh
   ```

## 2. Find Your Pi's IP Address

### On the Pi:
```bash
hostname -I
```

### From Windows (scan network):
```powershell
# Scan for Raspberry Pi devices
arp -a | findstr "b8-27-eb\|dc-a6-32\|e4-5f-01"
```

### From Router:
- Check your router's admin panel for connected devices
- Look for "raspberrypi" or similar hostname

## 3. Connect from Windows

### Using PowerShell (Built-in):
```powershell
ssh pi@192.168.1.100
# Replace 192.168.1.100 with your Pi's IP address
```

### Using PuTTY (GUI):
1. Download [PuTTY](https://www.putty.org/)
2. Enter Pi's IP address
3. Port: 22, Connection type: SSH
4. Click "Open"

## 4. First Connection
1. Accept the SSH key fingerprint (type "yes")
2. Enter password (default: `raspberry`)
3. **Change default password**: `passwd`

## 5. Deploy Rust-Spray

### Option A: Automated (using our script):
```powershell
# Run from your Rust-Spray directory
.\deploy-to-pi.ps1 -PiAddress "192.168.1.100" -PiUser "pi"
```

### Option B: Manual:
1. Transfer files:
   ```powershell
   scp raspi-setup.sh pi@192.168.1.100:~/
   docker save rust-spray:cross-aarch64 -o rust-spray.tar
   scp rust-spray.tar pi@192.168.1.100:~/
   ```

2. Setup on Pi:
   ```bash
   ssh pi@192.168.1.100
   chmod +x raspi-setup.sh && ./raspi-setup.sh
   docker load -i rust-spray.tar
   cd ~/rust-spray
   docker-compose up -d
   ```

## 6. Camera Setup

### USB Camera:
```bash
# List cameras
v4l2-ctl --list-devices

# Test camera
ffmpeg -f v4l2 -i /dev/video0 -t 5 test.mp4
```

### Pi Camera:
```bash
# Test Pi camera
libcamera-hello --timeout 5000

# List cameras
libcamera-hello --list-cameras
```

## 7. Troubleshooting

### SSH Connection Issues:
- Check Pi is powered on: `ping 192.168.1.100`
- Verify SSH is running: `sudo systemctl status ssh`
- Check firewall: `sudo ufw status`

### Camera Issues:
- Enable camera: `sudo raspi-config` → Interface Options → Camera
- Check permissions: `sudo usermod -a -G video pi`
- Reboot after changes: `sudo reboot`

### Docker Issues:
- Check Docker status: `sudo systemctl status docker`
- View logs: `docker-compose logs`
- Restart containers: `docker-compose restart`

## 8. Useful Commands

### System Management:
```bash
# Check system info
neofetch
htop

# Monitor resources
vcgencmd measure_temp
vcgencmd measure_volts
```

### Docker Management:
```bash
# View running containers
docker ps

# Container logs
docker logs rust-spray

# Update application
docker-compose pull && docker-compose up -d
```

### Network:
```bash
# Check IP
ip addr show

# WiFi status
iwconfig

# Network speed test
speedtest-cli
```

## 🚀 Quick Start Summary

1. Flash Pi OS with SSH enabled
2. Find Pi IP: `hostname -I` 
3. Connect: `ssh pi@<IP>`
4. Deploy: `.\deploy-to-pi.ps1 -PiAddress "<IP>"`
5. Start: `docker-compose up -d`

Your Rust-Spray should now be running on the Pi! 🎉
