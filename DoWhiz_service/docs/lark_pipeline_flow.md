# Lark (Feishu) Pipeline Flow

Complete implementation guide for Lark channel integration.

---

## 1. Channel Enum - scheduler_module/src/channel.rs

```rust
// Add to Channel enum (after WeChat):
    /// Lark (Feishu 飞书) via Open Platform API
    Lark,

// Add to Display impl:
impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // ... existing ...
            Channel::Lark => write!(f, "lark"),
        }
    }
}

// Add to FromStr impl:
impl std::str::FromStr for Channel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            // ... existing ...
            "lark" | "feishu" => Ok(Channel::Lark),
            _ => Err(format!("unknown channel: {}", s)),
        }
    }
}

// Add to ChannelMetadata struct:
    /// Lark-specific: App ID
    pub lark_app_id: Option<String>,
    /// Lark-specific: Tenant key (workspace identifier)
    pub lark_tenant_key: Option<String>,
    /// Lark-specific: User's open_id
    pub lark_open_id: Option<String>,
    /// Lark-specific: Chat ID (group or P2P chat)
    pub lark_chat_id: Option<String>,
    /// Lark-specific: Message ID
    pub lark_message_id: Option<String>,
```

---

## 2. Lark Adapter - scheduler_module/src/adapters/lark.rs (NEW FILE)

