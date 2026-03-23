use std::collections::{HashMap, HashSet};
use std::env;
use std::path::Path;

use crate::slack_store::SlackStore;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use super::state::AppState;
use super::{BoxError, ServiceConfig};

pub(crate) const CHAT_HISTORY_SCOPE_FILE_NAME: &str = ".chat_history_scope.json";

const CHAT_HISTORY_SCOPE_SIGNING_SECRET_ENV: &str = "CHAT_HISTORY_SCOPE_SIGNING_SECRET";
const CHAT_HISTORY_PROXY_BASE_URL_ENV: &str = "CHAT_HISTORY_PROXY_BASE_URL";
const DOWHIZ_API_URL_ENV: &str = "DOWHIZ_API_URL";
const FRONTEND_URL_ENV: &str = "FRONTEND_URL";
const POSTMARK_INBOUND_HOOK_URL_ENV: &str = "POSTMARK_INBOUND_HOOK_URL";
const SERVICE_URL_ENV: &str = "SERVICE_URL";
const RUN_TASK_EXECUTION_BACKEND_ENV: &str = "RUN_TASK_EXECUTION_BACKEND";
const CHAT_HISTORY_SCOPE_TTL_MINUTES_ENV: &str = "CHAT_HISTORY_SCOPE_TTL_MINUTES";
const CHAT_HISTORY_SLACK_MAX_HISTORY_PAGES_ENV: &str = "CHAT_HISTORY_SLACK_MAX_HISTORY_PAGES";
const CHAT_HISTORY_SLACK_MAX_THREAD_PAGES_ENV: &str = "CHAT_HISTORY_SLACK_MAX_THREAD_PAGES";
const CHAT_HISTORY_DISCORD_MAX_HISTORY_PAGES_PER_CHANNEL_ENV: &str =
    "CHAT_HISTORY_DISCORD_MAX_HISTORY_PAGES_PER_CHANNEL";
const CHAT_HISTORY_DISCORD_MAX_CHANNELS_ENV: &str = "CHAT_HISTORY_DISCORD_MAX_CHANNELS";

