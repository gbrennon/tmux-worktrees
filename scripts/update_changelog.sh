#!/usr/bin/env bash
set -euo pipefail

readonly VERSION="${VERSION:?VERSION is required}"
readonly GIT_USER_NAME="${GIT_USER_NAME:?GIT_USER_NAME is required}"
readonly GIT_USER_EMAIL="${GIT_USER_EMAIL:?GIT_USER_EMAIL is required}"
readonly CHANGELOG_FILE="${CHANGELOG_FILE:-CHANGELOG.md}"

git cliff --config cliff.toml --tag "$VERSION" --no-exec --output "$CHANGELOG_FILE"
if [[ ! -s "$CHANGELOG_FILE" ]]; then
  printf 'Generated changelog is empty\n' >&2
  exit 1
fi

git add -- "$CHANGELOG_FILE"
if git diff --cached --quiet -- "$CHANGELOG_FILE"; then
  printf 'Changelog is already current\n'
  exit 0
fi

git -c user.name="$GIT_USER_NAME" -c user.email="$GIT_USER_EMAIL" \
  commit --message 'chore(release): update changelog'
git push origin "HEAD:${TARGET_BRANCH:-$HEAD_REF}"
