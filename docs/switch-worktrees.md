# Switch between worktrees

Press `prefix` + `W` again. The interactive picker reads Git's registered
worktrees, not just the plugin's configured directory. It lists worktrees
outside
`@worktree-dir` too, using each worktree directory's name: `feature-foo`,
`fix-bar`, `chore-baz`, or whatever directory name Git reports.

Use the arrow keys (or keep typing to filter) and hit `Enter`. You'll jump
straight to that worktree's path in a tmux window and resume its branch. The configured directory
only controls where newly created worktrees are placed; it does not limit
which existing worktrees can be resumed or cleaned up.

Think of `prefix + W` as your "project dashboard" — one keystroke to see every
active branch and jump to any of them.
