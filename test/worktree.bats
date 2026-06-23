#!/usr/bin/env bats

load 'test_helper/bats-support/load'
load 'test_helper/bats-assert/load'
load 'test_helper'

@test "worktree_abspath — resolves path with realpath" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    run worktree_abspath "."

    teardown_temp_repo "$repo"

    assert_success
    assert_output "$repo"
}

@test "worktree_abspath — resolves subdirectory" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    mkdir -p "sub"
    run worktree_abspath "sub"

    teardown_temp_repo "$repo"

    assert_success
    assert_output "$repo/sub"
}

@test "worktree_branch — returns current branch in repo" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    run worktree_branch "."

    teardown_temp_repo "$repo"

    assert_success
    assert_output "main"
}

@test "worktree_branch — returns '?' for non-existent path" {
    run worktree_branch "/tmp/non-existent-path-xxxxx" 2>/dev/null

    assert_success
    assert_output "?"
}

@test "list_worktrees — empty when no worktrees" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    mkdir -p ".worktrees"
    run list_worktrees ".worktrees"

    teardown_temp_repo "$repo"

    assert_success
    assert_output ""
}

@test "create_worktree and remove_worktree — basic cycle" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    mkdir -p ".worktrees"
    output=$(create_worktree ".worktrees/test-feat" "test-feat" "main" 2>&1) || {
        teardown_temp_repo "$repo"
        skip "git worktree add failed: $output"
    }

    run test -d ".worktrees/test-feat"
    assert_success

    run worktree_branch ".worktrees/test-feat"
    assert_success
    assert_output "test-feat"

    remove_worktree ".worktrees/test-feat" >/dev/null 2>&1

    refute test -d ".worktrees/test-feat"
    teardown_temp_repo "$repo"
}

@test "remove_worktree — force removes dirty worktree" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    mkdir -p ".worktrees"
    output=$(create_worktree ".worktrees/dirty-feat" "dirty-feat" "main" 2>&1) || {
        teardown_temp_repo "$repo"
        skip "git worktree add failed: $output"
    }

    echo "uncommitted" >> ".worktrees/dirty-feat/init.txt"

    run remove_worktree ".worktrees/dirty-feat"
    assert_success

    refute test -d ".worktrees/dirty-feat"
    teardown_temp_repo "$repo"
}

@test "list_worktrees — returns entries when worktree exists" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    mkdir -p ".worktrees"
    output=$(create_worktree ".worktrees/test-feat" "test-feat" "main" 2>&1) || {
        teardown_temp_repo "$repo"
        skip "git worktree add failed: $output"
    }

    run list_worktrees ".worktrees"

    remove_worktree ".worktrees/test-feat" >/dev/null 2>&1
    teardown_temp_repo "$repo"

    assert_success
    refute_output ""
}

@test "delete_local_branch — removes an existing branch" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    git checkout -b "to-delete" >/dev/null 2>&1
    git checkout main >/dev/null 2>&1

    git show-ref --verify --quiet refs/heads/to-delete
    assert_equal "$?" "0" "branch should exist before deletion"

    run delete_local_branch "to-delete"
    assert_success
    assert_output ""

    run git show-ref --verify --quiet refs/heads/to-delete
    assert_failure "branch should NOT exist after deletion"

    teardown_temp_repo "$repo"
}

@test "delete_local_branch — does not error on non-existent branch" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    run delete_local_branch "non-existent-branch"
    assert_success
    assert_output ""

    teardown_temp_repo "$repo"
}

@test "list_worktrees — works with custom directory name" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    mkdir -p "custom-wt"
    output=$(create_worktree "custom-wt/test-feat" "test-feat" "main" 2>&1) || {
        teardown_temp_repo "$repo"
        skip "git worktree add failed: $output"
    }

    run list_worktrees "custom-wt"

    remove_worktree "custom-wt/test-feat" >/dev/null 2>&1
    teardown_temp_repo "$repo"

    assert_success
    refute_output ""
}

@test "create_worktree and remove_worktree — with custom directory name" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    mkdir -p "custom-wt"
    output=$(create_worktree "custom-wt/test-feat" "test-feat" "main" 2>&1) || {
        teardown_temp_repo "$repo"
        skip "git worktree add failed: $output"
    }

    assert test -d "custom-wt/test-feat"

    remove_worktree "custom-wt/test-feat" >/dev/null 2>&1
    refute test -d "custom-wt/test-feat"
    teardown_temp_repo "$repo"
}

@test "list_worktree_names — works with custom directory name" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    mkdir -p "custom-wt"
    output=$(create_worktree "custom-wt/test-feat" "test-feat" "main" 2>&1) || {
        teardown_temp_repo "$repo"
        skip "git worktree add failed: $output"
    }

    run list_worktree_names "custom-wt"

    remove_worktree "custom-wt/test-feat" >/dev/null 2>&1
    teardown_temp_repo "$repo"

    assert_success
    assert_output "test-feat"
}

@test "remove_worktree with branch — calls tmux_kill_window" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    mkdir -p ".worktrees"
    output=$(create_worktree ".worktrees/test-feat" "test-feat" "main" 2>&1) || {
        teardown_temp_repo "$repo"
        skip "git worktree add failed: $output"
    }

    local sentinel
    sentinel=$(mktemp)

    # Override tmux_kill_window to record the call
    tmux_kill_window() { echo "$1" > "$sentinel"; }
    export -f tmux_kill_window

    run remove_worktree ".worktrees/test-feat" "test-feat"

    assert_success
    run cat "$sentinel"
    assert_output "test-feat"
    refute test -d ".worktrees/test-feat"
    rm -f "$sentinel"
    teardown_temp_repo "$repo"
}

@test "remove_worktree without branch — does not call tmux_kill_window" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    mkdir -p ".worktrees"
    output=$(create_worktree ".worktrees/test-feat" "test-feat" "main" 2>&1) || {
        teardown_temp_repo "$repo"
        skip "git worktree add failed: $output"
    }

    local sentinel
    sentinel=$(mktemp)

    # Override tmux_kill_window to record any call
    tmux_kill_window() { echo "unexpected-call" > "$sentinel"; }
    export -f tmux_kill_window

    remove_worktree ".worktrees/test-feat" >/dev/null 2>&1

    run test -s "$sentinel"
    assert_failure
    refute test -d ".worktrees/test-feat"
    rm -f "$sentinel"
    teardown_temp_repo "$repo"
}

@test "remove_worktree with branch — still removes worktree if tmux not found" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    mkdir -p ".worktrees"
    output=$(create_worktree ".worktrees/test-feat" "test-feat" "main" 2>&1) || {
        teardown_temp_repo "$repo"
        skip "git worktree add failed: $output"
    }

    # Override tmux to simulate a missing tmux server
    tmux() { return 1; }
    export -f tmux

    run remove_worktree ".worktrees/test-feat" "test-feat"

    assert_success
    refute test -d ".worktrees/test-feat"
    teardown_temp_repo "$repo"
}
