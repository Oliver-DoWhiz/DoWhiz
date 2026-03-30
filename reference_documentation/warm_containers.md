# Warm Container Pool Architecture

## Overview

The warm container pool eliminates the 2-4 minute ACI cold start latency by pre-provisioning containers that poll an Azure Queue for tasks.

### Current Flow (scheduler-side)
```
scheduler: azcopy upload → az container create (with mount) → wait → azcopy download
```

### New Flow (container-side)
```
scheduler: az container create (no mount, pass SAS env vars)
container: azcopy download → run agent → azcopy upload → exit
```

## Architecture Diagram

```
Scheduler Host                          Azure
───────────────                         ─────

pool_manager.rs                         ACI Containers (N)
├─ provision N containers ────────────▶ [warm_worker.sh polling...]
│  (passes queue env vars)              [warm_worker.sh polling...]
│
codex.rs                                Azure Task Queue
├─ create share, upload                      │
├─ push task to queue ──────────────────────▶│
│                                            │     Inside ACI Container
│                                            └───▶ container pops from queue
│                                                  (warm_worker.sh)
│                                                  Gets SAS Token, WORKSPACE_SHARE_URL,
│                                                  agent_command from queue task JSON
│                                                  download, run, upload
│                                                  push completion msg to Azure Completion Queue
│                                                  exit
│                                            ┌────
├─ poll completion queue ◀───────────────────┘
├─ download results
├─ cleanup share
└─ pool_manager.replenish() ─────────────▶ [new container provisioned]
```

## Why Azure Queue?

The polling for a task happens **within the ACI container**. Since ACI doesn't support injecting environment variables after container creation, we need an external mechanism (the queue) to pass task-specific credentials (SAS token, share URL) to pre-provisioned containers.

## Implementation

### 1. bin/warm_worker.sh

Shell script that runs inside the container, polling for tasks:

```bash
#!/bin/bash
set -euo pipefail

QUEUE_NAME="${TASK_QUEUE_NAME:?TASK_QUEUE_NAME not set}"
STORAGE_ACCOUNT="${QUEUE_STORAGE_ACCOUNT:?QUEUE_STORAGE_ACCOUNT not set}"
STORAGE_KEY="${QUEUE_STORAGE_KEY:?QUEUE_STORAGE_KEY not set}"
POLL_INTERVAL="${POLL_INTERVAL:-2}"

echo "[warm_worker] Polling queue: $QUEUE_NAME"

while true; do
    # Get message from queue (visibility timeout 300s = 5min)
    MSG=$(az storage message get \
        --queue-name "$QUEUE_NAME" \
        --account-name "$STORAGE_ACCOUNT" \
        --account-key "$STORAGE_KEY" \
        --visibility-timeout 300 \
        --output json 2>/dev/null | jq -r '.[0] // empty')

    if [ -n "$MSG" ]; then
        MESSAGE_ID=$(echo "$MSG" | jq -r '.id')
        POP_RECEIPT=$(echo "$MSG" | jq -r '.popReceipt')
        CONTENT=$(echo "$MSG" | jq -r '.content' | base64 -d)

        echo "[warm_worker] Received task: $MESSAGE_ID"

        # Extract task info from Azure queue
        export WORKSPACE_SHARE_URL=$(echo "$CONTENT" | jq -r '.share_url')
        export WORKSPACE_SAS_TOKEN=$(echo "$CONTENT" | jq -r '.sas_token')
        export WORKSPACE_LOCAL_DIR="/app/.workspace/task"
        AGENT_COMMAND=$(echo "$CONTENT" | jq -r '.agent_command')
        COMPLETION_QUEUE=$(echo "$CONTENT" | jq -r '.completion_queue')
        TASK_ID=$(echo "$CONTENT" | jq -r '.task_id')

        # Process task
        echo "[warm_worker] Downloading workspace..."
        workspace_sync.sh download  # replaces mount

        echo "[warm_worker] Running agent..."
        AGENT_EXIT_CODE=0
        eval "$AGENT_COMMAND" || AGENT_EXIT_CODE=$?

        echo "[warm_worker] Uploading results..."
        workspace_sync.sh upload

        # Signal completion by pushing onto COMPLETION_QUEUE
        COMPLETION_MSG=$(jq -n --arg tid "$TASK_ID" --arg code "$AGENT_EXIT_CODE" \
            '{task_id: $tid, exit_code: $code}' | base64 -w0)
        az storage message put \
            --queue-name "$COMPLETION_QUEUE" \
            --account-name "$STORAGE_ACCOUNT" \
            --account-key "$STORAGE_KEY" \
            --content "$COMPLETION_MSG"

        # Delete processed message (pop from queue)
        az storage message delete \
            --queue-name "$QUEUE_NAME" \
            --account-name "$STORAGE_ACCOUNT" \
            --account-key "$STORAGE_KEY" \
            --id "$MESSAGE_ID" \
            --pop-receipt "$POP_RECEIPT"

        echo "[warm_worker] Task complete, exiting"
        exit "$AGENT_EXIT_CODE"
    fi

    sleep "$POLL_INTERVAL"
done
```