const DEFAULT_SCOPE_TTL_MINUTES: i64 = 12 * 60;
const DEFAULT_SEARCH_RESULT_LIMIT: usize = 20;
const MAX_SEARCH_RESULT_LIMIT: usize = 100;
const MAX_QUERY_CHARS: usize = 240;
const DEFAULT_SLACK_MAX_HISTORY_PAGES: usize = 100;
const DEFAULT_SLACK_MAX_THREAD_PAGES: usize = 20;
const DEFAULT_DISCORD_MAX_HISTORY_PAGES_PER_CHANNEL: usize = 40;
const DEFAULT_DISCORD_MAX_CHANNELS: usize = 200;
const DISCORD_TEXT_CHANNEL_TYPES: &[u8] = &[0, 5, 10, 11, 12, 15];
const AZURE_ACI_EXECUTION_BACKEND: &str = "azure_aci";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ChatHistoryPlatform {
    Slack,
    Discord,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ChatHistoryScopeMode {
    CurrentConversation,
    CurrentGuild,
    DirectMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SlackScopeGrant {
    team_id: String,
    channel_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    thread_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiscordScopeGrant {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    guild_id: Option<u64>,
    channel_id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    thread_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatHistoryScopeGrant {
    version: u8,
    platform: ChatHistoryPlatform,
    scope_mode: ChatHistoryScopeMode,
    employee_id: String,
    iat: usize,
    exp: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    slack: Option<SlackScopeGrant>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    discord: Option<DiscordScopeGrant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceChatHistoryScope {
    version: u8,
    platform: ChatHistoryPlatform,
    scope_mode: ChatHistoryScopeMode,
    description: String,
    generated_at: String,
    expires_at: String,
    search_endpoint: String,
    token: String,
    #[serde(flatten)]
    scope: WorkspaceChatHistoryScopeDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct WorkspaceChatHistoryScopeDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    employee_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    team_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    guild_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    channel_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    thread_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ChatHistorySearchRequest {
    pub(crate) query: String,
    #[serde(default)]
    pub(crate) limit: Option<usize>,
    #[serde(default)]
    pub(crate) channel_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ChatHistorySearchResponse {
    platform: ChatHistoryPlatform,
    scope_mode: ChatHistoryScopeMode,
    query: String,
    limit: usize,
    searched_channels: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    warnings: Vec<String>,
    results: Vec<ChatHistoryMatch>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ChatHistoryMatch {
    channel_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_name: Option<String>,
    message_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    thread_id: Option<String>,
    timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    author_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author_name: Option<String>,
    text: String,
    source: String,
}

#[derive(Debug, Clone, Serialize)]
struct ChatHistoryErrorResponse {
    error: String,
}

#[derive(Debug)]
struct ChatHistoryRequestError {
    status: StatusCode,
    message: String,
}

impl ChatHistoryRequestError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct SlackHistoryResponse {
    ok: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    messages: Vec<SlackHistoryMessage>,
    #[serde(default)]
    response_metadata: Option<SlackResponseMetadata>,
}

#[derive(Debug, Deserialize, Clone)]
struct SlackHistoryMessage {
    #[serde(default)]
    text: String,
    #[serde(default)]
    user: Option<String>,
    #[serde(default)]
    username: Option<String>,
    ts: String,
    #[serde(default)]
    thread_ts: Option<String>,
    #[serde(default)]
    reply_count: Option<u64>,
    #[serde(default)]
    files: Vec<SlackHistoryFile>,
}

#[derive(Debug, Deserialize, Clone)]
struct SlackHistoryFile {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SlackResponseMetadata {
    #[serde(default)]
    next_cursor: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct DiscordGuildChannel {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "type")]
    kind: u8,
}

#[derive(Debug, Deserialize)]
struct DiscordActiveThreadsResponse {
    #[serde(default)]
    threads: Vec<DiscordGuildChannel>,
}

#[derive(Debug, Deserialize)]
struct DiscordHistoryMessage {
    id: String,
    #[serde(default)]
    content: String,
    timestamp: String,
    author: DiscordAuthor,
    #[serde(default)]
    attachments: Vec<DiscordAttachment>,
}

#[derive(Debug, Deserialize)]
struct DiscordAuthor {
    id: String,
    username: String,
    #[serde(default)]
    global_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DiscordAttachment {
    filename: String,
}

pub(crate) fn write_slack_chat_history_scope_file(
    config: &ServiceConfig,
    workspace: &Path,
    message: &crate::channel::InboundMessage,
) -> Result<(), BoxError> {
    let team_id = message
        .metadata
        .slack_team_id
        .clone()
        .ok_or("missing slack team id for chat history scope")?;
    let channel_id = message
        .metadata
        .slack_channel_id
        .clone()
        .ok_or("missing slack channel id for chat history scope")?;
    let claims = build_scope_grant(
        config,
        ChatHistoryPlatform::Slack,
        ChatHistoryScopeMode::CurrentConversation,
        Some(SlackScopeGrant {
            team_id: team_id.clone(),
            channel_id: channel_id.clone(),
            thread_id: Some(message.thread_id.clone()),
        }),
        None,
    )?;
    let description = format!(
        "History search is limited to this Slack conversation ({channel_id}) and its threads. Cross-channel and cross-workspace access is blocked.",
    );
    let scope = WorkspaceChatHistoryScope {
        version: claims.version,
        platform: claims.platform,
        scope_mode: claims.scope_mode,
        description,
        generated_at: format_unix_ts(claims.iat)?,
        expires_at: format_unix_ts(claims.exp)?,
        search_endpoint: format!(
            "{}/internal/chat-history/search",
            chat_history_base_url(config)
        ),
        token: encode_scope_grant(config, &claims)?,
        scope: WorkspaceChatHistoryScopeDetails {
            employee_id: Some(config.employee_id.clone()),
            team_id: Some(team_id),
            channel_id: Some(channel_id),
            thread_id: Some(message.thread_id.clone()),
            ..WorkspaceChatHistoryScopeDetails::default()
        },
    };
    std::fs::write(
        workspace.join(CHAT_HISTORY_SCOPE_FILE_NAME),
        format!("{}\n", serde_json::to_string_pretty(&scope)?),
    )?;
    Ok(())
}

pub(crate) fn write_discord_chat_history_scope_file(
    config: &ServiceConfig,
    workspace: &Path,
    message: &crate::channel::InboundMessage,
) -> Result<(), BoxError> {
    let channel_id = message
        .metadata
        .discord_channel_id
        .ok_or("missing discord channel id for chat history scope")?;
    let guild_id = message.metadata.discord_guild_id;
    let scope_mode = if guild_id.is_some() {
        ChatHistoryScopeMode::CurrentGuild
    } else {
        ChatHistoryScopeMode::DirectMessage
    };
    let claims = build_scope_grant(
        config,
        ChatHistoryPlatform::Discord,
        scope_mode,
        None,
        Some(DiscordScopeGrant {
            guild_id,
            channel_id,
            thread_id: Some(message.thread_id.clone()),
        }),
    )?;
    let description = match guild_id {
        Some(guild_id) => format!(
            "History search is limited to the current Discord server ({guild_id}). Cross-server access is blocked."
        ),
        None => "History search is limited to the current Discord DM conversation.".to_string(),
    };
    let scope = WorkspaceChatHistoryScope {
        version: claims.version,
        platform: claims.platform,
        scope_mode: claims.scope_mode,
        description,
        generated_at: format_unix_ts(claims.iat)?,
        expires_at: format_unix_ts(claims.exp)?,
        search_endpoint: format!(
            "{}/internal/chat-history/search",
            chat_history_base_url(config)
        ),
        token: encode_scope_grant(config, &claims)?,
        scope: WorkspaceChatHistoryScopeDetails {
            employee_id: Some(config.employee_id.clone()),
            guild_id,
            channel_id: Some(channel_id.to_string()),
            thread_id: Some(message.thread_id.clone()),
            ..WorkspaceChatHistoryScopeDetails::default()
        },
    };
    std::fs::write(
        workspace.join(CHAT_HISTORY_SCOPE_FILE_NAME),
        format!("{}\n", serde_json::to_string_pretty(&scope)?),
    )?;
    Ok(())
}

pub(crate) async fn search_chat_history(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ChatHistorySearchRequest>,
) -> impl IntoResponse {
    let authorization = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim().to_string());

    let state_for_blocking = state.clone();
    let result = tokio::task::spawn_blocking(move || {
        let token = extract_bearer_token(authorization.as_deref())?;
        let claims = decode_scope_grant(&state_for_blocking.config, &token)?;
        if claims.employee_id != state_for_blocking.config.employee_id {
            return Err(ChatHistoryRequestError::new(
                StatusCode::FORBIDDEN,
                "chat history grant does not belong to this worker",
            ));
        }
        execute_chat_history_search(
            &state_for_blocking.config,
            &state_for_blocking.slack_store,
            claims,
            request,
        )
    })
    .await;

    match result {
        Ok(Ok(response)) => (StatusCode::OK, Json(response)).into_response(),
        Ok(Err(err)) => (
            err.status,
            Json(ChatHistoryErrorResponse { error: err.message }),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ChatHistoryErrorResponse {
                error: format!("chat history task join error: {err}"),
            }),
        )
            .into_response(),
    }
}

fn execute_chat_history_search(
    config: &ServiceConfig,
    slack_store: &SlackStore,
    claims: ChatHistoryScopeGrant,
    request: ChatHistorySearchRequest,
) -> Result<ChatHistorySearchResponse, ChatHistoryRequestError> {
    let query = request.query.trim();
    if query.is_empty() {
        return Err(ChatHistoryRequestError::new(
            StatusCode::BAD_REQUEST,
            "query is required",
        ));
    }
    if query.chars().count() > MAX_QUERY_CHARS {
        return Err(ChatHistoryRequestError::new(
            StatusCode::BAD_REQUEST,
            format!("query is too long (max {MAX_QUERY_CHARS} chars)"),
        ));
    }
    let limit = request
        .limit
        .unwrap_or(DEFAULT_SEARCH_RESULT_LIMIT)
        .clamp(1, MAX_SEARCH_RESULT_LIMIT);

    match claims.platform {
        ChatHistoryPlatform::Slack => {
            let slack = claims.slack.ok_or_else(|| {
                ChatHistoryRequestError::new(
                    StatusCode::FORBIDDEN,
                    "slack chat history grant is missing slack scope",
                )
            })?;
            if let Some(request_channel_id) = request.channel_id.as_deref() {
                if request_channel_id.trim() != slack.channel_id {
                    return Err(ChatHistoryRequestError::new(
                        StatusCode::FORBIDDEN,
                        "slack history search cannot access other channels",
                    ));
                }
            }
            let installation = slack_store
                .get_installation_or_env(&slack.team_id)
                .map_err(|err| {
                    ChatHistoryRequestError::new(
                        StatusCode::BAD_GATEWAY,
                        format!("failed to resolve Slack installation: {err}"),
                    )
                })?;
            let results = search_slack_channel_history(
                &installation.bot_token,
                &slack.channel_id,
                query,
                limit,
            )
            .map_err(|err| {
                ChatHistoryRequestError::new(
                    StatusCode::BAD_GATEWAY,
                    format!("Slack history search failed: {err}"),
                )
            })?;
            Ok(ChatHistorySearchResponse {
                platform: ChatHistoryPlatform::Slack,
                scope_mode: claims.scope_mode,
                query: query.to_string(),
                limit,
                searched_channels: 1,
                warnings: Vec::new(),
                results,
            })
        }
        ChatHistoryPlatform::Discord => {
            let discord = claims.discord.ok_or_else(|| {
                ChatHistoryRequestError::new(
                    StatusCode::FORBIDDEN,
                    "discord chat history grant is missing discord scope",
                )
            })?;
            let token = resolve_discord_bot_token(config).ok_or_else(|| {
                ChatHistoryRequestError::new(
                    StatusCode::BAD_GATEWAY,
                    "discord bot token is not configured",
                )
            })?;
            let (results, searched_channels, warnings) = search_discord_history(
                &token,
                &discord,
                request.channel_id.as_deref(),
                query,
                limit,
            )
            .map_err(|err| {
                ChatHistoryRequestError::new(
                    StatusCode::BAD_GATEWAY,
                    format!("Discord history search failed: {err}"),
                )
            })?;
            Ok(ChatHistorySearchResponse {
                platform: ChatHistoryPlatform::Discord,
                scope_mode: claims.scope_mode,
                query: query.to_string(),
                limit,
                searched_channels,
                warnings,
                results,
            })
        }
    }
}

fn build_scope_grant(
    config: &ServiceConfig,
    platform: ChatHistoryPlatform,
    scope_mode: ChatHistoryScopeMode,
    slack: Option<SlackScopeGrant>,
    discord: Option<DiscordScopeGrant>,
) -> Result<ChatHistoryScopeGrant, BoxError> {
    let issued_at = Utc::now();
    let expires_at = issued_at + Duration::minutes(scope_ttl_minutes());
    Ok(ChatHistoryScopeGrant {
        version: 1,
        platform,
        scope_mode,
        employee_id: config.employee_id.clone(),
        iat: issued_at.timestamp().max(0) as usize,
        exp: expires_at.timestamp().max(0) as usize,
        slack,
        discord,
    })
}

fn encode_scope_grant(
    config: &ServiceConfig,
    claims: &ChatHistoryScopeGrant,
) -> Result<String, BoxError> {
    let secret = chat_history_signing_secret(config)?;
    Ok(jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?)
}

fn decode_scope_grant(
    config: &ServiceConfig,
    token: &str,
) -> Result<ChatHistoryScopeGrant, ChatHistoryRequestError> {
    let secret = chat_history_signing_secret(config).map_err(|err| {
        ChatHistoryRequestError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("missing chat history signing secret: {err}"),
        )
    })?;
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.required_spec_claims.insert("exp".to_string());
    validation.required_spec_claims.insert("iat".to_string());
    let decoded = jsonwebtoken::decode::<ChatHistoryScopeGrant>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|err| {
        ChatHistoryRequestError::new(
            StatusCode::UNAUTHORIZED,
            format!("invalid chat history grant: {err}"),
        )
    })?;
    Ok(decoded.claims)
}

fn extract_bearer_token(value: Option<&str>) -> Result<String, ChatHistoryRequestError> {
    let header = value.ok_or_else(|| {
        ChatHistoryRequestError::new(StatusCode::UNAUTHORIZED, "missing Authorization header")
    })?;
    let token = header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ChatHistoryRequestError::new(
                StatusCode::UNAUTHORIZED,
                "Authorization header must be a Bearer token",
            )
        })?;
    Ok(token.to_string())
}

fn chat_history_signing_secret(config: &ServiceConfig) -> Result<String, BoxError> {
    if let Some(secret) = env_trimmed(CHAT_HISTORY_SCOPE_SIGNING_SECRET_ENV) {
        return Ok(secret);
    }
    if let Some(secret) = env_trimmed("SLACK_SIGNING_SECRET") {
        return Ok(secret);
    }
    if let Some(secret) = config
        .slack_client_secret
        .clone()
        .filter(|value| !value.trim().is_empty())
    {
        return Ok(secret);
    }
    let employee_prefix = config.employee_profile.id.to_uppercase().replace('-', "_");
    if let Some(secret) = env_trimmed(&format!("{employee_prefix}_DISCORD_BOT_TOKEN")) {
        return Ok(secret);
    }
    if let Some(secret) = env_trimmed(&format!("{employee_prefix}_SLACK_BOT_TOKEN")) {
        return Ok(secret);
    }
    if let Some(secret) = config
        .discord_bot_token
        .clone()
        .filter(|value| !value.trim().is_empty())
    {
        return Ok(secret);
    }
    if let Some(secret) = config
        .slack_bot_token
        .clone()
        .filter(|value| !value.trim().is_empty())
    {
        return Ok(secret);
    }
    Err("CHAT_HISTORY_SCOPE_SIGNING_SECRET (or another platform secret fallback) is not set".into())
}

fn chat_history_base_url(config: &ServiceConfig) -> String {
    if let Some(url) = env_trimmed(CHAT_HISTORY_PROXY_BASE_URL_ENV) {
        return url.trim_end_matches('/').to_string();
    }
    if let Some(url) = env_trimmed(DOWHIZ_API_URL_ENV) {
        return url.trim_end_matches('/').to_string();
    }
    if let Some(url) = env_trimmed(SERVICE_URL_ENV) {
        return url.trim_end_matches('/').to_string();
    }
    if chat_history_requires_public_proxy() {
        if let Some(url) = env_trimmed(POSTMARK_INBOUND_HOOK_URL_ENV)
            .and_then(|value| derive_public_service_base_url(&value))
        {
            return url;
        }
        if let Some(url) =
            env_trimmed(FRONTEND_URL_ENV).and_then(|value| derive_public_service_base_url(&value))
        {
            return url;
        }
    }
    let host = normalize_base_host(&config.host);
    format!("http://{host}:{}", config.port)
}

fn chat_history_requires_public_proxy() -> bool {
    env_trimmed(RUN_TASK_EXECUTION_BACKEND_ENV)
        .map(|value| value.eq_ignore_ascii_case(AZURE_ACI_EXECUTION_BACKEND))
        .unwrap_or(false)
}

fn derive_public_service_base_url(candidate: &str) -> Option<String> {
    let mut url = reqwest::Url::parse(candidate).ok()?;
    let raw_path = url.path().trim_end_matches('/');
    let normalized_path = if let Some(prefix) = raw_path.strip_suffix("/postmark/inbound") {
        let prefix = prefix.trim_end_matches('/');
        if prefix.is_empty() {
            "/service".to_string()
        } else {
            format!("{prefix}/service")
        }
    } else if raw_path.is_empty() || raw_path == "/" {
        "/service".to_string()
    } else if raw_path.ends_with("/service") {
        raw_path.to_string()
    } else {
        format!("{}/service", raw_path)
    };
    url.set_path(&normalized_path);
    url.set_query(None);
    url.set_fragment(None);
    Some(url.to_string().trim_end_matches('/').to_string())
}

fn normalize_base_host(host: &str) -> String {
    let trimmed = host.trim();
    let normalized = match trimmed {
        "" | "0.0.0.0" | "::" | "[::]" => "127.0.0.1",
        other => other,
    };
    if normalized.contains(':') && !normalized.starts_with('[') && !normalized.ends_with(']') {
        format!("[{normalized}]")
    } else {
        normalized.to_string()
    }
}

fn scope_ttl_minutes() -> i64 {
    env_trimmed(CHAT_HISTORY_SCOPE_TTL_MINUTES_ENV)
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_SCOPE_TTL_MINUTES)
}

fn env_trimmed(key: &str) -> Option<String> {
    env::var(key).ok().and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn format_unix_ts(seconds: usize) -> Result<String, BoxError> {
    let Some(dt) = DateTime::<Utc>::from_timestamp(seconds as i64, 0) else {
        return Err(format!("invalid unix timestamp: {seconds}").into());
    };
    Ok(dt.to_rfc3339())
}

fn history_client() -> Result<Client, BoxError> {
    Ok(Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?)
}

fn slack_max_history_pages() -> usize {
    env_trimmed(CHAT_HISTORY_SLACK_MAX_HISTORY_PAGES_ENV)
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_SLACK_MAX_HISTORY_PAGES)
}

fn slack_max_thread_pages() -> usize {
    env_trimmed(CHAT_HISTORY_SLACK_MAX_THREAD_PAGES_ENV)
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_SLACK_MAX_THREAD_PAGES)
}

fn discord_max_history_pages_per_channel() -> usize {
    env_trimmed(CHAT_HISTORY_DISCORD_MAX_HISTORY_PAGES_PER_CHANNEL_ENV)
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_DISCORD_MAX_HISTORY_PAGES_PER_CHANNEL)
}

