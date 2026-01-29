# TECH_DESIGN

## Scope
This document defines the MVP tech design for Icebrew "Digital Employee":
- Email-driven task intake for `main@icebrew.ai`
- Per-user workspace and thread memory with attachment versioning
- Task classification, billing quota, retries, and concurrency safety
- Modular, CLI-testable components

This doc borrows structure and rigor from both Moltbot tech designs (Codex + Claude), but adapts them to an email-first, single-agent MVP.

## Product goals
- Users work entirely from their email client.
- Tens of concurrent tasks without server instability.
- Clear, enforceable free-tier limits and upgrade messaging.
- Reliable, observable workflow with minimal manual intervention.
- MVP first; future-proof for tags, routing, monitoring, and research delivery.

## Non-goals (MVP)
- Real-time chat UI or multi-channel gateways.
- Long-running multi-agent orchestration.
- Advanced billing/usage metering beyond task counts.

## System overview
High-level flow:
1) Postmark inbound webhook receives email for `main@icebrew.ai`.
2) Webhook handler validates, normalizes, and enqueues a job quickly.
3) Celery workers process jobs asynchronously (Claude agent SDK runs here).
4) Workspace and thread memory are stored in Azure Blob.
5) PostgreSQL stores users, threads, messages, task status, and quotas.
6) Postmark outbound sends replies to the user.

Key design choice: keep webhook handler fast and stateless; all heavy work is async.

### Architecture diagram

```
[Postmark Inbound]
       |
       v
[FastAPI Webhook] --(enqueue)--> [Celery Queue Lanes]
       |                               |
       |                               v
       |                          [Workers]
       |                               |
       |                               v
       |                 [LLM Runtime + Workspace]
       |                    |                 |
       v                    v                 v
  [Postgres]           [Azure Blob]      [Postmark Outbound]
```

## Core concepts and identifiers
Inspired by Moltbot session scoping, Icebrew uses explicit IDs and idempotency keys:

| Concept | Identifier | Notes |
|---------|------------|-------|
| User | `user_id` (UUID) | Unique per sender email |
| Thread | `thread_id` (UUID) | Tied to email headers |
| Message | `message_id` (Message-ID) | Idempotency key |
| Task | `task_id` (UUID) | One billable run per task |
| Workspace | `workspace/{user_id}` | Blob prefix |

Thread key format (canonical, for logs/diagnostics):
- `user:{user_id}:thread:{thread_id}`

## Tech stack (MVP)
- **API**: Python + FastAPI (async inbound webhook, health, admin hooks)
- **Queue**: Celery + Redis (4 workers x concurrency 4 = 16 parallel tasks)
- **DB**: PostgreSQL (ACID for billing/quota and thread state)
- **Storage**: Azure Blob Storage (attachments + workspace)
- **Email provider**: Postmark (inbound + outbound, reliable deliverability)
- **LLM**: Claude Agent SDK (pipeline execution + LLM-as-judge)
- **Observability**: Sentry (errors), logs + OpenTelemetry (optional)

Rationale:
- Python ecosystem best fits Claude SDK and email parsing.
- Postgres ensures atomic quota enforcement.
- Celery provides stable retries and concurrency control.

## Execution lanes (queue separation)
Borrowing the Moltbot "lanes" concept, we split Celery into explicit queues to isolate workloads:

| Lane | Purpose | Concurrency |
|------|---------|-------------|
| `ingest` | Normalize + persist email metadata | High |
| `classify` | LLM-as-judge classification | Medium |
| `execute` | Task execution (agent run) | Medium |
| `notify` | Outbound email and post-processing | High |

Default single queue is acceptable for MVP; lane split is a safe upgrade path without code changes.

## Data model (PostgreSQL)
Core tables and key constraints:

### users
- `id` (uuid, PK)
- `email` (unique)
- `status` (active / blocked)
- `plan` (free / paid_20 / paid_200)
- `tasks_used` (int)
- `tasks_limit` (int or null for unlimited)
- `created_at`, `updated_at`

### threads
- `id` (uuid, PK)
- `user_id` (FK)
- `root_message_id`
- `last_message_id`
- `created_at`, `updated_at`

### messages
- `id` (uuid, PK)
- `thread_id` (FK)
- `message_id` (unique)
- `in_reply_to`
- `references` (jsonb array)
- `from_email`
- `subject`
- `body_text`
- `body_html`
- `received_at`
- `is_task` (bool nullable)
- `task_confidence` (float nullable)

### tasks
- `id` (uuid, PK)
- `message_id` (unique)
- `user_id` (FK)
- `thread_id` (FK)
- `status` (queued / running / complete / failed / skipped)
- `attempts` (int)
- `last_error` (text)
- `charged` (bool)
- `created_at`, `updated_at`

### attachments
- `id` (uuid, PK)
- `message_id` (FK)
- `filename`
- `blob_path`
- `version` (int)
- `content_type`
- `size_bytes`

Indexes and constraints:
- `messages.message_id` unique (idempotency)
- `tasks.message_id` unique (exactly one task per message)
- `attachments.message_id, filename, version` unique (deterministic versioning)

## Storage layout (Azure Blob)
Workspace structure:
```
workspace/{user_id}/
  current/
    thread.md
    attachments/
  threads/
    {thread_id}/
      thread.md
      attachments/
```