### 2. bin/workspace_sync.sh

Replaces mount with azcopy sync:

```bash
#!/bin/bash
set -euo pipefail

ACTION="${1:-}"
SHARE_URL="${WORKSPACE_SHARE_URL:?WORKSPACE_SHARE_URL not set}"
SAS="${WORKSPACE_SAS_TOKEN:?WORKSPACE_SAS_TOKEN not set}"
LOCAL_DIR="${WORKSPACE_LOCAL_DIR:-/app/.workspace/task}"

case "$ACTION" in
  download)
    mkdir -p "$LOCAL_DIR"
    azcopy copy "${SHARE_URL}/*?${SAS}" "$LOCAL_DIR" --recursive
    ;;
  upload)
    azcopy copy "${LOCAL_DIR}/*" "${SHARE_URL}?${SAS}" --recursive
    ;;
  *)
    echo "Usage: workspace_sync.sh <download|upload>" >&2
    exit 1
    ;;
esac
```

### 3. Dockerfile.aci additions

```dockerfile
# Add worker scripts
COPY DoWhiz_service/bin/workspace_sync.sh /app/bin/workspace_sync.sh
COPY DoWhiz_service/bin/warm_worker.sh /app/bin/warm_worker.sh
RUN chmod +x /app/bin/workspace_sync.sh /app/bin/warm_worker.sh
RUN ln -sf /app/bin/workspace_sync.sh /usr/local/bin/workspace_sync.sh
RUN ln -sf /app/bin/warm_worker.sh /usr/local/bin/warm_worker.sh
```

### 4. pool_manager.rs

Maintains a running pool of pre-provisioned containers:

