#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TAG="${1:-}"
if [[ -z "$TAG" ]]; then
  echo "Usage: $0 vX.Y.Z" >&2
  exit 1
fi

if [[ ! "$TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Tag must look like v0.1.0, got: $TAG" >&2
  exit 1
fi

VERSION="${TAG#v}"
CRATE_VERSION="$(grep '^version = ' Cargo.toml | head -n1 | sed 's/version = "\(.*\)"/\1/')"
if [[ "$VERSION" != "$CRATE_VERSION" ]]; then
  echo "Tag version ($VERSION) does not match Cargo.toml ($CRATE_VERSION)" >&2
  exit 1
fi

BODY="$(awk -v version="$VERSION" '
  $0 ~ "^#+ \\[" version "\\]" { capture=1; next }
  capture && (/^#+ \[/ || /^\[.*\]: http/) { exit }
  capture { print }
' CHANGELOG.md)"

if [[ -z "$BODY" ]]; then
  echo "No CHANGELOG section found for version $VERSION" >&2
  exit 1
fi

echo "Release $TAG looks valid."
echo
echo "Changelog preview:"
echo "$BODY"
