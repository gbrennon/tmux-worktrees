# Customise the shell

When a new tmux window opens, the plugin launches your login shell by default.
If you'd prefer a different command (e.g. `zsh`, `vim` or `emacs`), set:

```tmux
set -g @worktree-command 'zsh'
```
