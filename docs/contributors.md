# Contributor Guide

## Cloning the Repository

To clone this repository, use:
```bash
 git clone https://codeberg.org/gbrennon/tmux-worktrees.git
```

## Prerequisites

Ensure you have the following tools installed:
- Rust stable and Cargo: Used to build and test the application.
- `just`: Used to run tests and build the project.

### Installation Instructions

**Ubuntu**:
```bash
cargo install just
```

**macOS (Homebrew)**:
```bash
brew install just
```

## Running Tests

To run all tests, use the `just` command:
```bash
just test
```

This will execute the Rust test suite located in the `tests/` directory.
