show_error() {
    local msg="$1"
    printf '%s\n' "$msg" >&2
    local tmpfile
    tmpfile=$(mktemp)
    printf '%s\n' "$msg" > "$tmpfile"
    tmux display-popup -E -h 20 \
        "cat '$tmpfile'; echo; echo 'Press any key or wait 10s...'; read -t 10 -n1 2>/dev/null || true; rm -f '$tmpfile'" 2>/dev/null || \
        tmux display-message -d 5000 "tmux-worktrees: $msg" 2>/dev/null || true
}

tmux_shell_command() {
    local command
    command=$(tmux show-option -gv @worktree-command 2>/dev/null)
    if [[ -z "$command" ]]; then
        command=$(getent passwd "$(id -u)" | cut -d: -f7 2>/dev/null)
        command="${command:-$SHELL}"
        command="${command:-/bin/bash}"
    fi
    echo "$command"
}

tmux_select_or_create_window() {
    local branch="$1" cwd="$2" command="$3"
    if tmux list-windows -F '#{window_name}' 2>/dev/null | grep -Fxq "wt-$branch"; then
        tmux select-window -t "wt-$branch"
        tmux display-message "Resumed worktree: $branch"
    else
        tmux new-window -n "wt-$branch" -c "$cwd" "$command"
        tmux display-message "Created worktree: $branch"
    fi
}

tmux_kill_window() {
    local branch="$1"
    if tmux list-windows -F '#{window_name}' 2>/dev/null | grep -Fxq "wt-$branch"; then
        tmux kill-window -t "wt-$branch"
    fi
}
