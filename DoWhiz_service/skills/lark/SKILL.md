---
name: lark
description: Work with Lark Docs, Sheets, Bitable (database), and Drive. Use this skill when the user requests operations on Lark documents, spreadsheets, or databases.
allowed-tools: Bash(lark_cli:*)
---

# Lark Integration Skill

## Overview

This skill enables you to interact with Lark's productivity suite:
- **Docs** - Read and create documents
- **Sheets** - Read, write, and append spreadsheet data
- **Bitable** - Query and manage database records (like Airtable)
- **Drive** - List and manage files/folders
- **Sharing** - Share files with users via email

## Authentication

The CLI uses environment variables for authentication:
- `LARK_APP_ID` - Your Lark app ID
- `LARK_APP_SECRET` - Your Lark app secret

These are pre-configured in the environment.

## Available Commands

### Documents

```bash
# Get document metadata
lark_cli get-doc --document-id "<doc_token>"

# Read document content (returns blocks)
lark_cli read-doc --document-id "<doc_token>"

# Create a new document
lark_cli create-doc --title "Meeting Notes"
lark_cli create-doc --title "Report" --folder-token "<folder_token>"
```

### Sheets

```bash
# Get spreadsheet metadata
lark_cli get-sheet --spreadsheet-id "<sheet_token>"

# Read a range of cells
lark_cli read-range --spreadsheet-id "<sheet_token>" --sheet-id "Sheet1" --range "A1:C10"

# Write values to a range (JSON 2D array)
lark_cli write-range --spreadsheet-id "<sheet_token>" --sheet-id "Sheet1" --range "A1:B2" --values '[["Name","Age"],["John",30]]'

# Append rows to a sheet
lark_cli append-rows --spreadsheet-id "<sheet_token>" --sheet-id "Sheet1" --values '[["Jane",25],["Bob",35]]'
```

### Bitable (Database)

Bitable is Lark's database product (similar to Airtable/Notion databases).

```bash
# List tables in a bitable app
lark_cli list-tables --app-token "<app_token>"

# Get table metadata (field definitions)
lark_cli get-table --app-token "<app_token>" --table-id "<table_id>"

# Query records (with optional filter)
lark_cli query-records --app-token "<app_token>" --table-id "<table_id>"
lark_cli query-records --app-token "<app_token>" --table-id "<table_id>" --filter 'CurrentValue.[Status]="Done"' --page-size 50

# Create a record (fields as JSON object)
lark_cli create-record --app-token "<app_token>" --table-id "<table_id>" --fields '{"Name":"John","Status":"Active"}'

# Update a record
lark_cli update-record --app-token "<app_token>" --table-id "<table_id>" --record-id "<record_id>" --fields '{"Status":"Completed"}'

# Delete a record
lark_cli delete-record --app-token "<app_token>" --table-id "<table_id>" --record-id "<record_id>"
```

**Field Types:**
| Type | Example Value |
|------|---------------|
| Text | `"Hello"` |
| Number | `123` or `45.67` |
| Checkbox | `true` / `false` |
| Date | `1672531200000` (unix ms) |
| Single Select | `"Option A"` |
| Multi Select | `["Option A", "Option B"]` |
| Person | `[{"id": "ou_xxx"}]` |

### Drive

```bash
# List files (root folder)
lark_cli list-files

# List files in a specific folder
lark_cli list-files --folder-token "<folder_token>" --page-size 50

# Get file metadata
lark_cli get-file --file-token "<file_token>"

# Create a folder
lark_cli create-folder --name "Projects" --parent-token "<parent_folder_token>"
```

### Sharing

Share documents, sheets, or bitable apps with users:

```bash
# Share a document with a user by open_id (default, edit access)
lark_cli share-file --token "<doc_token>" --file-type docx --member-id "ou_xxx"

# Share with view-only access
lark_cli share-file --token "<doc_token>" --file-type docx --member-id "ou_xxx" --perm view

# Share by email instead of open_id
lark_cli share-file --token "<doc_token>" --file-type docx --member-type email --member-id "user@example.com"

# Share a spreadsheet
lark_cli share-file --token "<sheet_token>" --file-type sheet --member-id "ou_xxx"

# Share a bitable with full access
lark_cli share-file --token "<app_token>" --file-type bitable --member-id "ou_xxx" --perm full_access
```

**Parameters:**
- `--token` - The file/doc token from the create response
- `--file-type` - `docx`, `sheet`, `bitable`, `file`, `folder`
- `--member-type` - `openid` (default), `email`, or `userid`
- `--member-id` - The user's open_id (like "ou_xxx"), email, or user_id
- `--perm` - `view`, `edit` (default), or `full_access`

**Important:** When you create a document/sheet/bitable, it is owned by the bot app. You must share it with the user for them to access it.
- Use the user's open_id from "Lark Open IDs" in the User Context section
- If not available, the user needs to link their Lark account at dowhiz.com first

## Getting Tokens from URLs

Lark URLs contain tokens you'll need:

| URL Pattern | Token Location |
|-------------|----------------|
| `https://xxx.larksuite.com/docx/ABC123` | Document: `ABC123` |
| `https://xxx.larksuite.com/sheets/ABC123` | Spreadsheet: `ABC123` |
| `https://xxx.larksuite.com/base/ABC123` | Bitable App: `ABC123` |
| `https://xxx.larksuite.com/drive/folder/ABC123` | Folder: `ABC123` |

## Example Workflows

### Read and Summarize a Document

```bash
# 1. Read the document
lark_cli read-doc --document-id "doxcn4RM1Lxxx"

# 2. Parse the JSON response to extract text blocks
# 3. Summarize the content
```

### Log Data to a Bitable

```bash
# 1. Get table structure
lark_cli get-table --app-token "appXXX" --table-id "tblYYY"

# 2. Create a new record
lark_cli create-record --app-token "appXXX" --table-id "tblYYY" --fields '{"Task":"Review PR","Status":"Pending","Due":"2024-03-15"}'
```

### Update Spreadsheet from User Request

```bash
# 1. Read current data
lark_cli read-range --spreadsheet-id "shtXXX" --sheet-id "Sheet1" --range "A1:D10"

# 2. Append new rows
lark_cli append-rows --spreadsheet-id "shtXXX" --sheet-id "Sheet1" --values '[["New Item",100,"Active"]]'
```

## Error Handling

Common errors:
- **Permission denied**: User needs to grant the Lark app access to the document/table
- **Not found**: Check the token is correct and the resource exists
- **Invalid fields**: Ensure field names match exactly (case-sensitive) and values match field types

## Response Format

All commands output JSON. Parse the response to extract:
- `data` - The actual content/records
- `code` - 0 for success, non-zero for errors
- `msg` - Error message if failed
