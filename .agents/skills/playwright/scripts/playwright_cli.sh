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

derive_workspace_session_name() {
  local session_source="$1"

  if command -v python3 >/dev/null 2>&1; then
    PLAYWRIGHT_SESSION_SOURCE="$session_source" python3 - <<'PY'
import hashlib
import os

source = os.environ.get("PLAYWRIGHT_SESSION_SOURCE", "").strip()
if not source:
    source = "dowhiz"
digest = hashlib.sha256(source.encode("utf-8")).hexdigest()[:16]
print(f"dowhiz-{digest}")
PY
    return
  fi

  if command -v shasum >/dev/null 2>&1; then
    printf 'dowhiz-%s\n' "$(printf '%s' "$session_source" | shasum -a 256 | awk '{print substr($1, 1, 16)}')"
    return
  fi

  printf 'dowhiz-default\n'
}

ensure_workspace_cli_session() {
  if [[ "${has_session_flag}" == "true" || -n "${PLAYWRIGHT_CLI_SESSION:-}" ]]; then
    return 0
  fi

  local session_source="${BROWSERBASE_STATE_DIR:-${PWD}}"
  export PLAYWRIGHT_CLI_SESSION
  PLAYWRIGHT_CLI_SESSION="$(derive_workspace_session_name "$session_source")"
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
  ensure_workspace_cli_session
}

build_playwright_cli_cmd() {
  cmd=(npx --yes --package @playwright/cli playwright-cli)
  if [[ "${has_session_flag}" != "true" && -n "${PLAYWRIGHT_CLI_SESSION:-}" ]]; then
    cmd+=(--session "${PLAYWRIGHT_CLI_SESSION}")
  fi
}

browser_session_probe() {
  local output
  local status
  local -a probe_cmd

  build_playwright_cli_cmd
  probe_cmd=("${cmd[@]}")

  set +e
  output="$("${probe_cmd[@]}" eval "location.href" 2>&1)"
  status=$?
  set -e

  if [[ $status -eq 0 ]]; then
    return 0
  fi

  if [[ "$output" == *"is not open, please run open first"* ]]; then
    return 10
  fi

  echo "$output" >&2
  return 20
}

maybe_rewrite_open_to_goto() {
  if [[ -z "${PLAYWRIGHT_MCP_CDP_ENDPOINT:-}" ]]; then
    return 0
  fi
  if [[ "${has_session_flag}" == "true" ]]; then
    return 0
  fi
  if [[ $# -ne 2 || "$1" != "open" || "$2" == -* ]]; then
    return 0
  fi

  local probe_status=0
  set +e
  browser_session_probe
  probe_status=$?
  set -e

  case "$probe_status" in
    0)
      set -- "goto" "$2"
      ;;
    10)
      ;;
    *)
      exit 1
      ;;
  esac

  PLAYWRIGHT_WRAPPER_ARGS=("$@")
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

PLAYWRIGHT_WRAPPER_ARGS=("$@")
maybe_rewrite_open_to_goto "${PLAYWRIGHT_WRAPPER_ARGS[@]}"
build_playwright_cli_cmd
cmd+=("${PLAYWRIGHT_WRAPPER_ARGS[@]}")

exec "${cmd[@]}"
