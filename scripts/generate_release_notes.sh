#!/usr/bin/env bash
set -euo pipefail

readonly NOTES_FILE="${RELEASE_NOTES_FILE:-release-notes.md}"

git cliff --config cliff.toml --latest --no-exec --output "$NOTES_FILE"
if [[ ! -s "$NOTES_FILE" ]]; then
  printf 'Generated release notes are empty\n' >&2
  exit 1
fi
