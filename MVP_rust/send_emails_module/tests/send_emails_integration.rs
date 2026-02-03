use base64::{engine::general_purpose, Engine as _};
use mockito::{Matcher, Server};
use send_emails_module::{send_email, SendEmailRequest};
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

fn load_env_file(path: &Path) {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(_) => return,
    };

    for raw in contents.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || !line.contains('=') {
            continue;
        }
        let mut parts = line.splitn(2, '=');
        let key = match parts.next() {
            Some(key) => key.trim(),
            None => continue,
        };
        let value = match parts.next() {
            Some(value) => value.trim(),
            None => continue,
        };
        if env::var_os(key).is_some() {
            continue;
        }
        let cleaned = value.trim_matches('"').trim_matches('\'');
        env::set_var(key, cleaned);
    }
}

fn unique_subject(prefix: &str) -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs();
    format!("{prefix} {ts}")
}

fn poll_outbound(
    token: &str,
    recipient: &str,
    subject_hint: &str,
    timeout: Duration,
) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    let start = SystemTime::now();

    loop {
        let url = format!(
            "https://api.postmarkapp.com/messages/outbound?recipient={}&count=50&offset=0",
            recipient
        );
        let response = client
            .get(url)
            .header("Accept", "application/json")
            .header("X-Postmark-Server-Token", token)
            .send()?;
        let body = response.text()?;
        let payload: serde_json::Value = serde_json::from_str(&body)?;
        if let Some(messages) = payload.get("Messages").and_then(|value| value.as_array()) {
            for message in messages {
                let subject = message.get("Subject").and_then(|value| value.as_str()).unwrap_or("");
                if subject.contains(subject_hint) {
                    return Ok(Some(message.clone()));
                }
            }
        }

        if start.elapsed().unwrap_or_else(|_| Duration::from_secs(0)) > timeout {
            return Ok(None);
        }
        thread::sleep(Duration::from_secs(1));
    }
}

#[test]
fn send_payload_includes_recipients_and_attachments() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let html_path = temp.path().join("reply_email_draft.html");
    fs::write(&html_path, "<p>Hello</p>")?;

    let attachments_dir = temp.path().join("reply_email_attachments");
    fs::create_dir(&attachments_dir)?;
    let attachment_a = attachments_dir.join("data.json");
    let attachment_b = attachments_dir.join("notes.txt");
    fs::write(&attachment_a, "{\"ok\": true}")?;
    fs::write(&attachment_b, "hello world")?;

    let attachments = vec![attachment_a.clone(), attachment_b.clone()];
    let expected_attachments: Vec<serde_json::Value> = attachments
        .into_iter()
        .map(|path| {
            let payload = fs::read(&path).unwrap_or_default();
            let content_type = mime_guess::from_path(&path)
                .first_or_octet_stream()
                .essence_str()
                .to_string();
            json!({
                "Name": path.file_name().unwrap().to_string_lossy(),
                "Content": general_purpose::STANDARD.encode(payload),
                "ContentType": content_type,
            })
        })
        .collect();

    let expected_payload = json!({
        "From": "sender@example.com",
        "To": "to1@example.com, to2@example.com",
        "Cc": "cc@example.com",
        "Bcc": "bcc@example.com",
        "Subject": "Test subject",
        "HtmlBody": "<p>Hello</p>",
        "Attachments": expected_attachments,
    });

    let response_body = json!({
        "To": "to1@example.com, to2@example.com",
        "SubmittedAt": "2024-01-01T00:00:00Z",
        "MessageID": "test-message-id",
        "ErrorCode": 0,
        "Message": "OK",
    });

    let mut server = Server::new();
    let api_base_url = server.url();
    let mock = server
        .mock("POST", "/email")
        .match_header("x-postmark-server-token", "test-token")
        .match_header("accept", "application/json")
        .match_header("content-type", "application/json")
        .match_body(Matcher::Json(expected_payload))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create();

    let to_list = vec!["to1@example.com".to_string(), "to2@example.com".to_string()];
    let cc_list = vec!["cc@example.com".to_string()];
    let bcc_list = vec!["bcc@example.com".to_string()];
    let request = SendEmailRequest {
        html_path: html_path.as_path(),
        attachments_dir: attachments_dir.as_path(),
        to: &to_list,
        cc: &cc_list,
        bcc: &bcc_list,
        subject: "Test subject",
        from: Some("sender@example.com"),
        postmark_token: Some("test-token"),
        api_base_url: Some(api_base_url.as_str()),
    };

    let response = send_email(request)?;
    assert_eq!(response.message_id, "test-message-id");

    mock.assert();
    Ok(())
}

#[test]
fn live_postmark_delivery_with_attachments() -> Result<(), Box<dyn std::error::Error>> {
    load_env_file(&repo_root().join(".env"));

    if env::var("POSTMARK_LIVE_TEST").unwrap_or_default() != "1" {
        eprintln!("Skipping live Postmark test. Set POSTMARK_LIVE_TEST=1 to run it.");
        return Ok(());
    }

    let token = env::var("POSTMARK_SERVER_TOKEN")
        .expect("POSTMARK_SERVER_TOKEN must be set for live tests");
    let from = env::var("OUTBOUND_FROM").unwrap_or_else(|_| "oliver@dowhiz.com".to_string());
    let to_addr =
        env::var("POSTMARK_TEST_TO").unwrap_or_else(|_| "mini-mouse@deep-tutor.com".to_string());
    let cc_addr = env::var("POSTMARK_TEST_CC").ok();
    let bcc_addr = env::var("POSTMARK_TEST_BCC").ok();

    let temp = TempDir::new()?;
    let html_path = temp.path().join("reply_email_draft.html");
    fs::write(
        &html_path,
        "<html><body><p>MVP Rust Postmark test</p></body></html>",
    )?;

    let attachments_dir = temp.path().join("reply_email_attachments");
    fs::create_dir(&attachments_dir)?;
    fs::write(attachments_dir.join("note.txt"), "postmark live test")?;

    let subject = unique_subject("MVP Rust Postmark live test");

    let to_list = vec![to_addr.clone()];
    let cc_list = cc_addr.clone().map(|addr| vec![addr]).unwrap_or_default();
    let bcc_list = bcc_addr.clone().map(|addr| vec![addr]).unwrap_or_default();

    let request = SendEmailRequest {
        html_path: html_path.as_path(),
        attachments_dir: attachments_dir.as_path(),
        to: &to_list,
        cc: &cc_list,
        bcc: &bcc_list,
        subject: &subject,
        from: Some(&from),
        postmark_token: Some(&token),
        api_base_url: None,
    };

    let response = send_email(request)?;
    assert!(!response.message_id.is_empty());

    let message = poll_outbound(&token, &to_addr, &subject, Duration::from_secs(90))?
        .expect("Timed out waiting for outbound message");
    let status = message
        .get("Status")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    assert_eq!(status, "Delivered", "Expected Delivered status");

    Ok(())
}
