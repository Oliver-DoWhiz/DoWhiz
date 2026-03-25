use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::RunTaskTask;

const USER_SECRETS_DIR: &str = "secrets";
const USER_ENV_FILENAME: &str = ".env";
const USER_BROWSERBASE_DIR: &str = "browserbase";
const WORKSPACE_SECRETS_DIR: &str = ".secrets";
const WORKSPACE_BROWSERBASE_DIR: &str = "browserbase";

pub(crate) fn resolve_user_secrets_path(task: &RunTaskTask) -> Option<PathBuf> {
    if let Some(archive_root) = task.archive_root.as_ref() {
        if let Some(user_root) = archive_root.parent() {
            return Some(user_root.join(USER_SECRETS_DIR).join(USER_ENV_FILENAME));
        }
    }
    let user_root = task.workspace_dir.parent()?.parent()?;
    Some(user_root.join(USER_SECRETS_DIR).join(USER_ENV_FILENAME))
}

pub(crate) fn sync_user_secrets_to_workspace(
    user_env_path: &Path,
    workspace_dir: &Path,
) -> Result<(), io::Error> {
    let workspace_env = workspace_env_path(workspace_dir);
    if user_env_path.exists() {
        copy_file_with_fallback(user_env_path, &workspace_env)?;
        return Ok(());
    }
    if workspace_env.exists() {
        return Ok(());
    }
    if let Some(parent) = workspace_env.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(workspace_env, "")?;
    Ok(())
}

pub(crate) fn sync_workspace_secrets_to_user(
    workspace_dir: &Path,
    user_env_path: &Path,
) -> Result<(), io::Error> {
    let workspace_env = workspace_env_path(workspace_dir);
    if !workspace_env.exists() {
        return Ok(());
    }
    if let Some(parent) = user_env_path.parent() {
        fs::create_dir_all(parent)?;
    }
    copy_file_with_fallback(&workspace_env, user_env_path)?;
    Ok(())
}

pub(crate) fn resolve_user_browserbase_state_dir(task: &RunTaskTask) -> Option<PathBuf> {
    let secrets_path = resolve_user_secrets_path(task)?;
    let secrets_dir = secrets_path.parent()?;
    Some(secrets_dir.join(USER_BROWSERBASE_DIR))
}

pub(crate) fn sync_user_browserbase_state_to_workspace(
    user_browserbase_dir: &Path,
    workspace_dir: &Path,
) -> Result<(), io::Error> {
    let workspace_browserbase_dir = workspace_browserbase_dir(workspace_dir);
    mirror_optional_dir(user_browserbase_dir, &workspace_browserbase_dir)
}

pub(crate) fn sync_workspace_browserbase_state_to_user(
    workspace_dir: &Path,
    user_browserbase_dir: &Path,
) -> Result<(), io::Error> {
    let workspace_browserbase_dir = workspace_browserbase_dir(workspace_dir);
    mirror_optional_dir(&workspace_browserbase_dir, user_browserbase_dir)
}

fn copy_file_with_fallback(src: &Path, dest: &Path) -> Result<(), io::Error> {
    match fs::copy(src, dest) {
        Ok(_) => Ok(()),
        Err(err)
            if err.kind() == io::ErrorKind::PermissionDenied || err.raw_os_error() == Some(1) =>
        {
            // Some CIFS/Azure Files mounts reject kernel fast-copy syscalls.
            // Fall back to stream copy for compatibility.
            let mut input = fs::File::open(src)?;
            let mut output = fs::File::create(dest)?;
            io::copy(&mut input, &mut output)?;
            Ok(())
        }
        Err(err) => Err(err),
    }
}

fn workspace_env_path(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join(USER_ENV_FILENAME)
}

fn workspace_browserbase_dir(workspace_dir: &Path) -> PathBuf {
    workspace_dir
        .join(WORKSPACE_SECRETS_DIR)
        .join(WORKSPACE_BROWSERBASE_DIR)
}

fn mirror_optional_dir(src: &Path, dest: &Path) -> Result<(), io::Error> {
    if !src.exists() {
        if dest.exists() {
            fs::remove_dir_all(dest)?;
        }
        fs::create_dir_all(dest)?;
        return Ok(());
    }
    if !src.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("expected directory, found file: {}", src.display()),
        ));
    }
    if dest.exists() && !dest.is_dir() {
        fs::remove_file(dest)?;
    }
    fs::create_dir_all(dest)?;
    mirror_dir_recursive(src, dest)
}

fn mirror_dir_recursive(src: &Path, dest: &Path) -> Result<(), io::Error> {
    for entry in fs::read_dir(dest)? {
        let entry = entry?;
        let dest_path = entry.path();
        let src_path = src.join(entry.file_name());
        if !src_path.exists() {
            remove_path(&dest_path)?;
            continue;
        }
        if src_path.is_dir() != dest_path.is_dir() {
            remove_path(&dest_path)?;
        }
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            if dest_path.exists() && !dest_path.is_dir() {
                remove_path(&dest_path)?;
            }
            fs::create_dir_all(&dest_path)?;
            mirror_dir_recursive(&src_path, &dest_path)?;
        } else if src_path.is_file() {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            copy_file_with_fallback(&src_path, &dest_path)?;
        }
    }

    Ok(())
}

