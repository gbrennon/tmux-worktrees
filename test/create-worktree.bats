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
    display-message)
        if [[ "$2" == "-p" ]]; then
            echo "$(pwd)"
        else
            shift
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

    source "$SCRIPTS_DIR/create-worktree.sh"
    NOTIFY_MESSAGES_FILE="$MOCK_DIR/notify_messages"
    touch "$NOTIFY_MESSAGES_FILE"

    notify_stub() {
        echo "$*" >> "$MOCK_DIR/notify_messages"
    }

    source "$SCRIPTS_DIR/create-worktree.sh"
}

teardown() {
    rm -rf "$MOCK_DIR"
}

@test "create_worktree_and_open — handles empty branch name" {
    run create_worktree_and_open "" notify_stub

    assert_failure
    run cat "$MOCK_DIR/notify_messages"
    assert_output --partial "Branch name cannot be empty"
}

@test "create_worktree_and_open — shows status message when worktree already exists" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    mkdir -p ".worktrees/feat-test"

    run create_worktree_and_open "feat/test" notify_stub

    assert_success
    run cat "$MOCK_DIR/notify_messages"
    assert_output --partial "Worktree already exists: feat/test"

    teardown_temp_repo "$repo"
}

@test "create_worktree_and_open — shows status message when worktree creation fails" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    create_worktree() { echo "git error"; return 1; }

    run create_worktree_and_open "feat/fail" notify_stub

    assert_failure
    run cat "$MOCK_DIR/notify_messages"
    assert_output --partial "Worktree creation failed"

    teardown_temp_repo "$repo"
}

@test "create_worktree_and_open — creates worktree without notifying on success" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    run create_worktree_and_open "feat/test" notify_stub

    assert_success
    assert [ ! -s "$MOCK_DIR/notify_messages" ]

    teardown_temp_repo "$repo"
}
