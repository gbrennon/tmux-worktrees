find_repo_root() {
    local dir
    dir=$(tmux show-environment -g MAIN_PROJECT_PATH 2>/dev/null | cut -d= -f2)
    if [[ -n "$dir" ]]; then
        echo "$dir"
        return 0
    fi
    dir="$(tmux display-message -p '#{pane_current_path}')"
    while [[ "$dir" != "/" ]]; do
        if [[ -d "$dir/.git" ]]; then
            echo "$dir"
            return 0
        fi
        dir="$(dirname "$dir")"
    done
    if [[ -d "/.git" ]]; then
        echo "/"
        return 0
    fi
    return 1
}

resolve_default_branch() {
    local branch
    branch=$(git config --global init.defaultBranch 2>/dev/null)
    if [[ -z "$branch" ]]; then
        branch=$(git config init.defaultBranch 2>/dev/null)
    fi
    if [[ -z "$branch" ]]; then
        branch=$(git branch --show-current 2>/dev/null)
    fi
    echo "${branch:-main}"
}

ensure_worktrees_dir() {
    local worktrees_dir="$1"
    if [[ ! -d "$worktrees_dir" ]]; then
        mkdir "$worktrees_dir"
        if [[ -f ".git/info/exclude" ]]; then
            grep -qxF "$worktrees_dir/" .git/info/exclude 2>/dev/null ||
                echo "$worktrees_dir/" >> .git/info/exclude
        fi
    fi
}
