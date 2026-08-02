# list available recipes
default:
    @just --list

# compile release binary
build:
    cargo build --release

# install release binary to ~/.cargo/bin
install:
    cargo install --path .

# run all tests with terminal coverage report
test:
    cargo llvm-cov --text

# run filtered tests with terminal coverage report (e.g. `just test-filter merge_checker`)
test-filter FILTER:
    cargo llvm-cov --text -- {{FILTER}}

# run all tests with stdout/stderr shown and terminal coverage report
test-verbose:
    cargo llvm-cov --text -- --nocapture

# run tests with terminal coverage report
coverage:
    cargo llvm-cov --text
