test_select_sources_cleanly() {
    local funcs
    funcs=$(declare -F | awk '{print $3}' | grep -E '^fzf_')
    local count
    count=$(echo "$funcs" | wc -l)
    assert_eq "2" "$count" "should define 2 fzf functions"
}

test_select_functions_exist() {
    assert_true type fzf_select_worktree >/dev/null 2>&1
    assert_true type fzf_cleanup_picker >/dev/null 2>&1
}
