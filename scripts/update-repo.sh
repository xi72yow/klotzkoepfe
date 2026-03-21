#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="${SCRIPT_DIR}/.."
REPO_DIR="${REPO_ROOT}/repo"
NO_SIGN=false

if [[ "${1:-}" == "--no-sign" ]]; then
  NO_SIGN=true
fi

if [ -z "$(ls -A "${REPO_ROOT}"/target/debian/*.deb 2>/dev/null)" ]; then
  echo "Error: No .deb packages found in target/debian/"
  exit 1
fi

# Clean and create repo structure
rm -rf "${REPO_DIR}"
mkdir -p "${REPO_DIR}/pool/main"
mkdir -p "${REPO_DIR}/dists/stable/main/binary-amd64"

# Copy packages to pool
cp "${REPO_ROOT}"/target/debian/*.deb "${REPO_DIR}/pool/main/"

# Extract changelogs from .deb packages
for deb in "${REPO_DIR}"/pool/main/*.deb; do
  PKG_NAME=$(dpkg-deb -f "$deb" Package)
  PKG_VERSION=$(dpkg-deb -f "$deb" Version)
  PREFIX="${PKG_NAME:0:1}"
  CHANGELOG_DIR="${REPO_DIR}/main/${PREFIX}/${PKG_NAME}/${PKG_NAME}_${PKG_VERSION}"
  mkdir -p "${CHANGELOG_DIR}"
  TMPEXTRACT=$(mktemp -d)
  dpkg-deb -x "$deb" "$TMPEXTRACT"
  if [ -f "${TMPEXTRACT}/usr/share/doc/${PKG_NAME}/changelog.Debian.gz" ]; then
    zcat "${TMPEXTRACT}/usr/share/doc/${PKG_NAME}/changelog.Debian.gz" > "${CHANGELOG_DIR}/changelog"
  fi
  rm -rf "$TMPEXTRACT"
done

# Generate Packages file
cd "${REPO_DIR}"
dpkg-scanpackages --arch amd64 pool/main /dev/null > dists/stable/main/binary-amd64/Packages
gzip -k dists/stable/main/binary-amd64/Packages

# Generate Release file
cd "${REPO_DIR}/dists/stable"

cat > Release << EOF
Origin: klotzkoepfe
Label: Klotzkoepfe
Suite: stable
Codename: stable
Architectures: amd64
Components: main
Changelogs: https://xi72yow.github.io/klotzkoepfe/@CHANGEPATH@/changelog
Description: APT repository for Klotzkoepfe
$(apt-ftparchive release .)
EOF

if [ "$NO_SIGN" = false ]; then
  if [ -n "${GPG_PRIVATE_KEY:-}" ]; then
    echo "${GPG_PRIVATE_KEY}" | gpg --batch --import 2>/dev/null || true
  fi

  GPG_KEY_ID=$(gpg --list-secret-keys --keyid-format long 2>/dev/null | grep sec | head -1 | awk '{print $2}' | cut -d'/' -f2)

  if [ -z "${GPG_KEY_ID}" ]; then
    echo "Error: No GPG secret key found. Set GPG_PRIVATE_KEY or use --no-sign"
    exit 1
  fi

  gpg --batch --yes --armor --detach-sign --output Release.gpg Release
  gpg --batch --yes --armor --clearsign --output InRelease Release

  gpg --batch --yes --armor --export "${GPG_KEY_ID}" > "${REPO_DIR}/key.gpg"

  echo "Repository signed with key ${GPG_KEY_ID}"
else
  echo "Warning: Repository is NOT signed (--no-sign)"
fi

echo "Repository updated in ${REPO_DIR}/"
find "${REPO_DIR}" -type f | sort | sed "s|${REPO_DIR}/||"
