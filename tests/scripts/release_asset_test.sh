#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TEMP_DIR"' EXIT

mkdir -p "$TEMP_DIR/bin" "$TEMP_DIR/project" "$TEMP_DIR/dist"
printf '[package]\nname = "tmux-worktrees"\nversion = "0.4.0"\n' > "$TEMP_DIR/project/Cargo.toml"
cat > "$TEMP_DIR/bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$(dirname "${CARGO_MANIFEST:?}")/target/release"
printf '#!/usr/bin/env bash\nexit 0\n' > "$(dirname "$CARGO_MANIFEST")/target/release/tmux-worktrees"
chmod +x "$(dirname "$CARGO_MANIFEST")/target/release/tmux-worktrees"
EOF
chmod +x "$TEMP_DIR/bin/cargo"

PATH="$TEMP_DIR/bin:$PATH" \
  VERSION=v0.4.0 \
  CARGO_MANIFEST="$TEMP_DIR/project/Cargo.toml" \
  RELEASE_ASSET_DIR="$TEMP_DIR/dist" \
  bash "$ROOT/scripts/package_release_asset.sh" >/dev/null

asset="$TEMP_DIR/dist/tmux-worktrees-x86_64-unknown-linux-gnu.tar.gz"
test -s "$asset" || { echo 'release asset was not created' >&2; exit 1; }
tar -tzf "$asset" | grep -qx 'tmux-worktrees' || {
  echo 'release asset does not contain the binary' >&2
  exit 1
}
printf 'release asset script tests: PASS\n'
