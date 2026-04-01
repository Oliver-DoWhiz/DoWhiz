# Task Latency Analysis

## Download timing data from staging

```bash
scp dowhizstaging:/home/azureuser/server/DoWhiz/DoWhiz_service/task_timings.jsonl /tmp/
```

## Generate plots

```bash
cd DoWhiz/DoWhiz_service
python scripts/analyze_timings.py /tmp/task_timings.jsonl
```

## View plots manually

```bash
open /tmp/mean_breakdown.png
open /tmp/stacked_timeline.png
open /tmp/distribution.png
```

## Timing stages

- `setup_latency_ms` - Initial setup before ACI
- `ephemeral_share_create_ms` - Creating Azure file share
- `aci_cold_start_ms` - ACI container spin-up/provisioning from pre-built image
- `codex_execution_ms` - Actual codex running
- `result_download_ms` - Downloading results from ephemeral task fileshare to global fileshare @
`/home/azureuser/server/.dowhiz/DoWhiz/run_task`

## CLI Commands

The `timing_cli` binary provides commands for managing timing logs.

### Build

```bash
cd DoWhiz_service
cargo build -p run_task_module --bin timing_cli --release
```

### Commands (on Local Machine, after scp)

```bash
# Clear the timing log (start fresh)
TIMING_LOG_PATH=/tmp/task_timings.jsonl ./target/release/timing_cli clear

# Show the path to the timing log
TIMING_LOG_PATH=/tmp/task_timings.jsonl ./target/release/timing_cli path

# Display timing log contents
TIMING_LOG_PATH=/tmp/task_timings.jsonl ./target/release/timing_cli show

# Run Python analysis script to generate plots
TIMING_LOG_PATH=/tmp/task_timings.jsonl ./target/release/timing_cli analyze

# Help
./target/release/timing_cli help
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TIMING_LOG_PATH` | `./task_timings.jsonl` | Path to the JSONL timing log file |
| `ANALYZE_SCRIPT_PATH` | `scripts/analyze_timings.py` | Path to the Python analysis script |

---

## Warm Pool Architecture

Warm pool containers eliminate cold start latency by keeping pre-provisioned ACI containers polling for tasks.

### Warm Pool Timing Stages

| Stage | Description |
|-------|-------------|
| `ephemeral_share_create_ms` | Creating Azure File Share for workspace |
| `ephemeral_share_upload_ms` | Uploading workspace files to share |
| `codex_execution_ms` | Queue wait + agent execution time |
| `result_download_ms` | Downloading results from share |

Note: `setup_latency_ms` and `aci_cold_start_ms` do not apply to warm pool - setup happens on the scheduler before pushing to queue, and containers are already running.

---

## Addendum: Warm Pool Debugging Notes

Key fixes made during warm pool rollout debugging (March 2026):

### 1. Agent command `exit 0` killing worker script
**Commit:** `7f10966` - Remove incorrect exit 0 from AGENT_COMMAND

**Problem:** The agent command in `build_warm_pool_agent_command()` ended with `exit 0`, which when `eval`'d in `warm_worker.sh` killed the parent script before it could upload results or signal completion.

**Symptom:** Container logs showed "Running agent..." but no "Uploading results..." or completion message. Tasks appeared stuck.

**Fix:** Removed `exit 0` from the end of the agent command - let the script continue naturally after agent execution.

### 2. Missing PATH in warm container environment
**Commit:** `401e309` - Explicitly set PATH for ACI polling logic

**Problem:** The warm container's polling loop couldn't find `jq` and other tools because PATH wasn't properly set in the container environment.

**Symptom:** `jq: command not found` errors in container logs.

**Fix:** Added explicit PATH export at the top of `warm_worker.sh`:
```bash
export PATH="/app/bin:/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin:/sbin:$PATH"
```

### 3. Missing jq and Azure CLI in base Docker image
**Commit:** `4c9fb02` - Added jq, azure cli for dockerfile image creation

**Problem:** Base Docker image didn't include `jq` (for JSON parsing) or Azure CLI (for queue operations).

**Fix:** Added to `Dockerfile.base`:
```dockerfile
RUN apt-get install -y jq
RUN curl -fsSL https://aka.ms/InstallAzureCLIDeb | bash
```

### 4. Environment variables not passed to warm containers
**Commit:** `e0cf68c` - Added .codex_remote_prompt.txt, essential env vars to provisioning warm containers
**Commit:** `580c4e7` - Align env vars of warm containers, CLIs with original flow

**Problem:** Warm containers were missing critical environment variables that the regular ACI flow had (API keys, workspace paths, etc.).

**Symptom:** Agent couldn't authenticate or find workspace files.

**Fix:** Updated `pool_manager.rs` to pass the required env vars during container creation, matching the original ACI flow. Due to sheer complexity of initial ACI provisioning, not all env vars that existed in original provisioning logic were passed

### 5. Debug logging for queue polling
**Commit:** `64a3110` - Debug logging for warm_worker.sh

**Problem:** Hard to trace why containers weren't picking up tasks.

**Fix:** Added verbose logging to `warm_worker.sh`:
- Poll count tracking
- Queue peek before dequeue (shows queue depth)
- Message content logging
- Clear stage markers (downloading, running, uploading)

### 6. Azure Queue visibility timeout issues
**Commit:** `8c0d965` - True dequeue from azure queue within warm ACI container

**Problem:** Messages would reappear in queue after visibility timeout if container took too long.

**Fix:** Implemented "true dequeue" - delete message immediately after receiving, before processing. Scheduler handles retry logic based on completion message.

### 7. Async pool replenishment
**Commit:** `8566f25` - Pass runtime handler into replenish for async processing

**Problem:** 

```
Main Tokio runtime  ←── Handle points here
       ↑
       │ handle.spawn() sends work here
       │
Scheduler worker thread (not tokio thread, synchronous std::thread) 
replenish() called async function provision_warm_container() from this blocking worker thread, 
and thus needs to get a tokio thread from tokio pool for async processing
```

**Fix:** Store the Tokio runtime handle in `initialize()`, then use `handle.spawn()` to dispatch async work from the synchronous worker thread.

### 8. Timing instrumentation for warm pool
**Commit:** `4de2d89` - Initialize timing collector for warm setup

**Problem:** Warm pool tasks weren't appearing in timing logs.

**Fix:** Added `timing.set_task_id()` and `TIMING_COLLECTOR.record(timing.finish())` to `run_codex_warm_pool()` function.
