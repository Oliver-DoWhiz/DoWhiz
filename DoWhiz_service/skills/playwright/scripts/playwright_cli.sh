#!/usr/bin/env bash
set -euo pipefail

if ! command -v npx >/dev/null 2>&1; then
  echo "Error: npx is required but not found on PATH." >&2
  exit 1
fi

print_browserbase_error() {
  local raw_output="$1"
  local parsed_message

  if command -v python3 >/dev/null 2>&1; then
    if parsed_message="$(
      RAW_OUTPUT="$raw_output" python3 - <<'PY'
import json
import os
import sys

text = os.environ.get("RAW_OUTPUT", "").strip()
if not text:
    sys.exit(1)
try:
    payload = json.loads(text)
except json.JSONDecodeError:
    sys.exit(1)
message = str(payload.get("error", "")).strip()
if not message:
    sys.exit(1)
print(message)
PY
    )"; then
      echo "$parsed_message" >&2
      return
    fi
  fi

  echo "$raw_output" >&2
}

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
  if ! exports="$(browserbase_session_manager --state-dir "$state_dir" ensure-session --format shell 2>&1)"; then
    print_browserbase_error "$exports"
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