fn discord_max_channels() -> usize {
    env_trimmed(CHAT_HISTORY_DISCORD_MAX_CHANNELS_ENV)
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_DISCORD_MAX_CHANNELS)
}

fn search_slack_channel_history(
    bot_token: &str,
    channel_id: &str,
    query: &str,
    limit: usize,
) -> Result<Vec<ChatHistoryMatch>, BoxError> {
    let api_base =
        env_trimmed("SLACK_API_BASE_URL").unwrap_or_else(|| "https://slack.com/api".to_string());
    let client = history_client()?;
    let query_norm = query.to_ascii_lowercase();
    let mut results = Vec::new();
    let mut seen = HashSet::new();
    let mut cursor: Option<String> = None;

    for _ in 0..slack_max_history_pages() {
        let mut request = client
            .get(format!(
                "{}/conversations.history",
                api_base.trim_end_matches('/')
            ))
            .bearer_auth(bot_token)
            .query(&[("channel", channel_id), ("limit", "200")]);
        if let Some(cursor_value) = cursor.as_deref() {
            request = request.query(&[("cursor", cursor_value)]);
        }
        let response = request.send()?;
        let payload: SlackHistoryResponse = response.json()?;
        if !payload.ok {
            return Err(format!(
                "slack conversations.history returned error {}",
                payload.error.unwrap_or_else(|| "unknown_error".to_string())
            )
            .into());
        }

        for message in payload.messages {
            maybe_push_slack_match(
                &mut results,
                &mut seen,
                channel_id,
                None,
                &message,
                &query_norm,
            );
            if message.reply_count.unwrap_or(0) > 0 {
                let thread_ts = message.thread_ts.as_deref().unwrap_or(&message.ts);
                let replies = fetch_slack_thread_replies(
                    &client,
                    api_base.as_str(),
                    bot_token,
                    channel_id,
                    thread_ts,
                )?;
                for reply in replies {
                    maybe_push_slack_match(
                        &mut results,
                        &mut seen,
                        channel_id,
                        Some(thread_ts),
                        &reply,
                        &query_norm,
                    );
                }
            }
        }

        sort_matches_desc(&mut results);
        if results.len() >= limit {
            break;
        }

        cursor = payload
            .response_metadata
            .and_then(|meta| meta.next_cursor)
            .filter(|value| !value.trim().is_empty());
        if cursor.is_none() {
            break;
        }
    }

    results.truncate(limit);
    Ok(results)
}

