is_merged() {
    local wt_dir="$1" branch="$2" default_branch="$3"

    if git -C "$wt_dir" merge-base --is-ancestor HEAD "origin/$default_branch" 2>/dev/null; then
        return 0
    fi
    return 1
}
