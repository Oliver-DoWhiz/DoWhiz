//! Warm container pool manager with Azure Queue-based task assignment.
//!
//! Maintains a pool of pre-provisioned ACI containers that poll an Azure Queue
//! for task assignments. This eliminates the 2-4 minute cold start latency per task.
//!
//! Architecture:
//! - Pool manager provisions N containers at startup, each running `warm_worker.sh`
//! - Containers poll the task queue for work
//! - Scheduler pushes tasks to queue with share credentials
//! - Container downloads workspace, runs agent, uploads results, signals completion
//! - Scheduler polls completion queue, then calls replenish()

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use uuid::Uuid;

const DEFAULT_POOL_SIZE: usize = 5;
const CONTAINER_PREFIX: &str = "dwz-warm-";

/// Configuration for the warm container pool.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Azure resource group for ACI containers
    pub resource_group: String,
    /// Container image to use
    pub image: String,
    /// CPU cores per container
    pub cpu: String,
    /// Memory in GB per container
    pub memory_gb: String,
    /// Azure Storage account for queues
    pub queue_storage_account: String,
    /// Azure Storage account key
    pub queue_storage_key: String,
    /// Queue name for task submissions
    pub task_queue_name: String,
    /// Queue name for completion signals
    pub completion_queue_name: String,
    /// ACR registry server
    pub registry_server: String,
    /// ACR registry username
    pub registry_username: String,
    /// ACR registry password
    pub registry_password: String,
}

/// Manages a pool of warm ACI containers.
pub struct PoolManager {
    config: PoolConfig,
    active_count: AtomicUsize,
    target_size: usize,
}

impl PoolManager {
    /// Create a new pool manager with the given configuration.
    pub fn new(config: PoolConfig, target_size: Option<usize>) -> Self {
        Self {
            config,
            active_count: AtomicUsize::new(0),
            target_size: target_size.unwrap_or(DEFAULT_POOL_SIZE),
        }
    }

    /// Initialize the pool by provisioning N warm containers.
    /// Containers start polling the task queue immediately.
    pub async fn initialize(&self) -> Result<(), String> {
        eprintln!(
            "[pool_manager] Initializing warm pool with {} containers",
            self.target_size
        );

        // Ensure queues exist (idempotent)
        ensure_queue_exists(&self.config, &self.config.task_queue_name)?;
        ensure_queue_exists(&self.config, &self.config.completion_queue_name)?;

        let mut handles = Vec::new();
        for _ in 0..self.target_size {
            let config = self.config.clone();
            handles.push(tokio::spawn(async move {
                provision_warm_container(&config).await
            }));
        }

        for handle in handles {
            match handle.await {
                Ok(Ok(name)) => {
                    self.active_count.fetch_add(1, Ordering::SeqCst);
                    eprintln!("[pool_manager] Provisioned: {}", name);
                }
                Ok(Err(e)) => eprintln!("[pool_manager] Provision failed: {}", e),
                Err(e) => eprintln!("[pool_manager] Task join failed: {}", e),
            }
        }

        eprintln!(
            "[pool_manager] Pool ready with {} containers",
            self.active_count.load(Ordering::SeqCst)
        );
        Ok(())
    }

    /// Called after a task completes to replenish the pool.
    /// Decrements active count and spawns a background provision task.
    pub fn replenish(&self) {
        let previous = self.active_count.fetch_sub(1, Ordering::SeqCst);
        eprintln!(
            "[pool_manager] Container finished, active: {} -> {}",
            previous,
            previous - 1
        );

        let current = self.active_count.load(Ordering::SeqCst);
        if current >= self.target_size {
            eprintln!("[pool_manager] Pool at target size, skipping replenish");
            return;
        }

        let config = self.config.clone();
        let active_count = &self.active_count as *const AtomicUsize as usize;

        tokio::spawn(async move {
            eprintln!("[pool_manager] Replenishing pool...");
            match provision_warm_container(&config).await {
                Ok(name) => {
                    // Safe: we're just incrementing an atomic counter
                    let counter =
                        unsafe { &*(active_count as *const AtomicUsize) };
                    counter.fetch_add(1, Ordering::SeqCst);
                    eprintln!("[pool_manager] Replenished with: {}", name);
                }
                Err(e) => eprintln!("[pool_manager] Replenish failed: {}", e),
            }
        });
    }

    /// Get current number of active containers.
    pub fn active_count(&self) -> usize {
        self.active_count.load(Ordering::SeqCst)
    }

    /// Get target pool size.
    pub fn target_size(&self) -> usize {
        self.target_size
    }

    /// Get task queue name.
    pub fn task_queue(&self) -> &str {
        &self.config.task_queue_name
    }