fn fetch_slack_thread_replies(
    client: &Client,
    api_base: &str,
    bot_token: &str,
    channel_id: &str,
    thread_ts: &str,
) -> Result<Vec<SlackHistoryMessage>, BoxError> {
    let mut replies = Vec::new();
    let mut cursor: Option<String> = None;
    for _ in 0..slack_max_thread_pages() {
        let mut request = client
            .get(format!(
                "{}/conversations.replies",
                api_base.trim_end_matches('/')
            ))
            .bearer_auth(bot_token)
            .query(&[("channel", channel_id), ("ts", thread_ts), ("limit", "200")]);
        if let Some(cursor_value) = cursor.as_deref() {
            request = request.query(&[("cursor", cursor_value)]);
        }
        let response = request.send()?;
        let payload: SlackHistoryResponse = response.json()?;
        if !payload.ok {
            return Err(format!(
                "slack conversations.replies returned error {}",
                payload.error.unwrap_or_else(|| "unknown_error".to_string())
            )
            .into());
        }
        replies.extend(
            payload
                .messages
                .into_iter()
                .filter(|message| message.ts != thread_ts),
        );
        cursor = payload
            .response_metadata
            .and_then(|meta| meta.next_cursor)
            .filter(|value| !value.trim().is_empty());
        if cursor.is_none() {
            break;
        }
    }
    Ok(replies)
}

