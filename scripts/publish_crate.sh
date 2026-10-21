#!/usr/bin/env bash
set -euo pipefail

readonly TOKEN="${CARGO_REGISTRY_TOKEN:-}"
readonly VERSION="${VERSION:?VERSION is required}"
readonly EXPECTED_VERSION="${VERSION#v}"

if [[ -z "$TOKEN" ]]; then
  printf 'CARGO_REGISTRY_TOKEN is not configured\n' >&2
  exit 1
fi

manifest_version="$(sed -nE 's/^version[[:space:]]*=[[:space:]]*"([^"]+)"/\1/p' Cargo.toml | head -n 1)"
if [[ "$manifest_version" != "$EXPECTED_VERSION" ]]; then
  printf 'Cargo.toml version %s does not match release %s\n' \
    "$manifest_version" "$VERSION" >&2
  exit 1
fi

cargo publish --locked
