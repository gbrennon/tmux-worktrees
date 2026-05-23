#!/usr/bin/env bats

load 'test_helper/bats-support/load'
load 'test_helper/bats-assert/load'
load 'test_helper'

@test "tmux library — sources cleanly and defines the expected functions" {
    # test_helper already sources tmux.sh — verify functions exist
    run type show_error
    assert_success

    run type tmux_shell_command
    assert_success

    run type tmux_select_or_create_window
    assert_success

    run type tmux_kill_window
    assert_success
}

@test "tmux_shell_command — returns a non-empty shell path" {
    run tmux_shell_command

    assert_success
    refute_output ""
}

@test "tmux_shell_command — does not crash without SHELL set" {
    saved_shell=${SHELL:-}
    unset SHELL

    run tmux_shell_command

    SHELL="$saved_shell"

    assert_success
    refute_output ""
}
