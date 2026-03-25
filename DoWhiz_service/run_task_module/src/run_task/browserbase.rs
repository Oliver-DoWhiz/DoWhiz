use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::env::read_env_trimmed;
use super::errors::RunTaskError;
use super::utils::{run_command_with_timeout, tail_string};

pub(super) const BROWSERBASE_API_KEY_ENV_KEY: &str = "BROWSERBASE_API_KEY";
pub(super) const BROWSERBASE_PROJECT_ID_ENV_KEY: &str = "BROWSERBASE_PROJECT_ID";
pub(super) const BROWSERBASE_STATE_DIR_ENV_KEY: &str = "BROWSERBASE_STATE_DIR";
pub(super) const BROWSERBASE_REGISTRY_PATH_ENV_KEY: &str = "BROWSERBASE_REGISTRY_PATH";
pub(super) const BROWSERBASE_ACTIVE_SESSION_PATH_ENV_KEY: &str = "BROWSERBASE_ACTIVE_SESSION_PATH";
pub(super) const BROWSERBASE_API_BASE_URL_ENV_KEY: &str = "BROWSERBASE_API_BASE_URL";
pub(super) const BROWSERBASE_SESSION_TIMEOUT_SECONDS_ENV_KEY: &str =
    "BROWSERBASE_SESSION_TIMEOUT_SECONDS";
pub(super) const BROWSER_HANDOFF_BASE_URL_ENV_KEY: &str = "BROWSER_HANDOFF_BASE_URL";
pub(super) const BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY: &str = "BROWSER_HANDOFF_SIGNING_SECRET";

const BROWSERBASE_API_KEY_ALIAS_ENV_KEY: &str = "BROWSER_BASE_API_KEY";
const BROWSERBASE_PROJECT_ID_ALIAS_ENV_KEY: &str = "BROWSER_BASE_PROJECT_ID";
const DEFAULT_BROWSERBASE_STATE_DIR: &str = ".secrets/browserbase";
const DEFAULT_BROWSERBASE_REGISTRY_PATH: &str = ".secrets/browserbase/registry.json";
const DEFAULT_BROWSERBASE_ACTIVE_SESSION_PATH: &str = ".secrets/browserbase/active_session.json";
const DEFAULT_BROWSERBASE_API_BASE_URL: &str = "https://api.browserbase.com";
const DEFAULT_BROWSERBASE_SESSION_TIMEOUT_SECONDS: &str = "3600";
const CHAT_HISTORY_SCOPE_SIGNING_SECRET_ENV_KEY: &str = "CHAT_HISTORY_SCOPE_SIGNING_SECRET";
const SLACK_SIGNING_SECRET_ENV_KEY: &str = "SLACK_SIGNING_SECRET";
const DOWHIZ_API_URL_ENV_KEY: &str = "DOWHIZ_API_URL";
const SERVICE_URL_ENV_KEY: &str = "SERVICE_URL";
const POSTMARK_INBOUND_HOOK_URL_ENV_KEY: &str = "POSTMARK_INBOUND_HOOK_URL";
const FRONTEND_URL_ENV_KEY: &str = "FRONTEND_URL";
const SESSION_MANAGER_RELEASE_TIMEOUT: Duration = Duration::from_secs(30);

pub(super) fn collect_browserbase_env_overrides() -> Vec<(String, String)> {
    let mut overrides = vec![
        (
            BROWSERBASE_STATE_DIR_ENV_KEY.to_string(),
            DEFAULT_BROWSERBASE_STATE_DIR.to_string(),
        ),
        (
            BROWSERBASE_REGISTRY_PATH_ENV_KEY.to_string(),
            DEFAULT_BROWSERBASE_REGISTRY_PATH.to_string(),
        ),
        (
            BROWSERBASE_ACTIVE_SESSION_PATH_ENV_KEY.to_string(),
            DEFAULT_BROWSERBASE_ACTIVE_SESSION_PATH.to_string(),
        ),
        (
            BROWSERBASE_API_BASE_URL_ENV_KEY.to_string(),
            read_env_trimmed(BROWSERBASE_API_BASE_URL_ENV_KEY)
                .unwrap_or_else(|| DEFAULT_BROWSERBASE_API_BASE_URL.to_string()),
        ),
        (
            BROWSERBASE_SESSION_TIMEOUT_SECONDS_ENV_KEY.to_string(),
            read_env_trimmed(BROWSERBASE_SESSION_TIMEOUT_SECONDS_ENV_KEY)
                .unwrap_or_else(|| DEFAULT_BROWSERBASE_SESSION_TIMEOUT_SECONDS.to_string()),
        ),
    ];

    if let Some(api_key) = read_env_trimmed(BROWSERBASE_API_KEY_ENV_KEY)
        .or_else(|| read_env_trimmed(BROWSERBASE_API_KEY_ALIAS_ENV_KEY))
    {
        overrides.push((BROWSERBASE_API_KEY_ENV_KEY.to_string(), api_key));
    }

    if let Some(project_id) = read_env_trimmed(BROWSERBASE_PROJECT_ID_ENV_KEY)
        .or_else(|| read_env_trimmed(BROWSERBASE_PROJECT_ID_ALIAS_ENV_KEY))
    {
        overrides.push((BROWSERBASE_PROJECT_ID_ENV_KEY.to_string(), project_id));
    }

    if let Some(base_url) = resolve_browser_handoff_base_url() {
        overrides.push((BROWSER_HANDOFF_BASE_URL_ENV_KEY.to_string(), base_url));
    }

    if let Some(secret) = resolve_browser_handoff_signing_secret() {
        overrides.push((BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY.to_string(), secret));
    }

    overrides
}

