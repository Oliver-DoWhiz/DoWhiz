//! Slack installation store for multi-workspace support.
//!
//! Stores OAuth tokens and bot user IDs per workspace (team).

use chrono::{DateTime, Utc};
use mongodb::bson::{doc, Bson, DateTime as BsonDateTime, Document};
use mongodb::options::FindOptions;
use mongodb::options::IndexOptions;
use mongodb::sync::Collection;
use mongodb::IndexModel;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tracing::{info, warn};

use crate::mongo_store::{create_client_from_env, database_from_env, ensure_index_compatible};

/// A Slack workspace installation record.
#[derive(Debug, Clone)]
pub struct SlackInstallation {
    pub team_id: String,
    pub team_name: Option<String>,
    pub bot_token: String,
    pub bot_user_id: String,
    pub installed_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum SlackStoreError {
    #[error("mongodb error: {0}")]
    Mongo(#[from] mongodb::error::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("installation not found for team: {0}")]
    NotFound(String),
    #[error("datetime parse error: {0}")]
    DateTimeParse(#[from] chrono::ParseError),
    #[error("mongo config error: {0}")]
    MongoConfig(String),
}

/// Store for Slack workspace installations.
#[derive(Debug, Clone)]
pub struct SlackStore {
    mongo: MongoSlackStore,
}

#[derive(Debug, Clone)]
struct MongoSlackStore {
    installations: Collection<Document>,
}

impl SlackStore {
    /// Create a new SlackStore.
    pub fn new(_path: impl Into<PathBuf>) -> Result<Self, SlackStoreError> {
        Ok(Self {
            mongo: MongoSlackStore::new()?,
        })
    }

    /// Save or update an installation for a workspace.
    pub fn upsert_installation(
        &self,
        installation: &SlackInstallation,
    ) -> Result<(), SlackStoreError> {
        self.mongo.upsert_installation(installation)
    }

    /// Get installation by team_id.
    pub fn get_installation(&self, team_id: &str) -> Result<SlackInstallation, SlackStoreError> {
        self.mongo.get_installation(team_id)
    }

    /// Get installation by team_id, with fallback to environment variables.
    /// This provides backward compatibility with single-workspace setup.
    pub fn get_installation_or_env(
        &self,
        team_id: &str,
    ) -> Result<SlackInstallation, SlackStoreError> {
        self.resolve_installation_for_runtime(Some(team_id), None)
            .ok_or_else(|| SlackStoreError::NotFound(team_id.to_string()))
    }

    /// Delete an installation (e.g., when app is uninstalled).
    pub fn delete_installation(&self, team_id: &str) -> Result<bool, SlackStoreError> {
        self.mongo.delete_installation(team_id)
    }

    /// List all installations.
    pub fn list_installations(&self) -> Result<Vec<SlackInstallation>, SlackStoreError> {
        self.mongo.list_installations()
    }

    /// Resolve the Slack installation for a runtime message flow.
    ///
    /// Resolution order:
    /// 1. Stored installation by Slack team ID
    /// 2. Per-employee env credentials
    /// 3. Global env credentials
    pub fn resolve_installation_for_runtime(
        &self,
        team_id: Option<&str>,
        employee_id: Option<&str>,
    ) -> Option<SlackInstallation> {
        if let Some(team_id) = normalize_optional_value(team_id) {
            match self.get_installation(&team_id) {
                Ok(installation) => return Some(installation),
                Err(SlackStoreError::NotFound(_)) => {}
                Err(err) => warn!(
                    "failed to load Slack installation for team {} from store: {}",
                    team_id, err
                ),
            }
        }
        installation_from_env(employee_id, team_id)
    }
}

impl MongoSlackStore {
    fn new() -> Result<Self, SlackStoreError> {
        let client = create_client_from_env()
            .map_err(|err| SlackStoreError::MongoConfig(err.to_string()))?;
        let db = database_from_env(&client);
        let installations = db.collection::<Document>("slack_installations");
        ensure_index_compatible(
            &installations,
            IndexModel::builder()
                .keys(doc! { "team_id": 1 })
                .options(IndexOptions::builder().unique(Some(true)).build())
                .build(),
        )?;
        ensure_index_compatible(
            &installations,
            IndexModel::builder()
                .keys(doc! { "installed_at": -1 })
                .build(),
        )?;
        Ok(Self { installations })
    }

    fn upsert_installation(&self, installation: &SlackInstallation) -> Result<(), SlackStoreError> {
        self.installations.update_one(
            doc! {
                "team_id": installation.team_id.as_str(),
            },
            doc! {
                "$set": {
                    "team_id": installation.team_id.as_str(),
                    "team_name": installation.team_name.clone().map(Bson::from).unwrap_or(Bson::Null),
                    "bot_token": installation.bot_token.as_str(),
                    "bot_user_id": installation.bot_user_id.as_str(),
                    "installed_at": BsonDateTime::from_chrono(installation.installed_at),
                }
            },
            mongodb::options::UpdateOptions::builder()
                .upsert(true)
                .build(),
        )?;
        Ok(())
    }

    fn get_installation(&self, team_id: &str) -> Result<SlackInstallation, SlackStoreError> {
        let document = self
            .installations
            .find_one(
                doc! {
                    "team_id": team_id,
                },
                None,
            )?
            .ok_or_else(|| SlackStoreError::NotFound(team_id.to_string()))?;
        document_to_installation(document)
    }

    fn delete_installation(&self, team_id: &str) -> Result<bool, SlackStoreError> {
        let result = self.installations.delete_one(
            doc! {
                "team_id": team_id,
            },
            None,
        )?;
        Ok(result.deleted_count > 0)
    }

    fn list_installations(&self) -> Result<Vec<SlackInstallation>, SlackStoreError> {
        let mut values = Vec::new();
        let cursor = self.installations.find(
            doc! {},
            FindOptions::builder()
                .sort(doc! { "installed_at": -1 })
                .build(),
        )?;
        for row in cursor {
            values.push(document_to_installation(row?)?);
        }
        Ok(values)
    }
}

fn document_to_installation(document: Document) -> Result<SlackInstallation, SlackStoreError> {
    let team_id = document
        .get_str("team_id")
        .map_err(|err| SlackStoreError::MongoConfig(format!("missing team_id: {err}")))?
        .to_string();
    let team_name = match document.get("team_name") {
        Some(Bson::String(value)) => Some(value.to_string()),
        _ => None,
    };
    let bot_token = document
        .get_str("bot_token")
        .map_err(|err| SlackStoreError::MongoConfig(format!("missing bot_token: {err}")))?
        .to_string();
    let bot_user_id = document
        .get_str("bot_user_id")
        .map_err(|err| SlackStoreError::MongoConfig(format!("missing bot_user_id: {err}")))?
        .to_string();
    let installed_at = match document.get("installed_at") {
        Some(Bson::DateTime(value)) => value.to_chrono(),
        Some(Bson::String(value)) => parse_datetime(value)?,
        _ => {
            return Err(SlackStoreError::MongoConfig(
                "missing installed_at".to_string(),
            ))
        }
    };
    Ok(SlackInstallation {
        team_id,
        team_name,
        bot_token,
        bot_user_id,
        installed_at,
    })
}

fn parse_datetime(value: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    Ok(DateTime::parse_from_rfc3339(value)?.with_timezone(&Utc))
}

fn normalize_optional_value(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

fn env_var_trimmed(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn installation_from_env(
    employee_id: Option<&str>,
    team_id: Option<&str>,
) -> Option<SlackInstallation> {
    installation_from_employee_env(employee_id, team_id)
        .or_else(|| installation_from_global_env(team_id))
}

fn installation_from_employee_env(
    employee_id: Option<&str>,
    team_id: Option<&str>,
) -> Option<SlackInstallation> {
    let employee_id = normalize_optional_value(employee_id)?;
    let prefix = employee_id.to_uppercase().replace('-', "_");
    let token_key = format!("{prefix}_SLACK_BOT_TOKEN");
    let bot_token = env_var_trimmed(&token_key)?;
    let bot_user_id = env_var_trimmed(&format!("{prefix}_SLACK_BOT_USER_ID")).unwrap_or_default();
    Some(SlackInstallation {
        team_id: normalize_optional_value(team_id).unwrap_or_default(),
        team_name: None,
        bot_token,
        bot_user_id,
        installed_at: Utc::now(),
    })
}

fn installation_from_global_env(team_id: Option<&str>) -> Option<SlackInstallation> {
    let bot_token = env_var_trimmed("SLACK_BOT_TOKEN")?;
    let bot_user_id = env_var_trimmed("SLACK_BOT_USER_ID").unwrap_or_default();
    Some(SlackInstallation {
        team_id: normalize_optional_value(team_id).unwrap_or_default(),
        team_name: None,
        bot_token,
        bot_user_id,
        installed_at: Utc::now(),
    })
}

static SLACK_STORE: OnceLock<Option<Arc<SlackStore>>> = OnceLock::new();

/// Get or initialize the global SlackStore (returns None if not configured).
pub fn get_global_slack_store() -> Option<Arc<SlackStore>> {
    SLACK_STORE
        .get_or_init(|| {
            let path = std::env::var("SLACK_STORE_PATH").unwrap_or_default();
            match SlackStore::new(path) {
                Ok(store) => {
                    info!("SlackStore initialized for runtime token resolution");
                    Some(Arc::new(store))
                }
                Err(err) => {
                    info!(
                        "SlackStore not available ({}), Slack runtime resolution falling back to env",
                        err
                    );
                    None
                }
            }
        })
        .clone()
}

pub fn resolve_slack_installation_for_runtime(
    team_id: Option<&str>,
    employee_id: Option<&str>,
) -> Option<SlackInstallation> {
    if let Some(store) = get_global_slack_store() {
        return store.resolve_installation_for_runtime(team_id, employee_id);
    }
    installation_from_env(employee_id, team_id)
}

pub fn resolve_slack_bot_token_for_runtime(
    team_id: Option<&str>,
    employee_id: Option<&str>,
) -> Option<String> {
    resolve_slack_installation_for_runtime(team_id, employee_id)
        .map(|installation| installation.bot_token)
}

pub fn resolve_slack_bot_user_id_for_runtime(
    team_id: Option<&str>,
    employee_id: Option<&str>,
) -> Option<String> {
    resolve_slack_installation_for_runtime(team_id, employee_id)
        .map(|installation| installation.bot_user_id)
        .filter(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::{Mutex, OnceLock};
    use tempfile::TempDir;
    use uuid::Uuid;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvGuard {
        key: &'static str,
        original: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
            let original = env::var(key).ok();
            env::set_var(key, value);
            Self { key, original }
        }

        fn clear(key: &'static str) -> Self {
            let original = env::var(key).ok();
            env::remove_var(key);
            Self { key, original }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(value) => env::set_var(self.key, value),
                None => env::remove_var(self.key),
            }
        }
    }

    fn test_store() -> (TempDir, SlackStore) {
        let temp = TempDir::new().expect("tempdir");
        let path = temp.path().join("slack.db");
        let store = SlackStore::new(&path).expect("store");
        (temp, store)
    }

    fn unique_team_id(prefix: &str) -> String {
        format!("{prefix}-{}", Uuid::new_v4().simple())
    }

    #[test]
    fn upsert_and_get_installation() {
        let (_temp, store) = test_store();
        let team_id = unique_team_id("T12345");

        let installation = SlackInstallation {
            team_id: team_id.clone(),
            team_name: Some("Test Workspace".to_string()),
            bot_token: "xoxb-test-token".to_string(),
            bot_user_id: "U12345".to_string(),
            installed_at: Utc::now(),
        };

        store.upsert_installation(&installation).expect("upsert");

        let retrieved = store.get_installation(&team_id).expect("get");
        assert_eq!(retrieved.team_id, team_id);
        assert_eq!(retrieved.team_name, Some("Test Workspace".to_string()));
        assert_eq!(retrieved.bot_token, "xoxb-test-token");
        assert_eq!(retrieved.bot_user_id, "U12345");
    }

    #[test]
    fn upsert_updates_existing() {
        let (_temp, store) = test_store();
        let team_id = unique_team_id("T12345");

        let installation1 = SlackInstallation {
            team_id: team_id.clone(),
            team_name: Some("Old Name".to_string()),
            bot_token: "xoxb-old-token".to_string(),
            bot_user_id: "U12345".to_string(),
            installed_at: Utc::now(),
        };
        store.upsert_installation(&installation1).expect("upsert1");

        let installation2 = SlackInstallation {
            team_id: team_id.clone(),
            team_name: Some("New Name".to_string()),
            bot_token: "xoxb-new-token".to_string(),
            bot_user_id: "U67890".to_string(),
            installed_at: Utc::now(),
        };
        store.upsert_installation(&installation2).expect("upsert2");

        let retrieved = store.get_installation(&team_id).expect("get");
        assert_eq!(retrieved.team_name, Some("New Name".to_string()));
        assert_eq!(retrieved.bot_token, "xoxb-new-token");
        assert_eq!(retrieved.bot_user_id, "U67890");
    }

    #[test]
    fn get_not_found() {
        let (_temp, store) = test_store();

        let team_id = unique_team_id("TNOTEXIST");
        let result = store.get_installation(&team_id);
        assert!(matches!(result, Err(SlackStoreError::NotFound(_))));
    }

    #[test]
    fn delete_installation() {
        let (_temp, store) = test_store();
        let team_id = unique_team_id("T12345");

        let installation = SlackInstallation {
            team_id: team_id.clone(),
            team_name: None,
            bot_token: "xoxb-test".to_string(),
            bot_user_id: "U12345".to_string(),
            installed_at: Utc::now(),
        };
        store.upsert_installation(&installation).expect("upsert");

        let deleted = store.delete_installation(&team_id).expect("delete");
        assert!(deleted);

        let result = store.get_installation(&team_id);
        assert!(matches!(result, Err(SlackStoreError::NotFound(_))));
    }

    #[test]
    fn list_installations() {
        let (_temp, store) = test_store();
        let team_ids: Vec<String> = (1..=3).map(|i| unique_team_id(&format!("T{i}"))).collect();

        for (i, team_id) in team_ids.iter().enumerate() {
            let installation = SlackInstallation {
                team_id: team_id.clone(),
                team_name: Some(format!("Workspace {}", i + 1)),
                bot_token: format!("xoxb-token-{}", i + 1),
                bot_user_id: format!("U{}", i + 1),
                installed_at: Utc::now(),
            };
            store.upsert_installation(&installation).expect("upsert");
        }

        let list = store.list_installations().expect("list");
        let ids: std::collections::HashSet<String> =
            list.into_iter().map(|value| value.team_id).collect();
        for team_id in team_ids {
            assert!(ids.contains(&team_id));
        }
    }

    #[test]
    fn resolve_installation_for_runtime_prefers_store_team_over_env() {
        let _env_guard = env_lock().lock().expect("env lock");
        let (_temp, store) = test_store();
        let team_id = unique_team_id("T12345");
        let _employee_token = EnvGuard::set("LITTLE_BEAR_SLACK_BOT_TOKEN", "xoxb-env-token");
        let _employee_user = EnvGuard::set("LITTLE_BEAR_SLACK_BOT_USER_ID", "UENV");
        let _global_token = EnvGuard::set("SLACK_BOT_TOKEN", "xoxb-global-token");
        let _global_user = EnvGuard::set("SLACK_BOT_USER_ID", "UGLOBAL");

        let installation = SlackInstallation {
            team_id: team_id.clone(),
            team_name: Some("Store Workspace".to_string()),
            bot_token: "xoxb-store-token".to_string(),
            bot_user_id: "USTORE".to_string(),
            installed_at: Utc::now(),
        };
        store.upsert_installation(&installation).expect("upsert");

        let resolved = store
            .resolve_installation_for_runtime(Some(&team_id), Some("little_bear"))
            .expect("resolved installation");
        assert_eq!(resolved.bot_token, "xoxb-store-token");
        assert_eq!(resolved.bot_user_id, "USTORE");
        assert_eq!(resolved.team_id, team_id);
    }

    #[test]
    fn resolve_installation_for_runtime_falls_back_to_employee_env() {
        let _env_guard = env_lock().lock().expect("env lock");
        let (_temp, store) = test_store();
        let team_id = unique_team_id("TMISSING");
        let _employee_token = EnvGuard::set("LITTLE_BEAR_SLACK_BOT_TOKEN", "xoxb-employee-token");
        let _employee_user = EnvGuard::set("LITTLE_BEAR_SLACK_BOT_USER_ID", "UEMP");
        let _global_token = EnvGuard::clear("SLACK_BOT_TOKEN");
        let _global_user = EnvGuard::clear("SLACK_BOT_USER_ID");

        let resolved = store
            .resolve_installation_for_runtime(Some(&team_id), Some("little_bear"))
            .expect("resolved installation");
        assert_eq!(resolved.bot_token, "xoxb-employee-token");
        assert_eq!(resolved.bot_user_id, "UEMP");
        assert_eq!(resolved.team_id, team_id);
    }

    #[test]
    fn resolve_installation_for_runtime_falls_back_to_global_env() {
        let _env_guard = env_lock().lock().expect("env lock");
        let (_temp, store) = test_store();
        let team_id = unique_team_id("TMISSING");
        let _employee_token = EnvGuard::clear("LITTLE_BEAR_SLACK_BOT_TOKEN");
        let _employee_user = EnvGuard::clear("LITTLE_BEAR_SLACK_BOT_USER_ID");
        let _global_token = EnvGuard::set("SLACK_BOT_TOKEN", "xoxb-global-token");
        let _global_user = EnvGuard::set("SLACK_BOT_USER_ID", "UGLOBAL");

        let resolved = store
            .resolve_installation_for_runtime(Some(&team_id), Some("little_bear"))
            .expect("resolved installation");
        assert_eq!(resolved.bot_token, "xoxb-global-token");
        assert_eq!(resolved.bot_user_id, "UGLOBAL");
        assert_eq!(resolved.team_id, team_id);
    }
}