```rust
//! Lark (Feishu 飞书) adapter for inbound and outbound messages.
//!
//! This module provides adapters for handling Lark messages:
//! - `LarkInboundAdapter`: Parses Lark webhook payloads (JSON)
//! - `LarkOutboundAdapter`: Sends messages via Lark Open Platform API

use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use tracing::info;

use crate::channel::{
    AdapterError, Channel, ChannelMetadata, InboundAdapter,
    InboundMessage, OutboundAdapter,
    OutboundMessage, SendResult,
};

// ======================================================================
// Inbound Adapter
// ======================================================================

#[derive(Debug, Clone, Default)]
pub struct LarkInboundAdapter;

impl LarkInboundAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl InboundAdapter for LarkInboundAdapter {
    fn parse(&self, raw_payload: &[u8]) -> Result<InboundMessage, AdapterError> {
        let payload: LarkWebhookPayload = serde_json::from_slice(raw_payload)
            .map_err(|e| AdapterError::ParseError(format!("invalid JSON: {}", e)))?;

        // Handle different event types
        let event = payload.event.ok_or(AdapterError::MissingField("event"))?;

        // Only handle message events for now
        let message = event.message.ok_or(AdapterError::ParseError(
            "not a message event".to_string(),
        ))?;

        // Extract text content
        let content: LarkMessageContent = serde_json::from_str(&message.content)
            .map_err(|e| AdapterError::ParseError(format!("invalid message content: {}", e)))?;

        let sender_id = event
            .sender
            .as_ref()
            .and_then(|s| s.sender_id.as_ref())
            .and_then(|id| id.open_id.clone())
            .unwrap_or_default();

        let chat_id = message.chat_id.clone();
        let thread_id = format!("lark:{}:{}", chat_id, sender_id);

        Ok(InboundMessage {
            channel: Channel::Lark,
            sender: sender_id.clone(),
            sender_name: event.sender.as_ref().and_then(|s| s.sender_id.as_ref()).and_then(|id| id.name.clone()),
            recipient: chat_id.clone(),
            subject: None,
            text_body: Some(content.text),
            html_body: None,
            thread_id,
            message_id: Some(message.message_id.clone()),
            attachments: vec![],
            reply_to: vec![sender_id.clone()],
            raw_payload: raw_payload.to_vec(),
            metadata: ChannelMetadata {
                lark_app_id: payload.header.as_ref().and_then(|h| h.app_id.clone()),
                lark_tenant_key: payload.header.as_ref().and_then(|h| h.tenant_key.clone()),
                lark_open_id: Some(sender_id),
                lark_chat_id: Some(chat_id),
                lark_message_id: Some(message.message_id),
                ..Default::default()
            },
        })
    }

    fn channel(&self) -> Channel {
        Channel::Lark
    }
}

// ======================================================================
// Outbound Adapter
// ======================================================================

#[derive(Debug)]
pub struct LarkOutboundAdapter {
    pub app_id: String,
    pub app_secret: String,
    tenant_access_token_cache: RwLock<Option<CachedToken>>,
}

#[derive(Debug, Clone)]
struct CachedToken {
    token: String,
    expires_at: std::time::Instant,
}

impl LarkOutboundAdapter {
    pub fn new(app_id: String, app_secret: String) -> Self {
        Self {
            app_id,
            app_secret,
            tenant_access_token_cache: RwLock::new(None),
        }
    }

    pub fn from_env() -> Result<Self, AdapterError> {
        let app_id = std::env::var("LARK_APP_ID")
            .map_err(|_| AdapterError::ConfigError("LARK_APP_ID not set".to_string()))?;
        let app_secret = std::env::var("LARK_APP_SECRET")
            .map_err(|_| AdapterError::ConfigError("LARK_APP_SECRET not set".to_string()))?;
        Ok(Self::new(app_id, app_secret))
    }

    fn get_tenant_access_token(&self) -> Result<String, AdapterError> {
        // Check cache
        {
            let cache = self.tenant_access_token_cache.read().unwrap();
            if let Some(ref cached) = *cache {
                if cached.expires_at > std::time::Instant::now() {
                    return Ok(cached.token.clone());
                }
            }
        }

        // Fetch new token for outbound send using app_secret
        let url = "https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal";
        let request_body = serde_json::json!({
            "app_id": self.app_id,
            "app_secret": self.app_secret,
        });

        let client = reqwest::blocking::Client::new();
        let response: LarkTokenResponse = client
            .post(url)
            .json(&request_body)
            .send()
            .map_err(|e| AdapterError::SendError(format!("token request failed: {}", e)))?
            .json()
            .map_err(|e| AdapterError::SendError(format!("token parse failed: {}", e)))?;

        if response.code != 0 {
            return Err(AdapterError::SendError(format!(
                "Lark token error {}: {}",
                response.code,
                response.msg.unwrap_or_default()
            )));
        }

        let token = response
            .tenant_access_token
            .ok_or_else(|| AdapterError::SendError("no tenant_access_token".to_string()))?;

        // Cache with 110 minute expiry (tokens last 2 hours)
        let expires_at = std::time::Instant::now() + std::time::Duration::from_secs(110 * 60);
        {
            let mut cache = self.tenant_access_token_cache.write().unwrap();
            *cache = Some(CachedToken {
                token: token.clone(),
                expires_at,
            });
        }

        Ok(token)
    }
}

impl OutboundAdapter for LarkOutboundAdapter {
    fn send(&self, message: &OutboundMessage) -> Result<SendResult, AdapterError> {
        let token = self.get_tenant_access_token()?;

        let receive_id = message
            .to
            .first()
            .ok_or_else(|| AdapterError::ConfigError("no recipient specified".to_string()))?;

        // Determine receive_id_type based on format
        let receive_id_type = if receive_id.starts_with("ou_") {
            "open_id"
        } else if receive_id.starts_with("oc_") {
            "chat_id" //send POST to group chat
        } else {
            "open_id" // default, send POST to individual chat
        };

        let text = if message.text_body.is_empty() {
            message.html_body.clone()
        } else {
            message.text_body.clone()
        };

        let content = serde_json::json!({ "text": text });

        let url = format!(
            "https://open.feishu.cn/open-apis/im/v1/messages?receive_id_type={}",
            receive_id_type
        );

        let request_body = serde_json::json!({
            "receive_id": receive_id,
            "msg_type": "text",
            "content": content.to_string(),
        });

        let client = reqwest::blocking::Client::new();
        let response: LarkSendResponse = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&request_body)
            .send()
            .map_err(|e| AdapterError::SendError(format!("send request failed: {}", e)))?
            .json()
            .map_err(|e| AdapterError::SendError(format!("response parse failed: {}", e)))?;

        if response.code != 0 {
            return Ok(SendResult {
                success: false,
                message_id: String::new(),
                submitted_at: String::new(),
                error: Some(format!(
                    "Lark error {}: {}",
                    response.code,
                    response.msg.unwrap_or_default()
                )),
            });
        }

        let message_id = response
            .data
            .and_then(|d| d.message_id)
            .unwrap_or_default();

        info!("sent Lark message to {}", receive_id);

        Ok(SendResult {
            success: true,
            message_id,
            submitted_at: chrono::Utc::now().to_rfc3339(),
            error: None,
        })
    }

    fn channel(&self) -> Channel {
        Channel::Lark
    }
}

// ======================================================================
// Lark API Types
// ======================================================================

#[derive(Debug, Deserialize)]
pub struct LarkWebhookPayload {
    pub schema: Option<String>,
    pub header: Option<LarkEventHeader>,
    pub event: Option<LarkEvent>,
    /// URL verification challenge
    pub challenge: Option<String>,
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    /// Verification token (for URL verification)
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LarkEventHeader {
    pub event_id: Option<String>,
    pub event_type: Option<String>,
    pub create_time: Option<String>,
    pub token: Option<String>,
    pub app_id: Option<String>,
    pub tenant_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LarkEvent {
    pub sender: Option<LarkSender>,
    pub message: Option<LarkMessage>,
}

#[derive(Debug, Deserialize)]
pub struct LarkSender {
    pub sender_id: Option<LarkSenderId>,
    pub sender_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LarkSenderId {
    pub union_id: Option<String>,
    pub user_id: Option<String>,
    pub open_id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LarkMessage {
    pub message_id: String,
    pub root_id: Option<String>,
    pub parent_id: Option<String>,
    pub create_time: Option<String>,
    pub chat_id: String,
    pub chat_type: Option<String>,
    pub message_type: Option<String>,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct LarkMessageContent {
    pub text: String,
}

#[derive(Debug, Deserialize)]
struct LarkTokenResponse {
    code: i32,
    msg: Option<String>,
    tenant_access_token: Option<String>,
    expire: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct LarkSendResponse {
    code: i32,
    msg: Option<String>,
    data: Option<LarkSendData>,
}

#[derive(Debug, Deserialize)]
struct LarkSendData {
    message_id: Option<String>,
}

// ======================================================================
// Quick Response Helper
// ======================================================================

pub fn send_quick_lark_response(
    open_id: &str,
    text: &str,
) -> Result<(), AdapterError> {
    let adapter = LarkOutboundAdapter::from_env()?;
    let message = OutboundMessage {
        channel: Channel::Lark,
        from: None,
        to: vec![open_id.to_string()],
        cc: vec![],
        bcc: vec![],
        subject: String::new(),
        text_body: text.to_string(),
        html_body: String::new(),
        html_path: None,
        attachments_dir: None,
        thread_id: None,
        metadata: ChannelMetadata::default(),
    };
    adapter.send(&message)?;
    Ok(())
}
```

