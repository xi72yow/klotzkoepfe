#!/bin/bash
set -euo pipefail

TOOL="klotzkoepfe"
REPO_URL="https://xi72yow.github.io/${TOOL}"

echo "Installing ${TOOL}..."

# GPG-Key importieren
curl -fsSL "${REPO_URL}/key.gpg" | gpg --dearmor -o /usr/share/keyrings/${TOOL}.gpg

# APT-Source hinzufuegen
cat > /etc/apt/sources.list.d/${TOOL}.list << EOF
deb [signed-by=/usr/share/keyrings/${TOOL}.gpg] ${REPO_URL} stable main
EOF

# Installieren
apt-get update -o Dir::Etc::sourcelist="sources.list.d/${TOOL}.list" -o Dir::Etc::sourceparts="-" -o APT::Get::List-Cleanup="0"
apt-get install -y ${TOOL}

echo "${TOOL} installed successfully!"