fn maybe_push_slack_match(
    results: &mut Vec<ChatHistoryMatch>,
    seen: &mut HashSet<String>,
    channel_id: &str,
    thread_id: Option<&str>,
    message: &SlackHistoryMessage,
    query_norm: &str,
) {
    let text = slack_message_text(message);
    if !message_matches_query(&text, query_norm) {
        return;
    }
    let key = format!("{channel_id}:{}", message.ts);
    if !seen.insert(key) {
        return;
    }
    results.push(ChatHistoryMatch {
        channel_id: channel_id.to_string(),
        channel_name: None,
        message_id: message.ts.clone(),
        thread_id: thread_id
            .map(|value| value.to_string())
            .or_else(|| message.thread_ts.clone()),
        timestamp: slack_ts_to_rfc3339(&message.ts).unwrap_or_else(|| message.ts.clone()),
        author_id: message.user.clone(),
        author_name: message.username.clone(),
        text: truncate_match_text(&text),
        source: if thread_id.is_some() {
            "slack_thread_reply".to_string()
        } else {
            "slack_channel_message".to_string()
        },
    });
}

fn slack_message_text(message: &SlackHistoryMessage) -> String {
    let text = message.text.trim();
    if !text.is_empty() {
        return text.to_string();
    }
    let attachment_names = message
        .files
        .iter()
        .filter_map(|file| file.name.as_deref().or(file.title.as_deref()))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if attachment_names.is_empty() {
        "(empty Slack message)".to_string()
    } else {
        format!("[attachments: {}]", attachment_names.join(", "))
    }
}

fn slack_ts_to_rfc3339(ts: &str) -> Option<String> {
    let seconds_text = ts.split('.').next()?;
    let seconds = seconds_text.parse::<i64>().ok()?;
    DateTime::<Utc>::from_timestamp(seconds, 0).map(|dt| dt.to_rfc3339())
}

fn search_discord_history(
    bot_token: &str,
    scope: &DiscordScopeGrant,
    requested_channel_id: Option<&str>,
    query: &str,
    limit: usize,
) -> Result<(Vec<ChatHistoryMatch>, usize, Vec<String>), BoxError> {
    let api_base = env_trimmed("DISCORD_API_BASE_URL")
        .unwrap_or_else(|| "https://discord.com/api/v10".to_string());
    let client = history_client()?;
    let query_norm = query.to_ascii_lowercase();
    let mut warnings = Vec::new();
    let channels = resolve_discord_search_channels(
        &client,
        api_base.as_str(),
        bot_token,
        scope,
        requested_channel_id,
        &mut warnings,
    )?;
    let mut results = Vec::new();
    let mut seen = HashSet::new();
    let searched_channels = channels.len();

    for channel in channels {
        let channel_matches = search_discord_channel_messages(
            &client,
            api_base.as_str(),
            bot_token,
            &channel.id,
            channel.name.as_deref(),
            &query_norm,
            &mut warnings,
        )?;
        for entry in channel_matches {
            let key = format!("{}:{}", entry.channel_id, entry.message_id);
            if seen.insert(key) {
                results.push(entry);
            }
        }
    }

    sort_matches_desc(&mut results);
    results.truncate(limit);
    Ok((results, searched_channels, warnings))
}

fn resolve_discord_search_channels(
    client: &Client,
    api_base: &str,
    bot_token: &str,
    scope: &DiscordScopeGrant,
    requested_channel_id: Option<&str>,
    warnings: &mut Vec<String>,
) -> Result<Vec<DiscordGuildChannel>, BoxError> {
    if let Some(guild_id) = scope.guild_id {
        let mut channels = fetch_discord_guild_channels(client, api_base, bot_token, guild_id)?;
        let active_threads = fetch_discord_active_threads(client, api_base, bot_token, guild_id)
            .unwrap_or_else(|err| {
                warnings.push(format!(
                    "could not fetch active Discord threads in guild {guild_id}: {err}"
                ));
                Vec::new()
            });
        let mut channel_map = HashMap::new();
        for channel in channels.drain(..) {
            channel_map.insert(channel.id.clone(), channel);
        }
        for thread in active_threads {
            channel_map.insert(thread.id.clone(), thread);
        }
        let mut all_channels = channel_map.into_values().collect::<Vec<_>>();
        all_channels.sort_by(|left, right| left.id.cmp(&right.id));
        if let Some(requested) = requested_channel_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let selected = all_channels
                .into_iter()
                .find(|channel| channel.id == requested)
                .ok_or_else(|| {
                    format!(
                        "requested Discord channel {requested} is outside the allowed guild scope"
                    )
                })?;
            return Ok(vec![selected]);
        }
        if all_channels.len() > discord_max_channels() {
            warnings.push(format!(
                "Discord guild has {} searchable channels; only the first {} channels were scanned.",
                all_channels.len(),
                discord_max_channels()
            ));
            all_channels.truncate(discord_max_channels());
        }
        warnings.push(
            "Archived Discord threads are not scanned yet; active threads and regular text channels are included.".to_string(),
        );
        return Ok(all_channels);
    }

    let requested = requested_channel_id
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if let Some(requested_channel_id) = requested {
        if requested_channel_id != scope.channel_id.to_string() {
            return Err("Discord DM history cannot access another channel".into());
        }
    }
    Ok(vec![DiscordGuildChannel {
        id: scope.channel_id.to_string(),
        name: None,
        kind: 0,
    }])
}

