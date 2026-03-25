//! TPM (Technical Program Manager) CLI for agent use.
//!
//! Provides commands for TPM workflows:
//! - User contact directory lookup and management
//! - Task board status aggregation
//! - Cross-channel messaging coordination
//!
//! Usage:
//!   tpm_cli get-contact --notion-user-id <id> [--workspace-id <ws>]
//!   tpm_cli list-contacts [--workspace-id <ws>]
//!   tpm_cli update-contacted --contact-id <id>

use scheduler_module::account_store::{AccountStore, UserContact};
use serde_json::json;
use std::env;
use std::process::ExitCode;
use uuid::Uuid;

fn main() -> ExitCode {
    dotenvy::dotenv().ok();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return ExitCode::FAILURE;
    }

    let command = &args[1];
    match command.as_str() {
        "get-contact" => cmd_get_contact(&args[2..]),
        "list-contacts" => cmd_list_contacts(&args[2..]),
        "update-contacted" => cmd_update_contacted(&args[2..]),
        "help" | "--help" | "-h" => {
            print_usage();
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    eprintln!(
        r#"TPM CLI - Technical Program Manager tools

Usage:
  tpm_cli <command> [options]

Commands:
  get-contact       Get contact info for a Notion user
    --notion-user-id <id>    Notion person ID (from task assignee)
    --workspace-id <ws>      Notion workspace ID (required)

  list-contacts     List all configured contacts
    --workspace-id <ws>      Filter by Notion workspace (optional)

  update-contacted  Mark a contact as recently contacted
    --contact-id <id>        Contact entry UUID

Environment:
  SUPABASE_DB_URL        Required for database access
  ACCOUNT_ID             Account UUID (from .notion_context.json or env)

Context Files:
  .notion_context.json   Auto-loaded for account_id and workspace_id

Output:
  JSON to stdout on success, error message to stderr on failure.
"#
    );
}

/// Get account_id from environment or context file.
fn get_account_id() -> Option<Uuid> {
    // First try environment variable
    if let Ok(id) = env::var("ACCOUNT_ID") {
        if let Ok(uuid) = Uuid::parse_str(&id) {
            return Some(uuid);
        }
    }

    // Then try .notion_context.json
    let context_path = std::path::Path::new(".notion_context.json");
    if context_path.exists() {
        if let Ok(content) = std::fs::read_to_string(context_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(id) = json["account_id"].as_str() {
                    if let Ok(uuid) = Uuid::parse_str(id) {
                        return Some(uuid);
                    }
                }
            }
        }
    }

    None
}

/// Get workspace_id from environment or context file.
fn get_workspace_id() -> Option<String> {
    // First try argument (handled by caller)
    // Then try .notion_context.json
    let context_path = std::path::Path::new(".notion_context.json");
    if context_path.exists() {
        if let Ok(content) = std::fs::read_to_string(context_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(ws_id) = json["workspace_id"].as_str() {
                    return Some(ws_id.to_string());
                }
            }
        }
    }

    None
}

fn user_contact_to_json(contact: &UserContact) -> serde_json::Value {
    json!({
        "id": contact.id.to_string(),
        "account_id": contact.account_id.to_string(),
        "notion_user_id": contact.notion_user_id,
        "notion_workspace_id": contact.notion_workspace_id,
        "slack_user_id": contact.slack_user_id,
        "slack_workspace_id": contact.slack_workspace_id,
        "discord_user_id": contact.discord_user_id,
        "discord_guild_id": contact.discord_guild_id,
        "preferred_channel": contact.preferred_channel,
        "contact_frequency_days": contact.contact_frequency_days,
        "last_contacted_at": contact.last_contacted_at.map(|t| t.to_rfc3339()),
        "created_at": contact.created_at.to_rfc3339(),
    })
}

fn cmd_get_contact(args: &[String]) -> ExitCode {
    let mut notion_user_id: Option<String> = None;
    let mut workspace_id: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--notion-user-id" => {
                i += 1;
                notion_user_id = args.get(i).cloned();
            }
            "--workspace-id" => {
                i += 1;
                workspace_id = args.get(i).cloned();
            }
            _ => {}
        }
        i += 1;
    }

    let Some(notion_user_id) = notion_user_id else {
        eprintln!("Error: --notion-user-id is required");
        return ExitCode::FAILURE;
    };

    let workspace_id = workspace_id.or_else(get_workspace_id);
    let Some(workspace_id) = workspace_id else {
        eprintln!("Error: --workspace-id is required (or set via .notion_context.json)");
        return ExitCode::FAILURE;
    };

    let Some(account_id) = get_account_id() else {
        eprintln!("Error: ACCOUNT_ID not found (check env or .notion_context.json)");
        return ExitCode::FAILURE;
    };

    let store = match AccountStore::from_env() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Failed to connect to database: {}", e);
            return ExitCode::FAILURE;
        }
    };

    match store.get_user_contact_by_notion_user(account_id, &workspace_id, &notion_user_id) {
        Ok(Some(contact)) => {
            let output = user_contact_to_json(&contact);
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
            ExitCode::SUCCESS
        }
        Ok(None) => {
            let output = json!({
                "error": "not_found",
                "message": format!("No contact found for Notion user {} in workspace {}", notion_user_id, workspace_id)
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
            ExitCode::SUCCESS // Not an error, just no data
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn cmd_list_contacts(args: &[String]) -> ExitCode {
    let mut workspace_id: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--workspace-id" => {
                i += 1;
                workspace_id = args.get(i).cloned();
            }
            _ => {}
        }
        i += 1;
    }

    // Try to get workspace_id from context if not provided
    let workspace_id = workspace_id.or_else(get_workspace_id);

    let Some(account_id) = get_account_id() else {
        eprintln!("Error: ACCOUNT_ID not found (check env or .notion_context.json)");
        return ExitCode::FAILURE;
    };

    let store = match AccountStore::from_env() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Failed to connect to database: {}", e);
            return ExitCode::FAILURE;
        }
    };

    match store.list_user_contacts(account_id, workspace_id.as_deref()) {
        Ok(contacts) => {
            let output: Vec<serde_json::Value> =
                contacts.iter().map(user_contact_to_json).collect();
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn cmd_update_contacted(args: &[String]) -> ExitCode {
    let mut contact_id: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--contact-id" => {
                i += 1;
                contact_id = args.get(i).cloned();
            }
            _ => {}
        }
        i += 1;
    }

    let Some(contact_id) = contact_id else {
        eprintln!("Error: --contact-id is required");
        return ExitCode::FAILURE;
    };

    let contact_uuid = match Uuid::parse_str(&contact_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            eprintln!("Error: Invalid contact-id UUID format");
            return ExitCode::FAILURE;
        }
    };

    let store = match AccountStore::from_env() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Failed to connect to database: {}", e);
            return ExitCode::FAILURE;
        }
    };

    match store.update_user_contact_last_contacted(contact_uuid) {
        Ok(()) => {
            let output = json!({
                "status": "success",
                "message": format!("Updated last_contacted_at for contact {}", contact_id)
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}
