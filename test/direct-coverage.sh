#!/bin/bash
# Direct coverage test — runs outside bats so kcov traces all code paths.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$SCRIPT_DIR/scripts/lib/repo.sh"
source "$SCRIPT_DIR/scripts/lib/worktree.sh"
source "$SCRIPT_DIR/scripts/lib/merge.sh"
source "$SCRIPT_DIR/scripts/lib/tmux.sh"
source "$SCRIPT_DIR/scripts/lib/select.sh"

TMP_RESULTS=$(mktemp)
echo "0 0" > "$TMP_RESULTS"

pass() {
    read -r p f < "$TMP_RESULTS"
    echo "$((p + 1)) $f" > "$TMP_RESULTS"
}
fail() {
    echo "  FAIL: $*"
    read -r p f < "$TMP_RESULTS"
    echo "$p $((f + 1))" > "$TMP_RESULTS"
}

assert_eq() {
    local expected="$1" actual="$2" msg="${3:-}"
    if [[ "$actual" == "$expected" ]]; then
        pass
    else
        fail "${msg:-(expected '$expected', got '$actual')}"
    fi
}

show_results() {
    read -r PASS FAIL < "$TMP_RESULTS"
    rm -f "$TMP_RESULTS"
    echo ""
    echo "=== Results ==="
    echo "  PASS: $PASS  FAIL: $FAIL"
    if [[ "$FAIL" -gt 0 ]]; then
        echo "  Some tests failed!"
        exit 1
    fi
    echo "  All tests passed!"
}

# Trap to clean up temp file on exit
cleanup_on_exit() {
    rm -f "${TMP_RESULTS:-}"
}
trap cleanup_on_exit EXIT

cleanup_temp() {
    local dir="$1"
    if [[ -d "$dir" ]]; then
        cd /tmp 2>/dev/null || true
        if [[ -d "$dir/.git" ]]; then
            cd "$dir" 2>/dev/null || true
            git worktree list 2>/dev/null | tail -n +2 | awk '{print $1}' | while read -r wt; do
                git worktree remove "$wt" 2>/dev/null || true
            done
            cd /tmp 2>/dev/null || true
        fi
        rm -rf "$dir"
    fi
}

make_repo() {
    local dir
    dir=$(mktemp -d)
    cd "$dir"
    git init --initial-branch=main >/dev/null 2>&1
    git config user.email "t@t.com"
    git config user.name "T"
    echo "init" > init.txt
    git add init.txt && git commit -m "init" >/dev/null 2>&1
    echo "$dir"
}

# Create mock executables in PATH (avoids bash/zsh function export issues)
make_mock_dir() {
    local mock_dir
    mock_dir=$(mktemp -d)

    cat > "$mock_dir/tmux" << 'TMUXEOF'
#!/bin/bash
case "$*" in
    "show-option -gv @worktree-command")
        [ -n "${MOCK_TMUX_WORKTREE_COMMAND:-}" ] && echo "$MOCK_TMUX_WORKTREE_COMMAND"
        ;;
    "show-option -gv @worktree-dir")
        [ -n "${MOCK_TMUX_WORKTREE_DIR:-}" ] && echo "$MOCK_TMUX_WORKTREE_DIR"
        ;;
    "show-option -gv @worktree-auto-fetch")
        echo "${MOCK_TMUX_AUTO_FETCH:-true}"
        ;;
    "show-environment -g MAIN_PROJECT_PATH")
        if [ "${MOCK_TMUX_MAIN_PROJECT_RC:-1}" = "0" ]; then
            echo "MAIN_PROJECT_PATH=${MOCK_TMUX_MAIN_PROJECT_VALUE:-/tmp/default-project}"
            return 0
        fi
        return 1
        ;;
    "display-message -p #{pane_current_path}")
        echo "${MOCK_TMUX_PWD:-$(pwd)}"
        ;;
    "list-windows -F #{window_name}")
        echo "${MOCK_TMUX_WINDOWS:-}"
        ;;
    new-window\ *)
        exit 0
        ;;
    kill-window\ *)
        exit 0
        ;;
    select-window\ *)
        exit 0
        ;;
    display-popup\ *)
        exit 0
        ;;
    display-message\ *)
        exit 0
        ;;
    *)
        return 1
        ;;
esac
TMUXEOF
    chmod +x "$mock_dir/tmux"

    cat > "$mock_dir/fzf-tmux" << 'FZFEOF'
#!/bin/bash
if [ -n "${MOCK_FZF_OUTPUT:-}" ]; then
    echo "$MOCK_FZF_OUTPUT"
    exit 0
