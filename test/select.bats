#!/usr/bin/env bats

load 'test_helper/bats-support/load'
load 'test_helper/bats-assert/load'
load 'test_helper'

@test "select library — sources cleanly and defines fzf functions" {
    # test_helper already sources select.sh — verify functions exist
    run type fzf_select_worktree
    assert_success

    run type fzf_cleanup_picker
    assert_success
}

@test "fzf_select_worktree — handles empty input safely" {
    # fzf_select_worktree calls fzf-tmux which won't be available; stub it
    fzf-tmux() { echo ""; }
    export -f fzf-tmux

    run fzf_select_worktree ""

    # Should not crash or error — it may return empty string, which is fine
    assert_success
}

@test "fzf_cleanup_picker — handles empty input safely" {
    # fzf_cleanup_picker calls fzf-tmux which won't be available; stub it
    fzf-tmux() { echo ""; }
    export -f fzf-tmux

    run fzf_cleanup_picker ""

    # Should not crash or error
    assert_success
}
