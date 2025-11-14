#!/bin/bash
set -euo pipefail

SERVICE_NAME="super_v.service"
SYSTEM_PATH="/etc/systemd/system/${SERVICE_NAME}"
YDO_SERVICE_NAME="ydotoold.service"
YDO_SERVICE_PATH="/etc/systemd/system/${YDO_SERVICE_NAME}"
USER_PATH="${HOME}/.config/systemd/user/${SERVICE_NAME}"
LOG_FILE="/var/log/superv.log"

echo "[*] Stopping user service (if active)..."
(sudo rm /usr/local/bin/super_v) || echo "super_v cli already removed"
systemctl --user stop "${SERVICE_NAME}" 2>/dev/null || true
systemctl --user disable "${SERVICE_NAME}" 2>/dev/null || true

echo "[*] Disabling lingering for user (requires sudo)..."
sudo loginctl disable-linger "$(id -un)" 2>/dev/null || true

echo "[*] Removing user service file..."
rm -f "${USER_PATH}"

echo "[*] Stopping services (if active, requires sudo)..."
sudo systemctl stop "${SERVICE_NAME}" 2>/dev/null || true
sudo systemctl disable "${SERVICE_NAME}" 2>/dev/null || true
sudo systemctl stop "${YDO_SERVICE_NAME}" 2>/dev/null || true
sudo systemctl disable "${YDO_SERVICE_NAME}" 2>/dev/null || true

echo "[*] Removing system service file..."
sudo rm -f "/tmp/.ydotool_socket"
sudo rm -f "/usr/local/bin/ydotool"
sudo rm -f "/usr/local/bin/ydotoold"
sudo rm -f "${SYSTEM_PATH}"
sudo rm -f "${YDO_SERVICE_PATH}"

echo "[*] Reloading daemons..."
systemctl --user daemon-reload
sudo systemctl daemon-reload

echo "[*] Removing log file (optional)..."
sudo rm -f "${LOG_FILE}" 2>/dev/null || true
sudo rm -f ~/superv.log || true
echo "[*] All done. Service fully disabled and cleaned up."