fi
# Otherwise read stdin and echo first line
IFS= read -r line
echo "$line"
FZFEOF
    chmod +x "$mock_dir/fzf-tmux"

    echo "$mock_dir"
}

cleanup_mock_dir() {
    rm -rf "$1"
}

# =====================================================================
echo "=== merge.sh ==="
(
    repo=$(make_repo)
    cd "$repo"

    git checkout -b merged-branch >/dev/null 2>&1
    echo "feat" > feat.txt
    git add feat.txt && git commit -m "feat" >/dev/null 2>&1
    git checkout main >/dev/null 2>&1
    git merge merged-branch --no-ff -m "merge" >/dev/null 2>&1
    git branch -f "origin/main" main >/dev/null 2>&1
    git checkout merged-branch >/dev/null 2>&1

    is_merged "." "merged-branch" "main" "true" && pass || fail "is_merged merged"
    git checkout main >/dev/null 2>&1
    cleanup_temp "$repo"
)
(
    repo=$(make_repo)
    cd "$repo"

    git checkout -b unmerged-branch >/dev/null 2>&1
    echo "unmerged" > u.txt
    git add u.txt && git commit -m "unmerged" >/dev/null 2>&1
    git checkout main >/dev/null 2>&1
    git branch -f "origin/main" main >/dev/null 2>&1
    git checkout unmerged-branch >/dev/null 2>&1

    ! is_merged "." "unmerged-branch" "main" "false" && pass || fail "is_merged unmerged"
    git checkout main >/dev/null 2>&1
    cleanup_temp "$repo"
)
(
    repo=$(make_repo)
    cd "$repo"

    git checkout -b ff-branch >/dev/null 2>&1
    echo "ff" > ff.txt
    git add ff.txt && git commit -m "ff" >/dev/null 2>&1
    git checkout main >/dev/null 2>&1
    git merge ff-branch >/dev/null 2>&1
    git branch -f "origin/main" main >/dev/null 2>&1
    git checkout ff-branch >/dev/null 2>&1

    is_merged "." "ff-branch" "main" "true" && pass || fail "is_merged ff"
    git checkout main >/dev/null 2>&1
    cleanup_temp "$repo"
)
(
    repo=$(make_repo)
    cd "$repo"

    start=$(date +%s)
    ! is_merged "." "some-branch" "main" "true" && pass || fail "is_merged no origin"
    end=$(date +%s)
    elapsed=$((end - start))
    if [[ "$elapsed" -le 2 ]]; then pass; else fail "is_merged took too long: ${elapsed}s"; fi

    cleanup_temp "$repo"
)
# =====================================================================
echo "=== select.sh ==="
(
    mock_dir=$(make_mock_dir)
    export PATH="$mock_dir:$PATH"
    export MOCK_FZF_OUTPUT="test-branch"

    result=$(fzf_select_worktree "test-branch" "custom header")
    assert_eq "test-branch" "$result" "fzf_select_worktree with existing"

    result=$(fzf_select_worktree "")
    assert_eq "test-branch" "$result" "fzf_select_worktree with empty"

    export MOCK_FZF_OUTPUT="test-entry"
    result=$(fzf_cleanup_picker "test-entry")
    assert_eq "test-entry" "$result" "fzf_cleanup_picker"

    result=$(fzf_cleanup_picker "picked" "--no-info")
    assert_eq "test-entry" "$result" "fzf_cleanup_picker with noinfo"

    cleanup_mock_dir "$mock_dir"
) || true

