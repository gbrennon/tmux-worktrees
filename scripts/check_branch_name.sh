#!/usr/bin/env bash
# shellcheck disable=SC1091
set -euo pipefail

# Source common functions
source "$(dirname "$0")/lib/common.sh"

abort_if_branch_name_violates_naming_convention() {
  readonly VALID_BRANCH_PATTERN='^(main|master|develop|feature/.+|feat/.+|bugfix/.+|fix/.+|hotfix/.+|release/.+|chore/.+)$'
  local branch="$1"

  if [ -z "$branch" ]; then
    echo "Error: could not determine the current branch name" >&2
    exit 1
  fi

  if [[ ! "$branch" =~ $VALID_BRANCH_PATTERN ]]; then
    echo "Branch name '$branch' does not match allowed patterns" >&2
    exit 1
  fi
}

validate_branch_name() {
  local branch
  branch="$(resolve_current_branch_name)"
  echo "Branch: $branch"
  abort_if_branch_name_violates_naming_convention "$branch"
  echo "Branch name check: PASS"
}

validate_branch_name
