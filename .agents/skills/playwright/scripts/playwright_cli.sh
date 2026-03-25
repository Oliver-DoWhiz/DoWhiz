#!/usr/bin/env bash
set -euo pipefail

if ! command -v npx >/dev/null 2>&1; then
  echo "Error: npx is required but not found on PATH." >&2
  exit 1
fi

ensure_browserbase_session() {
  if [[ -z "${BROWSERBASE_API_KEY:-${BROWSER_BASE_API_KEY:-}}" ]]; then
    return 0
  fi
  if ! command -v browserbase_session_manager >/dev/null 2>&1; then
    echo "Error: browserbase_session_manager is required for Browserbase-backed playwright sessions." >&2
    exit 1
  fi

  local state_dir="${BROWSERBASE_STATE_DIR:-.secrets/browserbase}"
  local exports
  if ! exports="$(browserbase_session_manager ensure-session --state-dir "$state_dir" --format shell 2>&1)"; then
    echo "$exports" >&2
    exit 1
  fi
  eval "$exports"
}

has_session_flag="false"
for arg in "$@"; do
  case "$arg" in
    --session|--session=*)
      has_session_flag="true"
      break
      ;;
  esac
done

ensure_browserbase_session

cmd=(npx --yes --package @playwright/cli playwright-cli)
if [[ "${has_session_flag}" != "true" && -n "${PLAYWRIGHT_CLI_SESSION:-}" ]]; then
  cmd+=(--session "${PLAYWRIGHT_CLI_SESSION}")
fi
cmd+=("$@")

exec "${cmd[@]}"
