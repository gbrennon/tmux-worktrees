list_worktrees() {
    local worktrees_dir="$1"
    find "$worktrees_dir" -mindepth 2 -name '.git' -type f -printf '%h\n' 2>/dev/null
}

list_worktree_names() {
    local worktrees_dir="$1"
    find "$worktrees_dir" -mindepth 2 -name '.git' -type f \
        -printf '%P\n' 2>/dev/null | sed 's|/\.git$||' | sort
}

create_worktree() {
    local target="$1" branch="$2" base="$3"
    git worktree add "$target" -b "$branch" "$base" 2>&1
}

remove_worktree() {
    local target="$1"
    git worktree remove "$target" 2>&1
}

worktree_branch() {
    local wt_dir="$1"
    git -C "$wt_dir" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "?"
}

worktree_abspath() {
    local target="$1"
    if command -v realpath &>/dev/null; then
        realpath "$target"
    else
        (cd "$target" && pwd)
    fi
}

delete_local_branch() {
    local branch="$1"
    git branch -D "$branch" 2>&1 || true
}
