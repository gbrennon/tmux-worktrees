test_is_merged_ancestor_returns_true() {
    local repo result feat_head
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

    is_merged "." "test-feat" "main" "true"
    result=$?

    git checkout main >/dev/null 2>&1
    git branch -D "test-feat" >/dev/null 2>&1
    git branch -D "origin/main" >/dev/null 2>&1
    teardown_temp_repo "$repo"
    assert_eq "0" "$result" "merged branch should return 0"
}

test_is_merged_not_ancestor_returns_false() {
    local repo result
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    git checkout -b "test-unmerged" >/dev/null 2>&1
    echo "unmerged" > un.txt
    git add un.txt
    git commit -m "unmerged commit" >/dev/null 2>&1

    git checkout main >/dev/null 2>&1
    git branch -f "origin/main" main >/dev/null 2>&1

    git checkout "test-unmerged" >/dev/null 2>&1

    is_merged "." "test-unmerged" "main" "false"
    result=$?

    git checkout main >/dev/null 2>&1
    git branch -D "test-unmerged" >/dev/null 2>&1
    git branch -D "origin/main" >/dev/null 2>&1
    teardown_temp_repo "$repo"
    assert_eq "1" "$result" "unmerged branch should return 1"
}

test_is_merged_squash_not_ancestor() {
    local repo result
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

    is_merged "." "test-squash" "main" "false"
    result=$?

    git checkout main >/dev/null 2>&1
    git branch -D "test-squash" >/dev/null 2>&1
    git branch -D "origin/main" >/dev/null 2>&1
    teardown_temp_repo "$repo"
    assert_eq "1" "$result" "squash merge is not an ancestor"
}

test_is_merged_with_auto_fetch_disabled() {
    local repo result feat_head
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

    is_merged "." "test-no-auto" "main" "false"
    result=$?

    git checkout main >/dev/null 2>&1
    git branch -D "test-no-auto" >/dev/null 2>&1
    git branch -D "origin/main" >/dev/null 2>&1
    teardown_temp_repo "$repo"
    assert_eq "0" "$result" "should be detected via ancestry even with auto_fetch disabled"
}

test_is_merged_fast_forward_detected() {
    local repo result
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

    is_merged "." "test-ff" "main" "true"
    result=$?

    git checkout main >/dev/null 2>&1
    git branch -D "test-ff" >/dev/null 2>&1
    git branch -D "origin/main" >/dev/null 2>&1
    teardown_temp_repo "$repo"
    assert_eq "0" "$result" "fast-forward merge should be detected"
}
