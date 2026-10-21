#!/usr/bin/env bash
set -euo pipefail

readonly VERSION="${VERSION:?VERSION is required}"

if git ls-remote --exit-code --refs origin "refs/tags/$VERSION" >/dev/null 2>&1; then
  printf 'Release tag already exists remotely: %s\n' "$VERSION"
  exit 0
fi

git tag "$VERSION"
git push origin "refs/tags/$VERSION"
