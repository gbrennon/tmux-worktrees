#!/usr/bin/env bats

load 'test_helper/bats-support/load'
load 'test_helper/bats-assert/load'
load 'test_helper'

create_mock_tmux() {
    local mock_dir worktree_dir auto_fetch
    worktree_dir="$1"
    auto_fetch="${2:-}"
    mock_dir=$(mktemp -d)

    cat > "$mock_dir/tmux" << MOCK
#!/bin/bash
case "\$*" in
    "show-environment -g MAIN_PROJECT_PATH")
        exit 1
        ;;
    "display-message -p #{pane_current_path}")
        echo "\$REPO_PATH"
        ;;
    "show-option -gv @worktree-dir")
        echo "$worktree_dir"
        ;;
    "show-option -gv @worktree-command")
        ;;
    "show-option -gv @worktree-auto-fetch")
        echo "$auto_fetch"
        ;;
    "list-windows -F #{window_name}")
        ;;
    new-window\ *)
        exit 0
        ;;
    kill-window\ *)
        exit 0
        ;;
    display-message\ *)
        exit 0
        ;;
    *)
        exit 1
        ;;
esac
MOCK
    chmod +x "$mock_dir/tmux"

    cat > "$mock_dir/fzf-tmux" << 'MOCK'
#!/bin/bash
exit 0
MOCK
    chmod +x "$mock_dir/fzf-tmux"

    echo "$mock_dir"
}

setup() {
    export GIT_CONFIG_GLOBAL=$(mktemp)
}

teardown() {
    rm -f "${GIT_CONFIG_GLOBAL:-}"
}

@test "manager — uses custom directory from @worktree-dir option" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    mock_dir=$(create_mock_tmux "custom-dir")

    REPO_PATH="$repo" \
        PATH="$mock_dir:$PATH" \
        bash "$PROJECT_DIR/scripts/worktree-manager.sh" create-worktree "my-feat"

    assert test -f "$repo/custom-dir/my-feat/.git"
    teardown_temp_repo "$repo"
    rm -rf "$mock_dir"
}

@test "manager — defaults to .worktrees when @worktree-dir is unset" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    mock_dir=$(create_mock_tmux "")

    REPO_PATH="$repo" \
        PATH="$mock_dir:$PATH" \
        bash "$PROJECT_DIR/scripts/worktree-manager.sh" create-worktree "my-feat"

    assert test -f "$repo/.worktrees/my-feat/.git"
    teardown_temp_repo "$repo"
    rm -rf "$mock_dir"
}

@test "manager — custom directory is added to git exclude" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    mock_dir=$(create_mock_tmux "alt-trees")

    REPO_PATH="$repo" \
        PATH="$mock_dir:$PATH" \
        bash "$PROJECT_DIR/scripts/worktree-manager.sh" create-worktree "my-feat"

    run grep -qxF "alt-trees/" "$repo/.git/info/exclude"
    assert_success
    teardown_temp_repo "$repo"
    rm -rf "$mock_dir"
}

@test "manager — converts slashes in branch name to dashes with custom dir" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    mock_dir=$(create_mock_tmux "custom-dir")

    REPO_PATH="$repo" \
        PATH="$mock_dir:$PATH" \
        bash "$PROJECT_DIR/scripts/worktree-manager.sh" create-worktree "feat/my-feature"

    assert test -f "$repo/custom-dir/feat-my-feature/.git"
    teardown_temp_repo "$repo"
    rm -rf "$mock_dir"
}

@test "manager — cleanup with auto-fetch disabled does not crash" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    mock_dir=$(create_mock_tmux ".worktrees" "false")

    REPO_PATH="$repo" \
        PATH="$mock_dir:$PATH" \
        run bash "$PROJECT_DIR/scripts/worktree-manager.sh" cleanup

    assert_success
    teardown_temp_repo "$repo"
    rm -rf "$mock_dir"
}

@test "manager — cleanup removes a worktree" {
    repo=$(setup_temp_repo)
    cd "$repo" || return 1

    mkdir -p ".worktrees"
    output=$(git worktree add ".worktrees/test-feat" -b "test-feat" "main" 2>&1) || {
        teardown_temp_repo "$repo"
        skip "git worktree add failed: $output"
    }

    mock_dir=$(create_mock_tmux ".worktrees" "false")

    cat > "$mock_dir/fzf-tmux" << MOCK
#!/bin/bash
# Read stdin, find the line containing test-feat, output it
while IFS= read -r line; do
    if echo "\$line" | grep -q "test-feat"; then
        echo -e "\$line"
        exit 0
    fi
done
exit 0
MOCK
    chmod +x "$mock_dir/fzf-tmux"

    REPO_PATH="$repo" \
        PATH="$mock_dir:$PATH" \
        run bash "$PROJECT_DIR/scripts/worktree-manager.sh" cleanup

    assert_success
    refute test -d ".worktrees/test-feat"
    teardown_temp_repo "$repo"
    rm -rf "$mock_dir"
}
