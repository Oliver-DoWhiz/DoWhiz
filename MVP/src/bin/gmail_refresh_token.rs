use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use reqwest::Client;
use serde::Deserialize;

const DEFAULT_SCOPE: &str = "https://www.googleapis.com/auth/gmail.send";
const ENV_KEY: &str = "GMAIL_REFRESH_TOKEN";

#[derive(Debug, Deserialize)]
struct CredentialsFile {
    web: Option<Credentials>,
    installed: Option<Credentials>,
}

#[derive(Debug, Deserialize, Clone)]
struct Credentials {
    client_id: String,
    client_secret: String,
    auth_uri: String,
    token_uri: String,
    redirect_uris: Option<Vec<String>>,
}

#[derive(Debug)]
struct Args {
    credentials_path: PathBuf,
    env_path: PathBuf,
    redirect_uri: Option<String>,
    scopes: Vec<String>,
    prompt: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    let credentials = load_credentials(&args.credentials_path)?;
    let redirect_uri = resolve_redirect_uri(&credentials, args.redirect_uri)?;
    let scopes = if args.scopes.is_empty() {
        vec![DEFAULT_SCOPE.to_string()]
    } else {
        args.scopes
    };

    let auth_url = build_auth_url(&credentials, &redirect_uri, &scopes, args.prompt.as_deref())?;

    println!("Open this URL in your browser and approve access:\n\n{}\n", auth_url);
    println!("After approval, paste the FULL redirected URL here (preferred), or paste the code only:");
    print!("> ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    let code = extract_code(input).ok_or("No authorization code found in input")?;

    let refresh_token = exchange_code_for_refresh_token(&credentials, &redirect_uri, &code).await?;
    upsert_env_var(&args.env_path, ENV_KEY, &refresh_token)?;

    println!("Saved {} to {}", ENV_KEY, args.env_path.display());
    Ok(())
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let mut credentials_path = PathBuf::from(".gmail_client_secret.json");
    let mut env_path = PathBuf::from(".env");
    let mut redirect_uri = None;
    let mut scopes = Vec::new();
    let mut prompt = Some("consent".to_string());

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--credentials" => {
                let value = args.next().ok_or("--credentials requires a path")?;
                credentials_path = PathBuf::from(value);
            }
            "--env-path" => {
                let value = args.next().ok_or("--env-path requires a path")?;
                env_path = PathBuf::from(value);
            }
            "--redirect-uri" => {
                let value = args.next().ok_or("--redirect-uri requires a value")?;
                redirect_uri = Some(value);
            }
            "--scope" => {
                let value = args.next().ok_or("--scope requires a value")?;
                scopes.push(value);
            }
            "--no-prompt" => {
                prompt = None;
            }
            "--prompt" => {
                let value = args.next().ok_or("--prompt requires a value")?;
                prompt = Some(value);
            }
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            _ => return Err(format!("Unknown argument: {}", arg).into()),
        }
    }

    Ok(Args {
        credentials_path,
        env_path,
        redirect_uri,
        scopes,
        prompt,
    })
}

fn print_usage() {
    println!(
        "Usage: gmail_refresh_token [options]\n\n
Options:\n  \
  --credentials <path>   Path to OAuth client credentials JSON (default: .gmail_client_secret.json)\n  \
  --env-path <path>      Path to .env file (default: .env)\n  \
  --redirect-uri <uri>   Override redirect URI (defaults to first in credentials or http://localhost)\n  \
  --scope <scope>        Add a scope (repeatable; default: {scope})\n  \
  --prompt <value>       OAuth prompt value (default: consent)\n  \
  --no-prompt            Omit the prompt parameter\n  \
  -h, --help             Show this help\n",
        scope = DEFAULT_SCOPE
    );
}

fn load_credentials(path: &Path) -> Result<Credentials, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let file: CredentialsFile = serde_json::from_str(&contents)?;
    if let Some(web) = file.web {
        return Ok(web);
    }
    if let Some(installed) = file.installed {
        return Ok(installed);
    }
    Err("Expected top-level 'web' or 'installed' credentials".into())
}

fn resolve_redirect_uri(
    credentials: &Credentials,
    override_uri: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(uri) = override_uri {
        return Ok(uri);
    }
    if let Some(list) = credentials.redirect_uris.as_ref() {
        if let Some(uri) = list.first() {
            return Ok(uri.clone());
        }
    }
    Ok("http://localhost".to_string())
}

fn build_auth_url(
    credentials: &Credentials,
    redirect_uri: &str,
    scopes: &[String],
    prompt: Option<&str>,
) -> Result<reqwest::Url, Box<dyn std::error::Error>> {
    let mut url = reqwest::Url::parse(&credentials.auth_uri)?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("client_id", &credentials.client_id);
        pairs.append_pair("redirect_uri", redirect_uri);
        pairs.append_pair("response_type", "code");
        pairs.append_pair("scope", &scopes.join(" "));
        pairs.append_pair("access_type", "offline");
        if let Some(prompt_value) = prompt {
            pairs.append_pair("prompt", prompt_value);
        }
    }
    Ok(url)
}

fn extract_code(input: &str) -> Option<String> {
    if input.starts_with("http://") || input.starts_with("https://") {
        if let Ok(url) = reqwest::Url::parse(input) {
            for (key, value) in url.query_pairs() {
                if key == "code" {
                    return Some(value.into_owned());
                }
            }
        }
    }

    if let Some(pos) = input.find("code=") {
        let after = &input[pos + 5..];
        let raw = after.split('&').next().unwrap_or("");
        return Some(percent_decode(raw));
    }

    if input.is_empty() {
        None
    } else {
        Some(input.to_string())
    }
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(high), Some(low)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push(high * 16 + low);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            out.push(b' ');
        } else {
            out.push(bytes[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn from_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

async fn exchange_code_for_refresh_token(
    credentials: &Credentials,
    redirect_uri: &str,
    code: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client
        .post(&credentials.token_uri)
        .form(&[
            ("code", code),
            ("client_id", credentials.client_id.as_str()),
            ("client_secret", credentials.client_secret.as_str()),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_else(|_| "<failed to read body>".to_string());
        return Err(format!("Token exchange failed ({}): {}", status, body).into());
    }

    let payload: serde_json::Value = response.json().await?;
    if let Some(token) = payload.get("refresh_token").and_then(|v| v.as_str()) {
        if !token.is_empty() {
            return Ok(token.to_string());
        }
    }

    Err("No refresh_token returned. Re-consent may be required for existing grants.".into())
}

fn upsert_env_var(path: &Path, key: &str, value: &str) -> io::Result<()> {
    let mut contents = String::new();
    if path.exists() {
        let mut file = fs::File::open(path)?;
        file.read_to_string(&mut contents)?;
    }

    let mut found = false;
    let mut lines: Vec<String> = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(&format!("{}=", key)) {
            lines.push(format!("{}=\"{}\"", key, value));
            found = true;
        } else {
            lines.push(line.to_string());
        }
    }

    if !found {
        lines.push(format!("{}=\"{}\"", key, value));
    }

    let mut output = lines.join("\n");
    output.push('\n');
    fs::write(path, output)
}
