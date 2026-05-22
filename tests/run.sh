#!/bin/bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

PASS=0
FAIL=0

setup_temp_repo() {
    local dir
    dir=$(mktemp -d)
    cd "$dir" || return 1
    git init --initial-branch=main >/dev/null 2>&1
    git config user.email "test@test.com"
    git config user.name "Test"
    echo "init" > init.txt
    git add init.txt
    git commit -m "initial" >/dev/null 2>&1
    echo "$dir"
}

teardown_temp_repo() {
    local dir="$1"
    if [[ -d "$dir" ]]; then
        local in_dir
        in_dir=$(pwd 2>/dev/null || echo "/tmp")
        if [[ "$in_dir" == "$dir"* ]]; then
            cd /tmp 2>/dev/null || true
        fi
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

assert_eq() {
    local expected="$1" actual="$2" msg="${3:-}"
    if [[ "$expected" == "$actual" ]]; then
        return 0
    fi
    if [[ -n "$msg" ]]; then
        echo "    expected: $expected, actual: $actual — $msg" >&2
    else
        echo "    expected: $expected, actual: $actual" >&2
    fi
    return 1
}

assert_neq() {
    local unexpected="$1" actual="$2" msg="${3:-}"
    if [[ "$unexpected" != "$actual" ]]; then
        return 0
    fi
    if [[ -n "$msg" ]]; then
        echo "    unexpected: $unexpected, actual: $actual — $msg" >&2
    else
        echo "    unexpected: $unexpected, actual: $actual" >&2
    fi
    return 1
}

assert_true() {
    if "$@"; then
        return 0
    fi
    echo "    expected true, got false" >&2
    return 1
}

run_test() {
    local test_name="$1" test_file="$2"
    shift 2
    if "$@"; then
        ((PASS++))
        echo "  ✓ $test_name ($test_file)"
    else
        ((FAIL++))
        echo "  ✗ $test_name ($test_file)"
    fi
}

run_test_file() {
    local test_file="$1"
    local file_basename
    file_basename=$(basename "$test_file" .test.sh)
    echo ""
    echo "── $file_basename ──"

    source "$PROJECT_DIR/scripts/lib/$file_basename.sh"

    local before_funcs
    before_funcs=$(declare -F)

    source "$test_file"

    local after_funcs
    after_funcs=$(declare -F)
    local new_funcs
    new_funcs=$(comm -13 <(echo "$before_funcs" | sort) <(echo "$after_funcs" | sort) | awk '{print $3}')

    for func in $new_funcs; do
        run_test "$func" "$file_basename" "$func"
    done
}

echo "tmux-worktrees — test suite"
echo "────────────────────────"

for test_file in "$SCRIPT_DIR"/lib/*.test.sh; do
    if [[ -f "$test_file" ]]; then
        run_test_file "$test_file"
    fi
done

echo ""
echo "────────────────────────"
echo "Results: $PASS passed, $FAIL failed"
exit "$FAIL"
