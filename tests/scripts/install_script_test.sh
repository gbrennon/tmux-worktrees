#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TEMP_DIR"' EXIT

fail() {
  printf 'FAIL: %s\n' "$1" >&2
  exit 1
}

make_fixture() {
  local fixture="$1"
  mkdir -p "$fixture/bin" "$fixture/src"
  cp "$ROOT/worktree.tmux" "$fixture/worktree.tmux"
  printf '[package]\nname = "tmux-worktrees"\nversion = "0.4.0"\n' > "$fixture/Cargo.toml"
  printf 'fn main() {}\n' > "$fixture/src/main.rs"

  cat > "$fixture/bin/tmux" <<'EOF'
#!/usr/bin/env bash
case "${1:-}" in
  show-option|list-keys) exit 1 ;;
  *) exit 0 ;;
esac
EOF
  cat > "$fixture/bin/cargo" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "${CARGO_LOG:?CARGO_LOG is required}"
mkdir -p target/release
printf '#!/usr/bin/env bash\nexit 0\n' > target/release/tmux-worktrees
chmod +x target/release/tmux-worktrees
EOF
  cat > "$fixture/bin/curl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${CURL_FAIL_DOWNLOAD:-false}" == true && "$*" == *'releases/download'* ]]; then
  exit 1
fi
if [[ "$*" != *'releases/download'* ]]; then
  printf '{"tag_name":"v0.4.0"}\n'
  exit 0
fi
output=''
for ((i = 1; i <= $#; i++)); do
  eval "arg=\${$i}"
  if [[ "$arg" == -o ]]; then
    ((i++))
    eval "output=\${$i}"
  fi
done
mkdir -p "$(dirname "$output")"
tar -czf "$output" -C "${ARCHIVE_SOURCE:?ARCHIVE_SOURCE is required}" tmux-worktrees
EOF
  chmod +x "$fixture/bin"/*
}

run_installer() {
  local fixture="$1"
  (cd "$fixture" && HOME="$fixture/home" PATH="$fixture/bin:$PATH" \
    CARGO_LOG="$fixture/cargo.log" ARCHIVE_SOURCE="$fixture/archive" \
    bash "$fixture/worktree.tmux")
}

fixture="$TEMP_DIR/download"
make_fixture "$fixture"
mkdir -p "$fixture/archive" "$fixture/home"
printf '#!/usr/bin/env bash\nexit 0\n' > "$fixture/archive/tmux-worktrees"
chmod +x "$fixture/archive/tmux-worktrees"
run_installer "$fixture"
[[ -x "$fixture/home/.tmux/plugins/tmux-worktrees/tmux-worktrees" ]] || fail 'GitHub asset was not installed'
[[ ! -s "$fixture/cargo.log" ]] || fail 'Cargo build ran despite successful asset download'

fixture="$TEMP_DIR/fallback"
make_fixture "$fixture"
mkdir -p "$fixture/archive" "$fixture/home"
printf '#!/usr/bin/env bash\nexit 0\n' > "$fixture/archive/tmux-worktrees"
chmod +x "$fixture/archive/tmux-worktrees"
CURL_FAIL_DOWNLOAD=true run_installer "$fixture"
[[ -x "$fixture/home/.tmux/plugins/tmux-worktrees/tmux-worktrees" ]] || fail 'source fallback did not install binary'
grep -q 'build --release' "$fixture/cargo.log" || fail 'source fallback did not invoke cargo build'

printf 'installer script smoke tests: PASS\n'
