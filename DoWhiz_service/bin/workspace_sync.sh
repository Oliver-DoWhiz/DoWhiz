#!/bin/bash
set -euo pipefail

# Workspace sync script for warm pool containers.
# Uses azcopy to transfer files between Azure File Share and local workspace.
#
# Usage: workspace_sync.sh <download|upload>
#
# Required env vars:
#   WORKSPACE_SHARE_URL  - Azure File Share URL
#   WORKSPACE_SAS_TOKEN  - SAS token for authentication
#   WORKSPACE_LOCAL_DIR  - Local directory (default: /app/.workspace/task)

ACTION="${1:-}"
SHARE_URL="${WORKSPACE_SHARE_URL:?WORKSPACE_SHARE_URL not set}"
SAS="${WORKSPACE_SAS_TOKEN:?WORKSPACE_SAS_TOKEN not set}"
LOCAL_DIR="${WORKSPACE_LOCAL_DIR:-/app/.workspace/task}"

case "$ACTION" in
  download)
    echo "[workspace_sync] Downloading from share to $LOCAL_DIR"
    mkdir -p "$LOCAL_DIR"
    azcopy copy "${SHARE_URL}/*?${SAS}" "$LOCAL_DIR" --recursive --log-level=ERROR
    echo "[workspace_sync] Download complete"
    ;;
  upload)
    echo "[workspace_sync] Uploading from $LOCAL_DIR to share"
    azcopy copy "${LOCAL_DIR}/*" "${SHARE_URL}?${SAS}" --recursive --log-level=ERROR
    echo "[workspace_sync] Upload complete"
    ;;
  *)
    echo "Usage: workspace_sync.sh <download|upload>" >&2
    exit 1
    ;;
esac
