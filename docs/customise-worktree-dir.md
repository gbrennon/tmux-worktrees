# Customise the worktree directory

By default, all new worktrees are created inside `.workspaces/` at the repository
root.  You can change this with the `@worktree-dir` option:

```tmux
set -g @worktree-dir 'my-trees'
```

The value is relative to the repository root.  The plugin creates the directory
if it doesn't exist and adds it to `.git/info/exclude` so git ignores it. This
option is the default location for newly created worktrees; existing worktrees
are discovered from Git even when they are located elsewhere.

Set this **before** the plugin line in `~/.tmux.conf`:

```tmux
set -g @plugin 'gbrennon/tmux-worktrees'
```