pub(super) fn browserbase_enabled() -> bool {
    read_env_trimmed(BROWSERBASE_API_KEY_ENV_KEY)
        .or_else(|| read_env_trimmed(BROWSERBASE_API_KEY_ALIAS_ENV_KEY))
        .is_some()
}

pub(super) fn workspace_browserbase_state_dir(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join(DEFAULT_BROWSERBASE_STATE_DIR)
}

pub(super) struct BrowserbaseSessionCleanupGuard {
    enabled: bool,
    state_dir: PathBuf,
}

impl BrowserbaseSessionCleanupGuard {
    pub(super) fn new(workspace_dir: &Path) -> Self {
        Self {
            enabled: browserbase_enabled(),
            state_dir: workspace_browserbase_state_dir(workspace_dir),
        }
    }
}

impl Drop for BrowserbaseSessionCleanupGuard {
    fn drop(&mut self) {
        if !self.enabled {
            return;
        }
        if let Err(err) = release_active_browserbase_session(&self.state_dir) {
            eprintln!(
                "[run_task] warning: failed to release Browserbase session from {}: {}",
                self.state_dir.display(),
                err
            );
        }
    }
}

fn resolve_browser_handoff_signing_secret() -> Option<String> {
    read_env_trimmed(BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY)
        .or_else(|| read_env_trimmed(CHAT_HISTORY_SCOPE_SIGNING_SECRET_ENV_KEY))
        .or_else(|| read_env_trimmed(SLACK_SIGNING_SECRET_ENV_KEY))
}

fn resolve_browser_handoff_base_url() -> Option<String> {
    read_env_trimmed(BROWSER_HANDOFF_BASE_URL_ENV_KEY)
        .or_else(|| read_env_trimmed(DOWHIZ_API_URL_ENV_KEY))
        .or_else(|| read_env_trimmed(SERVICE_URL_ENV_KEY))
        .or_else(|| {
            read_env_trimmed(POSTMARK_INBOUND_HOOK_URL_ENV_KEY)
                .and_then(|value| derive_public_service_base_url(&value))
        })
        .or_else(|| {
            read_env_trimmed(FRONTEND_URL_ENV_KEY)
                .and_then(|value| derive_public_service_base_url(&value))
        })
        .map(|value| value.trim_end_matches('/').to_string())
}

fn derive_public_service_base_url(candidate: &str) -> Option<String> {
    let without_fragment = candidate.trim().split('#').next()?.trim();
    let without_query = without_fragment.split('?').next()?.trim_end_matches('/');
    if without_query.is_empty() {
        return None;
    }

    let normalized = if let Some(prefix) = without_query.strip_suffix("/postmark/inbound") {
        let prefix = prefix.trim_end_matches('/');
        if prefix.ends_with("/service") {
            prefix.to_string()
        } else if prefix.is_empty() {
            "/service".to_string()
        } else {
            format!("{prefix}/service")
        }
    } else if without_query.ends_with("/service") {
        without_query.to_string()
    } else {
        format!("{without_query}/service")
    };

    Some(normalized.trim_end_matches('/').to_string())
}

fn release_active_browserbase_session(state_dir: &Path) -> Result<(), RunTaskError> {
    if !state_dir.exists() {
        return Ok(());
    }

    let mut cmd = Command::new(resolve_session_manager_command());
    cmd.arg("release-active")
        .arg("--state-dir")
        .arg(state_dir)
        .env(
            BROWSERBASE_STATE_DIR_ENV_KEY,
            state_dir.to_string_lossy().into_owned(),
        );

    let output = run_command_with_timeout(
        cmd,
        SESSION_MANAGER_RELEASE_TIMEOUT,
        "browserbase_session_manager release-active",
    )?;
    if output.status.success() {
        return Ok(());
    }

    let mut combined = String::new();
    combined.push_str(&String::from_utf8_lossy(&output.stdout));
    combined.push_str(&String::from_utf8_lossy(&output.stderr));
    Err(RunTaskError::BrowserbaseFailed {
        action: "release-active",
        output: tail_string(&combined, 4000),
    })
}

