#!/usr/bin/env bash
# Common shared functions for tmux-worktrees scripts
# Source this file in other scripts: source "$(dirname "$0")/lib/common.sh"

set -euo pipefail

# ========================================
# Git Environment Helpers
# ========================================

resolve_current_branch_name() {
  local branch=""

  if [ "${CI_EVENT_NAME:-}" = "pull_request" ]; then
    branch="${CI_HEAD_REF:-}"
  else
    branch="${CI_REF:-}"
    branch="${branch#refs/heads/}"
  fi

  if [ -z "$branch" ]; then
    branch="${GITHUB_REF_NAME:-}"
  fi

  if [ -z "$branch" ]; then
    branch="$(git branch --show-current 2>/dev/null || true)"
  fi

  if [ -z "$branch" ] || [ "$branch" = "HEAD" ]; then
    branch="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || true)"
    [ "$branch" = "HEAD" ] && branch=""
  fi

  printf '%s' "$branch"
}

resolve_commit_range() {
  if [ "${CI_EVENT_NAME:-}" = "pull_request" ]; then
    local base="${CI_BASE_REF:-}"
    local head="${CI_HEAD_REF:-}"
    if [ -n "$base" ] && [ -n "$head" ]; then
      echo "origin/${base}..origin/${head}"
    fi
  else
    local before_sha="${CI_BEFORE_SHA:-}"
    local sha="${CI_SHA:-}"
    if [ -n "$before_sha" ] && [ -n "$sha" ]; then
      echo "${before_sha}..${sha}"
    fi
  fi
}
