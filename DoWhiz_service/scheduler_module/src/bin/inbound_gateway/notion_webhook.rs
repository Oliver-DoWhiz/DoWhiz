//! Notion webhook handler for comment.created events.
//!
//! This module handles incoming Notion webhooks, with:
//! - HMAC-SHA256 signature verification
//! - Self-trigger prevention (ignores comments posted by our own bot)
//! - Multi-environment routing (each env has its own integration_id)

use std::env;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, info, warn};

use scheduler_module::channel::{Channel, ChannelMetadata, InboundMessage};
use scheduler_module::notion_store::NotionStore;

use super::handlers::{build_envelope, enqueue_envelope};
use super::state::{GatewayState, RouteDecision};

/// Notion webhook event types we handle
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NotionEventType {
    #[serde(rename = "comment.created")]
    CommentCreated,
    #[serde(other)]
    Unknown,
}

/// Author information in webhook payload
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotionWebhookAuthor {
    pub id: String,
    #[serde(rename = "type")]
    pub author_type: String,
}

/// Rich text element in comment
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotionRichTextElement {
    #[serde(rename = "type")]
    pub element_type: String,
    pub plain_text: Option<String>,
    pub text: Option<NotionTextContent>,
    pub mention: Option<NotionMentionContent>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotionTextContent {
    pub content: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotionMentionContent {
    #[serde(rename = "type")]
    pub mention_type: Option<String>,
    pub user: Option<NotionMentionUser>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotionMentionUser {
    pub id: Option<String>,
    pub name: Option<String>,
}

/// Comment data in webhook payload
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotionCommentData {
    pub id: String,
    pub parent: Option<NotionCommentParent>,
    pub created_by: Option<NotionCommentCreatedBy>,
    pub rich_text: Option<Vec<NotionRichTextElement>>,
    pub discussion_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotionCommentParent {
    #[serde(rename = "type")]
    pub parent_type: String,
    pub page_id: Option<String>,
    pub block_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotionCommentCreatedBy {
    pub id: String,
    #[serde(rename = "type")]
    pub author_type: String,
    pub name: Option<String>,
}

/// Main webhook payload structure
/// Based on actual Notion webhook format (API version 2026-03-11)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotionWebhookPayload {
    /// Event ID
    pub id: String,
    /// Event type (e.g., "comment.created")
    #[serde(rename = "type")]
    pub event_type: Option<NotionEventType>,
    /// Timestamp of the event
    pub timestamp: Option<String>,
    /// The integration that received this webhook (same as bot_id from OAuth)
    pub integration_id: String,
    pub workspace_id: String,
    pub workspace_name: Option<String>,
    pub subscription_id: Option<String>,
    /// Authors who triggered this event
    pub authors: Option<Vec<NotionWebhookAuthor>>,
    /// Users/bots who can access the affected resource
    pub accessible_by: Option<Vec<NotionWebhookAuthor>>,
    /// Comment data (for comment.created events) - might be under "data" or "entity"
    pub data: Option<NotionCommentData>,
    #[serde(rename = "entity")]
    pub entity: Option<NotionCommentData>,
    /// Verification token for signature validation
    pub verification_token: Option<String>,
}

impl NotionWebhookPayload {
    /// Check if this event was triggered by our own bot posting a comment.
    ///
    /// Returns true if any author is a bot with the same ID as the integration_id,
    /// meaning our integration posted this comment and we should ignore it.
    pub fn is_self_triggered(&self) -> bool {
        let Some(authors) = &self.authors else {
            return false;
        };

        for author in authors {
            if author.author_type == "bot" && author.id == self.integration_id {
                return true;
            }
        }

        false
    }

    /// Get the comment data (from either `data` or `entity` field)
    fn comment_data(&self) -> Option<&NotionCommentData> {
        self.data.as_ref().or(self.entity.as_ref())
    }

    /// Extract the plain text content from the comment's rich_text array.
    pub fn extract_comment_text(&self) -> String {
        let Some(data) = self.comment_data() else {
            return String::new();
        };
        let Some(rich_text) = &data.rich_text else {
            return String::new();
        };

        rich_text
            .iter()
            .filter_map(|elem| {
                elem.plain_text
                    .clone()
                    .or_else(|| elem.text.as_ref().and_then(|t| t.content.clone()))
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// Get the page ID from the comment parent
    pub fn page_id(&self) -> Option<&str> {
        self.comment_data()
            .and_then(|d| d.parent.as_ref())
            .and_then(|p| p.page_id.as_deref())
    }

    /// Get the comment ID
    pub fn comment_id(&self) -> Option<&str> {
        self.comment_data().map(|d| d.id.as_str())
    }

    /// Get the author name (created_by.name)
    pub fn author_name(&self) -> Option<&str> {
        self.comment_data()
            .and_then(|d| d.created_by.as_ref())
            .and_then(|c| c.name.as_deref())
    }

    /// Get the author ID (created_by.id)
    pub fn author_id(&self) -> Option<&str> {
        self.comment_data()
            .and_then(|d| d.created_by.as_ref())
            .map(|c| c.id.as_str())
    }

    /// Check if the comment contains an @mention for a specific bot/integration.
    ///
    /// Looks for mention elements in rich_text where mention.user.id matches the integration_id.
    pub fn contains_bot_mention(&self, integration_id: &str) -> bool {
        let Some(data) = self.comment_data() else {
            return false;
        };
        let Some(rich_text) = &data.rich_text else {
            return false;
        };

        for elem in rich_text {
            if elem.element_type == "mention" {
                if let Some(mention) = &elem.mention {
                    if mention.mention_type.as_deref() == Some("user") {
                        if let Some(user) = &mention.user {
                            if user.id.as_deref() == Some(integration_id) {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    /// Check if this is a comment.created event
    pub fn is_comment_created(&self) -> bool {
        matches!(self.event_type, Some(NotionEventType::CommentCreated))
    }
}

/// Check if this webhook is for our environment's integration.
///
/// Compares payload.integration_id with NOTION_INTEGRATION_ID env var.
/// If no match, this event is for another environment (staging vs prod).
fn is_my_integration(payload: &NotionWebhookPayload) -> bool {
    let my_integration_id = env::var("NOTION_INTEGRATION_ID")
        .ok()
        .filter(|v| !v.trim().is_empty());

    match my_integration_id {
        Some(my_id) => payload.integration_id == my_id,
        // If not configured, accept all (for backwards compatibility during rollout)
        None => {
            debug!(
                "NOTION_INTEGRATION_ID not configured, accepting webhook for integration_id={}",
                payload.integration_id
            );
            true
        }
    }
}

/// Handle incoming Notion webhook POST request.
pub async fn ingest_notion_webhook(
    State(state): State<Arc<GatewayState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // Verify signature
    if let Err(reason) = super::verify::verify_notion(&headers, &body) {
        warn!("notion webhook signature verification failed: {}", reason);
        return (StatusCode::UNAUTHORIZED, Json(json!({"status": reason})));
    }

    // Parse payload
    let payload: NotionWebhookPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            let body_preview = String::from_utf8_lossy(&body[..body.len().min(500)]);
            warn!(
                "notion webhook failed to parse payload: {} - body preview: {}",
                e, body_preview
            );
            return (StatusCode::BAD_REQUEST, Json(json!({"status": "bad_json"})));
        }
    };

    info!(
        "notion webhook received: type={:?} integration_id={} workspace_id={}",
        payload.event_type, payload.integration_id, payload.workspace_id
    );

    // Only handle comment.created events
    if !payload.is_comment_created() {
        debug!(
            "notion webhook ignoring event type: {:?}",
            payload.event_type
        );
        return (
            StatusCode::OK,
            Json(json!({"status": "ignored", "reason": "unsupported_event_type"})),
        );
    }

    // Check if this webhook is for our environment
    if !is_my_integration(&payload) {
        info!(
            "notion webhook not for our integration: payload.integration_id={} (ignoring)",
            payload.integration_id
        );
        return (
            StatusCode::OK,
            Json(json!({"status": "ignored", "reason": "not_my_integration"})),
        );
    }

    // Check for self-trigger (our bot posted this comment)
    if payload.is_self_triggered() {
        info!(
            "notion webhook self-triggered: integration_id={} (ignoring)",
            payload.integration_id
        );
        return (
            StatusCode::OK,
            Json(json!({"status": "ignored", "reason": "self_triggered"})),
        );
    }

    // Look up credentials by integration_id (== bot_id)
    let notion_store = match NotionStore::new() {
        Ok(store) => store,
        Err(e) => {
            warn!("notion webhook failed to create NotionStore: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "store_error"})),
            );
        }
    };

    let credential = match notion_store.get_credential_by_bot_id(&payload.integration_id) {
        Ok(cred) => cred,
        Err(e) => {
            warn!(
                "notion webhook no credential found for integration_id={}: {}",
                payload.integration_id, e
            );
            return (
                StatusCode::OK,
                Json(json!({"status": "ignored", "reason": "no_credential"})),
            );
        }
    };

    // Build routing decision based on employee directory
    let route = resolve_notion_route(&payload, &credential, &state);
    let Some(route) = route else {
        info!(
            "notion webhook no route for workspace_id={}",
            payload.workspace_id
        );
        return (StatusCode::OK, Json(json!({"status": "no_route"})));
    };

    // Extract message details
    let comment_text = payload.extract_comment_text();
    let page_id = payload.page_id().unwrap_or("unknown").to_string();
    let comment_id = payload.comment_id().unwrap_or("unknown").to_string();
    let author_name = payload.author_name().map(|s| s.to_string());
    let author_id = payload.author_id().unwrap_or("unknown").to_string();

    info!(
        "notion webhook processing comment: page_id={} comment_id={} author={:?}",
        page_id, comment_id, author_name
    );

    // Build InboundMessage
    let thread_id = format!(
        "notion:{}:{}",
        payload.workspace_id,
        payload
            .data
            .as_ref()
            .and_then(|d| d.discussion_id.as_deref())
            .unwrap_or(&comment_id)
    );
    let message_id = format!("notion-comment-{}", comment_id);

    let message = InboundMessage {
        channel: Channel::Notion,
        sender: author_id.clone(),
        sender_name: author_name,
        recipient: payload.integration_id.clone(),
        subject: Some(format!("Notion comment on page {}", page_id)),
        text_body: Some(comment_text),
        html_body: None,
        thread_id,
        message_id: Some(message_id.clone()),
        attachments: Vec::new(),
        reply_to: vec![],
        raw_payload: body.to_vec(),
        metadata: ChannelMetadata {
            notion_page_id: Some(page_id.clone()),
            notion_comment_id: Some(comment_id.clone()),
            notion_workspace_id: Some(payload.workspace_id.clone()),
            ..Default::default()
        },
    };

    // Build and enqueue envelope
    let envelope = match build_envelope(
        route,
        Channel::Notion,
        Some(message_id),
        &message,
        &body,
    )
    .await
    {
        Ok(env) => env,
        Err(e) => {
            warn!("notion webhook failed to build envelope: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "envelope_build_error"})),
            );
        }
    };

    enqueue_envelope(state.queue.clone(), envelope).await
}

/// Resolve routing for Notion webhook based on workspace/integration mapping.
fn resolve_notion_route(
    payload: &NotionWebhookPayload,
    credential: &scheduler_module::notion_store::NotionCredential,
    state: &GatewayState,
) -> Option<RouteDecision> {
    // Try to find employee by workspace_id in routes
    let route_key = payload.workspace_id.clone();

    // Check explicit routes first
    if let Some(route) = super::routes::resolve_route(Channel::Notion, &route_key, state) {
        return Some(route);
    }

    // Fallback: use account_id from credential to find associated employee
    // For now, use default employee from config
    let employee_id = state
        .config
        .defaults
        .employee_id
        .clone()
        .unwrap_or_else(|| "oliver".to_string());
    let tenant_id = state
        .config
        .defaults
        .tenant_id
        .clone()
        .unwrap_or_else(|| "default".to_string());

    info!(
        "notion webhook using default route: employee_id={} tenant_id={} account_id={}",
        employee_id, tenant_id, credential.account_id
    );

    Some(RouteDecision {
        tenant_id,
        employee_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_payload(authors: Option<Vec<NotionWebhookAuthor>>) -> NotionWebhookPayload {
        NotionWebhookPayload {
            event_type: NotionEventType::CommentCreated,
            integration_id: "bot-123".to_string(),
            workspace_id: "ws-456".to_string(),
            authors,
            data: Some(NotionCommentData {
                id: "comment-1".to_string(),
                parent: Some(NotionCommentParent {
                    parent_type: "page_id".to_string(),
                    page_id: Some("page-789".to_string()),
                    block_id: None,
                }),
                created_by: Some(NotionCommentCreatedBy {
                    id: "user-111".to_string(),
                    author_type: "person".to_string(),
                    name: Some("Test User".to_string()),
                }),
                rich_text: Some(vec![NotionRichTextElement {
                    element_type: "text".to_string(),
                    plain_text: Some("Hello world".to_string()),
                    text: None,
                    mention: None,
                }]),
                discussion_id: Some("disc-222".to_string()),
            }),
            verification_token: None,
        }
    }

    #[test]
    fn test_is_self_triggered_false_for_person() {
        let payload = make_test_payload(Some(vec![NotionWebhookAuthor {
            id: "user-111".to_string(),
            author_type: "person".to_string(),
        }]));
        assert!(!payload.is_self_triggered());
    }

    #[test]
    fn test_is_self_triggered_true_for_matching_bot() {
        let payload = make_test_payload(Some(vec![NotionWebhookAuthor {
            id: "bot-123".to_string(), // Same as integration_id
            author_type: "bot".to_string(),
        }]));
        assert!(payload.is_self_triggered());
    }

    #[test]
    fn test_is_self_triggered_false_for_different_bot() {
        let payload = make_test_payload(Some(vec![NotionWebhookAuthor {
            id: "other-bot-999".to_string(),
            author_type: "bot".to_string(),
        }]));
        assert!(!payload.is_self_triggered());
    }

    #[test]
    fn test_is_self_triggered_false_for_no_authors() {
        let payload = make_test_payload(None);
        assert!(!payload.is_self_triggered());
    }

    #[test]
    fn test_extract_comment_text() {
        let payload = make_test_payload(None);
        assert_eq!(payload.extract_comment_text(), "Hello world");
    }

    #[test]
    fn test_extract_comment_text_multiple_elements() {
        let mut payload = make_test_payload(None);
        if let Some(data) = &mut payload.data {
            data.rich_text = Some(vec![
                NotionRichTextElement {
                    element_type: "text".to_string(),
                    plain_text: Some("Hello ".to_string()),
                    text: None,
                    mention: None,
                },
                NotionRichTextElement {
                    element_type: "mention".to_string(),
                    plain_text: Some("@Bot".to_string()),
                    text: None,
                    mention: Some(NotionMentionContent {
                        mention_type: Some("user".to_string()),
                        user: Some(NotionMentionUser {
                            id: Some("bot-123".to_string()),
                            name: Some("Bot".to_string()),
                        }),
                    }),
                },
                NotionRichTextElement {
                    element_type: "text".to_string(),
                    plain_text: Some(" please help".to_string()),
                    text: None,
                    mention: None,
                },
            ]);
        }
        assert_eq!(payload.extract_comment_text(), "Hello @Bot please help");
    }

    #[test]
    fn test_contains_bot_mention_true() {
        let mut payload = make_test_payload(None);
        if let Some(data) = &mut payload.data {
            data.rich_text = Some(vec![NotionRichTextElement {
                element_type: "mention".to_string(),
                plain_text: Some("@Bot".to_string()),
                text: None,
                mention: Some(NotionMentionContent {
                    mention_type: Some("user".to_string()),
                    user: Some(NotionMentionUser {
                        id: Some("bot-123".to_string()),
                        name: Some("Bot".to_string()),
                    }),
                }),
            }]);
        }
        assert!(payload.contains_bot_mention("bot-123"));
    }

    #[test]
    fn test_contains_bot_mention_false_different_id() {
        let mut payload = make_test_payload(None);
        if let Some(data) = &mut payload.data {
            data.rich_text = Some(vec![NotionRichTextElement {
                element_type: "mention".to_string(),
                plain_text: Some("@User".to_string()),
                text: None,
                mention: Some(NotionMentionContent {
                    mention_type: Some("user".to_string()),
                    user: Some(NotionMentionUser {
                        id: Some("user-other".to_string()),
                        name: Some("User".to_string()),
                    }),
                }),
            }]);
        }
        assert!(!payload.contains_bot_mention("bot-123"));
    }

    #[test]
    fn test_page_id() {
        let payload = make_test_payload(None);
        assert_eq!(payload.page_id(), Some("page-789"));
    }

    #[test]
    fn test_comment_id() {
        let payload = make_test_payload(None);
        assert_eq!(payload.comment_id(), Some("comment-1"));
    }

    #[test]
    fn test_author_name() {
        let payload = make_test_payload(None);
        assert_eq!(payload.author_name(), Some("Test User"));
    }

    #[test]
    fn test_deserialize_comment_created() {
        let json = r#"{
            "type": "comment.created",
            "integration_id": "abc-123",
            "workspace_id": "ws-456",
            "authors": [{"id": "user-1", "type": "person"}],
            "data": {
                "id": "comment-1",
                "parent": {"type": "page_id", "page_id": "page-1"},
                "created_by": {"id": "user-1", "type": "person", "name": "Alice"},
                "rich_text": [{"type": "text", "plain_text": "Hello"}]
            }
        }"#;

        let payload: NotionWebhookPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.event_type, NotionEventType::CommentCreated);
        assert_eq!(payload.integration_id, "abc-123");
        assert_eq!(payload.extract_comment_text(), "Hello");
    }

    #[test]
    fn test_deserialize_unknown_event_type() {
        let json = r#"{
            "type": "page.created",
            "integration_id": "abc",
            "workspace_id": "ws"
        }"#;

        let payload: NotionWebhookPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.event_type, NotionEventType::Unknown);
    }
}
