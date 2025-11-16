#!/bin/bash
# Installation script for Secure Cryptor auto-mount service on Linux

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Secure Cryptor Auto-Mount Service Installer${NC}"
echo

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Please run as root (use sudo)${NC}"
    exit 1
fi

# Check if secure-cryptor binary exists
if ! command -v secure-cryptor &> /dev/null; then
    echo -e "${RED}Error: secure-cryptor binary not found in PATH${NC}"
    echo "Please install Secure Cryptor first:"
    echo "  cargo install --path ."
    exit 1
fi

echo -e "${YELLOW}Installing systemd service...${NC}"

# Copy service file
cp secure-cryptor-automount.service /etc/systemd/system/
chmod 644 /etc/systemd/system/secure-cryptor-automount.service

# Copy environment file
cp secure-cryptor.conf /etc/default/secure-cryptor
chmod 644 /etc/default/secure-cryptor

# Create configuration directory
mkdir -p /etc/secure-cryptor
chmod 700 /etc/secure-cryptor

# Create runtime directory
mkdir -p /run/secure-cryptor
chmod 700 /run/secure-cryptor

# Create default configuration if it doesn't exist
if [ ! -f /etc/secure-cryptor/automount.json ]; then
    echo -e "${YELLOW}Creating default configuration...${NC}"
    cat > /etc/secure-cryptor/automount.json <<EOF
{
  "volumes": [],
  "global_timeout": 120,
  "background": true
}
EOF
    chmod 600 /etc/secure-cryptor/automount.json
    echo -e "${GREEN}Created /etc/secure-cryptor/automount.json${NC}"
    echo "Edit this file to add your encrypted volumes"
fi

# Reload systemd
echo -e "${YELLOW}Reloading systemd...${NC}"
systemctl daemon-reload

echo
echo -e "${GREEN}Installation complete!${NC}"
echo
echo "Next steps:"
echo "1. Edit configuration: sudo nano /etc/secure-cryptor/automount.json"
echo "2. Enable service: sudo systemctl enable secure-cryptor-automount.service"
echo "3. Start service: sudo systemctl start secure-cryptor-automount.service"
echo "4. Check status: sudo systemctl status secure-cryptor-automount.service"
echo
echo "Example configuration:"
echo '{
  "volumes": [
    {
      "id": "home-encrypted",
      "name": "Encrypted Home",
      "container_path": "/data/home.crypt",
      "mount_point": "/home/user/encrypted",
      "auth": {"method": "tpm", "pcr_indices": [0, 7]},
      "read_only": false,
      "required": true,
      "timeout": 60,
      "auto_unmount": true
    }
  ]
}'
