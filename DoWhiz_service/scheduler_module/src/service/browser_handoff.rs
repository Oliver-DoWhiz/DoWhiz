use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
#[cfg(test)]
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};

const BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY: &str = "BROWSER_HANDOFF_SIGNING_SECRET";
const CHAT_HISTORY_SCOPE_SIGNING_SECRET_ENV_KEY: &str = "CHAT_HISTORY_SCOPE_SIGNING_SECRET";
const SLACK_SIGNING_SECRET_ENV_KEY: &str = "SLACK_SIGNING_SECRET";
const BROWSERBASE_API_KEY_ENV_KEY: &str = "BROWSERBASE_API_KEY";
const BROWSERBASE_API_KEY_ALIAS_ENV_KEY: &str = "BROWSER_BASE_API_KEY";
const BROWSERBASE_API_BASE_URL_ENV_KEY: &str = "BROWSERBASE_API_BASE_URL";
const DEFAULT_BROWSERBASE_API_BASE_URL: &str = "https://api.browserbase.com";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BrowserHandoffGrant {
    version: u8,
    challenge_id: String,
    session_id: String,
    #[serde(default)]
    page_id: Option<String>,
    iat: usize,
    exp: usize,
}

#[derive(Debug, Deserialize)]
struct BrowserHandoffQuery {
    token: String,
}

#[derive(Debug, Deserialize)]
struct BrowserbaseDebugUrls {
    #[serde(rename = "debuggerUrl")]
    debugger_url: Option<String>,
    #[serde(rename = "debuggerFullscreenUrl")]
    debugger_fullscreen_url: Option<String>,
    #[serde(default)]
    pages: Vec<BrowserbaseDebugPage>,
}

#[derive(Debug, Deserialize)]
struct BrowserbaseDebugPage {
    id: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    title: String,
    #[serde(rename = "debuggerUrl")]
    debugger_url: Option<String>,
    #[serde(rename = "debuggerFullscreenUrl")]
    debugger_fullscreen_url: Option<String>,
}

pub fn browser_handoff_router() -> Router {
    Router::new().route("/auth/browser-handoff", get(browser_handoff_page))
}

async fn browser_handoff_page(Query(query): Query<BrowserHandoffQuery>) -> Response {
    let claims = match decode_browser_handoff_grant(&query.token) {
        Ok(claims) => claims,
        Err((status, message)) => {
            return html_error(status, "Browser Handoff Unavailable", &message)
        }
    };

    let debug_urls = match fetch_browserbase_debug_urls(&claims.session_id).await {
        Ok(payload) => payload,
        Err((status, message)) => {
            return html_error(status, "Browser Handoff Unavailable", &message)
        }
    };

    let Some(target_url) = select_debug_url(&claims, &debug_urls) else {
        return html_error(
            StatusCode::BAD_GATEWAY,
            "Browser Handoff Unavailable",
            "Browserbase did not return a usable live debugger URL for this session.",
        );
    };

    let page = render_browser_handoff_html(&claims, &target_url, &debug_urls);
    (StatusCode::OK, Html(page)).into_response()
}

fn decode_browser_handoff_grant(token: &str) -> Result<BrowserHandoffGrant, (StatusCode, String)> {
    let secret = resolve_browser_handoff_signing_secret().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Browser handoff signing secret is not configured.".to_string(),
        )
    })?;
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.required_spec_claims.insert("exp".to_string());
    validation.required_spec_claims.insert("iat".to_string());
    let decoded = jsonwebtoken::decode::<BrowserHandoffGrant>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|err| {
        (
            StatusCode::UNAUTHORIZED,
            format!("This browser handoff link is invalid or expired ({err})."),
        )
    })?;
    Ok(decoded.claims)
}

#[cfg(test)]
fn encode_browser_handoff_grant(
    grant: &BrowserHandoffGrant,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        grant,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

async fn fetch_browserbase_debug_urls(
    session_id: &str,
) -> Result<BrowserbaseDebugUrls, (StatusCode, String)> {
    let api_key = env_trimmed(BROWSERBASE_API_KEY_ENV_KEY)
        .or_else(|| env_trimmed(BROWSERBASE_API_KEY_ALIAS_ENV_KEY))
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Browserbase API key is not configured on the server.".to_string(),
            )
        })?;
    let api_base = env_trimmed(BROWSERBASE_API_BASE_URL_ENV_KEY)
        .unwrap_or_else(|| DEFAULT_BROWSERBASE_API_BASE_URL.to_string());
    let session_id = urlencoding::encode(session_id);
    let url = format!(
        "{}/v1/sessions/{}/debug",
        api_base.trim_end_matches('/'),
        session_id
    );

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("X-BB-API-Key", api_key)
        .send()
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_GATEWAY,
                format!("Failed to reach Browserbase debug API: {err}"),
            )
        })?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err((
            StatusCode::BAD_GATEWAY,
            format!(
                "Browserbase debug API returned {}: {}",
                status,
                truncate(&body, 500)
            ),
        ));
    }

    response
        .json::<BrowserbaseDebugUrls>()
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_GATEWAY,
                format!("Browserbase debug response could not be parsed: {err}"),
            )
        })
}

