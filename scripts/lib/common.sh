#!/usr/bin/env bash
# Common shared functions for tmux-worktrees scripts
# Source this file in other scripts: source "$(dirname "$0")/lib/common.sh"

set -euo pipefail

# ========================================
# Git Environment Helpers
# ========================================

resolve_current_branch_name() {
  if [ "${CI_EVENT_NAME:-}" = "pull_request" ]; then
    echo "${CI_HEAD_REF:-}"
  else
    echo "${CI_REF:-}" | sed 's|refs/heads/||'
  fi
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