fn fetch_discord_guild_channels(
    client: &Client,
    api_base: &str,
    bot_token: &str,
    guild_id: u64,
) -> Result<Vec<DiscordGuildChannel>, BoxError> {
    let response = client
        .get(format!(
            "{}/guilds/{guild_id}/channels",
            api_base.trim_end_matches('/')
        ))
        .header("Authorization", format!("Bot {bot_token}"))
        .send()?;
    if !response.status().is_success() {
        return Err(format!("discord guild channels api returned {}", response.status()).into());
    }
    let channels: Vec<DiscordGuildChannel> = response.json()?;
    Ok(channels
        .into_iter()
        .filter(|channel| DISCORD_TEXT_CHANNEL_TYPES.contains(&channel.kind))
        .collect())
}

fn fetch_discord_active_threads(
    client: &Client,
    api_base: &str,
    bot_token: &str,
    guild_id: u64,
) -> Result<Vec<DiscordGuildChannel>, BoxError> {
    let response = client
        .get(format!(
            "{}/guilds/{guild_id}/threads/active",
            api_base.trim_end_matches('/')
        ))
        .header("Authorization", format!("Bot {bot_token}"))
        .send()?;
    if !response.status().is_success() {
        return Err(format!("discord active threads api returned {}", response.status()).into());
    }
    let payload: DiscordActiveThreadsResponse = response.json()?;
    Ok(payload
        .threads
        .into_iter()
        .filter(|channel| DISCORD_TEXT_CHANNEL_TYPES.contains(&channel.kind))
        .collect())
}

fn search_discord_channel_messages(
    client: &Client,
    api_base: &str,
    bot_token: &str,
    channel_id: &str,
    channel_name: Option<&str>,
    query_norm: &str,
    warnings: &mut Vec<String>,
) -> Result<Vec<ChatHistoryMatch>, BoxError> {
    let mut results = Vec::new();
    let mut before: Option<String> = None;
    for _ in 0..discord_max_history_pages_per_channel() {
        let mut request = client
            .get(format!(
                "{}/channels/{channel_id}/messages",
                api_base.trim_end_matches('/')
            ))
            .header("Authorization", format!("Bot {bot_token}"))
            .query(&[("limit", "100")]);
        if let Some(before_id) = before.as_deref() {
            request = request.query(&[("before", before_id)]);
        }
        let response = request.send()?;
        if !response.status().is_success() {
            let status = response.status();
            if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::NOT_FOUND
            {
                let label = channel_name
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|value| format!("{value} ({channel_id})"))
                    .unwrap_or_else(|| channel_id.to_string());
                warnings.push(format!(
                    "Skipped Discord channel {label} because the bot could not read its history ({status})."
                ));
                return Ok(Vec::new());
            }
            return Err(format!(
                "discord channel history api for {channel_id} returned {}",
                status
            )
            .into());
        }
        let messages: Vec<DiscordHistoryMessage> = response.json()?;
        if messages.is_empty() {
            break;
        }

        for message in &messages {
            let text = discord_message_text(message);
            if !message_matches_query(&text, query_norm) {
                continue;
            }
            results.push(ChatHistoryMatch {
                channel_id: channel_id.to_string(),
                channel_name: channel_name.map(|value| value.to_string()),
                message_id: message.id.clone(),
                thread_id: None,
                timestamp: message.timestamp.clone(),
                author_id: Some(message.author.id.clone()),
                author_name: Some(
                    message
                        .author
                        .global_name
                        .clone()
                        .unwrap_or_else(|| message.author.username.clone()),
                ),
                text: truncate_match_text(&text),
                source: "discord_message".to_string(),
            });
        }

        before = messages.last().map(|message| message.id.clone());
    }
    Ok(results)
}

fn discord_message_text(message: &DiscordHistoryMessage) -> String {
    let text = message.content.trim();
    if !text.is_empty() {
        return text.to_string();
    }
    let attachment_names = message
        .attachments
        .iter()
        .map(|attachment| attachment.filename.trim())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if attachment_names.is_empty() {
        "(empty Discord message)".to_string()
    } else {
        format!("[attachments: {}]", attachment_names.join(", "))
    }
}

fn message_matches_query(text: &str, query_norm: &str) -> bool {
    text.to_ascii_lowercase().contains(query_norm)
}

fn truncate_match_text(text: &str) -> String {
    let trimmed = text.trim();
    let mut chars = trimmed.chars();
    let collected = chars.by_ref().take(1200).collect::<String>();
    if chars.next().is_some() {
        format!("{collected}...")
    } else {
        collected
    }
}

fn sort_matches_desc(matches: &mut [ChatHistoryMatch]) {
    matches.sort_by(|left, right| right.timestamp.cmp(&left.timestamp));
}

