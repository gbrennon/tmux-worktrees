#!/bin/bash

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$DIR/lib/repo.sh"
source "$DIR/lib/worktree.sh"
source "$DIR/lib/merge.sh"
source "$DIR/lib/tmux.sh"
source "$DIR/lib/select.sh"

create_or_resume() {
    local branch="$1"
    if [[ -z "$branch" ]]; then
        show_error "Branch name cannot be empty — type a name (e.g. feat/foo) or select an existing worktree"
        exit 0
    fi
    local repo_root
    repo_root=$(find_repo_root) || {
        show_error "Not in a git repository — no .git directory found when walking up from:\n\n  $(pwd)\n\nChecked all parent directories up to /."
        exit 0
    }
    cd "$repo_root" || { show_error "Cannot cd to $repo_root"; exit 0; }
    local worktree_dir
    worktree_dir=$(resolve_worktree_dir)
    ensure_worktrees_dir "$worktree_dir"
    local target
    target="$worktree_dir/$(echo "$branch" | tr '/' '-')"
    if [[ ! -d "$target" ]]; then
        local default_branch output
        default_branch=$(resolve_default_branch)
        output=$(create_worktree "$target" "$branch" "$default_branch") || {
            show_error "Worktree creation failed:\n\n$output"
            exit 0
        }
    fi
    local target_abs command
    target_abs=$(worktree_abspath "$target")
    command=$(tmux_shell_command)
    tmux_select_or_create_window "$branch" "$target_abs" "$command"
}

select_worktree() {
    local repo_root
    repo_root=$(find_repo_root) || {
        show_error "Not in a git repository — no .git directory found when walking up from:\n\n  $(pwd)\n\nChecked all parent directories up to /."
        exit 0
    }
    cd "$repo_root" || { show_error "Cannot cd to $repo_root"; exit 0; }
    local worktree_dir
    worktree_dir=$(resolve_worktree_dir)
    local existing branch
    existing=$(list_worktree_names "$worktree_dir")
    if [[ -z "$existing" ]]; then
        branch=$(fzf_select_worktree "" "No worktrees yet — type a name and press Enter to create")
    else
        branch=$(fzf_select_worktree "$existing")
    fi
    [[ -z "$branch" ]] && exit 0
    if [[ -d "$worktree_dir/$branch" ]]; then
        local resolved
        resolved=$(worktree_branch "$worktree_dir/$branch")
        [[ -n "$resolved" ]] && branch="$resolved"
    fi
    create_or_resume "$branch"
}

cleanup_worktrees() {
    local repo_root
    repo_root=$(find_repo_root) || {
        show_error "Not in a git repository — no .git directory found when walking up from:\n\n  $(pwd)\n\nChecked all parent directories up to /."
        exit 0
    }
    cd "$repo_root" || { show_error "Cannot cd to $repo_root"; exit 0; }
    local worktree_dir
    worktree_dir=$(resolve_worktree_dir)
    local default_branch auto_fetch
    default_branch=$(resolve_default_branch)
    auto_fetch=$(tmux show-option -gv @worktree-auto-fetch 2>/dev/null)
    auto_fetch="${auto_fetch:-true}"
    if [[ "$auto_fetch" != "false" ]]; then
        git fetch origin "$default_branch" --no-tags 2>/dev/null
    fi
    local fzf_input=""
    local wt_dir branch
    while IFS= read -r wt_dir; do
        branch=$(worktree_branch "$wt_dir")
        if is_merged "$wt_dir" "$branch" "$default_branch"; then
            fzf_input+="✓ merged  | $branch"$'\t'"$wt_dir"$'\n'
        else
            fzf_input+="✗ active  | $branch"$'\t'"$wt_dir"$'\n'
        fi
    done < <(list_worktrees "$worktree_dir")
    local noinfo=""
    if [[ -z "$fzf_input" ]]; then
        fzf_input="No worktrees found in $worktree_dir/ — press Enter to dismiss"
        noinfo="--no-info"
    fi
    local result
    result=$(fzf_cleanup_picker "$fzf_input" "$noinfo")
    [[ -z "$result" ]] && exit 0
    if [[ "$result" != *$'\t'* ]]; then
        exit 0
    fi
    branch=$(echo "$result" | cut -f1 | sed 's/^.*| *//')
    wt_dir=$(echo "$result" | cut -f2)
    local output
    output=$(remove_worktree "$wt_dir" "$branch") || {
        show_error "Worktree removal failed:\n\n$output"
        exit 0
    }
    delete_local_branch "$branch"
    tmux display-message "Removed worktree: $branch"
}

case "${1:-}" in
    "")        select_worktree ;;
    create-worktree) create_or_resume "${2:-}" ;;
    choose)    select_worktree ;;
    cleanup)   cleanup_worktrees ;;
    *)         show_error "Unknown command: $1"; exit 0 ;;
esac
