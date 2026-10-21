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
- `cargo-llvm-cov`: Used by the coverage-enforced test recipe.

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

To run the Rust test suite located in the `tests/` directory:
```bash
cargo test
```

For coverage enforcement, install `cargo-llvm-cov` and run:
```bash
just test
```
