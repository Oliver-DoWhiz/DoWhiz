use std::env;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, warn};
use uuid::Uuid;

use crate::account_store::{AccountStore, AnalyticsEventInsert, ChannelInstallOnboardingState};
use crate::adapters::discord::DiscordOutboundAdapter;
use crate::adapters::slack::SlackOutboundAdapter;
use crate::channel::{Channel, ChannelMetadata, OutboundAdapter, OutboundMessage};
use crate::slack_store::SlackInstallation;

const DEFAULT_REINSTALL_COOLDOWN_HOURS: u64 = 24 * 7;
const SLACK_SAFE_PUBLIC_CHANNEL_NAMES: &[&str] = &[
    "general",
    "team",
    "workspace",
    "chat",
    "lounge",
    "introductions",
];
const DISCORD_SAFE_PUBLIC_CHANNEL_NAMES: &[&str] = &[
    "general",
    "introductions",
    "start-here",
    "welcome",
    "team",
    "workspace",
    "chat",
    "lounge",
];
const DISCORD_TEXT_CHANNEL_TYPES: &[u8] = &[0, 5, 10, 11, 12, 15];

#[derive(Debug, Clone)]
pub struct InstallOnboardingConfig {
    pub(crate) slack_enabled: bool,
    pub(crate) discord_enabled: bool,
    pub(crate) reinstall_cooldown: Duration,
}

impl InstallOnboardingConfig {
    pub fn from_env() -> Self {
        Self {
            slack_enabled: env_flag("OLIVER_SLACK_INSTALL_ONBOARDING_ENABLED", false),
            discord_enabled: env_flag("OLIVER_DISCORD_INSTALL_ONBOARDING_ENABLED", false),
            reinstall_cooldown: Duration::from_secs(
                env::var("OLIVER_INSTALL_ONBOARDING_COOLDOWN_HOURS")
                    .ok()
                    .and_then(|value| value.parse::<u64>().ok())
                    .filter(|value| *value > 0)
                    .unwrap_or(DEFAULT_REINSTALL_COOLDOWN_HOURS)
                    * 60
                    * 60,
            ),
        }
    }

    fn enabled_for(&self, platform: InstallPlatform) -> bool {
        match platform {
            InstallPlatform::Slack => self.slack_enabled,
            InstallPlatform::Discord => self.discord_enabled,
        }
    }
}

