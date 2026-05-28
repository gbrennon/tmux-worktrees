#!/usr/bin/env bats

load 'test_helper/bats-support/load'
load 'test_helper/bats-assert/load'
load 'test_helper'

setup() {
    MOCK_DIR=$(mktemp -d)
    TMUX_MESSAGES_FILE="$MOCK_DIR/tmux_messages"
    TMUX_NEW_WINDOW_FILE="$MOCK_DIR/tmux_new_window"
    touch "$TMUX_MESSAGES_FILE" "$TMUX_NEW_WINDOW_FILE"

    cat > "$MOCK_DIR/tmux" <<'MOCK'
#!/bin/bash
MOCK_DIR="$(dirname "$0")"
case "$1" in
    show-environment)
        echo "MAIN_PROJECT_PATH="
        ;;
    display-popup)
        # Extract tmpfile path from command string and read its contents
        local cmd="$*"
        local tmpfile
        tmpfile=$(echo "$cmd" | grep -oP "cat '\K[^']+")
        if [[ -n "$tmpfile" && -f "$tmpfile" ]]; then
            cat "$tmpfile" >> "$MOCK_DIR/tmux_messages"
        fi
        ;;
    display-message)
        if [[ "$2" == "-p" ]]; then
            echo "$(pwd)"
        else
            shift
            # Skip -d flag and its argument if present
            while [[ $# -gt 0 && "$1" == "-d" ]]; do
                shift 2
            done
            echo "$*" >> "$MOCK_DIR/tmux_messages"
        fi
        ;;
    list-windows)
        echo ""
        ;;
    new-window)
        shift
        echo "$*" >> "$MOCK_DIR/tmux_new_window"
        ;;
    show-option)
        echo ""
        ;;
esac
MOCK
    chmod +x "$MOCK_DIR/tmux"
    export PATH="$MOCK_DIR:$PATH"

    MANAGER="$SCRIPTS_DIR/worktree-manager.sh"
}

teardown() {
    rm -rf "$MOCK_DIR"
}

@test "worktree-manager — create command creates worktree and opens window" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    run bash "$MANAGER" create "feat/test"

    assert_success
    assert [ -d "$repo/.worktrees/feat-test" ]
    run cat "$TMUX_NEW_WINDOW_FILE"
    assert_output --partial "wt-feat/test"

    teardown_temp_repo "$repo"
}

@test "worktree-manager — create command shows message when worktree exists" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    mkdir -p ".worktrees/feat-test"

    run bash "$MANAGER" create "feat/test"

    assert_success
    run cat "$TMUX_MESSAGES_FILE"
    assert_output --partial "Worktree already exists: feat/test"

    teardown_temp_repo "$repo"
}

@test "worktree-manager — create command handles empty branch" {
    run bash "$MANAGER" create ""

    assert_failure
    run cat "$TMUX_MESSAGES_FILE"
    assert_output --partial "Branch name cannot be empty"
}

@test "worktree-manager — resume command opens new window when worktree exists" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    create_worktree ".worktrees/feat-test" "feat/test" "main" >/dev/null 2>&1

    run bash "$MANAGER" resume "feat/test"

    assert_success
    run cat "$TMUX_MESSAGES_FILE"
    assert_output --partial "Opened worktree: feat/test"
    run cat "$TMUX_NEW_WINDOW_FILE"
    assert_output --partial "wt-feat/test"

    teardown_temp_repo "$repo"
}

@test "worktree-manager — resume command shows message when worktree not found" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    run bash "$MANAGER" resume "feat/nonexistent"

    assert_failure
    run cat "$TMUX_MESSAGES_FILE"
    assert_output --partial "Worktree not found: feat/nonexistent"

    teardown_temp_repo "$repo"
}

@test "worktree-manager — resume command handles empty branch" {
    run bash "$MANAGER" resume ""

    assert_failure
    run cat "$TMUX_MESSAGES_FILE"
    assert_output --partial "Branch name cannot be empty"
}

@test "worktree-manager — unknown command shows error" {
    run bash "$MANAGER" unknown-cmd 2>&1

    assert_success
    assert_output --partial "Unknown command: unknown-cmd"
}