---

## 3. Export in scheduler_module/src/adapters/mod.rs

```rust
// Add:
pub mod lark;

pub use lark::{send_quick_lark_response, LarkInboundAdapter, LarkOutboundAdapter, LarkWebhookPayload};
```

---

## 4. Verification - scheduler_module/src/bin/inbound_gateway/verify.rs

```rust
/// Verify Lark webhook signature using HMAC-SHA256.
/// Lark sends: X-Lark-Request-Timestamp, X-Lark-Request-Nonce, X-Lark-Signature
pub(super) fn verify_lark(headers: &HeaderMap, body: &[u8]) -> Result<(), &'static str> {
    let encrypt_key = std::env::var("LARK_ENCRYPT_KEY").ok();
    let Some(encrypt_key) = encrypt_key.filter(|v| !v.trim().is_empty()) else {
        // If encrypt_key not configured, skip verification
        return Ok(());
    };

    let timestamp = headers
        .get("X-Lark-Request-Timestamp")
        .and_then(|v| v.to_str().ok())
        .ok_or("missing_timestamp")?;

    let nonce = headers
        .get("X-Lark-Request-Nonce")
        .and_then(|v| v.to_str().ok())
        .ok_or("missing_nonce")?;

    let signature = headers
        .get("X-Lark-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or("missing_signature")?;

    // Signature = SHA256(timestamp + nonce + encrypt_key + body)
    use sha2::{Digest, Sha256};
    let body_str = std::str::from_utf8(body).unwrap_or("");
    let data = format!("{}{}{}{}", timestamp, nonce, encrypt_key, body_str);

    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let expected = hex::encode(hasher.finalize());

    if expected != signature {
        return Err("invalid_signature");
    }

    Ok(())
}

/// Check if this is a URL verification challenge from Lark.
/// Returns the challenge string if it is.
pub(super) fn verify_lark_challenge(body: &[u8]) -> Option<String> {
    let payload: serde_json::Value = serde_json::from_slice(body).ok()?;

    // Check for URL verification event
    if payload.get("type").and_then(|v| v.as_str()) == Some("url_verification") {
        return payload.get("challenge").and_then(|v| v.as_str()).map(|s| s.to_string());
    }

    // Also check schema 2.0 format
    if let Some(challenge) = payload.get("challenge").and_then(|v| v.as_str()) {
        if payload.get("token").is_some() {
            return Some(challenge.to_string());
        }
    }

    None
}
```

