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

@test "tmux_shell_command — reads @worktree-command option when set" {
    tmux() {
        if [[ "$*" == "show-option -gv @worktree-command" ]]; then
            echo "zsh"
        else
            exit 0
        fi
    }
    export -f tmux

    run tmux_shell_command

    assert_success
    assert_output "zsh"
}

@test "tmux_shell_command — falls back to SHELL when @worktree-command is empty" {
    tmux() {
        if [[ "$*" == "show-option -gv @worktree-command" ]]; then
            :
        else
            exit 0
        fi
    }
    getent() { return 1; }
    export -f tmux getent
    saved_shell=${SHELL:-}
    SHELL="/bin/bash"

    run tmux_shell_command

    SHELL="$saved_shell"

    assert_success
    assert_output "/bin/bash"
}