```rust
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use uuid::Uuid;

const DEFAULT_POOL_SIZE: usize = 5;
const CONTAINER_PREFIX: &str = "dwz-warm-";

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub resource_group: String,
    pub image: String,
    pub cpu: String,
    pub memory_gb: String,
    pub queue_storage_account: String,
    pub queue_storage_key: String,
    pub task_queue_name: String,
    pub completion_queue_name: String,
}

pub struct PoolManager {
    config: PoolConfig,
    active_count: AtomicUsize,
    target_size: usize,
}

impl PoolManager {
    pub fn new(config: PoolConfig, target_size: Option<usize>) -> Self {
        Self {
            config,
            active_count: AtomicUsize::new(0),
            target_size: target_size.unwrap_or(DEFAULT_POOL_SIZE),
        }
    }

    /// Initialize pool with N warm containers
    pub async fn initialize(&self) -> Result<(), String> {
        let mut handles = Vec::new();
        for _ in 0..self.target_size {
            let config = self.config.clone();
            handles.push(tokio::spawn(async move {
                provision_warm_container(&config).await
            }));
        }

        for handle in handles {
            match handle.await {
                Ok(Ok(_)) => {
                    self.active_count.fetch_add(1, Ordering::SeqCst);
                }
                Ok(Err(e)) => eprintln!("[pool_manager] Provision failed: {}", e),
                Err(e) => eprintln!("[pool_manager] Task failed: {}", e),
            }
        }
        Ok(())
    }

    /// Called when a container finishes - provision replacement
    pub fn replenish(&self) {
        self.active_count.fetch_sub(1, Ordering::SeqCst);

        let current = self.active_count.load(Ordering::SeqCst);
        if current >= self.target_size {
            return;
        }

        let config = self.config.clone();
        tokio::spawn(async move {
            if provision_warm_container(&config).await.is_ok() {
                // increment count
            }
        });
    }
}

async fn provision_warm_container(config: &PoolConfig) -> Result<String, String> {
    let container_name = format!("{}{}", CONTAINER_PREFIX, Uuid::new_v4().simple());

    let output = Command::new("az")
        .arg("container").arg("create")
        .arg("--resource-group").arg(&config.resource_group)
        .arg("--name").arg(&container_name)
        .arg("--image").arg(&config.image)
        .arg("--cpu").arg(&config.cpu)
        .arg("--memory").arg(&config.memory_gb)
        .arg("--restart-policy").arg("Never")
        .arg("--environment-variables")
        .arg(format!("TASK_QUEUE_NAME={}", config.task_queue_name))
        .arg(format!("QUEUE_STORAGE_ACCOUNT={}", config.queue_storage_account))
        .arg(format!("QUEUE_STORAGE_KEY={}", config.queue_storage_key))
        .arg("--command-line")
        .arg("/bin/bash -lc 'warm_worker.sh'")
        .output()
        .map_err(|e| format!("az command failed: {}", e))?;

    if !output.status.success() {
        return Err(format!("az container create failed: {}",
            String::from_utf8_lossy(&output.stderr)));
    }

    Ok(container_name)
}
```

### 5. codex.rs - run_codex_warm_pool

Submits tasks to the warm pool via Azure Queue:

```rust
pub fn run_codex_warm_pool(
    pool_manager: &PoolManager,
    workspace_dir: &Path,
    task_json: &serde_json::Value,
    timeout: Duration,
    timing: &mut TaskTimingBuilder,
) -> Result<RunTaskOutput, RunTaskError> {
    let config = load_azure_aci_config()?;
    let task_id = Uuid::new_v4().to_string();

    // 1. Create ephemeral share and upload workspace
    let share_name = format!("task-{}", Uuid::new_v4().simple());
    create_ephemeral_share(&config, &share_name)?;
    upload_workspace_to_share(&config, &share_name, workspace_dir)?;

    // 2. Generate SAS token
    let sas = generate_share_sas(&config, &share_name)?;
    let share_url = format!(
        "https://{}.file.core.windows.net/{}",
        config.storage_account, share_name
    );

    // 3. Build agent command
    let agent_command = build_agent_command(task_json)?;

    // 4. Push task to queue
    let task_msg = serde_json::json!({
        "task_id": task_id,
        "share_url": share_url,
        "sas_token": sas,
        "agent_command": agent_command,
        "completion_queue": pool_manager.completion_queue(),
    });
    push_to_queue(
        pool_manager.storage_account(),
        pool_manager.storage_key(),
        pool_manager.task_queue(),
        &task_msg,
    )?;

    // 5. Wait for completion message
    let completion = poll_completion_queue(
        pool_manager.storage_account(),
        pool_manager.storage_key(),
        pool_manager.completion_queue(),
        &task_id,
        timeout,
    )?;

    // 6. Download results
    download_workspace_from_share(&config, &share_name, workspace_dir)?;

    // 7. Cleanup share
    delete_ephemeral_share(&config, &share_name)?;

    // 8. Replenish pool (container exited after processing)
    pool_manager.replenish();

    Ok(RunTaskOutput { /* ... */ })
}
```

