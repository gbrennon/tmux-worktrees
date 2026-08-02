#!/usr/bin/env bash
# shellcheck disable=SC1091
set -euo pipefail

# Source common functions
source "$(dirname "$0")/lib/common.sh"

collect_commit_messages() {
  local range
  range="$(resolve_commit_range)"
  if [ -n "$range" ]; then
    git log --format=%s "$range" || true
  else
    git log --format=%s -n 20 || true
  fi
}

abort_if_any_commit_message_violates_conventional_commits() {
  local commits="$1"
  local types="feat|fix|docs|style|refactor|perf|test|chore|revert|ci|build"
  local merge="^Merge .+"
  local pattern="^(${types})(\(.+\))?: ?.+"

  echo "$commits" | while IFS= read -r msg; do
    if [ -z "$msg" ]; then
      continue
    fi

    if ! echo "$msg" | grep -qE "(${pattern}|${merge})"; then
      echo "Invalid conventional commit message: '$msg'" >&2
      echo "Allowed types: ${types//|/, }" >&2
      exit 1
    fi
  done
}

validate_commit_messages() {
  local commits
  commits="$(collect_commit_messages)"

  if [ -z "$commits" ]; then
    echo "No commits to check"
    return
  fi

  abort_if_any_commit_message_violates_conventional_commits "$commits"
  echo "Commit message check: PASS"
}

validate_commit_messages
