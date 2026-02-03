# send_emails_module

A small Rust module that sends a single Postmark email using an HTML draft file and an attachments folder.

## Manual usage

1) Set environment variables:

- `POSTMARK_SERVER_TOKEN` (required)
- `OUTBOUND_FROM` (optional, defaults to `oliver@dowhiz.com`)

2) Prepare files:

- `reply_email_draft.html` contains the HTML body.
- `reply_email_attachments/` contains files to attach (no subfolders).

3) Call the function:

```rust
use send_emails_module::{send_email, SendEmailRequest};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let to = vec!["mini-mouse@deep-tutor.com".to_string()];
    let cc: Vec<String> = Vec::new();
    let bcc: Vec<String> = Vec::new();

    let request = SendEmailRequest {
        html_path: Path::new("/path/to/reply_email_draft.html"),
        attachments_dir: Path::new("/path/to/reply_email_attachments"),
        to: &to,
        cc: &cc,
        bcc: &bcc,
        subject: "Reply from DoWhiz",
        from: None,
        postmark_token: None,
        api_base_url: None,
    };

    let response = send_email(request)?;
    println!("Message ID: {}", response.message_id);
    Ok(())
}
```

Notes:
- The module always sends immediately. There is no scheduling in this MVP.
- Attachments are read from the directory (files only) and sent as base64.
- Recipients are joined into comma-separated `To`, `Cc`, and `Bcc` fields.

## Tests

The tests live under `MVP_rust/send_emails_module/tests/` and include:

- A mock test that verifies recipients and attachments are serialized correctly.
- A live Postmark test that sends a real email and polls delivery status.

To run the live test, set:

- `POSTMARK_LIVE_TEST=1`
- `POSTMARK_SERVER_TOKEN`
- `OUTBOUND_FROM` (optional)
- `POSTMARK_TEST_TO` (optional, defaults to `mini-mouse@deep-tutor.com`)
- `POSTMARK_TEST_CC` / `POSTMARK_TEST_BCC` (optional)

Then run:

```bash
cargo test --package send_emails_module
```

## Folder layout

`MVP_rust/send_emails_module/`
- `Cargo.toml`: Rust crate metadata and dependencies.
- `src/lib.rs`: Re-exports for the module.
- `src/send_emails.rs`: The single-file module that builds payloads and calls Postmark.
- `tests/send_emails_integration.rs`: Mock and live integration tests.

Other files:
- The tests optionally read `.env` from the repo root to load environment variables for Postmark.
