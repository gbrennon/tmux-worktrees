#!/usr/bin/env bats

load 'test_helper/bats-support/load'
load 'test_helper/bats-assert/load'
load 'test_helper'

@test "resolve_default_branch — from global config" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    old=$(git config --global init.defaultBranch 2>/dev/null || true)
    git config --global init.defaultBranch "trunk"

    run resolve_default_branch

    if [[ -n "$old" ]]; then
        git config --global init.defaultBranch "$old"
    else
        git config --global --unset init.defaultBranch 2>/dev/null || true
    fi
    teardown_temp_repo "$repo"

    assert_success
    assert_output "trunk"
}

@test "resolve_default_branch — from local config" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    old_global=$(git config --global init.defaultBranch 2>/dev/null || true)
    git config --global --unset init.defaultBranch 2>/dev/null || true
    git config init.defaultBranch "develop"

    run resolve_default_branch

    git config --unset init.defaultBranch 2>/dev/null || true
    if [[ -n "$old_global" ]]; then
        git config --global init.defaultBranch "$old_global"
    fi
    teardown_temp_repo "$repo"

    assert_success
    assert_output "develop"
}

@test "resolve_default_branch — global takes precedence over local" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    old=$(git config --global init.defaultBranch 2>/dev/null || true)
    git config --global init.defaultBranch "global-main"
    git config init.defaultBranch "local-develop"

    run resolve_default_branch

    if [[ -n "$old" ]]; then
        git config --global init.defaultBranch "$old"
    else
        git config --global --unset init.defaultBranch 2>/dev/null || true
    fi
    git config --unset init.defaultBranch 2>/dev/null || true
    teardown_temp_repo "$repo"

    assert_success
    assert_output "global-main"
}

@test "resolve_default_branch — from current branch" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    old_global=$(git config --global init.defaultBranch 2>/dev/null || true)
    git config --global --unset init.defaultBranch 2>/dev/null || true
    git config --unset init.defaultBranch 2>/dev/null || true

    git checkout -b "release" >/dev/null 2>&1

    run resolve_default_branch

    git checkout main >/dev/null 2>&1
    if [[ -n "$old_global" ]]; then
        git config --global init.defaultBranch "$old_global"
    fi
    teardown_temp_repo "$repo"

    assert_success
    assert_output "release"
}

@test "resolve_default_branch — fallback to main" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    old_global=$(git config --global init.defaultBranch 2>/dev/null || true)
    git config --global --unset init.defaultBranch 2>/dev/null || true
    git config --unset init.defaultBranch 2>/dev/null || true

    git checkout --detach main >/dev/null 2>&1

    run resolve_default_branch

    if [[ -n "$old_global" ]]; then
        git config --global init.defaultBranch "$old_global"
    fi
    teardown_temp_repo "$repo"

    assert_success
    assert_output "main"
}

@test "ensure_worktrees_dir — creates directory" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    ensure_worktrees_dir ".worktrees"

    assert test -d "$repo/.worktrees"
    teardown_temp_repo "$repo"
}

@test "ensure_worktrees_dir — adds to .git/info/exclude" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    ensure_worktrees_dir ".worktrees"

    run grep -qxF ".worktrees/" .git/info/exclude

    teardown_temp_repo "$repo"
    assert_success
}

@test "ensure_worktrees_dir — idempotent (no duplicate exclude lines)" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    ensure_worktrees_dir ".worktrees"
    before=$(wc -l < .git/info/exclude)

    ensure_worktrees_dir ".worktrees"
    after=$(wc -l < .git/info/exclude)

    teardown_temp_repo "$repo"
    assert_equal "$before" "$after"
}
