test_resolve_from_global_config() {
    local repo old result
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    old=$(git config --global init.defaultBranch 2>/dev/null || true)
    git config --global init.defaultBranch "trunk"

    result=$(resolve_default_branch)

    if [[ -n "$old" ]]; then
        git config --global init.defaultBranch "$old"
    else
        git config --global --unset init.defaultBranch 2>/dev/null || true
    fi
    teardown_temp_repo "$repo"
    assert_eq "trunk" "$result"
}

test_resolve_from_local_config() {
    local repo result old_global
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    old_global=$(git config --global init.defaultBranch 2>/dev/null || true)
    git config --global --unset init.defaultBranch 2>/dev/null || true
    git config init.defaultBranch "develop"

    result=$(resolve_default_branch)

    git config --unset init.defaultBranch 2>/dev/null || true
    if [[ -n "$old_global" ]]; then
        git config --global init.defaultBranch "$old_global"
    fi
    teardown_temp_repo "$repo"
    assert_eq "develop" "$result"
}

test_resolve_global_takes_precedence() {
    local repo result old
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    old=$(git config --global init.defaultBranch 2>/dev/null || true)
    git config --global init.defaultBranch "global-main"
    git config init.defaultBranch "local-develop"

    result=$(resolve_default_branch)

    if [[ -n "$old" ]]; then
        git config --global init.defaultBranch "$old"
    else
        git config --global --unset init.defaultBranch 2>/dev/null || true
    fi
    git config --unset init.defaultBranch 2>/dev/null || true
    teardown_temp_repo "$repo"
    assert_eq "global-main" "$result" "global config should take precedence over local"
}

test_resolve_from_current_branch() {
    local repo result old_global
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    old_global=$(git config --global init.defaultBranch 2>/dev/null || true)
    git config --global --unset init.defaultBranch 2>/dev/null || true
    git config --unset init.defaultBranch 2>/dev/null || true

    git checkout -b "release" >/dev/null 2>&1

    result=$(resolve_default_branch)

    git checkout main >/dev/null 2>&1
    if [[ -n "$old_global" ]]; then
        git config --global init.defaultBranch "$old_global"
    fi
    teardown_temp_repo "$repo"
    assert_eq "release" "$result"
}

test_resolve_fallback_to_main() {
    local repo result old_global
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    old_global=$(git config --global init.defaultBranch 2>/dev/null || true)
    git config --global --unset init.defaultBranch 2>/dev/null || true
    git config --unset init.defaultBranch 2>/dev/null || true

    git checkout --detach main >/dev/null 2>&1

    result=$(resolve_default_branch)

    if [[ -n "$old_global" ]]; then
        git config --global init.defaultBranch "$old_global"
    fi
    teardown_temp_repo "$repo"
    assert_eq "main" "$result"
}

test_ensure_worktrees_dir_creates() {
    local repo
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    ensure_worktrees_dir ".worktrees"

    assert_true test -d ".worktrees"
    teardown_temp_repo "$repo"
}

test_ensure_worktrees_dir_adds_exclude() {
    local repo
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    ensure_worktrees_dir ".worktrees"

    assert_true grep -qxF ".worktrees/" .git/info/exclude
    teardown_temp_repo "$repo"
}

test_ensure_worktrees_dir_idempotent() {
    local repo before after
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    ensure_worktrees_dir ".worktrees"
    before=$(wc -l < .git/info/exclude)

    ensure_worktrees_dir ".worktrees"
    after=$(wc -l < .git/info/exclude)

    teardown_temp_repo "$repo"
    assert_eq "$before" "$after" "exclude line should not be duplicated"
}
