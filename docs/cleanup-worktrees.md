# Clean up worktrees

When a branch is merged (or you simply don't need it anymore), press
`prefix` + `D`.  The popup shows all your worktrees, each prefixed with:

- **✓ merged** — the branch is confirmed as an ancestor of
  `origin/<resolved-default-branch>`. The check depends on the remote-tracking
  ref being available and does not inspect uncommitted changes.
- **✗ active** — ancestry against that remote-tracking ref was not confirmed;
  a locally merged branch can still receive this mark. The plugin won't stop
  you from removing it (uses `git branch -D`), but the mark helps you decide.

Select one with `Enter` and the worktree directory, its tmux window, and the
local git branch are all removed in one shot.