---

## 5. Handler - scheduler_module/src/bin/inbound_gateway/handlers.rs

```rust
// Add imports at top:
use scheduler_module::adapters::lark::LarkInboundAdapter;
use super::verify::{verify_lark, verify_lark_challenge};

/// Handle Lark inbound messages (POST request)
pub(super) async fn ingest_lark(
    State(state): State<Arc<GatewayState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // Handle URL verification challenge
    if let Some(challenge) = verify_lark_challenge(&body) {
        info!("lark URL verification, returning challenge");
        return (StatusCode::OK, Json(json!({"challenge": challenge})));
    }

    // Verify signature
    if let Err(reason) = verify_lark(&headers, &body) {
        return (StatusCode::UNAUTHORIZED, Json(json!({"status": reason})));
    }

    let adapter = LarkInboundAdapter::new();
    let message = match adapter.parse(&body) {
        Ok(message) => message,
        Err(err) => {
            debug!("gateway ignoring lark event: {}", err);
            return (StatusCode::OK, Json(json!({"status": "ignored"})));
        }
    };

    let chat_id = message
        .metadata
        .lark_chat_id
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    //Maps message to correct employee in gateway.toml (currently just Oliver)
    let Some(route) = resolve_route(Channel::Lark, &chat_id, &state) else {
        info!("gateway no route for lark chat_id={}", chat_id);
        return (StatusCode::OK, Json(json!({"status": "no_route"})));
    };

    let external_message_id = message.message_id.clone();

    //Build envelope to send to gateway bus
    let envelope = match build_envelope(
        route,
        Channel::Lark,
        external_message_id,
        &message,
        &body,
    )
    .await
    {
        Ok(envelope) => envelope,
        Err(err) => {
            error!("gateway failed to store raw payload: {}", err);
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({"status": "payload_store_failed"})),
            );
        }
    };

    enqueue_envelope(state.queue.clone(), envelope).await
}
```

---

## 6. Route Registration - scheduler_module/src/bin/inbound_gateway.rs

```rust
// Add import:
use handlers::ingest_lark;

// Add route in Router (after wechat):
.route("/lark/webhook", post(ingest_lark))
```

---

## 7. Service Processor - scheduler_module/src/service/inbound/lark.rs (NEW FILE)

For additional context, global scheduler pattern matches `process_lark_event`:

```rust
// In ingestion.rs
match envelope.channel {
    Channel::Email => { ... }
    Channel::Slack => { process_slack_event(...) }
    Channel::Discord => { process_discord_inbound_message(...) }
    Channel::WeChat => { process_wechat_event(...) }
    Channel::Lark => { process_lark_event(...) }  // We'll add this
    // etc.
}
```

