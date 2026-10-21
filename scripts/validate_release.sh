#!/usr/bin/env bash
set -euo pipefail

readonly RELEASE_BRANCH="${HEAD_REF:?HEAD_REF is required}"
readonly MANIFEST="${CARGO_MANIFEST:-Cargo.toml}"
readonly RELEASE_PATTERN='^release/v[0-9]+\.[0-9]+\.[0-9]+$'

if [[ ! "$RELEASE_BRANCH" =~ $RELEASE_PATTERN ]]; then
  printf 'Invalid release branch: %s\n' "$RELEASE_BRANCH" >&2
  exit 1
fi

version="${RELEASE_BRANCH#release/}"
manifest_version="$(sed -nE 's/^version[[:space:]]*=[[:space:]]*"([^"]+)"/\1/p' "$MANIFEST" | head -n 1)"
if [[ "$manifest_version" != "${version#v}" ]]; then
  printf 'Cargo.toml version %s does not match release %s\n' \
    "$manifest_version" "$version" >&2
  exit 1
fi

if git ls-remote --exit-code --refs origin "refs/tags/$version" >/dev/null 2>&1; then
  printf 'Release tag already exists: %s\n' "$version" >&2
  exit 1
fi

printf 'Validated release %s\n' "$version"
printf 'version=%s\n' "$version" >> "${GITHUB_OUTPUT:?GITHUB_OUTPUT is required}"
