# Customise the worktree directory

By default, all worktrees are created inside `.worktrees/` at the repository
root.  You can change this with the `@worktree-dir` option:

```tmux
set -g @worktree-dir 'my-trees'
```

The value is relative to the repository root.  The plugin creates the directory
if it doesn't exist and adds it to `.git/info/exclude` so git ignores it.

Set this **before** the plugin line in `~/.tmux.conf`:

```tmux
set -g @plugin 'gbrennon/tmux-worktrees'
```