```rust
use std::path::Path;
use std::time::Duration;

use tracing::{info, warn};

use crate::channel::Channel;
use crate::index_store::IndexStore;
use crate::user_store::UserStore;
use crate::{ModuleExecutor, RunTaskTask, Scheduler, TaskKind};

use super::super::bump_thread_state;
use super::super::config::ServiceConfig;
use super::super::default_thread_state_path;
use super::super::scheduler::cancel_pending_thread_tasks;
use super::super::workspace::ensure_thread_workspace;
use super::super::BoxError;

pub(crate) fn process_lark_event(
    //called by global scheduler thread, which creates the worker thread
    config: &ServiceConfig,
    user_store: &UserStore,
    index_store: &IndexStore,
    message: &crate::channel::InboundMessage,
    raw_payload: &[u8],
) -> Result<(), BoxError> {
    info!("processing Lark event");

    info!(
        "Lark message from {} in chat {:?}: {:?}",
        message.sender, message.metadata.lark_chat_id, message.text_body
    );

    let chat_id = message
        .metadata
        .lark_chat_id
        .as_deref()
        .unwrap_or("default");

    let user = user_store.get_or_create_user("lark", &message.sender)?;
    let user_paths = user_store.user_paths(&config.users_root, &user.user_id);
    user_store.ensure_user_dirs(&user_paths)?;

    let thread_key = format!("lark:{}:{}", chat_id, message.sender);

    //set up workspace for worker thread
    let workspace = ensure_thread_workspace(
        &user_paths,
        &user.user_id,
        &thread_key,
        &config.employee_profile,
        config.skills_source_dir.as_deref(),
    )?;

    let thread_state_path = default_thread_state_path(&workspace);
    //bump_thread_state keeps replies in order, and also deduplication of message_id
    let thread_state =
        bump_thread_state(&thread_state_path, &thread_key, message.message_id.clone())?;

    append_lark_message(
        &workspace,
        message,
        raw_payload,
        thread_state.last_email_seq.try_into().unwrap_or(u32::MAX),
    )?;

    let model_name = match config.employee_profile.model.clone() {
        Some(model) => model,
        None => {
            if config
                .employee_profile
                .runner
                .eq_ignore_ascii_case("claude")
            {
                String::new()
            } else {
                config.codex_model.clone()
            }
        }
    };

    info!(
        "workspace ready at {} for user {} thread={} epoch={}",
        workspace.display(),
        user.user_id,
        thread_key,
        thread_state.epoch
    );

    let run_task = RunTaskTask {
        workspace_dir: workspace.clone(),
        input_email_dir: std::path::PathBuf::from("incoming_email"),
        input_attachments_dir: std::path::PathBuf::from("incoming_attachments"),
        memory_dir: std::path::PathBuf::from("memory"),
        reference_dir: std::path::PathBuf::from("references"),
        model_name,
        runner: config.employee_profile.runner.clone(),
        codex_disabled: config.codex_disabled,
        reply_to: vec![message.sender.clone()],
        reply_from: None,
        archive_root: Some(user_paths.mail_root.clone()),
        thread_id: Some(thread_key.clone()),
        thread_epoch: Some(thread_state.epoch),
        thread_state_path: Some(thread_state_path.clone()),
        channel: Channel::Lark,
        slack_team_id: None,
        employee_id: Some(config.employee_profile.id.clone()),
        requester_identifier_type: None,
        requester_identifier: None,
        account_id: None,
    };

    let mut scheduler = Scheduler::load(&user_paths.tasks_db_path, ModuleExecutor::default())?;
    if let Err(err) = cancel_pending_thread_tasks(&mut scheduler, &workspace, thread_state.epoch) {
        warn!(
            "failed to cancel pending thread tasks for {}: {}",
            workspace.display(),
            err
        );
    }
    //Add run_task to global scheduler via task_index db
    let task_id = scheduler.add_one_shot_in(Duration::from_secs(0), TaskKind::RunTask(run_task))?;
    index_store.sync_user_tasks(&user.user_id, scheduler.tasks())?;

    info!(
        "scheduler tasks enqueued user_id={} task_id={} message_id={:?} workspace={} thread_epoch={}",
        user.user_id,
        task_id,
        message.message_id,
        workspace.display(),
        thread_state.epoch
    );

    Ok(())
}

pub(super) fn append_lark_message(
    workspace: &Path,
    message: &crate::channel::InboundMessage,
    raw_payload: &[u8],
    seq: u32,
) -> Result<(), BoxError> {
    let incoming_dir = workspace.join("incoming_email");
    std::fs::create_dir_all(&incoming_dir)?;

    // Save raw JSON payload
    let json_filename = format!("{:04}_lark.json", seq);
    std::fs::write(incoming_dir.join(&json_filename), raw_payload)?;

    // Save text content
    if let Some(ref text) = message.text_body {
        let txt_filename = format!("{:04}_lark.txt", seq);
        let content = format!(
            "From: {}\nDate: {}\n\n{}",
            message.sender,
            chrono::Utc::now().to_rfc3339(),
            text
        );
        std::fs::write(incoming_dir.join(&txt_filename), content)?;
    }

    Ok(())
}
```

