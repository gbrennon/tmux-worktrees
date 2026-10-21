# tmux-worktrees

A tmux plugin to create, switch between, and clean up
[git worktrees][git-worktree] without leaving your terminal.

## Features

- Create worktrees from a branch name and open them in dedicated tmux windows.
- Resume and switch to Git-registered worktrees, including worktrees outside the
  configured directory. The picker resolves paths from the active repository
  context and avoids listing that resolved root itself.
- Keep managing external worktrees: switch to them, inspect their branches,
  and remove them through the same cleanup flow.
- Customize the worktree directory, keybindings, and shell used by new windows.
- Clean up merged or stale worktrees without leaving tmux windows behind.

**Demo**: `prefix + W` opens a picker — type a branch name, press Enter, and
you're in a fresh tmux window pointing at that worktree.  `prefix + D` removes
one just as fast.

---

## Install

Add to `~/.tmux.conf` and reload (`prefix + I`):

```tmux
set -g @plugin 'gbrennon/tmux-worktrees'
```

Also needs tmux >= 3.2, git >= 2.28, bash >= 4.0, `curl`, and `tar`.
TPM downloads a pre-built binary when one is available and otherwise builds it
with the Rust toolchain.

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
> set -g @worktree-dir 'my-trees'    # custom worktree directory
> ```
>
> More details: [Customise the keys](docs/customise-keys.md).
> You can also [change the shell](docs/customise-shell.md) that opens in new windows.
> Or [change the worktree directory](docs/customise-worktree-dir.md).

---

## Quick start

```
cd ~/my-project
prefix + W
```
Type `feat/foo` -> Enter. A worktree is created (by default at
`.workspaces/feat-foo`) and a new tmux window `ws-feat/foo` opens there.
Press `prefix + W` again to switch between worktrees, including existing
worktrees outside `.workspaces`; press `prefix + D` to clean up.

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
| [Customise worktree dir](docs/customise-worktree-dir.md) | Use a different worktree directory |
| [How it works](docs/how-it-works.md) | Under-the-hood architecture |
| [Releasing](docs/releasing.md) | Release branches and publishing |

---

## Testing

Run the Rust test suite from the project root:

```bash
cargo test
```

For the coverage-enforced project test recipe, install `just` and
`cargo-llvm-cov`, then run `just test`. Tests create and tear down temporary
git repositories in `/tmp` — no project files are touched.

---

## License

MIT

[git-worktree]: https://git-scm.com/docs/git-worktree
[tpm]: https://github.com/tmux-plugins/tpm
[conv-branch]: https://conventional-branch.github.io/
