# Contributor Guide

## Cloning the Repository

To clone this repository with submodules, use:
```bash
 git clone --recurse-submodules https://codeberg.org/gbrennon/tmux-worktree.git
```

## Prerequisites

Ensure you have the following tools installed:
- `just`: Used to run tests and build the project.
- `fzf`: For interactive worktree selection.

### Installation Instructions

**Ubuntu**:
```bash
cargo install just
sudo apt-get install fzf
```

**macOS (Homebrew)**:
```bash
brew install just fzf
```

## Running Tests

To run all tests, use the `just` command:
```bash
just test
```

This will execute the Rust test suite located in the `tests/` directory.
