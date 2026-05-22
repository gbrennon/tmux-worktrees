test_tmux_sources_cleanly() {
    local funcs
    funcs=$(declare -F | awk '{print $3}' | grep -E '^show_error$|^tmux_')
    local count
    count=$(echo "$funcs" | wc -l)
    assert_eq "4" "$count" "should define show_error + 3 tmux_* functions"
}

test_tmux_shell_command_default() {
    local command
    command=$(tmux_shell_command)
    assert_neq "" "$command" "should return a shell path"
}

test_tmux_shell_command_falls_back_to_bash() {
    local saved_shell
    saved_shell=${SHELL:-}
    unset SHELL

    local command
    command=$(tmux_shell_command 2>/dev/null)

    SHELL="$saved_shell"
    assert_neq "" "$command" "should not crash without SHELL"
}
