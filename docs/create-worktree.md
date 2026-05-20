# Create a worktree

1. Inside tmux, `cd` into **any** git repository.
2. Press `prefix` + `W`.
3. An fzf popup appears — type a branch name, e.g. `feat/foo`.
4. Press `Enter`.

If the branch doesn't exist yet, tmux-worktrees will:

- Create a worktree at `.worktrees/feat-foo` inside your repo
  (slashes in branch names are replaced with dashes for the directory),
  branching off `main` (or `master`, whichever exists).
- Open a new tmux window named `wt-feat/foo` with your shell's
  working directory already set to that worktree.

If the worktree already exists, the same keybinding switches you to its tmux
window — so `prefix + W` doubles as a fast project-wide window switcher.

> **Note:** examples use [conventional branch][conv-commits] prefixes like
> `feat/`, `fix/`, `chore/` for readability, but any branch name works —
> `my-branch` or `bugfix/login` are equally valid.

[conv-commits]: https://www.conventionalcommits.org/
