# Scoped Slack / Discord Chat History

This repo now supports on-demand Slack and Discord history search through workspace skills plus a backend-enforced scope grant.

## Why This Exists

The agent sometimes needs older Slack or Discord context that is not already in the current prompt or workspace files.

We do **not** give the agent raw unrestricted bot tokens for that.

Instead:

1. The worker writes a short-lived `.chat_history_scope.json` file into the task workspace.
2. That file contains a signed grant plus the internal search endpoint.
3. The workspace skill calls the internal endpoint with the signed grant.
4. The worker validates the grant and performs the real Slack / Discord API calls server-side.

This keeps the safety boundary in backend code rather than in prompt text alone.

## Isolation Model

- Slack: scoped to the **current conversation** only.
  - Current channel, DM, or MPIM.
  - Thread replies inside that same conversation are included.
  - Cross-channel and cross-workspace access is blocked.
- Discord guild message: scoped to the **current guild/server** only.
  - Cross-server access is blocked.
- Discord DM: scoped to the **current DM channel** only.

This directly prevents a user in server A from asking the agent about server B.

## Skills

The worker copies these into the task workspace:

- `.agents/skills/slack-history-search`
- `.agents/skills/discord-history-search`

Each skill reads `.chat_history_scope.json` and calls:

- `POST /internal/chat-history/search`

## Credential Safety

For local child-agent execution we now strip obvious bypass credentials from the child process environment, including:

- `SLACK_BOT_TOKEN`
- `*_SLACK_BOT_TOKEN`
- `DISCORD_BOT_TOKEN`
- `*_DISCORD_BOT_TOKEN`
- `MONGODB_URI`
- `DATABASE_URL`
- `INGESTION_DB_URL`
- `SUPABASE_DB_URL`
- `SLACK_STORE_PATH`

The intent is that the signed grant is the usable path for chat-history access.

## Runtime Configuration

Recommended:

- `CHAT_HISTORY_SCOPE_SIGNING_SECRET`
  - Dedicated HMAC/JWT signing secret for workspace history grants.
- `CHAT_HISTORY_PROXY_BASE_URL`
  - Public base URL the workspace helper should call.
  - Important when the agent runs outside the worker host, for example in a remote container.

Optional:

- `CHAT_HISTORY_SCOPE_TTL_MINUTES`
- `CHAT_HISTORY_SLACK_MAX_HISTORY_PAGES`
- `CHAT_HISTORY_SLACK_MAX_THREAD_PAGES`
- `CHAT_HISTORY_DISCORD_MAX_HISTORY_PAGES_PER_CHANNEL`
- `CHAT_HISTORY_DISCORD_MAX_CHANNELS`

If `CHAT_HISTORY_PROXY_BASE_URL` is missing, the worker falls back to `SERVICE_URL`, then to its local bind host and port.

## Current Discord Coverage

Discord search currently scans:

- searchable guild text channels
- active threads
- the current DM channel for DM-scoped requests

Archived Discord threads are not scanned yet; the API response includes a warning when that matters.
