# How it works (under the hood)

1. **Repo discovery** — the script walks up from the active pane's current
   directory until it finds a `.git` folder.  That becomes the repo root.
2. **Worktree storage** — all worktrees live in a `.worktrees/` directory at
   the repo root.  Branch slashes (`feat/foo`) are flattened to dashes
   (`feat-foo`) for directory names.  The plugin adds `.worktrees/` to
   `.git/info/exclude` automatically so git never sees them as untracked files.
3. **Window naming** — each worktree gets a tmux window named
   `wt-<branch-name>` (with the original branch slashes preserved).
   The plugin checks for an existing window with that name before creating a
   new one, so you never end up with duplicates.
4. **Cleanup safety** — the removal flow kills the tmux window first, then
   runs `git worktree remove` followed by `git branch -D` (force-delete
   even unmerged branches), so nothing is left dangling.
