# list available recipes
default:
    @just --list

# compile release binary
build:
    cargo build --release

# install release binary to ~/.cargo/bin
install:
    cargo install --path .

# run all tests with coverage summary table
test:
    cargo llvm-cov --fail-under-lines 70 --summary-only

# run filtered tests with coverage summary (e.g. `just test-filter merge_checker`)
test-filter FILTER:
    cargo llvm-cov --summary-only -- {{FILTER}}

# run all tests with stdout/stderr shown and coverage summary
test-verbose:
    cargo llvm-cov --summary-only -- --nocapture

# run tests with line-by-line coverage annotation in terminal
coverage:
    cargo llvm-cov --text

# format source files (optionally specify files)
fmt *FILES:
    cargo fmt {{FILES}}

# check formatting without modifying files
fmt-check:
    cargo fmt --check

# lint with zero warnings enforced
lint:
    cargo clippy -- -D warnings

# lint fixes (optionally specify files)
lint-fix +FILES='':
    cargo clippy --fix --allow-dirty --allow-staged {{FILES}}

# install required dev tools (rustfmt, clippy, cargo-llvm-cov)
tools:
    rustup component add rustfmt clippy
    cargo install cargo-llvm-cov --locked --force

# install git hooks via lefthook
install-hooks:
    pipx install lefthook
    lefthook install

# validate workflow files statically (requires actionlint)
lint-workflows:
    actionlint -config-file .actionlint.yaml .github/workflows/*.yml