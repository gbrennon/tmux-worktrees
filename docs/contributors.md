# Contributor Guide

## Cloning the Repository

To clone this repository with submodules, use:
```bash
 git clone --recurse-submodules https://codeberg.org/gbrennon/tmux-worktree.git
```

## Prerequisites

Ensure you have the following tools installed:
- `make`: Used to run tests and build the project.
- `fzf`: For interactive worktree selection.
- `bats`: For running BATS tests.

### Installation Instructions

**Ubuntu**:
```bash
 sudo apt-get install make fzf bats
```

**macOS (Homebrew)**:
```bash
 brew install make fzf bats
```

## Running Tests

To run all tests, use the `make` command:
```bash
 make test
```

This will execute the BATS test suite located in the `test/bats/` directory.
