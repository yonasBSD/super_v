#!/bin/bash
set -euo pipefail

# Config
CURRENT_DIR="$(pwd)"
USERNAME="$(id -un)"
USERHOME="${HOME}"
SERVICE_NAME="super_v.service"
USER_DIR="${USERHOME}/.config/systemd/user"
USER_PATH="${USER_DIR}/${SERVICE_NAME}"
LOG_PATH="/var/log/superv.log"
YDO_SERVICE_NAME="ydotoold.service"
YDO_SERVICE_PATH="/etc/systemd/system/${YDO_SERVICE_NAME}"

# Install ydotool if not present
echo "[*] Checking for ydotool..."
if [ ! -f /usr/local/bin/ydotoold ] || [ ! -f /usr/local/bin/ydotool ]; then
    echo "[!] ydotool not found. Installing from source (requires sudo)..."
    
    # Install dependencies (Debian/Ubuntu-based)
    # This may need adjustment for other distros
    sudo apt-get update
    sudo apt-get install -y git cmake scdoc build-essential
    
    # Clone, build, and install
    echo "[*] Cloning ydotool..."
    cd /tmp
    git clone https://github.com/ReimuNotMoe/ydotool.git
    cd ydotool
    
    echo "[*] Building ydotool..."
    cmake .
    make
    
    echo "[*] Installing ydotool..."
    sudo make install
    
    # Cleanup
    cd $CURRENT_DIR
    rm -rf /tmp/ydotool
    echo "[*] ydotool installed."
else
    echo "[*] ydotool found."
fi

# build the project as release 
cargo build --release
strip target/release/super_v

# Stop and remove any system service to avoid conflicts (requires sudo)
echo "[*] removing system service (if present)..."
sudo rm /usr/local/bin/super_v || echo "super_v cli already removed"
sudo systemctl stop "${SERVICE_NAME}" 2>/dev/null || true
sudo systemctl disable "${SERVICE_NAME}" 2>/dev/null || true
sudo rm -f "/etc/systemd/system/${SERVICE_NAME}" 2>/dev/null || true

# Stop and remove old ydotoold service ---
echo "[*] removing ydotoold system service (if present)..."
sudo systemctl stop "${YDO_SERVICE_NAME}" 2>/dev/null || true
sudo systemctl disable "${YDO_SERVICE_NAME}" 2>/dev/null || true
sudo rm -f "${YDO_SERVICE_PATH}" 2>/dev/null || true

sudo systemctl daemon-reload

echo "[*] Installing service..."
sudo cp ./target/release/super_v /usr/local/bin/
sudo chmod +x /usr/local/bin/super_v

# Install application icons so the GTK window resolves the app ID.
echo "[*] installing application icons..."
ICON_NAME="com.ecstra.super_v"
ICON_DIR="/usr/share/icons/hicolor"

install_icon() {
    local size="$1"
    local src="assets/icons/${size}x${size}.png"
    local dest="${ICON_DIR}/${size}x${size}/apps/${ICON_NAME}.png"

    if [ -f "${src}" ]; then
        sudo install -Dm644 "${src}" "${dest}"
    else
        echo "[!] missing ${src}, skipping ${size}x${size} icon"
    fi
}

install_icon 32
install_icon 192
install_icon 512

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    sudo gtk-update-icon-cache "${ICON_DIR}"
fi

echo "[*] installing desktop entry..."
sudo install -Dm644 assets/super_v.desktop \
    "/usr/share/applications/super_v.desktop"
if command -v update-desktop-database >/dev/null 2>&1; then
    sudo update-desktop-database /usr/share/applications
fi

# Write ydotoold system service
echo "[*] writing ydotoold system service to ${YDO_SERVICE_PATH}..."
# We use sudo bash -c to write to a root-owned file
sudo bash -c "cat > ${YDO_SERVICE_PATH}" <<EOF
[Unit]
Description=ydotool daemon
Wants=systemd-udev-settle.service
After=systemd-udev-settle.service

[Service]
Type=simple
# Load uinput module and set permissions *before* starting
ExecStartPre=/sbin/modprobe uinput
ExecStartPre=/bin/bash -c 'sudo chmod 0666 /dev/uinput'

# Start the daemon (no sudo needed) and tell *it* to set the socket mode
ExecStart=sudo ydotoold
TimeoutStartSec=5
ExecStartPost=/bin/bash -c 'sudo chmod 666 /tmp/.ydotool_socket'
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# Ensure user unit dir exists
echo "[*] creating user unit dir..."
mkdir -p "${USER_DIR}"

# Write user unit
echo "[*] writing user service to ${USER_PATH}..."
cat > "${USER_PATH}" <<EOF
[Unit]
Description=SuperV Clipboard Manager (user)
After=graphical-session.target

[Service]
Type=simple
Environment=RUST_BACKTRACE=1
Environment=RUST_LOG=info
Environment=DISPLAY=:0
Environment=XDG_RUNTIME_DIR=/run/user/1000
ExecStartPre=/usr/local/bin/super_v clean
ExecStart=/usr/local/bin/super_v start
Restart=on-failure
RestartSec=5
StandardOutput=append:/var/log/superv.log
StandardError=append:/var/log/superv.log

[Install]
WantedBy=default.target
EOF

echo "[*] cleaning up build file..."
# cargo clean

# Ensure the log file exists and is writable by the user (no sudo required)
echo "[*] creating user-writable log at ${LOG_PATH}..."
sudo touch "${LOG_PATH}"
sudo chmod 600 "${LOG_PATH}"
sudo chown "${USERNAME}:${USERNAME}" "${LOG_PATH}"

# Enable lingering so the service can run without an active login (requires sudo)
echo "[*] enabling linger for ${USERNAME} (requires sudo)..."
sudo loginctl enable-linger "${USERNAME}"

# Reload system daemon, enable and start ydotoold unit
echo "[*] reloading system systemd and starting ydotoold service..."
sudo systemctl daemon-reload
sudo systemctl enable --now "${YDO_SERVICE_NAME}"

# Reload user daemon, enable and start the unit
echo "[*] reloading user systemd and starting service..."
systemctl --user daemon-reload
systemctl --user enable --now "${SERVICE_NAME}"
sudo update-desktop-database /usr/share/applications

echo
echo "[*] status (ydotoold system unit):"
sudo systemctl status "${YDO_SERVICE_NAME}" --no-pager

echo
echo "[*] status (user unit):"
systemctl --user status "${SERVICE_NAME}" --no-pager