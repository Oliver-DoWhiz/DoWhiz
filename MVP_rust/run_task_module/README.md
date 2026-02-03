# Run task module (Codex CLI)

This folder contains a single Rust module for running the Codex CLI to draft an
HTML email reply from a workspace on disk. The module is designed to be copied
into an existing Rust crate and called as a helper.

## Manual usage

1) Install the Codex CLI and ensure it is on your PATH.
2) Set environment variables (or place them in a .env file at or above the workspace):
   - AZURE_OPENAI_API_KEY_BACKUP
   - AZURE_OPENAI_ENDPOINT_BACKUP
   - Optional: CODEX_MODEL (default: gpt-5.2-codex)
3) Create a workspace with input folders:

```
workspace/
  incoming_email/
  incoming_attachments/
  memory/
  references/
```

4) Call the module from your Rust binary (example):

```rust
mod run_task;

use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request = run_task::RunTaskRequest {
        workspace_dir: Path::new("/path/to/workspace"),
        input_email_dir: Path::new("incoming_email"),
        input_attachments_dir: Path::new("incoming_attachments"),
        memory_dir: Path::new("memory"),
        reference_dir: Path::new("references"),
    };

    let result = run_task::run_codex_task(request)?;
    println!("Draft written to {}", result.reply_html_path.display());
    Ok(())
}
```

After the call completes, the module expects:

```
workspace/
  reply_email_draft.html
  reply_email_attachments/
```

## Folder and file roles

Working files (MVP_rust/run_task_module/):
- MVP_rust/run_task_module/run_task.rs: the single Rust module that loads env
  variables, updates ~/.codex/config.toml, builds the prompt, and runs Codex CLI.
- MVP_rust/run_task_module/README.md: this usage guide.

Tests (MVP_rust/run_task_module/tests/):
- MVP_rust/run_task_module/tests/README.md: manual test checklist and notes.

Other files used by the module:
- ~/.codex/config.toml: updated or created to include Azure settings.
- .env: discovered by walking up from the workspace (or current directory) and
  loaded only if variables are not already set.

## Notes

- The input directories must be relative to the workspace and must exist.
- The prompt instructs Codex to use the highest attachment version when file
  names contain suffixes like _v1, _v2, etc.
- Codex is run with --dangerously-bypass-approvals-and-sandbox. Adjust the
  flags in run_task.rs if you need stricter controls.
