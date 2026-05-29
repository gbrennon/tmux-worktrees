list_worktrees() {
    local work_dir="${1:-.worktrees}"
    git worktree list --porcelain | awk '/^worktree / {print $2}' | { grep "^$(pwd)/$work_dir/" || true; }
}

list_worktree_names() {
    git worktree list --porcelain | awk '/^worktree / {print $2}' | xargs -I {} basename {} | sort
}

create_worktree() {
    local target="$1" branch="$2" base="$3"

    git fetch origin --quiet

    if [[ "$base" == "main" || "$base" == "master" ]]; then
        base="origin/$base"
    fi

    git worktree add "$target" -b "$branch" "$base" 2>&1
}

remove_worktree() {
    local target="$1"

    git worktree remove --force "$target" 2>&1
}

worktree_branch() {
    local wt_dir="$1"
    git -C "$wt_dir" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "?"
}

worktree_abspath() {
    local target="$1"

    if command -v realpath &>/dev/null; then
        realpath "$target"
    fi
}

delete_local_branch() {
    local branch="$1"

    git branch -D "$branch" >/dev/null 2>&1 || true
}