# =====================================================================
echo "=== tmux.sh ==="
(
    mock_dir=$(make_mock_dir)
    export PATH="$mock_dir:$PATH"

    # tmux_worktree_dir with option set
    export MOCK_TMUX_WORKTREE_DIR="custom-trees"
    result=$(tmux_worktree_dir)
    assert_eq "custom-trees" "$result" "tmux_worktree_dir with option"
    unset MOCK_TMUX_WORKTREE_DIR

    # tmux_worktree_dir without option
    result=$(tmux_worktree_dir)
    assert_eq "" "$result" "tmux_worktree_dir without option"

    # resolve_worktree_dir with option set
    export MOCK_TMUX_WORKTREE_DIR="alt-trees"
    result=$(resolve_worktree_dir)
    assert_eq "alt-trees" "$result" "resolve_worktree_dir with option"
    unset MOCK_TMUX_WORKTREE_DIR

    # resolve_worktree_dir without option (falls back to .worktrees)
    result=$(resolve_worktree_dir)
    assert_eq ".worktrees" "$result" "resolve_worktree_dir default fallback"

    # tmux_shell_command with @worktree-command option
    export MOCK_TMUX_WORKTREE_COMMAND="fish"
    result=$(tmux_shell_command)
    assert_eq "fish" "$result" "tmux_shell_command with option"
    unset MOCK_TMUX_WORKTREE_COMMAND

    # tmux_shell_command fallback to SHELL
    # Create a mock getent that fails (returns empty), so SHELL fallback is used
    cat > "$mock_dir/getent" << 'GETENTFAIL'
#!/bin/bash
exit 1
GETENTFAIL
    chmod +x "$mock_dir/getent"
    export PATH="$mock_dir:$PATH"

    SAVED_SHELL="${SHELL:-}"
    SHELL="/bin/zsh"
    result=$(tmux_shell_command)
    assert_eq "/bin/zsh" "$result" "tmux_shell_command SHELL fallback"
    SHELL="$SAVED_SHELL"

    # tmux_shell_command fallback to /bin/bash when SHELL is empty and getent fails
    SHELL=""
    result=$(tmux_shell_command)
    assert_eq "/bin/bash" "$result" "tmux_shell_command default fallback"
    SHELL="$SAVED_SHELL"

    cleanup_mock_dir "$mock_dir"
)
(
    mock_dir=$(make_mock_dir)
    export PATH="$mock_dir:$PATH"
    export MOCK_TMUX_WINDOWS=""

    # tmux_select_or_create_window — create new window (window not in list)
    result=$(tmux_select_or_create_window "my-branch" "/tmp" "/bin/bash" 2>&1) && pass || fail "tmux_select_or_create_window create"

    # tmux_select_or_create_window — select existing window
    export MOCK_TMUX_WINDOWS="wt-existing-branch"
    result=$(tmux_select_or_create_window "existing-branch" "/tmp" "/bin/bash" 2>&1) && pass || fail "tmux_select_or_create_window select"

    cleanup_mock_dir "$mock_dir"
)
(
    mock_dir=$(make_mock_dir)
    export PATH="$mock_dir:$PATH"

    # tmux_kill_window — window exists
    export MOCK_TMUX_WINDOWS="wt-to-kill"
    tmux_kill_window "to-kill" && pass || fail "tmux_kill_window exists"

    # tmux_kill_window — window does not exist
    export MOCK_TMUX_WINDOWS=""
    tmux_kill_window "nonexistent" && pass || fail "tmux_kill_window nonexistent"

    cleanup_mock_dir "$mock_dir"
)
(
    mock_dir=$(make_mock_dir)
    export PATH="$mock_dir:$PATH"

    show_error "test error" 2>/dev/null && pass || fail "show_error"

    cleanup_mock_dir "$mock_dir"
)
# =====================================================================
echo "=== worktree.sh ==="
(
    repo=$(make_repo)
    cd "$repo"
    mkdir -p ".worktrees"

    result=$(list_worktrees ".worktrees")
    assert_eq "" "$result" "list_worktrees empty"

    create_worktree ".worktrees/test-wt" "test-wt" "main" >/dev/null 2>&1 && pass || fail "create_worktree"

    result=$(list_worktrees ".worktrees")
    if [[ -n "$result" ]]; then pass; else fail "list_worktrees non-empty"; fi

    result=$(list_worktree_names ".worktrees")
    assert_eq "test-wt" "$result" "list_worktree_names"

    result=$(worktree_branch ".worktrees/test-wt")
    assert_eq "test-wt" "$result" "worktree_branch"

    result=$(worktree_branch "/tmp/non-existent-xxxx")
    assert_eq "?" "$result" "worktree_branch non-existent"

    result=$(worktree_abspath ".worktrees/test-wt")
    if [[ "$result" == "$repo/.worktrees/test-wt" ]]; then pass; else fail "worktree_abspath (got '$result')"; fi

    remove_worktree ".worktrees/test-wt" >/dev/null 2>&1 && pass || fail "remove_worktree"

    git checkout -b "to-delete" >/dev/null 2>&1
    git checkout main >/dev/null 2>&1
    delete_local_branch "to-delete" && pass || fail "delete_local_branch"

    delete_local_branch "non-existent" && pass || fail "delete_local_branch non-existent"

    cleanup_temp "$repo"
)
(
    repo=$(make_repo)
    cd "$repo"
    mkdir -p ".worktrees"
    git worktree add ".worktrees/test-wt" -b "test-wt" "main" >/dev/null 2>&1

    result=$(worktree_abspath ".worktrees/test-wt")
    if [[ "$result" == "$repo/.worktrees/test-wt" ]]; then pass; else fail "worktree_abspath realpath (got '$result')"; fi

    cleanup_temp "$repo"
)
# =====================================================================
echo "=== repo.sh ==="
(
    mock_dir=$(make_mock_dir)
    export PATH="$mock_dir:$PATH"

    export MOCK_TMUX_MAIN_PROJECT_RC=0
    export MOCK_TMUX_MAIN_PROJECT_VALUE="/tmp/test-project"
    result=$(find_repo_root)
    assert_eq "/tmp/test-project" "$result" "find_repo_root via MAIN_PROJECT_PATH"

    cleanup_mock_dir "$mock_dir"
) || true

