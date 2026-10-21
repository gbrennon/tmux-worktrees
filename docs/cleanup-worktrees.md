# Clean up worktrees

When a branch is merged (or you simply don't need it anymore), press
`prefix` + `D`.  The popup shows all your worktrees, each prefixed with:

- **✓ merged** — the branch is an ancestor of the repository's resolved
  default branch (from Git configuration or the current branch). This check
  does not inspect uncommitted changes.
- **✗ active** — the branch is not an ancestor of that default branch; the
  plugin won't stop you from removing it (uses `git branch -D`), but the mark
  helps you decide.

Select one with `Enter` and the worktree directory, its tmux window, and the
local git branch are all removed in one shot.
