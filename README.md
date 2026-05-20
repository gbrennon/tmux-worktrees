# tmux-worktrees

A tmux plugin to create, switch between, and clean up
[git worktrees][git-worktree] without leaving your terminal.

**Demo**: `prefix + W` opens a picker — type a branch name, press Enter, and
you're in a fresh tmux window pointing at that worktree.  `prefix + D` removes
one just as fast.

---

## Install

Add to `~/.tmux.conf` and reload (`prefix + I`):

```tmux
set -g @plugin 'gbrennon/tmux-worktrees'
```

**Requires** [`fzf`][fzf] on your `$PATH`.  Also needs tmux >= 2.4, git >= 2.5,
bash >= 4.0.

```bash
# Fedora        |  # Debian/Ubuntu    |  # Arch          |  # macOS
sudo dnf install fzf | sudo apt install fzf | sudo pacman -S fzf | brew install fzf
```

---

## Keybindings

| Keys           | What it does                  | Details |
|----------------|-------------------------------|---------|
| `prefix` + `W` | Create or switch to worktree  | [docs](docs/create-worktree.md) |
| `prefix` + `D` | Remove merged/stale worktrees | [docs](docs/cleanup-worktrees.md) |

> `prefix` is `Ctrl-b` by default.  Override the keys by setting these
> variables in `~/.tmux.conf` **before** the plugin line — for example:
>
> ```tmux
> set -g @worktree-key 'T'           # prefix + T to pick a worktree
> set -g @worktree-cleanup-key 'X'   # prefix + X to clean up
> ```
>
> More details: [Customise the keys](docs/customise-keys.md).
> You can also [change the shell](docs/customise-shell.md) that opens in new windows.

---

## Quick start

```
cd ~/my-project
prefix + W
```
Type `feat/foo` -> Enter.  A worktree is created at `.worktrees/feat-foo` and a
new tmux window `wt-feat/foo` opens there.  Press `prefix + W` again to switch
between worktrees, `prefix + D` to clean up.

> Examples use [conventional branch][conv-branch] prefixes (`feat/`, `fix/`,
> `chore/`), but any branch name works — `my-branch` or `bugfix/login` are
> equally valid.

---

## More

| Topic | |
|-------|---|
| [Creating worktrees](docs/create-worktree.md) | Step-by-step walkthrough |
| [Switching worktrees](docs/switch-worktrees.md) | Using the picker as a dashboard |
| [Cleaning up](docs/cleanup-worktrees.md) | Removing worktrees and branches |
| [Customise keys](docs/customise-keys.md) | Change `W` / `D` bindings |
| [Customise shell](docs/customise-shell.md) | Launch `bash`, `zsh`, `vim`, `emacs`, etc. |
| [How it works](docs/how-it-works.md) | Under-the-hood architecture |

---

## License

MIT

[git-worktree]: https://git-scm.com/docs/git-worktree
[tpm]: https://github.com/tmux-plugins/tpm
[fzf]: https://github.com/junegunn/fzf
[conv-branch]: https://conventional-branch.github.io/
