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

- Slack: scoped to the **current workspace/team** only.
  - Search can span readable public channels, private channels, DMs, and MPIMs in the same workspace.
  - The current origin conversation is always kept in scope.
  - Thread replies inside scanned conversations are included.
  - Cross-workspace access is blocked.
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

- `DOWHIZ_API_URL`
  - Explicit public `/service` base URL (for example `https://api.staging.dowhiz.com/service`).
- `CHAT_HISTORY_SCOPE_TTL_MINUTES`
- `CHAT_HISTORY_SLACK_MAX_HISTORY_PAGES`
- `CHAT_HISTORY_SLACK_MAX_THREAD_PAGES`
- `CHAT_HISTORY_SLACK_MAX_CHANNELS`
- `CHAT_HISTORY_DISCORD_MAX_HISTORY_PAGES_PER_CHANNEL`
- `CHAT_HISTORY_DISCORD_MAX_CHANNELS`

Fallback order:

1. `CHAT_HISTORY_PROXY_BASE_URL`
2. `DOWHIZ_API_URL`
3. `SERVICE_URL`
4. When `RUN_TASK_EXECUTION_BACKEND=azure_aci`, derive the public `/service` base from `POSTMARK_INBOUND_HOOK_URL` or `FRONTEND_URL`
5. Local bind host and port

The Azure ACI fallback avoids writing `127.0.0.1` into the workspace scope file when the agent runs in a separate container.

## Current Slack Coverage

Slack workspace search now enumerates readable conversations through `conversations.list`, then scans scoped history with `conversations.history` plus `conversations.replies`.

Whole-workspace Slack search depends on the installed Slack app having both:

- conversation listing/read scopes (`channels:read`, `groups:read`, `im:read`, `mpim:read`)
- history scopes (`channels:history`, `groups:history`, `im:history`, `mpim:history`)

Current Slack scan coverage includes:

- readable public channels
- readable private channels
- readable DMs and MPIMs
- thread replies inside scanned conversations

If conversation enumeration is unavailable, the backend falls back to the origin Slack conversation and returns a warning instead of breaking history search entirely.
If Slack cannot read a specific conversation or thread-reply page during a workspace scan, the backend keeps any partial matches it already found and reports the skip as a warning.

## Current Discord Coverage

Discord guild search now prefers the official Discord guild search endpoint first.

If that official search is unavailable, still indexing, or rate-limited, the backend automatically falls back to the scoped channel scan below.

The scoped Discord scan currently covers:

- searchable guild text channels
- active threads
- the current DM channel for DM-scoped requests

Archived Discord threads are not scanned yet; the API response includes a warning when that matters.
Guild-wide Discord search also skips channels or active threads that the bot cannot read (for example 403/404 responses) and reports those skips as warnings instead of failing the whole search.
When Discord returns 429 rate limits on a channel-history request, the backend now retries with the server-provided delay before eventually downgrading that single channel to a warning.
