create_worktree_and_open() {
    local branch="$1"
    local command
    local default_branch output
    local notify="${2:-tmux_show_status_message}"
    local repo_root
    local target
    local target_abs

    if [[ -z "$branch" ]]; then
        "$notify" "Branch name cannot be empty"
        return 1
    fi

    repo_root=$(find_repo_root) || {
        "$notify" "Not in a git repository"
        return 1
    }

    cd "$repo_root" || {
        "$notify" "Cannot cd to $repo_root"
        return 1
    }

    ensure_worktrees_dir ".worktrees"
    target=".worktrees/$(echo "$branch" | tr '/' '-')"

    if [[ -d "$target" ]]; then
        "$notify" "Worktree already exists: $branch"
        return 0
    fi

    default_branch=$(resolve_default_branch)
    output=$(create_worktree "$target" "$branch" "$default_branch") || {
        "$notify" "Worktree creation failed: $output"
        return 1
    }

    target_abs=$(worktree_abspath "$target")

    command=$(tmux_shell_command)
    tmux_create_window "$branch" "$target_abs" "$command"
}