fn resolve_session_manager_command() -> String {
    if let Some(path) = read_env_trimmed("DOWHIZ_BIN_DIR")
        .map(PathBuf::from)
        .map(|dir| dir.join("browserbase_session_manager"))
        .filter(|path| path.exists())
    {
        return path.to_string_lossy().into_owned();
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_wrapper = manifest_dir
        .parent()
        .map(|path| path.join("bin").join("browserbase_session_manager"));
    if let Some(path) = repo_wrapper.filter(|path| path.exists()) {
        return path.to_string_lossy().into_owned();
    }

    "browserbase_session_manager".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvGuard {
        key: &'static str,
        prev: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let prev = env::var(key).ok();
            env::set_var(key, value);
            Self { key, prev }
        }

        fn unset(key: &'static str) -> Self {
            let prev = env::var(key).ok();
            env::remove_var(key);
            Self { key, prev }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(value) = &self.prev {
                env::set_var(self.key, value);
            } else {
                env::remove_var(self.key);
            }
        }
    }

    #[test]
    fn collect_browserbase_env_overrides_canonicalizes_aliases() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _api_key = EnvGuard::set(BROWSERBASE_API_KEY_ALIAS_ENV_KEY, "bb_test");
        let _project_id = EnvGuard::set(BROWSERBASE_PROJECT_ID_ALIAS_ENV_KEY, "proj_test");
        let _unset_api = EnvGuard::unset(BROWSERBASE_API_KEY_ENV_KEY);
        let _unset_project = EnvGuard::unset(BROWSERBASE_PROJECT_ID_ENV_KEY);
        let _unset_direct_secret = EnvGuard::unset(BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY);
        let _unset_chat_secret = EnvGuard::unset(CHAT_HISTORY_SCOPE_SIGNING_SECRET_ENV_KEY);
        let _unset_slack_secret = EnvGuard::unset(SLACK_SIGNING_SECRET_ENV_KEY);
        let overrides = collect_browserbase_env_overrides();

        assert!(overrides
            .iter()
            .any(|(key, value)| { key == BROWSERBASE_API_KEY_ENV_KEY && value == "bb_test" }));
        assert!(overrides
            .iter()
            .any(|(key, value)| { key == BROWSERBASE_PROJECT_ID_ENV_KEY && value == "proj_test" }));
        assert!(overrides.iter().any(|(key, value)| {
            key == BROWSERBASE_STATE_DIR_ENV_KEY && value == DEFAULT_BROWSERBASE_STATE_DIR
        }));
    }

    #[test]
    fn collect_browserbase_env_overrides_uses_handoff_secret_fallbacks() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _unset_direct = EnvGuard::unset(BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY);
        let _chat_secret = EnvGuard::set(CHAT_HISTORY_SCOPE_SIGNING_SECRET_ENV_KEY, "scope-secret");
        let _unset_slack = EnvGuard::unset(SLACK_SIGNING_SECRET_ENV_KEY);

        let overrides = collect_browserbase_env_overrides();

        assert!(overrides.iter().any(|(key, value)| {
            key == BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY && value == "scope-secret"
        }));
    }

    #[test]
    fn resolve_browser_handoff_base_url_prefers_public_service_values() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _unset_direct = EnvGuard::unset(BROWSER_HANDOFF_BASE_URL_ENV_KEY);
        let _unset_dowhiz = EnvGuard::unset(DOWHIZ_API_URL_ENV_KEY);
        let _service = EnvGuard::set(SERVICE_URL_ENV_KEY, "https://api.example.com/service/");
        let _unset_postmark = EnvGuard::unset(POSTMARK_INBOUND_HOOK_URL_ENV_KEY);
        let _unset_frontend = EnvGuard::unset(FRONTEND_URL_ENV_KEY);

        assert_eq!(
            resolve_browser_handoff_base_url().as_deref(),
            Some("https://api.example.com/service")
        );
    }

    #[test]
    fn derive_public_service_base_url_strips_inbound_suffix_and_appends_service() {
        assert_eq!(
            derive_public_service_base_url("https://api.example.com/service/postmark/inbound")
                .as_deref(),
            Some("https://api.example.com/service")
        );
        assert_eq!(
            derive_public_service_base_url("https://api.example.com/postmark/inbound").as_deref(),
            Some("https://api.example.com/service")
        );
    }

    #[test]
    fn derive_public_service_base_url_appends_service_for_frontend_root() {
        assert_eq!(
            derive_public_service_base_url("https://api.example.com/").as_deref(),
            Some("https://api.example.com/service")
        );
    }
}
