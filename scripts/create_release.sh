#!/usr/bin/env bash
set -euo pipefail

readonly API_URL="${RELEASE_API_URL:?RELEASE_API_URL is required}"
readonly REPOSITORY="${RELEASE_REPOSITORY:?RELEASE_REPOSITORY is required}"
readonly TOKEN="${RELEASE_TOKEN:?RELEASE_TOKEN is required}"
readonly VERSION="${VERSION:?VERSION is required}"
readonly AUTH_SCHEME="${RELEASE_AUTH_SCHEME:-Bearer}"
readonly TARGET_COMMITISH="${RELEASE_TARGET_COMMITISH:-}"
readonly NOTES_FILE="${RELEASE_NOTES_FILE:-release-notes.md}"

if [[ ! -s "$NOTES_FILE" ]]; then
  printf 'Release notes are missing or empty: %s\n' "$NOTES_FILE" >&2
  exit 1
fi

payload_args=(
  --arg tag_name "$VERSION"
  --arg name "$VERSION"
  --arg target_commitish "$TARGET_COMMITISH"
  --rawfile body "$NOTES_FILE"
)

payload="$(jq -n "${payload_args[@]}" \
  '{tag_name: $tag_name, name: $name, body: $body, draft: false,
    prerelease: false} +
   (if ($target_commitish | length) > 0 then {target_commitish: $target_commitish} else {} end)')"

response_file="$(mktemp)"
trap 'rm -f "$response_file"' EXIT
status="$(curl --silent --show-error --request POST \
  --url "${API_URL%/}/repos/${REPOSITORY}/releases" \
  --header "Authorization: ${AUTH_SCHEME} ${TOKEN}" \
  --header 'Accept: application/json' \
  --header 'Content-Type: application/json' \
  --data "$payload" --output "$response_file" --write-out '%{http_code}')"

if [[ "$status" != 2* ]]; then
  printf 'Release API returned HTTP status: %s\n' "$status" >&2
  cat "$response_file" >&2
  exit 1
fi
release_id="$(jq -r '.id' "$response_file")"
upload_url="$(jq -r '.upload_url // empty' "$response_file" | sed 's/{?name,label}$//')"
if [[ -z "$release_id" || "$release_id" == null || -z "$upload_url" ]]; then
  printf 'Release API response did not include release id and upload URL\n' >&2
  exit 1
fi
printf 'release-id=%s\n' "$release_id" >> "${GITHUB_OUTPUT:-/dev/null}"
printf 'upload-url=%s\n' "$upload_url" >> "${GITHUB_OUTPUT:-/dev/null}"
cat "$response_file"
