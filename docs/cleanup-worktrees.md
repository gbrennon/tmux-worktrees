# Clean up worktrees

When a branch is merged (or you simply don't need it anymore), press
`prefix` + `D`.  The popup shows all your worktrees, each prefixed with:

- **✓ merged** — the branch is confirmed as an ancestor of
  `origin/<resolved-default-branch>`. The check depends on the remote-tracking
  ref being available and does not inspect uncommitted changes.
- **✗ active** — ancestry against that remote-tracking ref was not confirmed;
  a locally merged branch can still receive this mark. The plugin won't stop
  you from removing it (uses `git branch -D`), but the mark helps you decide.

Use Up/Down to move through the list and Space to toggle each worktree. After
toggling at least one, press Enter to continue to a confirmation popup. If no
worktrees are checked, Enter instead shows `No worktrees were selected`.
Choose **Yes** to remove every checked worktree, its tmux window, and its local
Git branch; choose **No** or press `Esc` to leave everything unchanged.

Worktrees are processed in display order. If removal or branch deletion fails,
the cleanup stops at that item: earlier successful removals remain applied, no
success message is shown for the failed item, and later selections are not
processed.
