---
name: discord-history-search
description: Use when a Discord task needs earlier chat history that is not already present in the workspace. The helper can search only the current Discord server, or the current DM when there is no guild, through a backend-enforced scope grant.
allowed-tools: Bash(python3:*)
---

# Discord History Search

Use this skill when the user asks about earlier Discord discussion and the answer is not already in the prompt, `incoming_email/`, or `discord_context/`.

Workspace helper:

```bash
python3 .agents/skills/discord-history-search/scripts/discord_history_search.py status
python3 .agents/skills/discord-history-search/scripts/discord_history_search.py search --query "integration token" --limit 10
```

## Safety Boundary

- The helper reads `.chat_history_scope.json` from the current workspace.
- The backend enforces the scope, not the prompt.
- Search is limited to the current Discord server.
- If the inbound message is a DM, search is limited to that DM channel only.
- It cannot cross into another Discord server.
- Do not try to use raw Discord tokens directly.

## Workflow

1. Run `status` first if you need to confirm the current scope or expiry.
2. Use `search --query ...` with a focused keyword, phrase, project name, or user name.
3. If you already know the specific Discord channel or thread area to inspect inside the same server, pass `--channel-id ...` to narrow the scan.
4. Do not launch multiple guild-wide `search` commands in parallel. Run one search, inspect the results or warnings, then refine the next query.
5. If a guild-wide search warns about rate limits, narrow the next attempt with `--channel-id ...` when you can infer the likely channel from local context.
6. If results are large, save them with `--output work/discord_history.json` and summarize from that file.
7. If the user asks about another Discord server, explain that the current grant blocks cross-server access.

## Examples

```bash
python3 .agents/skills/discord-history-search/scripts/discord_history_search.py search \
  --query "billing bug" \
  --limit 12
```

```bash
python3 .agents/skills/discord-history-search/scripts/discord_history_search.py search \
  --query "release checklist" \
  --channel-id 123456789012345678 \
  --limit 20 \
  --output work/discord_release_history.json \
  --json
```