fn resolve_discord_bot_token(config: &ServiceConfig) -> Option<String> {
    let employee_prefix = config.employee_profile.id.to_uppercase().replace('-', "_");
    env_trimmed(&format!("{employee_prefix}_DISCORD_BOT_TOKEN"))
        .or_else(|| config.discord_bot_token.clone())
        .filter(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner())
    }

    struct EnvGuard {
        key: String,
        previous: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &str, value: &str) -> Self {
            let previous = env::var(key).ok();
            env::set_var(key, value);
            Self {
                key: key.to_string(),
                previous,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.take() {
                env::set_var(&self.key, previous);
            } else {
                env::remove_var(&self.key);
            }
        }
    }

    fn test_config() -> ServiceConfig {
        let profile = crate::employee_config::EmployeeProfile {
            id: "little_bear".to_string(),
            display_name: Some("Little Bear".to_string()),
            runner: "codex".to_string(),
            model: None,
            addresses: Vec::new(),
            address_set: HashSet::new(),
            runtime_root: None,
            agents_path: None,
            claude_path: None,
            soul_path: None,
            skills_dir: None,
            discord_enabled: true,
            slack_enabled: true,
            bluebubbles_enabled: false,
        };
        let employee_directory = crate::employee_config::EmployeeDirectory {
            default_employee_id: Some("little_bear".to_string()),
            service_addresses: HashSet::new(),
            employee_by_id: HashMap::from([(profile.id.clone(), profile.clone())]),
            employees: vec![profile.clone()],
        };
        ServiceConfig {
            host: "0.0.0.0".to_string(),
            port: 9001,
            employee_id: "little_bear".to_string(),
            employee_config_path: std::path::PathBuf::from("employee.toml"),
            employee_profile: profile,
            employee_directory,
            workspace_root: std::path::PathBuf::from("/tmp/workspaces"),
            scheduler_state_path: std::path::PathBuf::from("/tmp/scheduler_state"),
            processed_ids_path: std::path::PathBuf::from("/tmp/processed_ids"),
            ingestion_db_url: "postgres://example".to_string(),
            ingestion_poll_interval: std::time::Duration::from_secs(1),
            users_root: std::path::PathBuf::from("/tmp/users"),
            users_db_path: std::path::PathBuf::from("/tmp/users.db"),
            task_index_path: std::path::PathBuf::from("/tmp/task_index.db"),
            codex_model: "gpt-5.4".to_string(),
            codex_disabled: false,
            scheduler_poll_interval: std::time::Duration::from_secs(1),
            scheduler_max_concurrency: 1,
            scheduler_user_max_concurrency: 1,
            inbound_body_max_bytes: 1024,
            skills_source_dir: None,
            slack_bot_token: Some("xoxb-secret".to_string()),
            slack_bot_user_id: Some("U123".to_string()),
            slack_store_path: std::path::PathBuf::from("/tmp/slack.db"),
            slack_client_id: None,
            slack_client_secret: Some("slack-client-secret".to_string()),
            slack_redirect_uri: None,
            discord_bot_token: Some("discord-secret".to_string()),
            discord_bot_user_id: Some(123),
            google_docs_enabled: false,
            bluebubbles_url: None,
            bluebubbles_password: None,
            telegram_bot_token: None,
            whatsapp_access_token: None,
            whatsapp_phone_number_id: None,
            whatsapp_verify_token: None,
        }
    }

    #[test]
    fn scope_grants_round_trip() {
        let _lock = env_lock();
        let _secret = EnvGuard::set(CHAT_HISTORY_SCOPE_SIGNING_SECRET_ENV, "scope-secret");
        let config = test_config();
        let claims = build_scope_grant(
            &config,
            ChatHistoryPlatform::Slack,
            ChatHistoryScopeMode::CurrentConversation,
            Some(SlackScopeGrant {
                team_id: "T123".to_string(),
                channel_id: "C123".to_string(),
                thread_id: Some("1700.1".to_string()),
            }),
            None,
        )
        .expect("build scope");
        let token = encode_scope_grant(&config, &claims).expect("encode");
        let decoded = decode_scope_grant(&config, &token).expect("decode");
        assert_eq!(decoded.platform, ChatHistoryPlatform::Slack);
        assert_eq!(
            decoded.scope_mode,
            ChatHistoryScopeMode::CurrentConversation
        );
        let slack = decoded.slack.expect("slack scope");
        assert_eq!(slack.team_id, "T123");
        assert_eq!(slack.channel_id, "C123");
    }

    #[test]
    fn write_discord_scope_file_uses_service_url_override() {
        let _lock = env_lock();
        let _secret = EnvGuard::set(CHAT_HISTORY_SCOPE_SIGNING_SECRET_ENV, "scope-secret");
        let _service_url = EnvGuard::set(SERVICE_URL_ENV, "https://worker.example.com");
        let workspace = tempfile::tempdir().expect("tempdir");
        let config = test_config();
        let message = crate::channel::InboundMessage {
            channel: crate::channel::Channel::Discord,
            sender: "123".to_string(),
            sender_name: Some("User".to_string()),
            recipient: "456".to_string(),
            subject: None,
            text_body: Some("hello".to_string()),
            html_body: None,
            thread_id: "789".to_string(),
            message_id: Some("789".to_string()),
            attachments: Vec::new(),
            reply_to: vec!["123".to_string()],
            raw_payload: Vec::new(),
            metadata: crate::channel::ChannelMetadata {
                discord_guild_id: Some(42),
                discord_channel_id: Some(84),
                ..crate::channel::ChannelMetadata::default()
            },
        };
        write_discord_chat_history_scope_file(&config, workspace.path(), &message)
            .expect("write scope");
        let payload = std::fs::read_to_string(workspace.path().join(CHAT_HISTORY_SCOPE_FILE_NAME))
            .expect("scope file");
        assert!(payload.contains("https://worker.example.com/internal/chat-history/search"));
        assert!(payload.contains("\"guild_id\": 42"));
    }

    #[test]
    fn write_discord_scope_file_uses_public_service_base_for_azure_aci() {
        let _lock = env_lock();
        let _secret = EnvGuard::set(CHAT_HISTORY_SCOPE_SIGNING_SECRET_ENV, "scope-secret");
        let _run_task_backend =
            EnvGuard::set(RUN_TASK_EXECUTION_BACKEND_ENV, AZURE_ACI_EXECUTION_BACKEND);
        let _proxy = EnvGuard::set(CHAT_HISTORY_PROXY_BASE_URL_ENV, "");
        let _dowhiz_api = EnvGuard::set(DOWHIZ_API_URL_ENV, "");
        let _service_url = EnvGuard::set(SERVICE_URL_ENV, "");
        let _frontend_url = EnvGuard::set(FRONTEND_URL_ENV, "");
        let _postmark_hook = EnvGuard::set(
            POSTMARK_INBOUND_HOOK_URL_ENV,
            "https://api.staging.dowhiz.com/postmark/inbound",
        );
        let workspace = tempfile::tempdir().expect("tempdir");
        let config = test_config();
        let message = crate::channel::InboundMessage {
            channel: crate::channel::Channel::Discord,
            sender: "123".to_string(),
            sender_name: Some("User".to_string()),
            recipient: "456".to_string(),
            subject: None,
            text_body: Some("hello".to_string()),
            html_body: None,
            thread_id: "789".to_string(),
            message_id: Some("789".to_string()),
            attachments: Vec::new(),
            reply_to: vec!["123".to_string()],
            raw_payload: Vec::new(),
            metadata: crate::channel::ChannelMetadata {
                discord_guild_id: Some(42),
                discord_channel_id: Some(84),
                ..crate::channel::ChannelMetadata::default()
            },
        };
        write_discord_chat_history_scope_file(&config, workspace.path(), &message)
            .expect("write scope");
        let payload = std::fs::read_to_string(workspace.path().join(CHAT_HISTORY_SCOPE_FILE_NAME))
            .expect("scope file");
        assert!(
            payload.contains("https://api.staging.dowhiz.com/service/internal/chat-history/search")
        );
    }

    #[test]
    fn write_discord_scope_file_keeps_local_base_without_remote_execution_backend() {
        let _lock = env_lock();
        let _secret = EnvGuard::set(CHAT_HISTORY_SCOPE_SIGNING_SECRET_ENV, "scope-secret");
        let _run_task_backend = EnvGuard::set(RUN_TASK_EXECUTION_BACKEND_ENV, "");
        let _proxy = EnvGuard::set(CHAT_HISTORY_PROXY_BASE_URL_ENV, "");
        let _dowhiz_api = EnvGuard::set(DOWHIZ_API_URL_ENV, "");
        let _service_url = EnvGuard::set(SERVICE_URL_ENV, "");
        let _frontend_url = EnvGuard::set(FRONTEND_URL_ENV, "https://api.staging.dowhiz.com/");
        let _postmark_hook = EnvGuard::set(
            POSTMARK_INBOUND_HOOK_URL_ENV,
            "https://api.staging.dowhiz.com/postmark/inbound",
        );
        let workspace = tempfile::tempdir().expect("tempdir");
        let config = test_config();
        let message = crate::channel::InboundMessage {
            channel: crate::channel::Channel::Discord,
            sender: "123".to_string(),
            sender_name: Some("User".to_string()),
            recipient: "456".to_string(),
            subject: None,
            text_body: Some("hello".to_string()),
            html_body: None,
            thread_id: "789".to_string(),
            message_id: Some("789".to_string()),
            attachments: Vec::new(),
            reply_to: vec!["123".to_string()],
            raw_payload: Vec::new(),
            metadata: crate::channel::ChannelMetadata {
                discord_guild_id: Some(42),
                discord_channel_id: Some(84),
                ..crate::channel::ChannelMetadata::default()
            },
        };
        write_discord_chat_history_scope_file(&config, workspace.path(), &message)
            .expect("write scope");
        let payload = std::fs::read_to_string(workspace.path().join(CHAT_HISTORY_SCOPE_FILE_NAME))
            .expect("scope file");
        assert!(payload.contains("http://127.0.0.1:9001/internal/chat-history/search"));
    }

    #[test]
    fn slack_history_search_reads_thread_replies() {
        let _lock = env_lock();
        let mut server = mockito::Server::new();
        let _api = EnvGuard::set("SLACK_API_BASE_URL", &server.url());

        let history_mock = server
            .mock("GET", "/conversations.history")
            .match_header("authorization", "Bearer xoxb-secret")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("channel".into(), "C123".into()),
                mockito::Matcher::UrlEncoded("limit".into(), "200".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{"ok":true,"messages":[{"ts":"1700000001.000100","text":"Root message","reply_count":1},{"ts":"1700000000.000100","text":"Older note"}],"response_metadata":{"next_cursor":""}}"#,
            )
            .create();
        let replies_mock = server
            .mock("GET", "/conversations.replies")
            .match_header("authorization", "Bearer xoxb-secret")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("channel".into(), "C123".into()),
                mockito::Matcher::UrlEncoded("ts".into(), "1700000001.000100".into()),
                mockito::Matcher::UrlEncoded("limit".into(), "200".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{"ok":true,"messages":[{"ts":"1700000001.000100","text":"Root message"},{"ts":"1700000001.000200","text":"Needle in a thread"}],"response_metadata":{"next_cursor":""}}"#,
            )
            .create();

        let results =
            search_slack_channel_history("xoxb-secret", "C123", "needle", 10).expect("search");
        history_mock.assert();
        replies_mock.assert();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].thread_id.as_deref(), Some("1700000001.000100"));
        assert!(results[0].text.contains("Needle"));
    }

    #[test]
    fn discord_dm_scope_rejects_other_channel() {
        let scope = DiscordScopeGrant {
            guild_id: None,
            channel_id: 999,
            thread_id: Some("999".to_string()),
        };
        let mut warnings = Vec::new();
        let error = resolve_discord_search_channels(
            &history_client().expect("client"),
            "https://discord.com/api/v10",
            "discord-secret",
            &scope,
            Some("123"),
            &mut warnings,
        )
        .expect_err("expected scope violation");
        assert!(error.to_string().contains("cannot access another channel"));
    }

    #[test]
    fn discord_guild_search_skips_forbidden_channels() {
        let _lock = env_lock();
        let mut server = mockito::Server::new();
        let _api = EnvGuard::set("DISCORD_API_BASE_URL", &server.url());

        let guild_channels = server
            .mock("GET", "/guilds/42/channels")
            .match_header("authorization", "Bot discord-secret")
            .with_status(200)
            .with_body(
                r#"[{"id":"111","name":"restricted","type":0},{"id":"222","name":"general","type":0}]"#,
            )
            .create();
        let active_threads = server
            .mock("GET", "/guilds/42/threads/active")
            .match_header("authorization", "Bot discord-secret")
            .with_status(200)
            .with_body(r#"{"threads":[]}"#)
            .create();
        let restricted = server
            .mock("GET", "/channels/111/messages")
            .match_header("authorization", "Bot discord-secret")
            .match_query(mockito::Matcher::UrlEncoded("limit".into(), "100".into()))
            .with_status(403)
            .create();
        let general_page_one = server
            .mock("GET", "/channels/222/messages")
            .match_header("authorization", "Bot discord-secret")
            .match_query(mockito::Matcher::UrlEncoded("limit".into(), "100".into()))
            .with_status(200)
            .with_body(
                r#"[{"id":"200","content":"Needle from general","timestamp":"2026-03-23T00:00:00Z","author":{"id":"user-1","username":"alice","global_name":"Alice"},"attachments":[]}]"#,
            )
            .create();
        let general_page_two = server
            .mock("GET", "/channels/222/messages")
            .match_header("authorization", "Bot discord-secret")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("limit".into(), "100".into()),
                mockito::Matcher::UrlEncoded("before".into(), "200".into()),
            ]))
            .with_status(200)
            .with_body("[]")
            .create();

        let scope = DiscordScopeGrant {
            guild_id: Some(42),
            channel_id: 222,
            thread_id: Some("thread-1".to_string()),
        };
        let (results, searched_channels, warnings) =
            search_discord_history("discord-secret", &scope, None, "needle", 10)
                .expect("search should succeed");

        guild_channels.assert();
        active_threads.assert();
        restricted.assert();
        general_page_one.assert();
        general_page_two.assert();
        assert_eq!(searched_channels, 2);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].channel_id, "222");
        assert!(warnings.iter().any(|warning| {
            warning.contains("Skipped Discord channel") && warning.contains("111")
        }));
    }
}
