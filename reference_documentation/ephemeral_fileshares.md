# Ephemeral File Share Isolation for ACI Tasks

This document describes the ephemeral file share feature that provides per-task workspace isolation when running tasks in Azure Container Instances (ACI).

## Problem Statement

### Current Architecture (Legacy)

The VM and ACI containers share access to the same Azure Files share:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Azure Files Share                                     │
│                      "dowhiz-run-task-prod"                                  │
│  ├── little_bear/                                                           │
│  │   └── users/                                                             │
│  │       ├── abc/workspaces/thread_123/  ◄── Task A can see this           │
│  │       └── xyz/workspaces/thread_456/  ◄── Task A can ALSO see this      │
│  └── other_employee/                                                        │
│      └── users/...                       ◄── Task A can see ALL of this    │
└─────────────────────────────────────────────────────────────────────────────┘
```

| Component | Protocol | Mount Point |
|-----------|----------|-------------|
| VM | CIFS/SMB (Linux kernel mount via `/etc/fstab`) | `/home/azureuser/server/.dowhiz/DoWhiz/run_task` |
| ACI Container | Azure File Volume (`--azure-file-volume-*` args) | `/mnt/dowhiz-share` |

### The Problem

1. **No isolation**: When ACI mounts the file share, it sees the entire share contents - all employees, all users, all workspaces
2. **Azure Files limitation**: Azure Files does not support mounting subdirectories - you must mount the entire share

### The Solution

Create an ephemeral (temporary) file share for each task that contains only that task's workspace:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Main File Share (persistent, ground truth)                                  │
│ dowhiz-run-task-prod                                                        │
│ └── little_bear/users/abc/workspaces/thread_123/                           │
│     ├── .codex_remote_prompt.txt                                           │
│     ├── incoming_email/                                                     │
│     └── memory/                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                               │
                               │ 1. Upload workspace to ephemeral share
                               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ Ephemeral File Share (per-task, temporary)                                  │
│ task-dwz-codex-1711234567890-12345-0                                        │
│ ├── .codex_remote_prompt.txt    ◄── flat structure at root                 │
│ ├── incoming_email/                                                         │
│ └── memory/                                                                 │
└─────────────────────────────────────────────────────────────────────────────┘
                               │
                               │ 2. ACI mounts ONLY this share
                               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ ACI Container                                                               │
│ /mnt/dowhiz-share/              ◄── only sees this one workspace           │
│ ├── .codex_remote_prompt.txt                                               │
│ ├── incoming_email/                                                         │
│ ├── memory/                                                                 │
│ └── .codex_remote_output.log    ◄── written by codex                       │
└─────────────────────────────────────────────────────────────────────────────┘
                               │
                               │ 3. Download results, overwrite shared files
                               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ Main File Share (updated with results)                                      │
│ └── little_bear/users/abc/workspaces/thread_123/                           │
│     ├── .codex_remote_output.log   ◄── synced back                         │
│     └── ...                                                                 │
└─────────────────────────────────────────────────────────────────────────────┘
                               │
                               │ 4. Delete ephemeral share
                               ▼
                          [Cleanup complete]
```

## Configuration

Enable ephemeral file share isolation by setting:

```bash
RUN_TASK_AZURE_ACI_EPHEMERAL_SHARE=1
```

Valid values: `1`, `true` (enabled) or `0`, `false`, unset (disabled)

## Implementation

### Key Components

Located in `run_task_module/src/run_task/codex.rs`:

| Component | Purpose |
|-----------|---------|
| `EPHEMERAL_SHARE_PREFIX` | Constant `"task-"` prefixed to share names |
| `use_ephemeral_share()` | Checks if feature is enabled via env var |
| `create_ephemeral_share()` | Creates Azure file share via `az storage share create` |
| `delete_ephemeral_share()` | Deletes Azure file share via `az storage share delete` |
| `upload_workspace_to_share()` | Uploads workspace via `azcopy` |
| `download_workspace_from_share()` | Downloads results via `azcopy` |
| `EphemeralShareGuard` | RAII struct for automatic cleanup on drop |

### EphemeralShareGuard

```rust
struct EphemeralShareGuard<'a> {
    config: &'a AzureAciConfig,
    share_name: String,
    workspace_dir: PathBuf,
}
```

- **`new()`**: Creates ephemeral share and uploads workspace
- **`share_name()`**: Returns the share name for ACI mount
- **`download_back()`**: Syncs results back to host workspace
- **`Drop`**: Automatically deletes the ephemeral share when guard goes out of scope

### Share Name Format

```
task-dwz-codex-{timestamp_ms}-{process_id}-{sequence}

Example: task-dwz-codex-1711234567890-12345-0
```

### Path Differences

| Mode | File Share | Container Workspace Path |
|------|------------|-------------------------|
| Legacy | `dowhiz-run-task-prod` | `/mnt/dowhiz-share/little_bear/users/abc/workspaces/thread_123` |
| Ephemeral | `task-dwz-codex-xxx` | `/mnt/dowhiz-share` (root, files uploaded flat) |

The `file_share` parameter flows through the call chain:

```
run_codex_task_azure_aci
    │
    ├─ effective_share = ephemeral ? guard.share_name() : config.file_share
    │
    └─► run_azure_aci_execution(..., file_share)
            │
            └─► create_aci_container(..., file_share)
                    │
                    └─► build_aci_create_command(..., file_share)
                            │
                            └─► --azure-file-volume-share-name {file_share}
```

