# Group Project Coordination

Use this skill when users ask to set up or coordinate a group project, team workspace, or shared resources.

## Core Principle

When coordinating multiple people, you often need contact info (email, GitHub) that you don't have. **Don't stop and report "I don't have their email" - proactively collect it.**

## Workflow

1. **Identify team members**: Ask the user who's in the group (Discord @mentions, names, etc.)

2. **Look up known identities**: 
   ```bash
   # Check if team members have DoWhiz accounts with linked identities
   identity_lookup_cli guild-identities <guild_id> --type email,github
   ```

3. **Collect missing info**: For members without linked accounts, reach out directly:
   ```bash
   # DM the person to ask for their email/GitHub
   discord_cli send-dm <user_id> "Hi! [Organizer] is setting up a shared workspace for [project]. Could you reply with your email so I can invite you to the Google Drive folder?"
   ```

4. **Create shared resources**:
   - Google Drive folder: `google-docs create-folder "Project Name" && google-docs share <folder_id> <email> editor`
   - GitHub repo: `gh repo create <org>/<name> --private && gh api repos/<org>/<name>/collaborators/<username> -X PUT`

5. **Report back** with links and who still needs to respond

## Key Commands

| Task | Command |
|------|---------|
| List guild members | `discord_cli list-guild-members <guild_id>` |
| Look up linked email | `identity_lookup_cli discord-to-email <discord_user_id>` |
| Look up linked GitHub | `identity_lookup_cli discord-to-github <discord_user_id>` |
| Batch lookup | `identity_lookup_cli guild-identities <guild_id>` |
| Send DM | `discord_cli send-dm <user_id> "<message>"` |
| Create Drive folder | `google-docs create-folder "<name>"` |
| Share with user | `google-docs share <file_id> <email> editor` |

## Tips

- **Don't wait for everyone**: Create resources as soon as you have some emails, then add others as they respond
- **Track who's missing**: Keep a list in your reply of who you're still waiting on
- **Follow up**: If someone doesn't respond to DM, mention it to the organizer so they can nudge
- **Remember for next time**: Once you learn someone's email, save it to memory for future tasks
