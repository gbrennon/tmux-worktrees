#!/bin/bash

CURRENT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

SCRIPT="$CURRENT_DIR/scripts/worktree-manager.sh"

# Read key bindings from tmux options (with defaults)
# Users can override by setting these before the plugin line in ~/.tmux.conf:
#   set -g @worktree-key 't'
#   set -g @worktree-cleanup-key 'x'
WORKTREE_KEY="$(tmux show-option -gv @worktree-key 2>/dev/null)"
WORKTREE_KEY="${WORKTREE_KEY:-W}"

CLEANUP_KEY="$(tmux show-option -gv @worktree-cleanup-key 2>/dev/null)"
CLEANUP_KEY="${CLEANUP_KEY:-D}"

# Main: fzf popup to select or create a worktree (prefix + W by default)
tmux bind-key -T prefix "$WORKTREE_KEY" run-shell -b "$SCRIPT choose"

# Cleanup: fzf popup to remove merged / stale worktrees (prefix + D by default)
tmux bind-key -T prefix "$CLEANUP_KEY" run-shell -b "$SCRIPT cleanup"
