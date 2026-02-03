# Tests for run_task_module

There is no standalone Cargo crate in MVP_rust/run_task_module yet, so tests are
tracked as manual checks for now.

## Manual smoke test

1) Prepare a workspace with the required folders:

```
workspace/
  incoming_email/
  incoming_attachments/
  memory/
  references/
```

2) Ensure AZURE_OPENAI_API_KEY_BACKUP and AZURE_OPENAI_ENDPOINT_BACKUP are set.
3) Run a small Rust harness that calls run_codex_task from run_task.rs.
4) Verify the outputs:
   - reply_email_draft.html exists in the workspace root.
   - reply_email_attachments/ exists (and contains any generated attachments).

## Future automated tests

- Add a lightweight Cargo crate under MVP_rust/run_task_module and wire
  integration tests that mock the Codex CLI via a wrapper script.
