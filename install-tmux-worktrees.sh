#!/bin/bash

# Install tmux-worktrees as a tmux plugin

set -euo pipefail

PLUGIN_NAME="tmux-worktrees"
PLUGIN_DIR="$HOME/.tmux/plugins/$PLUGIN_NAME"
REPO_DIR="$(dirname "$(realpath "$0")")"
TMUX_CONF="$HOME/.tmux.conf"
MARKER="# tmux-worktrees plugin"

mkdir -p "$PLUGIN_DIR"

echo "Building $PLUGIN_NAME..."
cargo build --release --manifest-path "$REPO_DIR/Cargo.toml" --quiet

echo "Configuring tmux..."
if [ -f "$TMUX_CONF" ]; then
    tmpfile=$(mktemp)
    grep -vF "$MARKER" "$TMUX_CONF" > "$tmpfile" || true
    grep -vE '^\s*(set -g @worktree-dir|run-shell).*tmux-worktrees' "$tmpfile" > "${tmpfile}.2" || true
    mv "${tmpfile}.2" "$TMUX_CONF"
    rm -f "$tmpfile"
fi

cat >> "$TMUX_CONF" <<EOF

# tmux-worktrees plugin
set -g @worktree-dir ".worktrees"

# Keybindings
run-shell "$REPO_DIR/worktree.tmux"
EOF

echo "Reloading tmux..."
tmux source-file "$TMUX_CONF" 2>/dev/null || true

echo "Verifying installation..."
if [ -x "$REPO_DIR/target/release/tmux-worktrees" ]; then
    echo "$PLUGIN_NAME installed successfully."
    echo "Press 'prefix + W' to create/resume a workspace."
    echo "Press 'prefix + D' to clean up workspaces."
else
    echo "Failed to install $PLUGIN_NAME."
    exit 1
fi