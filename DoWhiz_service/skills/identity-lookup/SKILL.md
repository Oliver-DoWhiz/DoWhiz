---
name: identity-lookup
description: Look up DoWhiz-linked identities (email, GitHub) for Discord users. Use when you need to invite Discord guild members to shared resources like Google Docs, GitHub repos, or other collaboration tools.
allowed-tools: Bash(identity_lookup_cli:*)
---

# Identity Lookup Skill

## Overview

This skill enables you to find DoWhiz-linked identities for Discord users. When a user asks you to share resources with Discord guild members, use this skill to look up their email addresses or GitHub usernames.

## When to Use This Skill

Use this skill when:
- You need to share Google Docs/Sheets/Slides with Discord guild members
- You need to add Discord users to GitHub repos as collaborators
- You need to invite Discord users to any service that requires email/GitHub identity
- A user asks to "invite everyone" or "share with the team" in a Discord context

## CLI Commands Reference

### Look Up Single User

```bash
# Get email for a Discord user
identity_lookup_cli discord-to-email <discord_user_id>
# Returns: {"email": "user@example.com"} or {"email": null}

# Get GitHub username for a Discord user
identity_lookup_cli discord-to-github <discord_user_id>
# Returns: {"github": "username"} or {"github": null}

# Get all identities for a Discord user
identity_lookup_cli lookup <discord_user_id>
# Returns: {"account_id": "uuid", "email": "...", "github": "..."}
```

### Batch Look Up Guild Members

```bash
# Get identities for all members of a Discord guild
identity_lookup_cli guild-identities <guild_id>
# Returns list of {discord_user_id, email, github, account_id} for each member

# Filter to specific identity types
identity_lookup_cli guild-identities <guild_id> --type email
identity_lookup_cli guild-identities <guild_id> --type github
identity_lookup_cli guild-identities <guild_id> --type email,github
```

## Workflow Example

### Sharing a Google Doc with Discord Guild Members

1. **List guild members:**
```bash
discord_cli list-guild-members 1234567890
```

2. **Look up their emails:**
```bash
identity_lookup_cli guild-identities 1234567890 --type email
```

3. **Share the document with each member who has a linked email:**
```bash
google-docs share <doc_id> user@example.com editor
```

### Adding Discord Users to GitHub Repo

1. **Look up GitHub username:**
```bash
identity_lookup_cli discord-to-github 987654321098765432
```

2. **Add as collaborator (if GitHub found):**
```bash
gh api repos/OWNER/REPO/collaborators/USERNAME -X PUT
```

## Handling Missing Identities

- If a user has no DoWhiz account, the CLI returns `null` for all fields
- Tell the user which members need to link their accounts at dowhiz.com
- Example response: "I was able to share with Alice and Bob. Charlie and Dave don't have DoWhiz accounts linked - they can sign up at dowhiz.com to get access."

## Notes

- Only verified identities are returned (unverified accounts are excluded)
- The CLI queries the Supabase `account_identifiers` table
- Requires `SUPABASE_DB_URL` environment variable (already configured in production/staging)