---

## 8. Export in scheduler_module/src/service/inbound/mod.rs

```rust
// Add:
mod lark;

pub(super) use lark::process_lark_event;
```

---

## 9. UserIdentities - run_task_module/src/run_task/types.rs

```rust
pub struct UserIdentities {
    // ... existing fields ...

    /// Lark (Feishu) user IDs (open_id format, e.g., "ou_xxx")
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lark_user_ids: Vec<String>,

    /// WeChat Work user IDs
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wechat_user_ids: Vec<String>,
}
```

---

## 10. Prompt Updates - run_task_module/src/run_task/prompt.rs

### 10a. Reply Instruction (add after "wechat" case, ~line 70):

```rust
            "lark" | "feishu" => {
                "2. After finishing the task (step one), write a plain text reply in reply_message.txt in the workspace root. Keep the reply concise and conversational. Lark supports basic markdown: **bold**, *italic*, ~~strikethrough~~, `code`. If there are files to attach, put them in reply_attachments/ and mention them in the reply. Do not pretend the job has been done without actually doing it."
            }
```

### 10b. Cross-Channel Routing Schema (update ~line 360):

```rust
// Update the channel enum in the JSON schema:
"channel": "email" | "slack" | "discord" | "telegram" | "sms" | "whatsapp" | "bluebubbles" | "wechat" | "lark",

// Add to identifier format per channel section:
- lark: Lark open_id (e.g., "ou_xxxxxxxxxxxxxxxxx")
- wechat: WeChat Work UserID (e.g., "zhangsan")

// Add to target channel format section:
- lark target: reply_message.txt (**bold**, *italic*, `code`)
- wechat target: reply_message.txt (plain text)
```

### 10c. User Identities Display (add in build_user_identities_section, ~line 345):

```rust
//identities.lark_user_id populated via Lark OAuth (separate flow)
if !identities.lark_user_ids.is_empty() {
    channels.push(format!(
        "- Lark User IDs: {}",
        identities.lark_user_ids.join(", ")
    ));
}

if !identities.wechat_user_ids.is_empty() {
    channels.push(format!(
        "- WeChat User IDs: {}",
        identities.wechat_user_ids.join(", ")
    ));
}
```

---

## 11. schedule_auto_reply in actions.rs