fn env_flag(key: &str, default: bool) -> bool {
    match env::var(key) {
        Ok(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "y" | "on"
        ),
        Err(_) => default,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum InstallPlatform {
    Slack,
    Discord,
}

impl InstallPlatform {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            InstallPlatform::Slack => "slack",
            InstallPlatform::Discord => "discord",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InstallOnboardingTrigger {
    InstallSuccess,
    ManualResend,
}

impl InstallOnboardingTrigger {
    fn as_str(self) -> &'static str {
        match self {
            InstallOnboardingTrigger::InstallSuccess => "install_success",
            InstallOnboardingTrigger::ManualResend => "manual_resend",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DeliveryStatus {
    NotAttempted,
    Sent,
    Failed,
    Skipped,
}

impl DeliveryStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            DeliveryStatus::NotAttempted => "not_attempted",
            DeliveryStatus::Sent => "sent",
            DeliveryStatus::Failed => "failed",
            DeliveryStatus::Skipped => "skipped",
        }
    }

    fn from_optional(value: Option<&str>) -> Self {
        match value.unwrap_or_default() {
            "sent" => DeliveryStatus::Sent,
            "failed" => DeliveryStatus::Failed,
            "skipped" => DeliveryStatus::Skipped,
            _ => DeliveryStatus::NotAttempted,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct InstallOnboardingStateRecord {
    pub(crate) account_id: Uuid,
    pub(crate) platform: InstallPlatform,
    pub(crate) workspace_id: String,
    pub(crate) workspace_name: Option<String>,
    pub(crate) installer_identifier: Option<String>,
    pub(crate) installer_identifier_source: Option<String>,
    pub(crate) last_public_channel_id: Option<String>,
    pub(crate) last_public_channel_name: Option<String>,
    pub(crate) last_dm_recipient_identifier: Option<String>,
    pub(crate) last_dm_recipient_source: Option<String>,
    pub(crate) last_event_key: Option<String>,
    pub(crate) last_public_status: DeliveryStatus,
    pub(crate) last_public_error: Option<String>,
    pub(crate) last_dm_status: DeliveryStatus,
    pub(crate) last_dm_error: Option<String>,
    pub(crate) last_skip_reason: Option<String>,
    pub(crate) last_attempted_at: Option<DateTime<Utc>>,
    pub(crate) last_succeeded_at: Option<DateTime<Utc>>,
    pub(crate) last_manual_resend_at: Option<DateTime<Utc>>,
}

impl InstallOnboardingStateRecord {
    fn new(account_id: Uuid, platform: InstallPlatform, workspace_id: impl Into<String>) -> Self {
        Self {
            account_id,
            platform,
            workspace_id: workspace_id.into(),
            workspace_name: None,
            installer_identifier: None,
            installer_identifier_source: None,
            last_public_channel_id: None,
            last_public_channel_name: None,
            last_dm_recipient_identifier: None,
            last_dm_recipient_source: None,
            last_event_key: None,
            last_public_status: DeliveryStatus::NotAttempted,
            last_public_error: None,
            last_dm_status: DeliveryStatus::NotAttempted,
            last_dm_error: None,
            last_skip_reason: None,
            last_attempted_at: None,
            last_succeeded_at: None,
            last_manual_resend_at: None,
        }
    }

    fn from_account_state(state: ChannelInstallOnboardingState) -> Self {
        Self {
            account_id: state.account_id,
            platform: match state.platform.as_str() {
                "discord" => InstallPlatform::Discord,
                _ => InstallPlatform::Slack,
            },
            workspace_id: state.workspace_id,
            workspace_name: state.workspace_name,
            installer_identifier: state.installer_identifier,
            installer_identifier_source: state.installer_identifier_source,
            last_public_channel_id: state.public_channel_id,
            last_public_channel_name: state.public_channel_name,
            last_dm_recipient_identifier: state.dm_recipient_identifier,
            last_dm_recipient_source: state.dm_recipient_source,
            last_event_key: state.last_event_key,
            last_public_status: DeliveryStatus::from_optional(state.last_public_status.as_deref()),
            last_public_error: state.last_public_error,
            last_dm_status: DeliveryStatus::from_optional(state.last_dm_status.as_deref()),
            last_dm_error: state.last_dm_error,
            last_skip_reason: state.last_skip_reason,
            last_attempted_at: state.last_attempted_at,
            last_succeeded_at: state.last_succeeded_at,
            last_manual_resend_at: state.last_manual_resend_at,
        }
    }

    fn into_account_state(self) -> ChannelInstallOnboardingState {
        ChannelInstallOnboardingState {
            account_id: self.account_id,
            platform: self.platform.as_str().to_string(),
            workspace_id: self.workspace_id,
            workspace_name: self.workspace_name,
            installer_identifier: self.installer_identifier,
            installer_identifier_source: self.installer_identifier_source,
            public_channel_id: self.last_public_channel_id,
            public_channel_name: self.last_public_channel_name,
            dm_recipient_identifier: self.last_dm_recipient_identifier,
            dm_recipient_source: self.last_dm_recipient_source,
            last_event_key: self.last_event_key,
            last_public_status: Some(self.last_public_status.as_str().to_string()),
            last_public_error: self.last_public_error,
            last_dm_status: Some(self.last_dm_status.as_str().to_string()),
            last_dm_error: self.last_dm_error,
            last_skip_reason: self.last_skip_reason,
            last_attempted_at: self.last_attempted_at,
            last_succeeded_at: self.last_succeeded_at,
            last_manual_resend_at: self.last_manual_resend_at,
        }
    }
}

pub(crate) trait InstallOnboardingStateStore: Send + Sync {
    fn load(
        &self,
        account_id: Uuid,
        platform: InstallPlatform,
        workspace_id: &str,
    ) -> Result<Option<InstallOnboardingStateRecord>, String>;

    fn save(&self, state: &InstallOnboardingStateRecord) -> Result<(), String>;
}

#[derive(Clone)]
pub(crate) struct AccountStoreInstallOnboardingStateStore {
    account_store: Arc<AccountStore>,
}

impl AccountStoreInstallOnboardingStateStore {
    pub(crate) fn new(account_store: Arc<AccountStore>) -> Self {
        Self { account_store }
    }
}

impl InstallOnboardingStateStore for AccountStoreInstallOnboardingStateStore {
    fn load(
        &self,
        account_id: Uuid,
        platform: InstallPlatform,
        workspace_id: &str,
    ) -> Result<Option<InstallOnboardingStateRecord>, String> {
        self.account_store
            .get_channel_install_onboarding_state(account_id, platform.as_str(), workspace_id)
            .map(|state| state.map(InstallOnboardingStateRecord::from_account_state))
            .map_err(|err| err.to_string())
    }

    fn save(&self, state: &InstallOnboardingStateRecord) -> Result<(), String> {
        self.account_store
            .upsert_channel_install_onboarding_state(&state.clone().into_account_state())
            .map(|_| ())
            .map_err(|err| err.to_string())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct InstallOnboardingRequest {
    pub(crate) account_id: Uuid,
    pub(crate) auth_user_id: Option<Uuid>,
    pub(crate) platform: InstallPlatform,
    pub(crate) workspace_id: String,
    pub(crate) workspace_name: Option<String>,
    pub(crate) installer_identifier: Option<String>,
    pub(crate) installer_identifier_source: Option<String>,
    pub(crate) linked_owner_identifier: Option<String>,
    pub(crate) linked_owner_identifier_source: Option<String>,
    pub(crate) public_channel_hint: Option<String>,
    pub(crate) event_key: Option<String>,
    pub(crate) route_path: Option<String>,
    pub(crate) trigger: InstallOnboardingTrigger,
    pub(crate) force: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct OnboardingChannelTarget {
    pub(crate) id: String,
    pub(crate) name: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct SendAttemptOutcome {
    pub(crate) success: bool,
    pub(crate) message_id: Option<String>,
    pub(crate) error: Option<String>,
}

pub(crate) trait InstallOnboardingPlatformClient: Send + Sync {
    fn platform(&self) -> InstallPlatform;
    fn workspace_name(&self) -> Option<&str>;
    fn mention_user(&self, user_id: &str) -> String;
    fn public_targets(
        &self,
        hint_channel_id: Option<&str>,
    ) -> Result<Vec<OnboardingChannelTarget>, String>;
    fn send_public_message(
        &self,
        target: &OnboardingChannelTarget,
        text: &str,
    ) -> Result<SendAttemptOutcome, String>;
    fn send_direct_message(
        &self,
        recipient_user_id: &str,
        text: &str,
    ) -> Result<SendAttemptOutcome, String>;
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct InstallOnboardingRunResult {
    pub(crate) skipped: bool,
    pub(crate) skip_reason: Option<String>,
    pub(crate) public_status: DeliveryStatus,
    pub(crate) dm_status: DeliveryStatus,
}

pub(crate) fn run_install_onboarding(
    config: &InstallOnboardingConfig,
    state_store: &dyn InstallOnboardingStateStore,
    analytics_store: &Arc<AccountStore>,
    client: &dyn InstallOnboardingPlatformClient,
    request: InstallOnboardingRequest,
) -> Result<InstallOnboardingRunResult, String> {
    let mut state = state_store
        .load(request.account_id, request.platform, &request.workspace_id)?
        .unwrap_or_else(|| {
            InstallOnboardingStateRecord::new(
                request.account_id,
                request.platform,
                request.workspace_id.clone(),
            )
        });
    state.workspace_name = request
        .workspace_name
        .clone()
        .or_else(|| state.workspace_name.clone());

    if request.trigger == InstallOnboardingTrigger::ManualResend {
        state.last_manual_resend_at = Some(Utc::now());
        track_onboarding_event(
            analytics_store,
            "install_onboarding_manual_resend_requested",
            &request,
            json!({
                "platform": request.platform.as_str(),
                "workspace_id": request.workspace_id,
            }),
        );
    }

    if !request.force && !config.enabled_for(request.platform) {
        state.last_skip_reason = Some("feature_disabled".to_string());
        state_store.save(&state)?;
        track_onboarding_event(
            analytics_store,
            "install_onboarding_skipped",
            &request,
            json!({
                "platform": request.platform.as_str(),
                "workspace_id": request.workspace_id,
                "reason": "feature_disabled",
            }),
        );
        return Ok(InstallOnboardingRunResult {
            skipped: true,
            skip_reason: Some("feature_disabled".to_string()),
            public_status: state.last_public_status,
            dm_status: state.last_dm_status,
        });
    }

    if !request.force {
        if let (Some(previous), Some(current)) = (
            state.last_event_key.as_deref(),
            request.event_key.as_deref(),
        ) {
            if previous == current && state.last_attempted_at.is_some() {
                state.last_skip_reason = Some("duplicate_event".to_string());
                state_store.save(&state)?;
                track_onboarding_event(
                    analytics_store,
                    "install_onboarding_skipped",
                    &request,
                    json!({
                        "platform": request.platform.as_str(),
                        "workspace_id": request.workspace_id,
                        "reason": "duplicate_event",
                    }),
                );
                return Ok(InstallOnboardingRunResult {
                    skipped: true,
                    skip_reason: Some("duplicate_event".to_string()),
                    public_status: state.last_public_status,
                    dm_status: state.last_dm_status,
                });
            }
        }
        if let Some(last_attempted_at) = state.last_attempted_at {
            let elapsed = Utc::now() - last_attempted_at;
            let cooldown = chrono::Duration::from_std(config.reinstall_cooldown)
                .map_err(|err| format!("invalid onboarding cooldown: {err}"))?;
            if elapsed < cooldown {
                state.last_skip_reason = Some("cooldown_active".to_string());
                state_store.save(&state)?;
                track_onboarding_event(
                    analytics_store,
                    "install_onboarding_skipped",
                    &request,
                    json!({
                        "platform": request.platform.as_str(),
                        "workspace_id": request.workspace_id,
                        "reason": "cooldown_active",
                        "cooldown_hours": config.reinstall_cooldown.as_secs() / 3600,
                    }),
                );
                return Ok(InstallOnboardingRunResult {
                    skipped: true,
                    skip_reason: Some("cooldown_active".to_string()),
                    public_status: state.last_public_status,
                    dm_status: state.last_dm_status,
                });
            }
        }
    }

    let public_message = build_public_message(client, &request);
    let direct_message = build_direct_message(client, &request);
    let direct_recipient = request
        .installer_identifier
        .clone()
        .map(|id| {
            (
                id,
                request
                    .installer_identifier_source
                    .clone()
                    .unwrap_or_else(|| "installer".to_string()),
            )
        })
        .or_else(|| {
            request.linked_owner_identifier.clone().map(|id| {
                (
                    id,
                    request
                        .linked_owner_identifier_source
                        .clone()
                        .unwrap_or_else(|| "linked_owner".to_string()),
                )
            })
        });

    let public_targets = client
        .public_targets(
            request
                .public_channel_hint
                .as_deref()
                .or(state.last_public_channel_id.as_deref()),
        )
        .unwrap_or_else(|err| {
            warn!(
                "install onboarding {} channel discovery failed for {}: {}",
                request.platform.as_str(),
                request.workspace_id,
                err
            );
            Vec::new()
        });

    let mut public_status = DeliveryStatus::Skipped;
    let mut public_error: Option<String> = None;
    let mut public_target_used: Option<OnboardingChannelTarget> = None;
    if public_targets.is_empty() {
        public_error = Some("no_safe_public_target".to_string());
    } else {
        for target in public_targets {
            match client.send_public_message(&target, &public_message) {
                Ok(outcome) if outcome.success => {
                    public_status = DeliveryStatus::Sent;
                    public_target_used = Some(target);
                    public_error = None;
                    track_onboarding_event(
                        analytics_store,
                        "install_onboarding_public_sent",
                        &request,
                        json!({
                            "platform": request.platform.as_str(),
                            "workspace_id": request.workspace_id,
                            "channel_id": public_target_used.as_ref().map(|value| value.id.clone()),
                            "channel_name": public_target_used.as_ref().and_then(|value| value.name.clone()),
                            "message_id": outcome.message_id,
                        }),
                    );
                    break;
                }
                Ok(outcome) => {
                    public_status = DeliveryStatus::Failed;
                    public_error = outcome
                        .error
                        .or_else(|| Some("unknown_public_error".to_string()));
                }
                Err(err) => {
                    public_status = DeliveryStatus::Failed;
                    public_error = Some(err);
                }
            }
        }
        if public_status == DeliveryStatus::Failed {
            track_onboarding_event(
                analytics_store,
                "install_onboarding_public_failed",
                &request,
                json!({
                    "platform": request.platform.as_str(),
                    "workspace_id": request.workspace_id,
                    "error": public_error,
                }),
            );
        }
    }

    let mut dm_status = DeliveryStatus::Skipped;
    let mut dm_error: Option<String> = None;
    let mut dm_recipient_used: Option<(String, String)> = None;
    if let Some((recipient_identifier, recipient_source)) = direct_recipient.clone() {
        match client.send_direct_message(&recipient_identifier, &direct_message) {
            Ok(outcome) if outcome.success => {
                dm_status = DeliveryStatus::Sent;
                dm_recipient_used = Some((recipient_identifier.clone(), recipient_source.clone()));
                track_onboarding_event(
                    analytics_store,
                    "install_onboarding_dm_sent",
                    &request,
                    json!({
                        "platform": request.platform.as_str(),
                        "workspace_id": request.workspace_id,
                        "recipient_identifier": recipient_identifier,
                        "recipient_source": recipient_source,
                        "message_id": outcome.message_id,
                    }),
                );
            }
            Ok(outcome) => {
                dm_status = DeliveryStatus::Failed;
                dm_error = outcome
                    .error
                    .or_else(|| Some("unknown_dm_error".to_string()));
                track_onboarding_event(
                    analytics_store,
                    "install_onboarding_dm_failed",
                    &request,
                    json!({
                        "platform": request.platform.as_str(),
                        "workspace_id": request.workspace_id,
                        "recipient_identifier": recipient_identifier,
                        "recipient_source": recipient_source,
                        "error": dm_error,
                    }),
                );
            }
            Err(err) => {
                dm_status = DeliveryStatus::Failed;
                dm_error = Some(err.clone());
                track_onboarding_event(
                    analytics_store,
                    "install_onboarding_dm_failed",
                    &request,
                    json!({
                        "platform": request.platform.as_str(),
                        "workspace_id": request.workspace_id,
                        "recipient_identifier": recipient_identifier,
                        "recipient_source": recipient_source,
                        "error": err,
                    }),
                );
            }
        }
    } else {
        dm_error = Some("no_direct_recipient".to_string());
    }

    state.installer_identifier = request
        .installer_identifier
        .clone()
        .or_else(|| state.installer_identifier.clone());
    state.installer_identifier_source = request
        .installer_identifier_source
        .clone()
        .or_else(|| state.installer_identifier_source.clone());
    state.last_public_status = public_status;
    state.last_public_error = public_error.clone();
    state.last_dm_status = dm_status;
    state.last_dm_error = dm_error.clone();
    state.last_event_key = request.event_key.clone();
    state.last_attempted_at = Some(Utc::now());
    state.last_skip_reason = match (public_status, dm_status) {
        (DeliveryStatus::Skipped, DeliveryStatus::Skipped) => Some("no_delivery_path".to_string()),
        _ => None,
    };
    if public_status == DeliveryStatus::Sent || dm_status == DeliveryStatus::Sent {
        state.last_succeeded_at = state.last_attempted_at;
    }
    if let Some(target) = public_target_used {
        state.last_public_channel_id = Some(target.id);
        state.last_public_channel_name = target.name;
    }
    if let Some((identifier, source)) = dm_recipient_used {
        state.last_dm_recipient_identifier = Some(identifier);
        state.last_dm_recipient_source = Some(source);
    }
    state_store.save(&state)?;

    if state.last_skip_reason.is_some() {
        track_onboarding_event(
            analytics_store,
            "install_onboarding_skipped",
            &request,
            json!({
                "platform": request.platform.as_str(),
                "workspace_id": request.workspace_id,
                "reason": state.last_skip_reason,
            }),
        );
    }

    info!(
        "install onboarding processed platform={} workspace={} public_status={} dm_status={}",
        request.platform.as_str(),
        request.workspace_id,
        public_status.as_str(),
        dm_status.as_str()
    );

    Ok(InstallOnboardingRunResult {
        skipped: state.last_skip_reason.is_some(),
        skip_reason: state.last_skip_reason,
        public_status,
        dm_status,
    })
}

fn build_public_message(
    client: &dyn InstallOnboardingPlatformClient,
    request: &InstallOnboardingRequest,
) -> String {
    let greeting = request
        .installer_identifier
        .as_deref()
        .map(|identifier| format!("Hey {} - ", client.mention_user(identifier)))
        .unwrap_or_else(|| "Hi team - ".to_string());

    format!(
        "{greeting}I'm Oliver, an AI coworker that helps turn conversation into follow-through.\n\n\
Here are a few things I can help with:\n\
- Summarize a thread into decisions, owners, and next steps\n\
- Draft follow-up messages or docs from what the team is discussing\n\
- Coordinate work across chat, email, GitHub, and Google Docs\n\n\
Try {cta}",
        cta = match client.platform() {
            InstallPlatform::Slack => "`@Oliver` in this channel or DM me directly.",
            InstallPlatform::Discord => "`@Oliver` in this channel or DM me directly.",
        }
    )
}

fn build_direct_message(
    client: &dyn InstallOnboardingPlatformClient,
    request: &InstallOnboardingRequest,
) -> String {
    let workspace_label = request
        .workspace_name
        .clone()
        .or_else(|| client.workspace_name().map(ToOwned::to_owned))
        .unwrap_or_else(|| match client.platform() {
            InstallPlatform::Slack => "your Slack workspace".to_string(),
            InstallPlatform::Discord => "your Discord server".to_string(),
        });

    format!(
        "Oliver is live in {workspace_label}.\n\n\
Best first ask:\n\
- \"Summarize the latest thread into owners and next steps.\"\n\n\
Starter prompts:\n\
1. \"Turn this discussion into action items with owners and due dates.\"\n\
2. \"Draft a follow-up message I can send after this conversation.\"\n\
3. \"Organize what the team decided and what is still blocked.\"\n\n\
Reply here anytime if you want me to help the team get first value quickly."
    )
}

fn track_onboarding_event(
    store: &Arc<AccountStore>,
    event_name: &str,
    request: &InstallOnboardingRequest,
    properties: serde_json::Value,
) {
    let environment = env::var("DEPLOY_TARGET")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "production".to_string());
    let event = AnalyticsEventInsert {
        event_name: event_name.to_string(),
        source: "server".to_string(),
        event_timestamp: Utc::now(),
        account_id: Some(request.account_id),
        auth_user_id: request.auth_user_id,
        anonymous_id: None,
        session_id: None,
        workspace_id: Some(format!(
            "{}:{}",
            request.platform.as_str(),
            request.workspace_id
        )),
        org_id: None,
        plan_type: None,
        environment: Some(environment),
        app_version: None,
        page_path: None,
        route_path: request.route_path.clone(),
        referrer: None,
        utm_source: None,
        utm_medium: None,
        utm_campaign: None,
        utm_term: None,
        utm_content: None,
        device_type: None,
        browser: None,
        os: None,
        event_key: request.event_key.clone().map(|value| {
            format!(
                "{}:{}:{}:{}",
                event_name,
                request.platform.as_str(),
                request.workspace_id,
                value
            )
        }),
        properties: merge_json(
            json!({
                "platform": request.platform.as_str(),
                "workspace_id": request.workspace_id,
                "trigger": request.trigger.as_str(),
                "force": request.force,
            }),
            properties,
        ),
    };
    store.record_analytics_event_detached(event, "install_onboarding");
}

fn merge_json(base: serde_json::Value, extra: serde_json::Value) -> serde_json::Value {
    let mut merged = base.as_object().cloned().unwrap_or_default();
    for (key, value) in extra.as_object().cloned().unwrap_or_default() {
        merged.insert(key, value);
    }
    serde_json::Value::Object(merged)
}

pub(crate) struct SlackInstallOnboardingClient {
    installation: SlackInstallation,
    api_base: String,
}

impl SlackInstallOnboardingClient {
    pub(crate) fn new(installation: SlackInstallation) -> Self {
        Self {
            installation,
            api_base: env::var("SLACK_API_BASE_URL")
                .unwrap_or_else(|_| "https://slack.com/api".to_string()),
        }
    }
}

impl InstallOnboardingPlatformClient for SlackInstallOnboardingClient {
    fn platform(&self) -> InstallPlatform {
        InstallPlatform::Slack
    }

    fn workspace_name(&self) -> Option<&str> {
        self.installation.team_name.as_deref()
    }

    fn mention_user(&self, user_id: &str) -> String {
        format!("<@{}>", user_id)
    }

    fn public_targets(
        &self,
        hint_channel_id: Option<&str>,
    ) -> Result<Vec<OnboardingChannelTarget>, String> {
        let channels = list_slack_public_channels(
            &self.api_base,
            &self.installation.bot_token,
            &self.installation.team_id,
        )?;
        Ok(select_slack_public_targets(&channels, hint_channel_id))
    }

    fn send_public_message(
        &self,
        target: &OnboardingChannelTarget,
        text: &str,
    ) -> Result<SendAttemptOutcome, String> {
        let adapter = SlackOutboundAdapter::new(self.installation.bot_token.clone());
        let result = adapter
            .send(&OutboundMessage {
                channel: Channel::Slack,
                from: None,
                to: vec![target.id.clone()],
                cc: vec![],
                bcc: vec![],
                subject: String::new(),
                text_body: text.to_string(),
                html_body: String::new(),
                html_path: None,
                attachments_dir: None,
                thread_id: None,
                metadata: ChannelMetadata {
                    slack_channel_id: Some(target.id.clone()),
                    slack_team_id: Some(self.installation.team_id.clone()),
                    ..Default::default()
                },
            })
            .map_err(|err| err.to_string())?;
        Ok(SendAttemptOutcome {
            success: result.success,
            message_id: (!result.message_id.is_empty()).then_some(result.message_id),
            error: result.error,
        })
    }

    fn send_direct_message(
        &self,
        recipient_user_id: &str,
        text: &str,
    ) -> Result<SendAttemptOutcome, String> {
        let adapter = SlackOutboundAdapter::new(self.installation.bot_token.clone());
        let result = adapter
            .send(&OutboundMessage {
                channel: Channel::Slack,
                from: None,
                to: vec![recipient_user_id.to_string()],
                cc: vec![],
                bcc: vec![],
                subject: String::new(),
                text_body: text.to_string(),
                html_body: String::new(),
                html_path: None,
                attachments_dir: None,
                thread_id: None,
                metadata: ChannelMetadata {
                    slack_team_id: Some(self.installation.team_id.clone()),
                    ..Default::default()
                },
            })
            .map_err(|err| err.to_string())?;
        Ok(SendAttemptOutcome {
            success: result.success,
            message_id: (!result.message_id.is_empty()).then_some(result.message_id),
            error: result.error,
        })
    }
}

pub(crate) struct DiscordInstallOnboardingClient {
    guild_id: String,
    guild_name: Option<String>,
    bot_token: String,
    api_base: String,
}

impl DiscordInstallOnboardingClient {
    pub(crate) fn new(
        guild_id: impl Into<String>,
        guild_name: Option<String>,
        bot_token: impl Into<String>,
    ) -> Self {
        Self {
            guild_id: guild_id.into(),
            guild_name,
            bot_token: bot_token.into(),
            api_base: env::var("DISCORD_API_BASE_URL")
                .unwrap_or_else(|_| "https://discord.com/api/v10".to_string()),
        }
    }
}

impl InstallOnboardingPlatformClient for DiscordInstallOnboardingClient {
    fn platform(&self) -> InstallPlatform {
        InstallPlatform::Discord
    }

    fn workspace_name(&self) -> Option<&str> {
        self.guild_name.as_deref()
    }

    fn mention_user(&self, user_id: &str) -> String {
        format!("<@{}>", user_id)
    }

    fn public_targets(
        &self,
        hint_channel_id: Option<&str>,
    ) -> Result<Vec<OnboardingChannelTarget>, String> {
        let channels =
            list_discord_guild_channels(&self.api_base, &self.bot_token, &self.guild_id)?;
        let resolved_hint = hint_channel_id.map(ToOwned::to_owned).or_else(|| {
            get_discord_system_channel_id(&self.api_base, &self.bot_token, &self.guild_id)
                .ok()
                .flatten()
        });
        Ok(select_discord_public_targets(
            &channels,
            resolved_hint.as_deref(),
        ))
    }

    fn send_public_message(
        &self,
        target: &OnboardingChannelTarget,
        text: &str,
    ) -> Result<SendAttemptOutcome, String> {
        let adapter = DiscordOutboundAdapter::new(self.bot_token.clone());
        let result = adapter
            .send(&OutboundMessage {
                channel: Channel::Discord,
                from: None,
                to: vec!["ignored".to_string(), target.id.clone()],
                cc: vec![],
                bcc: vec![],
                subject: String::new(),
                text_body: text.to_string(),
                html_body: String::new(),
                html_path: None,
                attachments_dir: None,
                thread_id: None,
                metadata: ChannelMetadata {
                    discord_guild_id: self.guild_id.parse::<u64>().ok(),
                    discord_channel_id: target.id.parse::<u64>().ok(),
                    ..Default::default()
                },
            })
            .map_err(|err| err.to_string())?;
        Ok(SendAttemptOutcome {
            success: result.success,
            message_id: (!result.message_id.is_empty()).then_some(result.message_id),
            error: result.error,
        })
    }

    fn send_direct_message(
        &self,
        recipient_user_id: &str,
        text: &str,
    ) -> Result<SendAttemptOutcome, String> {
        let adapter = DiscordOutboundAdapter::new(self.bot_token.clone());
        let result = adapter
            .send(&OutboundMessage {
                channel: Channel::Discord,
                from: None,
                to: vec![recipient_user_id.to_string()],
                cc: vec![],
                bcc: vec![],
                subject: String::new(),
                text_body: text.to_string(),
                html_body: String::new(),
                html_path: None,
                attachments_dir: None,
                thread_id: None,
                metadata: Default::default(),
            })
            .map_err(|err| err.to_string())?;
        Ok(SendAttemptOutcome {
            success: result.success,
            message_id: (!result.message_id.is_empty()).then_some(result.message_id),
            error: result.error,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
struct SlackConversation {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    is_general: bool,
    #[serde(default)]
    is_archived: bool,
    #[serde(default)]
    is_private: bool,
    #[serde(default)]
    is_im: bool,
    #[serde(default)]
    is_mpim: bool,
}

#[derive(Debug, Deserialize)]
struct SlackConversationListResponse {
    ok: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    channels: Vec<SlackConversation>,
}

fn list_slack_public_channels(
    api_base: &str,
    bot_token: &str,
    team_id: &str,
) -> Result<Vec<SlackConversation>, String> {
    let client = Client::new();
    let response = client
        .get(format!(
            "{}/conversations.list",
            api_base.trim_end_matches('/')
        ))
        .bearer_auth(bot_token)
        .query(&[
            ("limit", "200"),
            ("types", "public_channel"),
            ("team_id", team_id),
        ])
        .send()
        .map_err(|err| format!("slack conversations.list request failed: {err}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "slack conversations.list returned {}",
            response.status()
        ));
    }
    let payload: SlackConversationListResponse = response
        .json()
        .map_err(|err| format!("slack conversations.list payload unreadable: {err}"))?;
    if !payload.ok {
        return Err(payload
            .error
            .unwrap_or_else(|| "slack conversations.list returned error".to_string()));
    }
    Ok(payload.channels)
}

fn select_slack_public_targets(
    channels: &[SlackConversation],
    hint_channel_id: Option<&str>,
) -> Vec<OnboardingChannelTarget> {
    if let Some(hint) = hint_channel_id {
        if let Some(channel) = channels.iter().find(|channel| channel.id == hint.trim()) {
            return vec![OnboardingChannelTarget {
                id: channel.id.clone(),
                name: channel.name.clone(),
            }];
        }
    }

    if let Some(channel) = channels
        .iter()
        .find(|channel| channel.is_general && !channel.is_archived)
    {
        return vec![OnboardingChannelTarget {
            id: channel.id.clone(),
            name: channel.name.clone(),
        }];
    }

    let mut scored = channels
        .iter()
        .filter(|channel| {
            !channel.is_archived && !channel.is_private && !channel.is_im && !channel.is_mpim
        })
        .filter_map(|channel| {
            let name = channel.name.clone().unwrap_or_default();
            score_safe_channel_name(&name, SLACK_SAFE_PUBLIC_CHANNEL_NAMES).map(|score| {
                (
                    score,
                    OnboardingChannelTarget {
                        id: channel.id.clone(),
                        name: channel.name.clone(),
                    },
                )
            })
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.id.cmp(&right.1.id)));
    scored.into_iter().map(|(_, target)| target).collect()
}

#[derive(Debug, Clone, Deserialize)]
struct DiscordGuildChannel {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "type")]
    kind: u8,
}

#[derive(Debug, Deserialize)]
struct DiscordGuildInfo {
    #[serde(default)]
    system_channel_id: Option<String>,
}

fn list_discord_guild_channels(
    api_base: &str,
    bot_token: &str,
    guild_id: &str,
) -> Result<Vec<DiscordGuildChannel>, String> {
    let client = Client::new();
    let response = client
        .get(format!(
            "{}/guilds/{}/channels",
            api_base.trim_end_matches('/'),
            guild_id
        ))
        .header("Authorization", format!("Bot {bot_token}"))
        .send()
        .map_err(|err| format!("discord channel list request failed: {err}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "discord guild channels api returned {}",
            response.status()
        ));
    }
    let payload: Vec<DiscordGuildChannel> = response
        .json()
        .map_err(|err| format!("discord guild channels payload unreadable: {err}"))?;
    Ok(payload
        .into_iter()
        .filter(|channel| DISCORD_TEXT_CHANNEL_TYPES.contains(&channel.kind))
        .collect())
}

fn get_discord_system_channel_id(
    api_base: &str,
    bot_token: &str,
    guild_id: &str,
) -> Result<Option<String>, String> {
    let client = Client::new();
    let response = client
        .get(format!(
            "{}/guilds/{}",
            api_base.trim_end_matches('/'),
            guild_id
        ))
        .header("Authorization", format!("Bot {bot_token}"))
        .send()
        .map_err(|err| format!("discord guild info request failed: {err}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "discord guild info api returned {}",
            response.status()
        ));
    }
    let payload: DiscordGuildInfo = response
        .json()
        .map_err(|err| format!("discord guild info payload unreadable: {err}"))?;
    Ok(payload
        .system_channel_id
        .filter(|value| !value.trim().is_empty()))
}

fn select_discord_public_targets(
    channels: &[DiscordGuildChannel],
    hint_channel_id: Option<&str>,
) -> Vec<OnboardingChannelTarget> {
    if let Some(hint) = hint_channel_id {
        if let Some(channel) = channels.iter().find(|channel| channel.id == hint.trim()) {
            return vec![OnboardingChannelTarget {
                id: channel.id.clone(),
                name: channel.name.clone(),
            }];
        }
    }

    let mut scored = channels
        .iter()
        .filter_map(|channel| {
            let name = channel.name.clone().unwrap_or_default();
            score_safe_channel_name(&name, DISCORD_SAFE_PUBLIC_CHANNEL_NAMES).map(|score| {
                (
                    score,
                    OnboardingChannelTarget {
                        id: channel.id.clone(),
                        name: channel.name.clone(),
                    },
                )
            })
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.id.cmp(&right.1.id)));
    scored.into_iter().map(|(_, target)| target).collect()
}

fn score_safe_channel_name(name: &str, safe_names: &[&str]) -> Option<(u8, String)> {
    let normalized = name.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }

    if let Some(index) = safe_names
        .iter()
        .position(|candidate| normalized == *candidate)
    {
        return Some((index as u8, normalized));
    }

    if let Some(index) = safe_names
        .iter()
        .position(|candidate| normalized.starts_with(candidate) || normalized.contains(candidate))
    {
        return Some(((safe_names.len() + index) as u8, normalized));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemoryStateStore {
        rows: Mutex<HashMap<(Uuid, InstallPlatform, String), InstallOnboardingStateRecord>>,
    }

    impl InstallOnboardingStateStore for MemoryStateStore {
        fn load(
            &self,
            account_id: Uuid,
            platform: InstallPlatform,
            workspace_id: &str,
        ) -> Result<Option<InstallOnboardingStateRecord>, String> {
            Ok(self
                .rows
                .lock()
                .expect("lock")
                .get(&(account_id, platform, workspace_id.to_string()))
                .cloned())
        }

        fn save(&self, state: &InstallOnboardingStateRecord) -> Result<(), String> {
            self.rows.lock().expect("lock").insert(
                (state.account_id, state.platform, state.workspace_id.clone()),
                state.clone(),
            );
            Ok(())
        }
    }

    #[derive(Debug, Clone)]
    struct FakePlatformClient {
        platform: InstallPlatform,
        workspace_name: Option<String>,
        public_targets: Vec<OnboardingChannelTarget>,
        public_outcomes: Vec<SendAttemptOutcome>,
        dm_outcomes: Vec<SendAttemptOutcome>,
        public_attempts: Arc<Mutex<Vec<String>>>,
        dm_attempts: Arc<Mutex<Vec<String>>>,
    }

    impl FakePlatformClient {
        fn new(platform: InstallPlatform) -> Self {
            Self {
                platform,
                workspace_name: Some("Workspace".to_string()),
                public_targets: vec![],
                public_outcomes: vec![],
                dm_outcomes: vec![],
                public_attempts: Arc::new(Mutex::new(Vec::new())),
                dm_attempts: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl InstallOnboardingPlatformClient for FakePlatformClient {
        fn platform(&self) -> InstallPlatform {
            self.platform
        }

        fn workspace_name(&self) -> Option<&str> {
            self.workspace_name.as_deref()
        }

        fn mention_user(&self, user_id: &str) -> String {
            format!("@{user_id}")
        }

        fn public_targets(
            &self,
            _hint_channel_id: Option<&str>,
        ) -> Result<Vec<OnboardingChannelTarget>, String> {
            Ok(self.public_targets.clone())
        }

        fn send_public_message(
            &self,
            target: &OnboardingChannelTarget,
            _text: &str,
        ) -> Result<SendAttemptOutcome, String> {
            self.public_attempts
                .lock()
                .expect("lock")
                .push(target.id.clone());
            Ok(self
                .public_outcomes
                .get(self.public_attempts.lock().expect("lock").len() - 1)
                .cloned()
                .unwrap_or(SendAttemptOutcome {
                    success: false,
                    message_id: None,
                    error: Some("missing_public_outcome".to_string()),
                }))
        }

        fn send_direct_message(
            &self,
            recipient_user_id: &str,
            _text: &str,
        ) -> Result<SendAttemptOutcome, String> {
            self.dm_attempts
                .lock()
                .expect("lock")
                .push(recipient_user_id.to_string());
            Ok(self
                .dm_outcomes
                .get(self.dm_attempts.lock().expect("lock").len() - 1)
                .cloned()
                .unwrap_or(SendAttemptOutcome {
                    success: false,
                    message_id: None,
                    error: Some("missing_dm_outcome".to_string()),
                }))
        }
    }

    fn test_account_store() -> Arc<AccountStore> {
        Arc::new(AccountStore::detached_for_tests())
    }

    fn base_request(platform: InstallPlatform) -> InstallOnboardingRequest {
        InstallOnboardingRequest {
            account_id: Uuid::new_v4(),
            auth_user_id: None,
            platform,
            workspace_id: "workspace-1".to_string(),
            workspace_name: Some("Workspace".to_string()),
            installer_identifier: Some("user-1".to_string()),
            installer_identifier_source: Some("installer".to_string()),
            linked_owner_identifier: Some("owner-1".to_string()),
            linked_owner_identifier_source: Some("linked_owner".to_string()),
            public_channel_hint: None,
            event_key: Some("event-1".to_string()),
            route_path: Some("/auth/test".to_string()),
            trigger: InstallOnboardingTrigger::InstallSuccess,
            force: false,
        }
    }

    fn base_config() -> InstallOnboardingConfig {
        InstallOnboardingConfig {
            slack_enabled: true,
            discord_enabled: true,
            reinstall_cooldown: Duration::from_secs(3600),
        }
    }

    #[test]
    fn public_message_avoids_banned_phrases() {
        let client = FakePlatformClient::new(InstallPlatform::Slack);
        let request = base_request(InstallPlatform::Slack);
        let message = build_public_message(&client, &request);
        assert!(!message.contains("reading everything here"));
        assert!(!message.contains("read the whole workspace"));
        assert!(message.contains("Summarize a thread"));
    }

    #[test]
    fn direct_message_includes_three_starter_prompts() {
        let client = FakePlatformClient::new(InstallPlatform::Discord);
        let request = base_request(InstallPlatform::Discord);
        let message = build_direct_message(&client, &request);
        assert!(message.contains("1."));
        assert!(message.contains("2."));
        assert!(message.contains("3."));
    }

    #[test]
    fn dedupe_skips_repeated_event_after_attempt() {
        let store = MemoryStateStore::default();
        let analytics = test_account_store();
        let config = base_config();
        let request = base_request(InstallPlatform::Slack);
        let client = FakePlatformClient {
            public_targets: vec![OnboardingChannelTarget {
                id: "C1".to_string(),
                name: Some("general".to_string()),
            }],
            public_outcomes: vec![SendAttemptOutcome {
                success: true,
                message_id: Some("m1".to_string()),
                error: None,
            }],
            dm_outcomes: vec![SendAttemptOutcome {
                success: true,
                message_id: Some("dm1".to_string()),
                error: None,
            }],
            ..FakePlatformClient::new(InstallPlatform::Slack)
        };

        let first = run_install_onboarding(&config, &store, &analytics, &client, request.clone())
            .expect("first onboarding");
        assert!(!first.skipped);

        let second = run_install_onboarding(&config, &store, &analytics, &client, request)
            .expect("second onboarding");
        assert!(second.skipped);
        assert_eq!(second.skip_reason.as_deref(), Some("duplicate_event"));
        assert_eq!(client.public_attempts.lock().expect("lock").len(), 1);
    }

    #[test]
    fn cooldown_skips_reinstall_attempts() {
        let store = MemoryStateStore::default();
        let analytics = test_account_store();
        let config = base_config();
        let request = base_request(InstallPlatform::Slack);
        let mut existing = InstallOnboardingStateRecord::new(
            request.account_id,
            InstallPlatform::Slack,
            request.workspace_id.clone(),
        );
        existing.last_attempted_at = Some(Utc::now());
        store.save(&existing).expect("save existing");

        let client = FakePlatformClient::new(InstallPlatform::Slack);
        let result = run_install_onboarding(&config, &store, &analytics, &client, request)
            .expect("cooldown result");
        assert!(result.skipped);
        assert_eq!(result.skip_reason.as_deref(), Some("cooldown_active"));
    }

    #[test]
    fn no_public_target_falls_back_to_dm_only() {
        let store = MemoryStateStore::default();
        let analytics = test_account_store();
        let config = base_config();
        let request = base_request(InstallPlatform::Discord);
        let client = FakePlatformClient {
            public_targets: vec![],
            public_outcomes: vec![],
            dm_outcomes: vec![SendAttemptOutcome {
                success: true,
                message_id: Some("dm1".to_string()),
                error: None,
            }],
            ..FakePlatformClient::new(InstallPlatform::Discord)
        };

        let result = run_install_onboarding(&config, &store, &analytics, &client, request)
            .expect("dm fallback");
        assert!(!result.skipped);
        assert_eq!(result.public_status, DeliveryStatus::Skipped);
        assert_eq!(result.dm_status, DeliveryStatus::Sent);
    }

    #[test]
    fn dm_failure_does_not_break_public_success() {
        let store = MemoryStateStore::default();
        let analytics = test_account_store();
        let config = base_config();
        let request = base_request(InstallPlatform::Discord);
        let client = FakePlatformClient {
            public_targets: vec![OnboardingChannelTarget {
                id: "123".to_string(),
                name: Some("general".to_string()),
            }],
            public_outcomes: vec![SendAttemptOutcome {
                success: true,
                message_id: Some("pub1".to_string()),
                error: None,
            }],
            dm_outcomes: vec![SendAttemptOutcome {
                success: false,
                message_id: None,
                error: Some("dm_closed".to_string()),
            }],
            ..FakePlatformClient::new(InstallPlatform::Discord)
        };

        let result = run_install_onboarding(&config, &store, &analytics, &client, request)
            .expect("public succeeds");
        assert!(!result.skipped);
        assert_eq!(result.public_status, DeliveryStatus::Sent);
        assert_eq!(result.dm_status, DeliveryStatus::Failed);
    }

    #[test]
    fn manual_resend_bypasses_cooldown() {
        let store = MemoryStateStore::default();
        let analytics = test_account_store();
        let config = base_config();
        let mut request = base_request(InstallPlatform::Slack);
        request.trigger = InstallOnboardingTrigger::ManualResend;
        request.force = true;

        let mut existing = InstallOnboardingStateRecord::new(
            request.account_id,
            InstallPlatform::Slack,
            request.workspace_id.clone(),
        );
        existing.last_attempted_at = Some(Utc::now());
        store.save(&existing).expect("save existing");

        let client = FakePlatformClient {
            public_targets: vec![OnboardingChannelTarget {
                id: "C1".to_string(),
                name: Some("general".to_string()),
            }],
            public_outcomes: vec![SendAttemptOutcome {
                success: true,
                message_id: Some("pub1".to_string()),
                error: None,
            }],
            dm_outcomes: vec![SendAttemptOutcome {
                success: true,
                message_id: Some("dm1".to_string()),
                error: None,
            }],
            ..FakePlatformClient::new(InstallPlatform::Slack)
        };

        let result = run_install_onboarding(&config, &store, &analytics, &client, request)
            .expect("manual resend");
        assert!(!result.skipped);
        assert_eq!(client.public_attempts.lock().expect("lock").len(), 1);
    }

    #[test]
    fn slack_target_selection_prefers_general_and_ignores_unknown_channels() {
        let channels = vec![
            SlackConversation {
                id: "C2".to_string(),
                name: Some("build-alerts".to_string()),
                is_general: false,
                is_archived: false,
                is_private: false,
                is_im: false,
                is_mpim: false,
            },
            SlackConversation {
                id: "C1".to_string(),
                name: Some("general".to_string()),
                is_general: true,
                is_archived: false,
                is_private: false,
                is_im: false,
                is_mpim: false,
            },
        ];

        let targets = select_slack_public_targets(&channels, None);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].id, "C1");
    }

    #[test]
    fn discord_target_selection_prefers_safe_names_only() {
        let channels = vec![
            DiscordGuildChannel {
                id: "1".to_string(),
                name: Some("deployments".to_string()),
                kind: 0,
            },
            DiscordGuildChannel {
                id: "2".to_string(),
                name: Some("general".to_string()),
                kind: 0,
            },
        ];

        let targets = select_discord_public_targets(&channels, None);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].id, "2");
    }

    #[test]
    fn direct_recipient_prefers_installer_over_linked_owner() {
        let store = MemoryStateStore::default();
        let analytics = test_account_store();
        let config = base_config();
        let request = base_request(InstallPlatform::Slack);
        let client = FakePlatformClient {
            public_targets: vec![],
            public_outcomes: vec![],
            dm_outcomes: vec![SendAttemptOutcome {
                success: true,
                message_id: Some("dm".to_string()),
                error: None,
            }],
            ..FakePlatformClient::new(InstallPlatform::Slack)
        };

        let _ = run_install_onboarding(&config, &store, &analytics, &client, request)
            .expect("run onboarding");
        let attempts = client.dm_attempts.lock().expect("lock");
        assert_eq!(attempts.as_slice(), &["user-1".to_string()]);
    }

    #[test]
    fn linked_owner_is_used_when_installer_missing() {
        let store = MemoryStateStore::default();
        let analytics = test_account_store();
        let config = base_config();
        let mut request = base_request(InstallPlatform::Discord);
        request.installer_identifier = None;
        request.installer_identifier_source = None;
        let client = FakePlatformClient {
            public_targets: vec![],
            public_outcomes: vec![],
            dm_outcomes: vec![SendAttemptOutcome {
                success: true,
                message_id: Some("dm".to_string()),
                error: None,
            }],
            ..FakePlatformClient::new(InstallPlatform::Discord)
        };

        let _ = run_install_onboarding(&config, &store, &analytics, &client, request)
            .expect("run onboarding");
        let attempts = client.dm_attempts.lock().expect("lock");
        assert_eq!(attempts.as_slice(), &["owner-1".to_string()]);
    }
}
