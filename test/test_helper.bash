#!/bin/bash

# Path to the project root
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPTS_DIR="$PROJECT_DIR/scripts"

# Source all library scripts
source "$SCRIPTS_DIR/lib/repo.sh"
source "$SCRIPTS_DIR/lib/worktree.sh"
source "$SCRIPTS_DIR/lib/merge.sh"
source "$SCRIPTS_DIR/lib/tmux.sh"
source "$SCRIPTS_DIR/lib/select.sh"

# Load bats libraries via 'load' — bats_load_library requires bats-support pre-loaded,
# so we use the path-based load approach from each .bats file instead.

# ── Shared test helpers ─────────────────────────────────────────────

setup_temp_repo() {
    local dir
    dir=$(mktemp -d)
    cd "$dir" || return 1
    git init --initial-branch=main >/dev/null 2>&1
    git config user.email "test@test.com"
    git config user.name "Test"
    echo "init" > init.txt
    git add init.txt
    git commit -m "initial" >/dev/null 2>&1
    echo "$dir"
}

teardown_temp_repo() {
    local dir="$1"
    if [[ -d "$dir" ]]; then
        local in_dir
        in_dir=$(pwd 2>/dev/null || echo "/tmp")
        if [[ "$in_dir" == "$dir"* ]]; then
            cd /tmp 2>/dev/null || true
        fi
        if [[ -d "$dir/.git" ]]; then
            cd "$dir" 2>/dev/null || true
            git worktree list 2>/dev/null | tail -n +2 | awk '{print $1}' | while read -r wt; do
                git worktree remove "$wt" 2>/dev/null || true
            done
            cd /tmp 2>/dev/null || true
        fi
        rm -rf "$dir"
    fi
}