### 6. warm_pool.rs - Global Pool Manager

```rust
use run_task_module::{PoolConfig, PoolManager};
use std::env;
use std::sync::Arc;

static POOL_MANAGER: std::sync::OnceLock<Option<Arc<PoolManager>>> =
    std::sync::OnceLock::new();

pub fn is_warm_pool_enabled() -> bool {
    env::var("USE_WARM_POOL")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

pub async fn initialize_global_pool_manager() -> Result<bool, String> {
    if !is_warm_pool_enabled() {
        POOL_MANAGER.get_or_init(|| None);
        return Ok(false);
    }

    let config = load_pool_config_from_env()?;
    let pool_size = env::var("WARM_POOL_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok());

    let manager = PoolManager::new(config, pool_size);
    manager.initialize().await?;

    POOL_MANAGER.get_or_init(|| Some(Arc::new(manager)));
    Ok(true)
}

pub fn get_global_pool_manager() -> Option<Arc<PoolManager>> {
    POOL_MANAGER.get().and_then(|opt| opt.clone())
}
```

### 7. server.rs - Initialization

```rust
// Initialize warm container pool if enabled
if let Err(err) = crate::warm_pool::initialize_global_pool_manager().await {
    warn!("Failed to initialize warm pool: {} (falling back to direct ACI)", err);
}
```

### 8. executor.rs - Routing

```rust
let output = if is_warm_pool_enabled() {
    if let Some(pool_manager) = get_global_pool_manager() {
        let task_json = json!({ "model": task.model_name });
        let timeout = Duration::from_secs(3600);
        let mut timing = TaskTimingBuilder::new(&task.workspace_dir.display().to_string());

        run_task_module::run_codex_warm_pool(
            &pool_manager,
            &task.workspace_dir,
            &task_json,
            timeout,
            &mut timing,
        )?
    } else {
        // Fallback to direct ACI
        run_task_module::run_task(&params)?
    }
} else {
    run_task_module::run_task(&params)?
};
```

## Environment Variables

| Env Var | Fallback | Required |
|---------|----------|----------|
| `USE_WARM_POOL` | - | Yes (set to `1` to enable) |
| `WARM_POOL_RESOURCE_GROUP` | `RUN_TASK_AZURE_ACI_RESOURCE_GROUP` | No |
| `WARM_POOL_IMAGE` | `RUN_TASK_AZURE_ACI_IMAGE` or `RUN_TASK_DOCKER_IMAGE` | No |
| `WARM_POOL_STORAGE_ACCOUNT` | `RUN_TASK_AZURE_ACI_STORAGE_ACCOUNT` | No |
| `WARM_POOL_STORAGE_KEY` | `RUN_TASK_AZURE_ACI_STORAGE_KEY` | No |
| `WARM_POOL_SIZE` | `5` | No |
| `WARM_POOL_TASK_QUEUE` | `dowhiz-tasks` | No |
| `WARM_POOL_COMPLETION_QUEUE` | `dowhiz-completions` | No |
| `WARM_POOL_CPU` | `RUN_TASK_AZURE_ACI_CPU` or `2.0` | No |
| `WARM_POOL_MEMORY_GB` | `RUN_TASK_AZURE_ACI_MEMORY_GB` or `4.0` | No |

## Minimal Setup

If you already have the `RUN_TASK_AZURE_ACI_*` environment variables configured, you only need:

```bash
export USE_WARM_POOL=1
```

All other values will fall back to existing ACI configuration.

## Data Flow Summary

```
Scheduler                          Share                     Container
─────────                          ─────                     ─────────
upload_workspace_to_share ────────▶ [data]
                                          ──▶ workspace_sync.sh download
                                              (works on local files)
                                          ◀── workspace_sync.sh upload
download_workspace_from_share ◀─── [results]
```

The `workspace_sync.sh` script replaces what the **mount** used to do inside the ACI. The scheduler-side `upload_workspace_to_share` / `download_workspace_from_share` remain unchanged.