Attachment versioning:
- If `report.pdf` exists, rename to `report_v1.pdf`, `report_v2.pdf`, ...
- Keep original filename as base for versioning.
- Store version integer in DB to avoid re-listing blobs.

Current workspace behavior:
- `current/` mirrors the latest thread for that user.
- `threads/{thread_id}/` holds immutable snapshots for memory.

## Processing pipeline detail

### Inbound webhook (FastAPI)
1) Validate Postmark signature.
2) Normalize payload to internal schema.
3) Insert `messages` row if not exists (idempotent).
4) Enqueue `process_email(message_id)`.
5) Return 200 quickly.

### Worker: process_email
1) Load message + user + thread context.
2) If new user: auto-register (plan=free).
3) Resolve or create thread.
4) Save attachments to Azure Blob:
   - `threads/{thread_id}/attachments/`
   - `current/attachments/`
5) Update `thread.md` in both `threads/{thread_id}` and `current/`.
6) Run LLM-as-judge for `is_task`.
7) If not task: respond with clarification or acknowledgment.
8) If task:
   - Check quota in a DB transaction.
   - Call Claude agent SDK for task execution.
   - Send reply.
   - Update status and quota.

### Outbound email
- Always include `In-Reply-To` and `References` for threading.
- Use deterministic subject prefixing (no repeated "Re:").

## Thread detection rules
Thread detection uses headers with fallback logic:
- Primary: `In-Reply-To`
- Secondary: `References`
- Fallback: new thread per unique `Message-ID` if no headers

Implementation detail:
- Store `message_id`, `in_reply_to`, and `references` in DB.
- Build threads by walking `references` if needed.
- Do not rely solely on Postmark's thread id.

## Task classification (LLM-as-judge)
Goal: determine if email is a billable task vs. a clarification.
- Use Claude to classify `is_task` and `confidence`.
- If confidence < threshold (e.g. 0.6), default to non-task.
- Cache classification per message to avoid rework.

## Quota management
Rules:
- 5 free tasks per user.
- After limit, refuse and direct to `icebrew.ai` upgrade flow.
- Paid plans:
  - $20/mo -> 50 tasks/year
  - $200/mo -> unlimited

Enforcement:
- Check quota at task start, inside a DB transaction.
- Increment `tasks_used` only once per confirmed task.
- Do not charge for non-task emails.
- Retries must not double-charge.

## Retry policy
- 1 retry (2 total attempts) per task.
- Capture exception and log reason.
- Mark status `failed` after final retry.

## Concurrency + idempotency
Controls:
- Webhook: `message_id` unique constraint.
- Queue: dedupe on `message_id`.
- Worker: idempotent updates, use `tasks.message_id` unique.
- Quota update: DB row locking / atomic UPDATE.

## Security considerations
- Validate inbound signatures.
- Encrypt secrets in env/secret manager.
- Restrict Azure Blob access to service principal.
- Rate-limit webhook endpoint.
- Store minimal PII (email only).

## Observability
- Log correlation ID = `message_id`.
- Emit task lifecycle events: queued, running, complete, failed.
- Sentry for exceptions; optional OpenTelemetry spans for pipeline steps.

## CLI-testable modules
Every module exposes a CLI entry for independent testing. All commands accept JSON via file/stdin and emit JSON to stdout.

Base command (design target): `icebrew` (or `python -m icebrew.cli`).

### 1) email.parse
- Input: raw Postmark JSON
- Output: normalized `message` object
- Example: `icebrew email parse --postmark-json inbound.json`

### 2) email.thread
- Input: `message_id` + headers
- Output: `thread_id`
- Example: `icebrew email thread --message-id <id>`

### 3) storage.attachments
- Input: attachments list + `thread_id`
- Output: blob paths + versioning map
- Example: `icebrew storage attachments --thread-id <id> --attachments-json files.json`

### 4) workspace.update
- Input: message + thread
- Output: updated `thread.md` content + blob paths
- Example: `icebrew workspace update --thread-id <id> --message-id <id>`

### 5) task.judge
- Input: message body
- Output: `is_task` + `confidence`
- Example: `icebrew task judge --message-id <id>`

### 6) quota.check
- Input: `user_id`
- Output: allow/deny + remaining
- Example: `icebrew quota check --user-id <id>`

### 7) task.run
- Input: `message_id`
- Output: response email body + metadata
- Example: `icebrew task run --message-id <id>`

### 8) email.send
- Input: to, subject, body
- Output: Postmark response (stubbed in local mode)
- Example: `icebrew email send --to test@example.com --subject "Hello" --body "..."`

### 9) pipeline.simulate
- Input: raw Postmark JSON
- Output: full pipeline trace (no external side-effects)
- Example: `icebrew pipeline simulate --postmark-json inbound.json --dry-run`

## Testing strategy
- Unit tests for each CLI module.
- Integration test pipeline using mocked Postmark payload.
- End-to-end test with Postmark sandbox + Azure Blob emulator.
- Load test: simulate 20-50 concurrent tasks via Celery.

## Operations
- Deployment: Docker + Azure App Service.
- Config via env vars (Postmark token, DB URL, Azure creds, Claude key).
- Metrics: task latency, failure rate, quota usage.

## References
- TECH_DESIGN_of_MOLTBOT_by_Codex.md
- TECH_DESIGN_of_MOLTBOT_by_CLAUDE.md
