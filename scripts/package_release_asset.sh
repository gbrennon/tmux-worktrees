#!/usr/bin/env bash
set -euo pipefail

readonly VERSION="${VERSION:?VERSION is required}"
readonly BINARY_NAME="tmux-worktrees"
readonly MANIFEST="${CARGO_MANIFEST:-Cargo.toml}"
readonly OUTPUT_DIR="${RELEASE_ASSET_DIR:-dist}"

platform_triple() {
  local arch os
  arch="$(uname -m)"
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  case "$arch" in
    x86_64) arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *) printf 'Unsupported architecture: %s\n' "$arch" >&2; exit 1 ;;
  esac
  case "$os" in
    linux) os="unknown-linux-gnu" ;;
    darwin) os="apple-darwin" ;;
    *) printf 'Unsupported operating system: %s\n' "$os" >&2; exit 1 ;;
  esac
  printf '%s-%s\n' "$arch" "$os"
}

triple="$(platform_triple)"
asset_name="${BINARY_NAME}-${triple}.tar.gz"
mkdir -p "$OUTPUT_DIR"
cargo build --release --locked --manifest-path "$MANIFEST"

binary_dir="$(dirname "$MANIFEST")/target/release"
if [[ ! -x "$binary_dir/$BINARY_NAME" ]]; then
  printf 'Release binary not found or not executable: %s\n' "$binary_dir/$BINARY_NAME" >&2
  exit 1
fi

tar -czf "$OUTPUT_DIR/$asset_name" -C "$binary_dir" "$BINARY_NAME"
printf 'Packaged %s for %s\n' "$OUTPUT_DIR/$asset_name" "$VERSION"
printf 'asset=%s\n' "$OUTPUT_DIR/$asset_name" >> "${GITHUB_OUTPUT:-/dev/null}"