## Execution Flow

### With Ephemeral Share Enabled

```rust
fn run_codex_task_azure_aci(...) {
    let config = load_azure_aci_config()?;
    let host_workspace_dir = canonicalize_dir(request.workspace_dir)?;

    // 1. Create ephemeral share guard (creates share + uploads workspace)
    let ephemeral_guard = if use_ephemeral_share() {
        match EphemeralShareGuard::new(&config, &container_name, &host_workspace_dir) {
            Ok(guard) => Some(guard),
            Err(e) => None,  // Falls back to legacy mode
        }
    } else {
        None
    };

    // 2. Determine effective share and container workspace path
    let (effective_share, effective_container_workspace) = match &ephemeral_guard {
        Some(guard) => (guard.share_name().to_string(), config.container_share_root.clone()),
        None => (config.file_share.clone(), container_workspace_dir.clone()),
    };

    // 3. Run ACI with the appropriate share
    let execution = run_azure_aci_execution(
        &config,
        &container_name,
        &effective_container_workspace,
        ...,
        &effective_share,  // Ephemeral or main share
    );

    // 4. Download results back from ephemeral share
    if let Some(ref guard) = ephemeral_guard {
        guard.download_back()?;
    }

    // 5. Guard drops here -> ephemeral share deleted automatically
}
```

## Azure CLI Commands

### Create Ephemeral Share

```bash
az storage share create \
    --name task-dwz-codex-1711234567890-12345-0 \
    --account-name $STORAGE_ACCOUNT \
    --account-key $STORAGE_KEY
```

### Upload Workspace

Uses `azcopy` for faster parallel uploads:

```bash
export AZURE_STORAGE_ACCOUNT=$STORAGE_ACCOUNT
export AZURE_STORAGE_KEY=$STORAGE_KEY

azcopy copy \
    "/mnt/dowhiz-share/little_bear/users/abc/workspaces/thread_123/*" \
    "https://$STORAGE_ACCOUNT.file.core.windows.net/task-dwz-codex-1711234567890-12345-0/" \
    --recursive
```

### ACI Container Create (with ephemeral share)

```bash
az container create \
    --name dwz-codex-1711234567890-12345-0 \
    --resource-group $RESOURCE_GROUP \
    --image $IMAGE \
    --azure-file-volume-account-name $STORAGE_ACCOUNT \
    --azure-file-volume-account-key $STORAGE_KEY \
    --azure-file-volume-share-name task-dwz-codex-1711234567890-12345-0 \
    --azure-file-volume-mount-path /mnt/dowhiz-share \
    --command-line "/bin/bash -lc 'cd /mnt/dowhiz-share && codex ...'"
```

### Download Results

```bash
export AZURE_STORAGE_ACCOUNT=$STORAGE_ACCOUNT
export AZURE_STORAGE_KEY=$STORAGE_KEY

azcopy copy \
    "https://$STORAGE_ACCOUNT.file.core.windows.net/task-dwz-codex-1711234567890-12345-0/*" \
    "/mnt/dowhiz-share/little_bear/users/abc/workspaces/thread_123" \
    --recursive
```

### Delete Ephemeral Share

```bash
az storage share delete \
    --name task-dwz-codex-1711234567890-12345-0 \
    --account-name $STORAGE_ACCOUNT \
    --account-key $STORAGE_KEY \
    --delete-snapshots include
```

## Files Synced Back

| File | Purpose | Written By |
|------|---------|------------|
| `.codex_remote_output.log` | Codex stdout/stderr | Shell script in ACI |
| `.codex_remote_exit_code` | Exit code for polling | Shell script in ACI |
| `reply_email_draft.html` | Email reply content | Codex |
| `reply_message.txt` | Chat reply (Slack/Discord/Lark) | Codex |
| `reply_routing.json` | Reply metadata | Codex |
| `memory/*.md` | Updated memories | Codex |
| Any user files | Code changes, generated files | Codex |

## Download Behavior

The `azcopy copy` command performs a **merge with overwrite**:

- Files in ephemeral share **overwrite** same-named files in host workspace
- Files that exist only in host workspace (not in ephemeral) are **kept**
- It does NOT delete files that weren't in the ephemeral share

## Unit Tests

Tests are located in `run_task_module/src/run_task/codex.rs`:

| Test | Verifies |
|------|----------|
| `test_use_ephemeral_share_disabled_by_default` | Feature off when env unset |
| `test_use_ephemeral_share_enabled_with_1` | `=1` enables feature |
| `test_use_ephemeral_share_enabled_with_true` | `=true` enables feature |
| `test_use_ephemeral_share_disabled_with_0` | `=0` disables feature |
| `test_ephemeral_share_prefix_format` | Share name format `task-{id}` |
| `test_create_ephemeral_share_calls_az_storage_share_create` | Correct az CLI args |
| `test_delete_ephemeral_share_calls_az_storage_share_delete` | Correct az CLI args |
| `test_upload_workspace_to_share_calls_azcopy` | Correct azcopy args |
| `test_download_workspace_from_share_calls_azcopy` | Correct azcopy args |
| `test_build_aci_create_command_uses_provided_file_share` | `file_share` param used |

Run tests:

```bash
cargo test -p run_task_module -- ephemeral --nocapture
cargo test -p run_task_module -- azcopy --nocapture
```

## Related Documentation

- `azure_vm_worker.md` - VM setup and Azure Files CIFS mount
- `staging_production_deploy.md` - Deployment configuration
