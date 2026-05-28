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

    NOTIFY_MESSAGES_FILE="$MOCK_DIR/notify_messages"
    touch "$NOTIFY_MESSAGES_FILE"

    notify_stub() {
        echo "$*" >> "$MOCK_DIR/notify_messages"
    }

    source "$SCRIPTS_DIR/resume-worktree.sh"
}

teardown() {
    rm -rf "$MOCK_DIR"
}

@test "resume_worktree — sources cleanly and defines the function" {
    run type resume_worktree
    assert_success
}

@test "resume_worktree — opens new window when worktree exists" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    create_worktree ".worktrees/feat-test" "feat/test" "main" >/dev/null 2>&1

    run resume_worktree "feat/test" notify_stub

    assert_success
    run cat "$TMUX_MESSAGES_FILE"
    assert_output --partial "Opened worktree: feat/test"
    run cat "$TMUX_NEW_WINDOW_FILE"
    assert_output --partial "wt-feat/test"
    assert [ ! -s "$MOCK_DIR/notify_messages" ]

    teardown_temp_repo "$repo"
}

@test "resume_worktree — shows status message when worktree doesn't exist" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    run resume_worktree "feat/nonexistent" notify_stub

    assert_failure
    run cat "$MOCK_DIR/notify_messages"
    assert_output --partial "Worktree not found: feat/nonexistent"

    teardown_temp_repo "$repo"
}

@test "resume_worktree — handles empty branch name" {
    run resume_worktree "" notify_stub

    assert_failure
    run cat "$MOCK_DIR/notify_messages"
    assert_output --partial "Branch name cannot be empty"
}

@test "resume_worktree — handles missing git repo" {
    local dir
    dir=$(mktemp -d)
    cd "$dir" || return 1

    run resume_worktree "feat/test" notify_stub

    rm -rf "$dir"

    assert_failure
    run cat "$MOCK_DIR/notify_messages"
    assert_output --partial "Not in a git repository"
}