```rust
pub(crate) fn schedule_auto_reply<E: TaskExecutor>(
    scheduler: &mut Scheduler<E>,
    task: &RunTaskTask,
) -> Result<bool, SchedulerError> {
    // ... validation ...

    // Determine reply file based on channel
    let (reply_filename, attachments_dirname) = match target_channel {
        Channel::Slack | Channel::Discord | Channel::WeChat | Channel::Lark => {
            ("reply_message.txt", "reply_attachments")
        }
        Channel::Email => ("reply_email_draft.html", "reply_email_attachments"),
        // ...
    };

    // Read the reply file path
    let html_path = task.workspace_dir.join(reply_filename);
    if !html_path.exists() {
        return Ok(false); // No reply to send
    }

    // Create SendReplyTask
    let send_task = SendReplyTask {
        channel: target_channel.clone(),
        subject: reply_context.subject,
        html_path,                          // ← Path to reply_message.txt
        attachments_dir,
        from: reply_from,
        to: target_recipients,              // ← From task.reply_to (the sender's open_id)
        cc: Vec::new(),
        bcc: Vec::new(),
        in_reply_to,
        references,
        archive_root: task.archive_root.clone(),
        thread_epoch: task.thread_epoch,
        thread_state_path: task.thread_state_path.clone(),
        employee_id: task.employee_id.clone(),
    };

    // Schedule it immediately
    let task_id = scheduler.add_one_shot_in(
        Duration::from_secs(0),
        TaskKind::SendReply(send_task)
    )?;

    Ok(true)
}
```

---

## 12. executor.rs pattern-matches Channel::Lark to execute_lark_send in outbound.rs

```rust
// In execute_send_reply or similar
fn execute_send_reply(task: &SendReplyTask) -> Result<(), SchedulerError> {
    match task.channel {
        Channel::Slack => execute_slack_send(task)?,
        Channel::WeChat => execute_wechat_send(task)?,
        Channel::Lark => execute_lark_send(task)?,    // ← Dispatches here
        Channel::Email => execute_email_send(task)?,
        // ...
    }
    Ok(())
}
```

---

## 13. Execute SendReplyTask in outbound.rs

```rust
/// Execute a SendReplyTask via Lark (飞书).
pub(crate) fn execute_lark_send(task: &SendReplyTask) -> Result<(), SchedulerError> {
    use crate::adapters::lark::LarkOutboundAdapter;
    use crate::channel::{ChannelMetadata, OutboundAdapter, OutboundMessage};

    dotenvy::dotenv().ok();

    let adapter = LarkOutboundAdapter::from_env()
        .map_err(|err| SchedulerError::TaskFailed(format!("Lark config error: {}", err)))?;

    // Read plain text content from reply_message.txt
    let text_body = if task.html_path.exists() {
        fs::read_to_string(&task.html_path).unwrap_or_default()
    } else {
        String::new()
    };

    let message = OutboundMessage {
        channel: Channel::Lark,
        from: task.from.clone(),
        to: task.to.clone(),
        cc: vec![],
        bcc: vec![],
        subject: task.subject.clone(),
        text_body,
        html_body: String::new(),
        html_path: Some(task.html_path.clone()),
        attachments_dir: Some(task.attachments_dir.clone()),
        thread_id: task.in_reply_to.clone(),
        metadata: ChannelMetadata {
            lark_open_id: task.to.first().cloned(),
            ..Default::default()
        },
    };

    let result = adapter
        .send(&message)
        .map_err(|err| SchedulerError::TaskFailed(format!("Lark send failed: {}", err)))?;

    if !result.success {
        return Err(SchedulerError::TaskFailed(format!(
            "Lark API error: {}",
            result.error.unwrap_or_default()
        )));
    }

    info!(
        "sent Lark message to {:?}, message_id={}",
        task.to, result.message_id
    );

    Ok(())
}
```

---

## 14. Gateway Config - gateway.toml

```toml
# Lark (Feishu) routing
[[routes]]
channel = "lark"
key = "*"
employee_id = "little_bear"
tenant_id = "default"
```

---

## 15. Environment Variables

```bash
# Lark Open Platform credentials
LARK_APP_ID=cli_xxxxxxxxxxxx
LARK_APP_SECRET=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
LARK_ENCRYPT_KEY=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
LARK_VERIFICATION_TOKEN=xxxxxxxxxxxxxxxxxxxxxxxx
```