fn remove_path(path: &Path) -> Result<(), io::Error> {
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn base_task(workspace_dir: PathBuf, archive_root: Option<PathBuf>) -> RunTaskTask {
        RunTaskTask {
            workspace_dir,
            input_email_dir: PathBuf::from("incoming_email"),
            input_attachments_dir: PathBuf::from("incoming_attachments"),
            memory_dir: PathBuf::from("memory"),
            reference_dir: PathBuf::from("references"),
            model_name: "test-model".to_string(),
            runner: "codex".to_string(),
            codex_disabled: true,
            reply_to: Vec::new(),
            reply_from: None,
            archive_root,
            thread_id: None,
            thread_epoch: None,
            thread_state_path: None,
            channel: crate::channel::Channel::default(),
            slack_team_id: None,
            employee_id: None,
            requester_identifier_type: None,
            requester_identifier: None,
            account_id: None,
            channel_metadata: Default::default(),
        }
    }

    #[test]
    fn resolve_user_secrets_path_prefers_archive_root() {
        let temp = TempDir::new().expect("tempdir");
        let user_root = temp.path().join("users").join("user_1");
        let archive_root = user_root.join("mail");
        let workspace_dir = user_root.join("workspaces").join("thread_1");
        let task = base_task(workspace_dir, Some(archive_root));
        let expected = user_root.join("secrets").join(".env");
        let resolved = resolve_user_secrets_path(&task).expect("secrets path");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn sync_user_secrets_to_workspace_copies_env() {
        let temp = TempDir::new().expect("tempdir");
        let workspace_dir = temp.path().join("workspace");
        fs::create_dir_all(&workspace_dir).expect("workspace");
        let user_env = temp.path().join("secrets").join(".env");
        fs::create_dir_all(user_env.parent().unwrap()).expect("secrets dir");
        fs::write(&user_env, "TOKEN=origin").expect("user env");

        sync_user_secrets_to_workspace(&user_env, &workspace_dir).expect("sync");

        let workspace_env = workspace_dir.join(".env");
        let contents = fs::read_to_string(workspace_env).expect("workspace env");
        assert_eq!(contents, "TOKEN=origin");
    }

    #[test]
    fn sync_user_secrets_to_workspace_creates_empty_env_when_missing() {
        let temp = TempDir::new().expect("tempdir");
        let workspace_dir = temp.path().join("workspace");
        fs::create_dir_all(&workspace_dir).expect("workspace");
        let user_env = temp.path().join("secrets").join(".env");

        sync_user_secrets_to_workspace(&user_env, &workspace_dir).expect("sync");

        let workspace_env = workspace_dir.join(".env");
        let contents = fs::read_to_string(workspace_env).expect("workspace env");
        assert!(contents.is_empty());
    }

    #[test]
    fn resolve_user_browserbase_state_dir_uses_user_secrets_dir() {
        let temp = TempDir::new().expect("tempdir");
        let user_root = temp.path().join("users").join("user_1");
        let workspace_dir = user_root.join("workspaces").join("thread_1");
        let task = base_task(workspace_dir, None);

        let resolved = resolve_user_browserbase_state_dir(&task).expect("browserbase dir");

        assert_eq!(resolved, user_root.join("secrets").join("browserbase"));
    }

    #[test]
    fn sync_user_browserbase_state_to_workspace_mirrors_directory() {
        let temp = TempDir::new().expect("tempdir");
        let workspace_dir = temp.path().join("workspace");
        fs::create_dir_all(&workspace_dir).expect("workspace");
        let user_browserbase = temp.path().join("secrets").join("browserbase");
        fs::create_dir_all(user_browserbase.join("nested")).expect("user browserbase dir");
        fs::write(
            user_browserbase.join("registry.json"),
            "{\"context\":\"ctx_1\"}",
        )
        .expect("registry");
        fs::write(user_browserbase.join("nested").join("note.txt"), "hello").expect("nested file");

        sync_user_browserbase_state_to_workspace(&user_browserbase, &workspace_dir).expect("sync");

        let workspace_browserbase = workspace_dir.join(".secrets").join("browserbase");
        assert_eq!(
            fs::read_to_string(workspace_browserbase.join("registry.json")).expect("registry"),
            "{\"context\":\"ctx_1\"}"
        );
        assert_eq!(
            fs::read_to_string(workspace_browserbase.join("nested").join("note.txt"))
                .expect("nested"),
            "hello"
        );
    }

    #[test]
    fn sync_workspace_browserbase_state_to_user_removes_deleted_files() {
        let temp = TempDir::new().expect("tempdir");
        let workspace_dir = temp.path().join("workspace");
        let workspace_browserbase = workspace_dir.join(".secrets").join("browserbase");
        fs::create_dir_all(workspace_browserbase.join("nested")).expect("workspace browserbase");
        fs::write(
            workspace_browserbase.join("registry.json"),
            "{\"context\":\"ctx_2\"}",
        )
        .expect("registry");
        let user_browserbase = temp.path().join("user-secrets").join("browserbase");
        fs::create_dir_all(user_browserbase.join("nested")).expect("user browserbase");
        fs::write(user_browserbase.join("stale.json"), "stale").expect("stale file");
        fs::write(user_browserbase.join("nested").join("old.txt"), "old").expect("old file");

        sync_workspace_browserbase_state_to_user(&workspace_dir, &user_browserbase).expect("sync");

        assert!(user_browserbase.join("stale.json").exists() == false);
        assert!(user_browserbase.join("nested").join("old.txt").exists() == false);
        assert_eq!(
            fs::read_to_string(user_browserbase.join("registry.json")).expect("registry"),
            "{\"context\":\"ctx_2\"}"
        );
    }
}
