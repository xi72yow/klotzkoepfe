#!/bin/bash
set -euo pipefail

PKG_NAME="klotzkoepfe"
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
REVISION="${1:-1}"
FULL_VERSION="${VERSION}-${REVISION}"

CHANGELOG="debian/changelog"
mkdir -p debian

# Find last tag to only include new commits
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

{
  echo "${PKG_NAME} (${FULL_VERSION}) stable; urgency=low"
  echo ""
  if [ -n "$LAST_TAG" ]; then
    git log "${LAST_TAG}..HEAD" --pretty=format:"  * %s" --no-merges
  else
    git log --pretty=format:"  * %s" --no-merges
  fi
  echo ""
  echo ""
  AUTHOR=$(git log -1 --pretty=format:"%an <%ae>")
  DATE=$(date -R)
  echo " -- ${AUTHOR}  ${DATE}"
} > "${CHANGELOG}"

echo "Generated ${CHANGELOG}"
