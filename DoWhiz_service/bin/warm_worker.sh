#!/bin/bash
set -euo pipefail

# Ensure standard paths are available (login shell may reset PATH)
export PATH="/app/bin:/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin:/sbin:$PATH"

# Warm pool worker script.
# Polls Azure Queue for tasks, processes them, and signals completion.
#
# Required env vars:
#   TASK_QUEUE_NAME       - Queue to poll for tasks
#   COMPLETION_QUEUE_NAME - Queue to signal completion
#   QUEUE_STORAGE_ACCOUNT - Azure Storage account name
#   QUEUE_STORAGE_KEY     - Azure Storage account key
#
# Optional env vars:
#   POLL_INTERVAL         - Seconds between polls (default: 2)
#   WORKSPACE_LOCAL_DIR   - Local workspace directory (default: /app/.workspace/task)

TASK_QUEUE="${TASK_QUEUE_NAME:?TASK_QUEUE_NAME not set}"
COMPLETION_QUEUE="${COMPLETION_QUEUE_NAME:?COMPLETION_QUEUE_NAME not set}"
STORAGE_ACCOUNT="${QUEUE_STORAGE_ACCOUNT:?QUEUE_STORAGE_ACCOUNT not set}"
STORAGE_KEY="${QUEUE_STORAGE_KEY:?QUEUE_STORAGE_KEY not set}"
POLL_INTERVAL="${POLL_INTERVAL:-2}"
export WORKSPACE_LOCAL_DIR="${WORKSPACE_LOCAL_DIR:-/app/.workspace/task}"

echo "[warm_worker] Starting, polling queue: $TASK_QUEUE"

while true; do
    # Get message from queue
    AZ_OUTPUT=$(az storage message get \
        --queue-name "$TASK_QUEUE" \
        --account-name "$STORAGE_ACCOUNT" \
        --account-key "$STORAGE_KEY" \
        --output json 2>&1) || {
        echo "[warm_worker] az command failed: $AZ_OUTPUT" >&2
        sleep "$POLL_INTERVAL"
        continue
    }
    MSG=$(echo "$AZ_OUTPUT" | jq -r '.[0] // empty' 2>/dev/null || echo "")

    if [ -n "$MSG" ]; then
        MESSAGE_ID=$(echo "$MSG" | jq -r '.id')
        POP_RECEIPT=$(echo "$MSG" | jq -r '.popReceipt')
        CONTENT=$(echo "$MSG" | jq -r '.content' | base64 -d)

        TASK_ID=$(echo "$CONTENT" | jq -r '.task_id')
        echo "[warm_worker] Received task: $TASK_ID"

        # True dequeue: delete immediately to avoid visibility timeout issues
        az storage message delete \
            --queue-name "$TASK_QUEUE" \
            --account-name "$STORAGE_ACCOUNT" \
            --account-key "$STORAGE_KEY" \
            --id "$MESSAGE_ID" \
            --pop-receipt "$POP_RECEIPT" \
            --output none

        # Extract task info and export for workspace_sync.sh
        export WORKSPACE_SHARE_URL=$(echo "$CONTENT" | jq -r '.share_url')
        export WORKSPACE_SAS_TOKEN=$(echo "$CONTENT" | jq -r '.sas_token')
        AGENT_COMMAND=$(echo "$CONTENT" | jq -r '.agent_command')

        # Process task
        echo "[warm_worker] Downloading workspace..."
        workspace_sync.sh download

        echo "[warm_worker] Running agent..."
        AGENT_EXIT_CODE=0
        eval "$AGENT_COMMAND" || AGENT_EXIT_CODE=$?
        echo "[warm_worker] Agent exited with code: $AGENT_EXIT_CODE"

        echo "[warm_worker] Uploading results..."
        workspace_sync.sh upload

        # Signal completion (scheduler handles retry logic based on exit_code)
        COMPLETION_MSG=$(jq -n \
            --arg tid "$TASK_ID" \
            --argjson code "$AGENT_EXIT_CODE" \
            '{task_id: $tid, exit_code: $code}' | base64 -w0)

        az storage message put \
            --queue-name "$COMPLETION_QUEUE" \
            --account-name "$STORAGE_ACCOUNT" \
            --account-key "$STORAGE_KEY" \
            --content "$COMPLETION_MSG" \
            --output none

        echo "[warm_worker] Task $TASK_ID complete, exiting"
        exit "$AGENT_EXIT_CODE"
    fi

    sleep "$POLL_INTERVAL"
done