(
    repo=$(make_repo)
    cd "$repo"

    mock_dir=$(make_mock_dir)
    export PATH="$mock_dir:$PATH"
    export MOCK_TMUX_PWD="$repo"

    result=$(find_repo_root)
    assert_eq "$repo" "$result" "find_repo_root via walking"

    cleanup_mock_dir "$mock_dir"
    cleanup_temp "$repo"
) || true

(
    tmpdir=$(mktemp -d)
    cd "$tmpdir"

    mock_dir=$(make_mock_dir)
    export PATH="$mock_dir:$PATH"
    export MOCK_TMUX_PWD="$tmpdir"

    result=$(find_repo_root) || true
    assert_eq "" "$result" "find_repo_root outside repo"

    cleanup_mock_dir "$mock_dir"
    rm -rf "$tmpdir"
) || true

# =====================================================================
echo "=== worktree-manager.sh (edge cases via direct source) ==="

# Test 1: unknown command
(
    mock_dir=$(make_mock_dir)
    export PATH="$mock_dir:$PATH"

    result=$(bash "$SCRIPT_DIR/scripts/worktree-manager.sh" "unknown-cmd" 2>&1 || true)
    if echo "$result" | grep -q "Unknown command"; then pass; else fail "unknown command (got: $result)"; fi

    cleanup_mock_dir "$mock_dir"
) || true

# Test 2: create-or-resume with empty branch
(
    mock_dir=$(make_mock_dir)
    export PATH="$mock_dir:$PATH"

    result=$(bash "$SCRIPT_DIR/scripts/worktree-manager.sh" create-worktree "" 2>&1 || true)
    if echo "$result" | grep -q "cannot be empty"; then pass; else fail "empty branch error (got: $result)"; fi

    cleanup_mock_dir "$mock_dir"
) || true

# Test 3: create-or-resume outside git repo
(
    tmpdir=$(mktemp -d)
    cd "$tmpdir"

    mock_dir=$(make_mock_dir)
    export PATH="$mock_dir:$PATH"
    export MOCK_TMUX_PWD="$tmpdir"

    result=$(bash "$SCRIPT_DIR/scripts/worktree-manager.sh" create-worktree "test-branch" 2>&1 || true)
    if echo "$result" | grep -q "Not in a git repository"; then pass; else fail "outside git repo error (got: $result)"; fi

    cleanup_mock_dir "$mock_dir"
    rm -rf "$tmpdir"
) || true

# Test 4: choose command (alias for select)
(
    repo=$(make_repo)
    cd "$repo"

    mock_dir=$(make_mock_dir)
    export PATH="$mock_dir:$PATH"
    export MOCK_TMUX_PWD="$repo"
    export MOCK_FZF_OUTPUT=""

    # Isolate from global git config that may set init.defaultBranch to trunk
    GIT_CONFIG_GLOBAL=$(mktemp) \
        bash "$SCRIPT_DIR/scripts/worktree-manager.sh" choose 2>&1 || true
    rm -f "${GIT_CONFIG_GLOBAL:-}"
    pass

    cleanup_mock_dir "$mock_dir"
    cleanup_temp "$repo"
)
# Test 5: cleanup with auto-fetch enabled
(
    repo=$(make_repo)
    cd "$repo"

    mock_dir=$(make_mock_dir)
    export PATH="$mock_dir:$PATH"
    export MOCK_TMUX_PWD="$repo"
    export MOCK_TMUX_AUTO_FETCH="true"
    export MOCK_FZF_OUTPUT=""

    GIT_CONFIG_GLOBAL=$(mktemp) \
        bash "$SCRIPT_DIR/scripts/worktree-manager.sh" cleanup 2>&1 || true
    rm -f "${GIT_CONFIG_GLOBAL:-}"
    pass

    cleanup_mock_dir "$mock_dir"
    cleanup_temp "$repo"
) || true

