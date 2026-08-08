#!/bin/bash
enable_strict_mode

# tmux-worktrees orchestrator
# This script is sourced by tpm/tpack and will build/install the Rust binary on first use

set -euo pipefail

CURRENT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_NAME="tmux-worktrees"
PLUGIN_DIR="$HOME/.tmux/plugins/$PLUGIN_NAME"
BINARY_NAME="tmux-worktrees"
BINARY_PATH="$PLUGIN_DIR/$BINARY_NAME"
REPO_DIR="$CURRENT_DIR"
CARGO_MANIFEST="$REPO_DIR/Cargo.toml"
GITHUB_REPO="gbrennon/tmux-worktrees"

# Read key bindings from tmux options (with defaults)
WORKTREE_KEY="$(tmux show-option -gv @worktree-key 2>/dev/null || echo "W")"
CLEANUP_KEY="$(tmux show-option -gv @worktree-cleanup-key 2>/dev/null || echo "D")"

enable_strict_mode() {
    set -euo pipefail
}

ensure_plugin_dir() {
    mkdir -p "$PLUGIN_DIR"
}

get_latest_release_version() {
    curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/latest" \
        | grep '"tag_name":' \
        | sed -E 's/.*"([^"]+)".*/\1/'
}

download_prebuilt_binary() {
    local version="$1"
    local arch, os
    arch="$(uname -m)"
    os="$(uname -s | tr '[:upper:]' '[:lower:]')"

    case "$arch" in
        x86_64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) return 1 ;;
    esac

    case "$os" in
        linux) os="unknown-linux-gnu" ;;
        darwin) os="apple-darwin" ;;
        *) return 1 ;;
    esac

    local asset_name="${BINARY_NAME}-${arch}-${os}.tar.gz"
    local download_url="https://github.com/${GITHUB_REPO}/releases/download/${version}/${asset_name}"

    echo "Downloading pre-built binary (${version})..." >&2

    local temp_dir
    temp_dir="$(mktemp -d)"
    trap 'rm -rf "$temp_dir"' RETURN

    if curl -fsSL "$download_url" -o "$temp_dir/$asset_name"; then
        tar -xzf "$temp_dir/$asset_name" -C "$temp_dir"
        if [[ -x "$temp_dir/$BINARY_NAME" ]]; then
            cp "$temp_dir/$BINARY_NAME" "$BINARY_PATH"
            chmod +x "$BINARY_PATH"
            return 0
        fi
    fi

    return 1
}

build_from_source() {
    if ! command -v cargo >/dev/null 2>&1; then
        echo "Error: cargo not found. Please install Rust toolchain (https://rustup.rs/) or ensure a pre-built binary is available for your platform." >&2
        return 1
    fi

    echo "Building $PLUGIN_NAME from source..." >&2
    cargo build --release --manifest-path "$CARGO_MANIFEST" --quiet
    cp "$REPO_DIR/target/release/$BINARY_NAME" "$BINARY_PATH"
    chmod +x "$BINARY_PATH"
}

build_binary_if_needed() {
    if [[ -x "$BINARY_PATH" ]] && [[ "$REPO_DIR/src/main.rs" -ot "$BINARY_PATH" ]] && [[ "$CARGO_MANIFEST" -ot "$BINARY_PATH" ]]; then
        return 0
    fi

    local version
    version="$(get_latest_release_version 2>/dev/null || true)"

    if [[ -n "$version" ]] && download_prebuilt_binary "$version"; then
        return 0
    fi

    build_from_source
}

unbind_stale_keys() {
    for key in $(tmux list-keys -T prefix 2>/dev/null | grep -F "$BINARY_NAME" | awk '{print $4}'); do
        tmux unbind-key -T prefix "$key" 2>/dev/null || true
    done
}

bind_keys() {
    tmux bind-key -T prefix "$WORKTREE_KEY" run-shell -b "$BINARY_PATH choose"
    tmux bind-key -T prefix "$CLEANUP_KEY" run-shell -b "$BINARY_PATH cleanup"
}

main() {
    ensure_plugin_dir
    build_binary_if_needed
    unbind_stale_keys
    bind_keys
}

main