    /// Get completion queue name.
    pub fn completion_queue(&self) -> &str {
        &self.config.completion_queue_name
    }

    /// Get storage account name.
    pub fn storage_account(&self) -> &str {
        &self.config.queue_storage_account
    }

    /// Get storage account key.
    pub fn storage_key(&self) -> &str {
        &self.config.queue_storage_key
    }

    /// Get the pool configuration.
    pub fn config(&self) -> &PoolConfig {
        &self.config
    }
}

/// Provision a single warm container that polls the task queue.
async fn provision_warm_container(config: &PoolConfig) -> Result<String, String> {
    let container_name = format!("{}{}", CONTAINER_PREFIX, Uuid::new_v4().simple());

    eprintln!("[pool_manager] Provisioning container: {}", container_name);

    let output = tokio::task::spawn_blocking({
        let config = config.clone();
        let container_name = container_name.clone();
        move || {
            Command::new("az")
                .arg("container")
                .arg("create")
                .arg("--resource-group")
                .arg(&config.resource_group)
                .arg("--name")
                .arg(&container_name)
                .arg("--image")
                .arg(&config.image)
                .arg("--cpu")
                .arg(&config.cpu)
                .arg("--memory")
                .arg(&config.memory_gb)
                .arg("--restart-policy")
                .arg("Never")
                .arg("--registry-login-server")
                .arg(&config.registry_server)
                .arg("--registry-username")
                .arg(&config.registry_username)
                .arg("--registry-password")
                .arg(&config.registry_password)
                .arg("--environment-variables")
                .arg(format!("TASK_QUEUE_NAME={}", config.task_queue_name))
                .arg(format!("COMPLETION_QUEUE_NAME={}", config.completion_queue_name))
                .arg(format!("QUEUE_STORAGE_ACCOUNT={}", config.queue_storage_account))
                .arg(format!("QUEUE_STORAGE_KEY={}", config.queue_storage_key))
                .arg("--command-line")
                .arg("/bin/bash -lc 'warm_worker.sh'")
                .output()
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("az command failed: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "az container create failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(container_name)
}

/// Ensure an Azure Storage Queue exists (idempotent).
fn ensure_queue_exists(config: &PoolConfig, queue_name: &str) -> Result<(), String> {
    eprintln!("[pool_manager] Ensuring queue exists: {}", queue_name);

    let output = Command::new("az")
        .arg("storage")
        .arg("queue")
        .arg("create")
        .arg("--name")
        .arg(queue_name)
        .arg("--account-name")
        .arg(&config.queue_storage_account)
        .arg("--account-key")
        .arg(&config.queue_storage_key)
        .arg("--output")
        .arg("none")
        .output()
        .map_err(|e| format!("az command failed: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "az storage queue create failed for {}: {}",
            queue_name,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Delete an ACI container.
pub fn delete_container(resource_group: &str, container_name: &str) -> Result<(), String> {
    let output = Command::new("az")
        .arg("container")
        .arg("delete")
        .arg("--resource-group")
        .arg(resource_group)
        .arg("--name")
        .arg(container_name)
        .arg("--yes")
        .output()
        .map_err(|e| format!("az command failed: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "az container delete failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_manager_new() {
        let config = PoolConfig {
            resource_group: "test-rg".to_string(),
            image: "test-image".to_string(),
            cpu: "2.0".to_string(),
            memory_gb: "4.0".to_string(),
            queue_storage_account: "teststorage".to_string(),
            queue_storage_key: "testkey".to_string(),
            task_queue_name: "test-tasks".to_string(),
            completion_queue_name: "test-completions".to_string(),
            registry_server: "testregistry.azurecr.io".to_string(),
            registry_username: "testuser".to_string(),
            registry_password: "testpass".to_string(),
        };

        let manager = PoolManager::new(config, Some(5));
        assert_eq!(manager.target_size(), 5);
        assert_eq!(manager.active_count(), 0);
        assert_eq!(manager.task_queue(), "test-tasks");
        assert_eq!(manager.completion_queue(), "test-completions");
    }

    #[test]
    fn test_default_pool_size() {
        let config = PoolConfig {
            resource_group: "test-rg".to_string(),
            image: "test-image".to_string(),
            cpu: "2.0".to_string(),
            memory_gb: "4.0".to_string(),
            queue_storage_account: "teststorage".to_string(),
            queue_storage_key: "testkey".to_string(),
            task_queue_name: "test-tasks".to_string(),
            completion_queue_name: "test-completions".to_string(),
            registry_server: "testregistry.azurecr.io".to_string(),
            registry_username: "testuser".to_string(),
            registry_password: "testpass".to_string(),
        };

        let manager = PoolManager::new(config, None);
        assert_eq!(manager.target_size(), 5); // DEFAULT_POOL_SIZE
    }
}