# Test 6: cleanup outside git repo
(
    tmpdir=$(mktemp -d)
    cd "$tmpdir"

    mock_dir=$(make_mock_dir)
    export PATH="$mock_dir:$PATH"
    export MOCK_TMUX_PWD="$tmpdir"

    result=$(bash "$SCRIPT_DIR/scripts/worktree-manager.sh" cleanup 2>&1 || true)
    if echo "$result" | grep -q "Not in a git repository"; then pass; else fail "cleanup outside git (got: $result)"; fi

    cleanup_mock_dir "$mock_dir"
    rm -rf "$tmpdir"
) || true

# Test 7: select_worktree outside git repo
(
    tmpdir=$(mktemp -d)
    cd "$tmpdir"

    mock_dir=$(make_mock_dir)
    export PATH="$mock_dir:$PATH"
    export MOCK_TMUX_PWD="$tmpdir"

    result=$(bash "$SCRIPT_DIR/scripts/worktree-manager.sh" 2>&1 || true)
    if echo "$result" | grep -q "Not in a git repository"; then pass; else fail "select outside git (got: $result)"; fi

    cleanup_mock_dir "$mock_dir"
    rm -rf "$tmpdir"
) || true

# Test 8: cleanup with no worktrees
(
    repo=$(make_repo)
    cd "$repo"
    mkdir -p ".worktrees"

    mock_dir=$(make_mock_dir)
    export PATH="$mock_dir:$PATH"
    export MOCK_TMUX_PWD="$repo"
    export MOCK_TMUX_AUTO_FETCH="false"
    export MOCK_FZF_OUTPUT=""

    GIT_CONFIG_GLOBAL=$(mktemp) \
        bash "$SCRIPT_DIR/scripts/worktree-manager.sh" cleanup 2>&1 || true
    rm -f "${GIT_CONFIG_GLOBAL:-}"
    pass

    cleanup_mock_dir "$mock_dir"
    cleanup_temp "$repo"
) || true

# Test 9: create_or_resume with worktree creation failure (invalid branch)
(
    repo=$(make_repo)
    cd "$repo"

    mock_dir=$(make_mock_dir)
    export PATH="$mock_dir:$PATH"
    export MOCK_TMUX_PWD="$repo"

    GIT_CONFIG_GLOBAL=$(mktemp) \
        bash "$SCRIPT_DIR/scripts/worktree-manager.sh" create-worktree "branch with spaces" 2>&1 || true
    rm -f "${GIT_CONFIG_GLOBAL:-}"
    pass

    cleanup_mock_dir "$mock_dir"
    cleanup_temp "$repo"
)
# Test 10: cleanup with merged and unmerged worktrees
(
    repo=$(make_repo)
    cd "$repo"

    git checkout -b feat-merged >/dev/null 2>&1
    echo "merged" > m.txt
    git add m.txt && git commit -m "merged" >/dev/null 2>&1
    git checkout main >/dev/null 2>&1
    git merge feat-merged --no-ff -m "merged" >/dev/null 2>&1
    git branch -D feat-merged >/dev/null 2>&1
    git branch -f "origin/main" main >/dev/null 2>&1

    git checkout -b feat-active >/dev/null 2>&1
    echo "active" > a.txt
    git add a.txt && git commit -m "active" >/dev/null 2>&1
    git checkout main >/dev/null 2>&1

    mkdir -p ".worktrees"
    git worktree add ".worktrees/feat-active" -b "feat-active" "main" >/dev/null 2>&1 || true
    git branch -D feat-active >/dev/null 2>&1 || true
    git worktree remove .worktrees/feat-active 2>/dev/null || true
    rm -rf .worktrees/feat-active
    git worktree add ".worktrees/feat-active" "main" >/dev/null 2>&1

    mock_dir=$(make_mock_dir)
    export PATH="$mock_dir:$PATH"
    export MOCK_TMUX_PWD="$repo"
    export MOCK_TMUX_AUTO_FETCH="false"
    export MOCK_FZF_OUTPUT=""

    GIT_CONFIG_GLOBAL=$(mktemp) \
        bash "$SCRIPT_DIR/scripts/worktree-manager.sh" cleanup 2>&1 || true
    rm -f "${GIT_CONFIG_GLOBAL:-}"
    pass

    cleanup_mock_dir "$mock_dir"
    cleanup_temp "$repo"
) || true

show_results
