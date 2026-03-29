# Group Project Coordination Skill

This skill enables you to coordinate group projects by creating shared resources, assigning tasks, and sending reminders to team members.

## When to Use

Use this skill when users ask you to:
- Set up a new group project
- Create and share documents with team members
- Track project tasks and deadlines
- Send reminders to team members
- Coordinate schedules and meetings

## Capabilities

### 1. Create Shared Documents

You can create Google Docs, Sheets, or Slides and share them with team members:

```bash
# Create a new Google Doc
google-docs create-document --title="Project Notes"

# Share with team members (role: reader, commenter, or writer)
google-docs share <doc_id> --email="alice@example.com" --role="writer" --notify
google-docs share <doc_id> --email="bob@example.com" --role="writer" --notify

# Get the shareable link
google-docs get-link <doc_id>
```

For spreadsheets (tracking, data):
```bash
google-sheets create-spreadsheet --title="Project Tracker"
google-sheets share <sheet_id> --email="alice@example.com" --role="writer" --notify
```

For presentations:
```bash
google-slides create-presentation --title="Final Presentation"
google-slides share <presentation_id> --email="alice@example.com" --role="writer" --notify
```

### 2. Send Messages to Team Members

**Discord:**
```bash
# Send a direct message to a user
discord_cli send-dm --user-id <user_id> --message "Don't forget we have a meeting tomorrow at 3pm!"

# Send to a channel
discord_cli send-channel --channel-id <channel_id> --message "Project documents are ready: <link>"
```

**Slack:**
```bash
# Send a direct message
slack_cli send-dm --user <user_id_or_email> --message "Reminder: deadline is Friday!"

# Send to a channel
slack_cli send-channel --channel <channel_id_or_name> --message "Weekly standup reminder!"
```

### 3. Create Calendar Events

```bash
# Create a meeting with attendees
gws calendar create-event \
  --title "Group Project Kickoff" \
  --start "2024-03-30T14:00:00" \
  --end "2024-03-30T15:00:00" \
  --attendees "alice@example.com,bob@example.com" \
  --meet  # Adds Google Meet link
```

### 4. Schedule Reminders

Use the scheduler to set up recurring reminders:

**In your response, emit scheduled tasks:**
```
SCHEDULED_TASKS_JSON_BEGIN
[{
  "type": "send_discord_dm",
  "delay_minutes": 1440,
  "user_id": "123456789",
  "message": "Reminder: Project deadline tomorrow!"
}]
SCHEDULED_TASKS_JSON_END
```

## Example Workflows

### Setting Up a New Group Project

When a user says: "Help me set up a group project with Alice, Bob, and Carol for our CS 101 final"

1. **Create shared documents:**
   - Google Doc for notes/collaboration
   - Google Sheet for task tracking (optional)

2. **Share with all members:**
   - Share documents with each team member as "writer"
   - Use --notify to send email notifications

3. **Send notification:**
   - Post to the group's Discord/Slack channel with document links
   - Or DM each member individually

4. **Set up reminders (if deadline given):**
   - Schedule reminder 3 days before deadline
   - Schedule reminder 1 day before deadline

### Tracking Progress

When checking in on a project:

1. Read the shared document to see current state
2. Check who has contributed
3. Send gentle reminders to inactive members
4. Update any tracking spreadsheets

### Sending Reminders

For deadline reminders:
- Be friendly but clear
- Include the deadline date/time
- Provide document links for quick access
- Offer to help if they're stuck

## Best Practices

1. **Always confirm actions**: Tell the user what you've created and shared
2. **Include links**: Provide direct links to all created resources
3. **Be proactive but not spammy**: Space out reminders appropriately
4. **Respect privacy**: Only message people who are part of the project
5. **Track what you've done**: Keep notes in the shared doc about setup and reminders sent

## Error Handling

- If sharing fails, check if the email address is valid
- If the user doesn't have Google Workspace access, offer alternatives
- If Discord/Slack DMs fail, try channel messages instead
- Always report what succeeded and what failed
