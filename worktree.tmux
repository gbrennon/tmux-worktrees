#!/bin/bash

CURRENT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

WORKTREE_KEY="$(tmux show-option -gv @worktree-key 2>/dev/null)"
WORKTREE_KEY="${WORKTREE_KEY:-W}"

CLEANUP_KEY="$(tmux show-option -gv @worktree-cleanup-key 2>/dev/null)"
CLEANUP_KEY="${CLEANUP_KEY:-D}"

SCRIPT="$CURRENT_DIR/scripts/worktree-manager.sh"

# Main: fzf popup to select or create a worktree
tmux bind-key -T prefix "$WORKTREE_KEY" run-shell -b "$SCRIPT choose"

# Legacy: command-prompt to type a branch name directly
tmux bind-key -T prefix "M-$WORKTREE_KEY" command-prompt -p "Worktree branch:" \
    "run-shell -b '$SCRIPT' create-worktree '%%'"

# Cleanup: fzf popup to remove merged / stale worktrees
tmux bind-key -T prefix "$CLEANUP_KEY" run-shell -b "$SCRIPT cleanup"
