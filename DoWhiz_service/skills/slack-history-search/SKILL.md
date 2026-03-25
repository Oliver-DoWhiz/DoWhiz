---
name: slack-history-search
description: Use when a Slack task needs earlier chat history that is not already present in the workspace. The helper can search readable conversations inside the current Slack workspace/team through a backend-enforced scope grant.
allowed-tools: Bash(python3:*)
---

# Slack History Search

Use this skill when the user asks about earlier Slack discussion and the answer is not already in the prompt, `incoming_email/`, or other workspace files.

Workspace helper:

```bash
python3 .agents/skills/slack-history-search/scripts/slack_history_search.py status
python3 .agents/skills/slack-history-search/scripts/slack_history_search.py search --query "refund policy" --limit 10
python3 .agents/skills/slack-history-search/scripts/slack_history_search.py search --query "launch checklist" --channel-id C12345678
```

## Safety Boundary

- The helper reads `.chat_history_scope.json` from the current workspace.
- The backend enforces the scope, not the prompt.
- Search is limited to readable Slack conversations in the current Slack workspace/team.
- It cannot cross into other Slack workspaces.
- Do not try to use raw Slack tokens directly, even if you think they exist somewhere else.

## Workflow

1. Run `status` first if you need to confirm the current scope or expiry.
2. Use `search --query ...` with a focused keyword, phrase, or name.
3. Use `--channel-id ...` when you want to narrow the search to one Slack conversation inside the current workspace scope.
4. If results are large, write them to a file with `--output work/slack_history.json` and summarize from that file.
5. If the user asks about a different Slack workspace, explain that the current grant does not allow that access.

## Examples

```bash
python3 .agents/skills/slack-history-search/scripts/slack_history_search.py search \
  --query "launch checklist" \
  --limit 8 \
  --channel-id C12345678
```

```bash
python3 .agents/skills/slack-history-search/scripts/slack_history_search.py search \
  --query "invoice 4821" \
  --limit 20 \
  --output work/slack_invoice_history.json \
  --json
```
