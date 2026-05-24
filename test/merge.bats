#!/usr/bin/env bats

load 'test_helper/bats-support/load'
load 'test_helper/bats-assert/load'
load 'test_helper'

@test "is_merged — merged ancestor returns true" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    git checkout -b "test-feat" >/dev/null 2>&1
    echo "feature" > feat.txt
    git add feat.txt
    git commit -m "feature commit" >/dev/null 2>&1
    feat_head=$(git rev-parse HEAD)

    git checkout main >/dev/null 2>&1
    git merge "test-feat" --no-ff -m "merge test-feat" >/dev/null 2>&1
    git branch -f "origin/main" main >/dev/null 2>&1

    git checkout "$feat_head" >/dev/null 2>&1

    run is_merged "." "test-feat" "main" "true"

    git checkout main >/dev/null 2>&1
    git branch -D "test-feat" >/dev/null 2>&1
    git branch -D "origin/main" >/dev/null 2>&1
    teardown_temp_repo "$repo"

    assert_success
}

@test "is_merged — unmerged branch returns false" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    git checkout -b "test-unmerged" >/dev/null 2>&1
    echo "unmerged" > un.txt
    git add un.txt
    git commit -m "unmerged commit" >/dev/null 2>&1

    git checkout main >/dev/null 2>&1
    git branch -f "origin/main" main >/dev/null 2>&1

    git checkout "test-unmerged" >/dev/null 2>&1

    run is_merged "." "test-unmerged" "main" "false"

    git checkout main >/dev/null 2>&1
    git branch -D "test-unmerged" >/dev/null 2>&1
    git branch -D "origin/main" >/dev/null 2>&1
    teardown_temp_repo "$repo"

    assert_failure
}

@test "is_merged — squash merge is not an ancestor" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    git checkout -b "test-squash" >/dev/null 2>&1
    echo "squash" > sq.txt
    git add sq.txt
    git commit -m "squash commit" >/dev/null 2>&1

    git checkout main >/dev/null 2>&1
    git branch -f "origin/main" main >/dev/null 2>&1

    git merge --squash "test-squash" >/dev/null 2>&1
    git commit -m "squash merge" >/dev/null 2>&1
    git branch -f "origin/main" main >/dev/null 2>&1

    git checkout "test-squash" >/dev/null 2>&1

    run is_merged "." "test-squash" "main" "false"

    git checkout main >/dev/null 2>&1
    git branch -D "test-squash" >/dev/null 2>&1
    git branch -D "origin/main" >/dev/null 2>&1
    teardown_temp_repo "$repo"

    assert_failure
}

@test "is_merged — works with auto_fetch disabled" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    git checkout -b "test-no-auto" >/dev/null 2>&1
    echo "noauto" > na.txt
    git add na.txt
    git commit -m "no auto" >/dev/null 2>&1
    feat_head=$(git rev-parse HEAD)

    git checkout main >/dev/null 2>&1
    git merge "test-no-auto" --no-ff -m "merge no-auto" >/dev/null 2>&1
    git branch -f "origin/main" main >/dev/null 2>&1

    git checkout "$feat_head" >/dev/null 2>&1

    run is_merged "." "test-no-auto" "main" "false"

    git checkout main >/dev/null 2>&1
    git branch -D "test-no-auto" >/dev/null 2>&1
    git branch -D "origin/main" >/dev/null 2>&1
    teardown_temp_repo "$repo"

    assert_success
}

@test "is_merged — fast-forward merge detected" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    git checkout -b "test-ff" >/dev/null 2>&1
    echo "ff" > ff.txt
    git add ff.txt
    git commit -m "ff commit" >/dev/null 2>&1

    git checkout main >/dev/null 2>&1
    git merge "test-ff" >/dev/null 2>&1
    git branch -f "origin/main" main >/dev/null 2>&1

    git checkout "test-ff" >/dev/null 2>&1

    run is_merged "." "test-ff" "main" "true"

    git checkout main >/dev/null 2>&1
    git branch -D "test-ff" >/dev/null 2>&1
    git branch -D "origin/main" >/dev/null 2>&1
    teardown_temp_repo "$repo"

    assert_success
}

@test "is_merged — fails fast without network calls" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    # is_merged should complete instantly — no origin remote
    start=$(date +%s)
    run is_merged "." "some-branch" "main" "true"
    end=$(date +%s)
    elapsed=$((end - start))

    teardown_temp_repo "$repo"

    # Without origin/main ref the ancestry check fails → not merged
    assert_failure

    # Must complete quickly (no network calls)
    assert [ "$elapsed" -le 2 ]
}
