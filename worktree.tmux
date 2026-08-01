#!/bin/bash

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

# Read key bindings from tmux options (with defaults)
WORKTREE_KEY="$(tmux show-option -gv @worktree-key 2>/dev/null || echo "W")"
CLEANUP_KEY="$(tmux show-option -gv @worktree-cleanup-key 2>/dev/null || echo "D")"

# Ensure plugin directory exists
mkdir -p "$PLUGIN_DIR"

# Build and install the Rust binary if not present or outdated
if [[ ! -x "$BINARY_PATH" ]] || [[ "$REPO_DIR/src/main.rs" -nt "$BINARY_PATH" ]] || [[ "$CARGO_MANIFEST" -nt "$BINARY_PATH" ]]; then
    echo "Building $PLUGIN_NAME..." >&2
    cargo build --release --manifest-path "$CARGO_MANIFEST" --quiet
    cp "$REPO_DIR/target/release/$BINARY_NAME" "$BINARY_PATH"
    chmod +x "$BINARY_PATH"
fi

# Unbind stale keys from previous loads
for key in $(tmux list-keys -T prefix 2>/dev/null | grep -F "$BINARY_NAME" | awk '{print $4}'); do
    tmux unbind-key -T prefix "$key" 2>/dev/null || true
done

# Bind keys to the Rust binary
tmux bind-key -T prefix "$WORKTREE_KEY" run-shell -b "$BINARY_PATH choose"
tmux bind-key -T prefix "$CLEANUP_KEY" run-shell -b "$BINARY_PATH cleanup"