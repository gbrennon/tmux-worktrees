# How it works (under the hood)

1. **Repo discovery** — the plugin first honors a valid
   `TMUX_WORKTREES_ROOT`, then walks from the process working directory. If
   that fails it checks `MAIN_PROJECT_PATH`, and finally walks from the active
   tmux pane path. A `.git` directory or worktree file identifies the root.
2. **Worktree discovery and storage** — Git's registered worktrees are read
   with `git worktree list --porcelain`, so the picker includes worktrees
   wherever they are located—not only inside the configured directory. New
   worktrees use the configurable directory (default `.worktrees/`) at the
   repo root. Branch slashes (`feat/foo`) are flattened to dashes
   (`feat-foo`) for new directory names. Set `@worktree-dir` in
   `~/.tmux.conf` to use a different path.
3. **External worktree management** — an existing worktree outside the
   configured directory is resumed using its Git-registered path. Switching,
   branch inspection, tmux window management, and cleanup work the same as
   for worktrees created by the plugin.
4. **Window naming** — each worktree gets a tmux window named
   `wt-<branch-name>` (with the original branch slashes preserved).
   The plugin checks for an existing window with that name before creating a
   new one, so you never end up with duplicates.
5. **Cleanup safety** — the removal flow kills the tmux window first, then
   runs `git worktree remove --force` followed by `git branch -D`. This can
   discard uncommitted work and force-delete an unmerged branch, so review the
   selected worktree before confirming cleanup.
