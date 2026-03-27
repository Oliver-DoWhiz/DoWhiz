# Task Latency Analysis

## Download timing data from staging

```bash
scp dowhizstaging:/home/azureuser/server/DoWhiz/DoWhiz_service/task_timings.jsonl /tmp/
```

## Generate plots

```bash
cd /Users/dylantang/DoWhiz/DoWhiz_service
python scripts/analyze_timings.py /tmp/task_timings.jsonl
```

## View plots

```bash
open /tmp/mean_breakdown.png
open /tmp/stacked_timeline.png
open /tmp/distribution.png
```

## Timing stages

- `setup_latency_ms` - Initial setup before ACI
- `ephemeral_share_create_ms` - Creating Azure file share
- `aci_cold_start_ms` - ACI container spin-up
- `codex_execution_ms` - Actual codex running
- `result_download_ms` - Downloading results from ephemeral task fileshare to global fileshare @
`/home/azureuser/server/.dowhiz/DoWhiz/run_task`
