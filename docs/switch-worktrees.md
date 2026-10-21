# Switch between worktrees

Press `prefix` + `W` again. The fzf picker reads Git's registered worktrees,
not just the plugin's configured directory. It lists worktrees outside
`@worktree-dir` too, by branch name: `feat/foo`, `fix/bar`, `chore/baz`, or
whatever you named them.

Use the arrow keys (or keep typing to filter) and hit `Enter`. You'll jump
straight to that worktree's path in a tmux window. The configured directory
only controls where newly created worktrees are placed; it does not limit
which existing worktrees can be resumed or cleaned up.

Think of `prefix + W` as your "project dashboard" — one keystroke to see every
active branch and jump to any of them.