fn select_debug_url(
    claims: &BrowserHandoffGrant,
    payload: &BrowserbaseDebugUrls,
) -> Option<String> {
    if let Some(page_id) = claims.page_id.as_deref() {
        if let Some(url) = payload
            .pages
            .iter()
            .find(|page| page.id == page_id)
            .and_then(page_debug_url)
        {
            return Some(url);
        }
    }

    if let Some(page) = select_fallback_page(payload) {
        if let Some(url) = page_debug_url(page) {
            return Some(url);
        }
    }

    payload
        .debugger_fullscreen_url
        .clone()
        .or_else(|| payload.debugger_url.clone())
        .or_else(|| payload.pages.iter().find_map(page_debug_url))
}

fn page_debug_url(page: &BrowserbaseDebugPage) -> Option<String> {
    page.debugger_fullscreen_url
        .clone()
        .or_else(|| page.debugger_url.clone())
}

fn select_fallback_page(payload: &BrowserbaseDebugUrls) -> Option<&BrowserbaseDebugPage> {
    payload
        .pages
        .iter()
        .rev()
        .find(|page| page_debug_url(page).is_some() && page_looks_live(page))
        .or_else(|| {
            payload
                .pages
                .iter()
                .rev()
                .find(|page| page_debug_url(page).is_some())
        })
}

fn page_looks_live(page: &BrowserbaseDebugPage) -> bool {
    !page_text_is_blank(&page.url) || !page_text_is_blank(&page.title)
}

fn page_text_is_blank(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    normalized.is_empty()
        || normalized == "about:blank"
        || normalized == "new tab"
        || normalized.starts_with("chrome://newtab")
        || normalized.starts_with("edge://newtab")
        || normalized.starts_with("chrome-search://local-ntp")
}

