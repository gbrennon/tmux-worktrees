# Clean up worktrees

When a branch is merged (or you simply don't need it anymore), press
`prefix` + `D`.  The popup shows all your worktrees, each prefixed with:

- **✓ merged** — the branch has been merged into `main`/`master` and is safe
  to delete.
- **✗ active** — the branch still has unmerged commits; the plugin won't stop
  you from removing it (uses `git branch -D`), but the mark helps you decide.

Select one with `Enter` and the worktree directory, its tmux window, and the
local git branch are all removed in one shot.
