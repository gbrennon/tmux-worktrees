is_merged() {
    local wt_dir="$1" branch="$2" default_branch="$3" auto_fetch="$4"
    if git -C "$wt_dir" merge-base --is-ancestor HEAD "origin/$default_branch" 2>/dev/null; then
        return 0
    fi
    if [[ "$auto_fetch" != "false" ]] && \
       ! git ls-remote --exit-code origin "refs/heads/$branch" 2>/dev/null; then
        return 0
    fi
    return 1
}