fn render_browser_handoff_html(
    claims: &BrowserHandoffGrant,
    target_url: &str,
    payload: &BrowserbaseDebugUrls,
) -> String {
    let challenge_id = escape_html(&claims.challenge_id);
    let session_id = escape_html(&claims.session_id);
    let live_url = escape_html_attr(target_url);
    let open_url = escape_html(target_url);
    let page_count = payload.pages.len();
    let multi_page_note = if page_count > 1 {
        format!(
            "<p class=\"meta\">This session currently has {} open tabs. The live inspector should preserve the active tab, but you can switch tabs inside the embedded Browserbase UI if needed.</p>",
            page_count
        )
    } else {
        String::new()
    };
    let page_summary = payload
        .pages
        .iter()
        .take(3)
        .map(|page| {
            format!(
                "<li><strong>{}</strong><span>{}</span></li>",
                escape_html(if page.title.trim().is_empty() {
                    "Untitled tab"
                } else {
                    page.title.trim()
                }),
                escape_html(page.url.trim())
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let page_list = if page_summary.is_empty() {
        String::new()
    } else {
        format!("<ul class=\"pages\">{page_summary}</ul>")
    };

    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>DoWhiz Browser Handoff</title>
    <style>
      :root {{
        color-scheme: light;
        --bg: #f4f1ea;
        --panel: rgba(255, 255, 255, 0.92);
        --ink: #131312;
        --muted: #5c5b57;
        --line: rgba(19, 19, 18, 0.12);
        --accent: #1f6feb;
      }}
      * {{ box-sizing: border-box; }}
      body {{
        margin: 0;
        font-family: ui-sans-serif, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
        color: var(--ink);
        background:
          radial-gradient(circle at top right, rgba(31, 111, 235, 0.18), transparent 28rem),
          linear-gradient(180deg, #fbfaf7 0%, var(--bg) 100%);
      }}
      .layout {{
        min-height: 100vh;
        display: grid;
        grid-template-rows: auto 1fr;
        gap: 1rem;
        padding: 1rem;
      }}
      .panel {{
        background: var(--panel);
        border: 1px solid var(--line);
        border-radius: 18px;
        box-shadow: 0 20px 60px rgba(19, 19, 18, 0.08);
      }}
      .topbar {{
        padding: 1.25rem 1.5rem;
      }}
      h1 {{
        margin: 0 0 0.4rem;
        font-size: 1.4rem;
      }}
      p {{
        margin: 0.3rem 0;
        color: var(--muted);
        line-height: 1.45;
      }}
      .meta {{
        font-size: 0.95rem;
      }}
      .actions {{
        display: flex;
        gap: 0.75rem;
        flex-wrap: wrap;
        margin-top: 0.9rem;
      }}
      .button {{
        display: inline-flex;
        align-items: center;
        justify-content: center;
        padding: 0.8rem 1.1rem;
        border-radius: 999px;
        text-decoration: none;
        font-weight: 600;
        border: 1px solid transparent;
      }}
      .button.primary {{
        background: var(--ink);
        color: #fff;
      }}
      .button.secondary {{
        background: transparent;
        color: var(--ink);
        border-color: var(--line);
      }}
      .iframe-shell {{
        min-height: 0;
        overflow: hidden;
        padding: 0.75rem;
      }}
      iframe {{
        width: 100%;
        height: calc(100vh - 16rem);
        min-height: 38rem;
        border: 0;
        border-radius: 14px;
        background: #ffffff;
      }}
      .pages {{
        margin: 0.9rem 0 0;
        padding-left: 1.2rem;
        color: var(--muted);
      }}
      .pages li {{
        margin: 0.35rem 0;
      }}
      .pages span {{
        display: block;
        font-size: 0.92rem;
        color: var(--muted);
      }}
      @media (max-width: 768px) {{
        .layout {{ padding: 0.7rem; }}
        .topbar {{ padding: 1rem; }}
        iframe {{ height: calc(100vh - 18rem); min-height: 28rem; }}
      }}
    </style>
  </head>
  <body>
    <main class="layout">
      <section class="panel topbar">
        <h1>Browser handoff is ready</h1>
        <p>Challenge <strong>{challenge_id}</strong> is waiting on Browserbase session <strong>{session_id}</strong>.</p>
        <p class="meta">Complete the manual step in the live browser below, then reply to the original DoWhiz email thread so the agent knows it can continue.</p>
        {multi_page_note}
        {page_list}
        <div class="actions">
          <a class="button primary" href="{open_url}" target="_blank" rel="noopener noreferrer">Open in new tab</a>
          <a class="button secondary" href="mailto:">Reply to email after you finish</a>
        </div>
      </section>
      <section class="panel iframe-shell">
        <iframe
          src="{live_url}"
          title="DoWhiz Browser Handoff"
          allow="clipboard-read; clipboard-write"
          sandbox="allow-same-origin allow-scripts allow-forms allow-downloads allow-popups"
        ></iframe>
      </section>
    </main>
  </body>
</html>"#
    )
}

fn html_error(status: StatusCode, title: &str, message: &str) -> Response {
    let body = format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>{title}</title>
    <style>
      body {{
        margin: 0;
        min-height: 100vh;
        display: grid;
        place-items: center;
        background: linear-gradient(180deg, #faf7ef 0%, #f0ede6 100%);
        font-family: ui-sans-serif, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
        color: #171717;
      }}
      .card {{
        width: min(40rem, calc(100vw - 2rem));
        background: rgba(255, 255, 255, 0.95);
        border: 1px solid rgba(23, 23, 23, 0.12);
        border-radius: 18px;
        padding: 1.5rem;
        box-shadow: 0 18px 48px rgba(23, 23, 23, 0.08);
      }}
      h1 {{
        margin-top: 0;
        margin-bottom: 0.7rem;
      }}
      p {{
        margin: 0.35rem 0;
        line-height: 1.45;
        color: #57534e;
      }}
    </style>
  </head>
  <body>
    <section class="card">
      <h1>{title}</h1>
      <p>{message}</p>
      <p>Ask DoWhiz to send a fresh help email if you still need to continue this task.</p>
    </section>
  </body>
</html>"#,
        title = escape_html(title),
        message = escape_html(message),
    );
    (status, Html(body)).into_response()
}

fn resolve_browser_handoff_signing_secret() -> Option<String> {
    env_trimmed(BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY)
        .or_else(|| env_trimmed(CHAT_HISTORY_SCOPE_SIGNING_SECRET_ENV_KEY))
        .or_else(|| env_trimmed(SLACK_SIGNING_SECRET_ENV_KEY))
}

fn env_trimmed(key: &str) -> Option<String> {
    let value = std::env::var(key).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn truncate(value: &str, limit: usize) -> String {
    if value.len() <= limit {
        return value.to_string();
    }
    format!("{}...", &value[..limit.saturating_sub(3)])
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn escape_html_attr(value: &str) -> String {
    escape_html(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EnvGuard {
        key: &'static str,
        prev: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let prev = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, prev }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(value) = &self.prev {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    #[test]
    fn select_debug_url_prefers_specific_page() {
        let claims = BrowserHandoffGrant {
            version: 1,
            challenge_id: "hag-123".to_string(),
            session_id: "sess-123".to_string(),
            page_id: Some("page-b".to_string()),
            iat: 1,
            exp: usize::MAX,
        };
        let payload = BrowserbaseDebugUrls {
            debugger_url: Some("https://session.example.com".to_string()),
            debugger_fullscreen_url: Some("https://session-full.example.com".to_string()),
            pages: vec![
                BrowserbaseDebugPage {
                    id: "page-a".to_string(),
                    url: "https://a.example.com".to_string(),
                    title: "A".to_string(),
                    debugger_url: Some("https://page-a.example.com".to_string()),
                    debugger_fullscreen_url: Some("https://page-a-full.example.com".to_string()),
                },
                BrowserbaseDebugPage {
                    id: "page-b".to_string(),
                    url: "https://b.example.com".to_string(),
                    title: "B".to_string(),
                    debugger_url: Some("https://page-b.example.com".to_string()),
                    debugger_fullscreen_url: Some("https://page-b-full.example.com".to_string()),
                },
            ],
        };

        assert_eq!(
            select_debug_url(&claims, &payload).as_deref(),
            Some("https://page-b-full.example.com")
        );
    }

    #[test]
    fn select_debug_url_prefers_live_page_over_session_level_blank_debugger() {
        let claims = BrowserHandoffGrant {
            version: 1,
            challenge_id: "hag-123".to_string(),
            session_id: "sess-123".to_string(),
            page_id: None,
            iat: 1,
            exp: usize::MAX,
        };
        let payload = BrowserbaseDebugUrls {
            debugger_url: Some("https://session.example.com".to_string()),
            debugger_fullscreen_url: Some("https://session-full.example.com".to_string()),
            pages: vec![
                BrowserbaseDebugPage {
                    id: "page-a".to_string(),
                    url: "about:blank".to_string(),
                    title: "about:blank".to_string(),
                    debugger_url: Some("https://page-a.example.com".to_string()),
                    debugger_fullscreen_url: Some("https://page-a-full.example.com".to_string()),
                },
                BrowserbaseDebugPage {
                    id: "page-live".to_string(),
                    url: "https://example.com/login".to_string(),
                    title: "Login".to_string(),
                    debugger_url: Some("https://page-live.example.com".to_string()),
                    debugger_fullscreen_url: Some("https://page-live-full.example.com".to_string()),
                },
            ],
        };

        assert_eq!(
            select_debug_url(&claims, &payload).as_deref(),
            Some("https://page-live-full.example.com")
        );
    }

    #[test]
    fn decode_browser_handoff_grant_accepts_valid_token() {
        let _secret = EnvGuard::set(BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY, "secret");
        let grant = BrowserHandoffGrant {
            version: 1,
            challenge_id: "hag-123".to_string(),
            session_id: "sess-123".to_string(),
            page_id: None,
            iat: 1,
            exp: usize::MAX,
        };
        let token = encode_browser_handoff_grant(&grant, "secret").expect("token");

        let decoded = decode_browser_handoff_grant(&token).expect("decode");

        assert_eq!(decoded.challenge_id, "hag-123");
        assert_eq!(decoded.session_id, "sess-123");
    }

    #[test]
    fn render_browser_handoff_html_includes_iframe_and_summary() {
        let claims = BrowserHandoffGrant {
            version: 1,
            challenge_id: "hag-456".to_string(),
            session_id: "sess-456".to_string(),
            page_id: None,
            iat: 1,
            exp: usize::MAX,
        };
        let payload = BrowserbaseDebugUrls {
            debugger_url: Some("https://session.example.com".to_string()),
            debugger_fullscreen_url: Some("https://session-full.example.com".to_string()),
            pages: vec![BrowserbaseDebugPage {
                id: "page-a".to_string(),
                url: "https://example.com/login".to_string(),
                title: "Login".to_string(),
                debugger_url: Some("https://page-a.example.com".to_string()),
                debugger_fullscreen_url: Some("https://page-a-full.example.com".to_string()),
            }],
        };

        let html =
            render_browser_handoff_html(&claims, "https://page-a-full.example.com", &payload);

        assert!(html.contains("Browser handoff is ready"));
        assert!(html.contains("iframe"));
        assert!(html.contains("https://page-a-full.example.com"));
        assert!(html.contains("https://example.com/login"));
    }
}
