# Customise the keys

Don't like `W` and `D`?  Set your own keys in `~/.tmux.conf` **before** the
plugin line:

```tmux
set -g @worktree-key 't'           # prefix + T to pick a worktree
set -g @worktree-cleanup-key 'x'   # prefix + X to clean up
```
