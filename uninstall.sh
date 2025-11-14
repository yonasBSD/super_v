#!/bin/bash
set -euo pipefail

# --- Config ---
SERVICE_NAME="super_v.service"
YDO_SERVICE_NAME="ydotoold.service"
USERNAME="$(id -un)"

# Paths
USER_SERVICE_PATH="${HOME}/.config/systemd/user/${SERVICE_NAME}"
YDO_SYSTEM_SERVICE_PATH="/etc/systemd/system/${YDO_SERVICE_NAME}"
YDO_USER_SERVICE_PATH="/usr/lib/systemd/user/ydotoold.service" # Default file from 'make install'
SUPERV_BIN_PATH="/usr/local/bin/super_v"
YDO_BIN_PATH="/usr/local/bin/ydotool"
YDOOLD_BIN_PATH="/usr/local/bin/ydotoold"
YDO_MAN1_PATH="/usr/local/share/man/man1/ydotool.1"
YDOOLD_MAN8_PATH="/usr/local/share/man/man8/ydotoold.8"
LOG_PATH="/var/log/superv.log"
OLD_LOG_PATH="${HOME}/superv.log" # From your old script
YDO_SOCKET_PATH="/tmp/.ydotool_socket"

echo "[*] 1. Stopping & disabling services..."
# Stop/disable USER super_v service
systemctl --user stop "${SERVICE_NAME}" 2>/dev/null || true
systemctl --user disable "${SERVICE_NAME}" 2>/dev/null || true

# Stop/disable SYSTEM ydotoold service (requires sudo)
sudo systemctl stop "${YDO_SERVICE_NAME}" 2>/dev/null || true
sudo systemctl disable "${YDO_SERVICE_NAME}" 2>/dev/null || true

# Stop/disable the default USER ydotoold service (just in case)
systemctl --user stop "${YDO_SERVICE_NAME}" 2>/dev/null || true
systemctl --user disable "${YDO_SERVICE_NAME}" 2>/dev/null || true

echo "[*] 2. Disabling linger (requires sudo)..."
sudo loginctl disable-linger "${USERNAME}" 2>/dev/null || true

echo "[*] 3. Removing systemd unit files..."
# Remove your custom USER service file
rm -f "${USER_SERVICE_PATH}"

# Remove your custom SYSTEM service file (requires sudo)
sudo rm -f "${YDO_SYSTEM_SERVICE_PATH}"

# Remove the default ydotool USER service file (from 'make install')
sudo rm -f "${YDO_USER_SERVICE_PATH}"

echo "[*] 4. Reloading daemons..."
systemctl --user daemon-reload
sudo systemctl daemon-reload

echo "[*] 5. Removing binaries and man pages (requires sudo)..."
sudo rm -f "${SUPERV_BIN_PATH}"
sudo rm -f "${YDO_BIN_PATH}"
sudo rm -f "${YDOOLD_BIN_PATH}"
sudo rm -f "${YDO_MAN1_PATH}"
sudo rm -f "${YDOOLD_MAN8_PATH}"

echo "[*] 6. Removing log and socket files..."
sudo rm -f "${LOG_PATH}"
rm -f "${OLD_LOG_PATH}" 2>/dev/null || true
sudo rm -f "${YDO_SOCKET_PATH}" # Socket might be root-owned

echo
echo "[*] Uninstall complete. All components removed."