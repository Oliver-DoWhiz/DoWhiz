//! Lark (飞书) adapter for inbound and outbound messages via Open Platform API.
//!
//! This module provides adapters for handling Lark messages:
//! - `LarkInboundAdapter`: Parses Lark webhook payloads (JSON)
//! - `LarkOutboundAdapter`: Sends messages via Lark Open Platform API

use aes::cipher::{BlockDecryptMut, KeyIvInit};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::sync::RwLock;
use tracing::info;

use crate::channel::{
    AdapterError, Channel, ChannelMetadata, InboundAdapter, InboundMessage, OutboundAdapter,
    OutboundMessage, SendResult,
};

/// Adapter for parsing Lark webhook payloads.
#[derive(Debug, Clone, Default)]
pub struct LarkInboundAdapter;

impl LarkInboundAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Decrypt Lark encrypted payload using AES-256-CBC.
    /// Key derivation: SHA256(encrypt_key)
    /// Format: base64(IV + ciphertext)
    fn decrypt_payload(encrypted: &str, encrypt_key: &str) -> Result<Vec<u8>, AdapterError> {
        // Derive AES key from encrypt_key using SHA256
        let key = Sha256::digest(encrypt_key.as_bytes());

        // Decode base64
        let encrypted_bytes = BASE64
            .decode(encrypted)
            .map_err(|e| AdapterError::ParseError(format!("invalid base64: {}", e)))?;

        if encrypted_bytes.len() < 16 {
            return Err(AdapterError::ParseError(
                "encrypted data too short".to_string(),
            ));
        }

        // First 16 bytes are IV, rest is ciphertext
        let (iv, ciphertext) = encrypted_bytes.split_at(16);

        // Decrypt using AES-256-CBC
        type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
        let mut buf = ciphertext.to_vec();
        let decrypted = Aes256CbcDec::new(key.as_slice().into(), iv.into())
            .decrypt_padded_mut::<aes::cipher::block_padding::Pkcs7>(&mut buf)
            .map_err(|e| AdapterError::ParseError(format!("decryption failed: {}", e)))?;

        Ok(decrypted.to_vec())
    }
}

impl InboundAdapter for LarkInboundAdapter {
    fn parse(&self, raw_payload: &[u8]) -> Result<InboundMessage, AdapterError> {
        let payload: LarkWebhookPayload = serde_json::from_slice(raw_payload)
            .map_err(|e| AdapterError::ParseError(format!("invalid JSON: {}", e)))?;

        // Handle encrypted payloads
        let payload = if let Some(encrypted) = payload.encrypt {
            let encrypt_key = std::env::var("LARK_ENCRYPT_KEY").map_err(|_| {
                AdapterError::ConfigError(
                    "LARK_ENCRYPT_KEY required for encrypted payloads".to_string(),
                )
            })?;
            let decrypted = Self::decrypt_payload(&encrypted, &encrypt_key)?;
            info!("lark payload decrypted, len={}", decrypted.len());
            serde_json::from_slice::<LarkWebhookPayload>(&decrypted)
                .map_err(|e| AdapterError::ParseError(format!("invalid decrypted JSON: {}", e)))?
        } else {
            payload
        };

        // Handle different event types
        let event = payload
            .event
            .ok_or(AdapterError::ParseError("not a message event".to_string()))?;

        // Only handle message events for now
        let message = event
            .message
            .ok_or(AdapterError::ParseError("not a message event".to_string()))?;

        // Extract text content from JSON string
        let content: LarkMessageContent = serde_json::from_str(&message.content)
            .map_err(|e| AdapterError::ParseError(format!("invalid message content: {}", e)))?;

        let sender_id = event
            .sender
            .as_ref()
            .and_then(|s| s.sender_id.as_ref())
            .and_then(|id| id.open_id.clone())
            .unwrap_or_default();

        let chat_id = message.chat_id.clone();
        let thread_id = format!("lark:{}:{}", chat_id, sender_id);

        Ok(InboundMessage {
            channel: Channel::Lark,
            sender: sender_id.clone(),
            sender_name: event
                .sender
                .as_ref()
                .and_then(|s| s.sender_id.as_ref())
                .and_then(|id| id.name.clone()),
            recipient: chat_id.clone(),
            subject: None,
            text_body: Some(content.text),
            html_body: None,
            thread_id,
            message_id: Some(message.message_id.clone()),
            attachments: vec![],
            reply_to: vec![sender_id.clone()],
            raw_payload: raw_payload.to_vec(),
            metadata: ChannelMetadata {
                lark_app_id: payload.header.as_ref().and_then(|h| h.app_id.clone()),
                lark_tenant_key: payload.header.as_ref().and_then(|h| h.tenant_key.clone()),
                lark_open_id: Some(sender_id),
                lark_chat_id: Some(chat_id),
                lark_message_id: Some(message.message_id),
                ..Default::default()
            },
        })
    }

    fn channel(&self) -> Channel {
        Channel::Lark
    }
}

/// Adapter for sending messages via Lark Open Platform API.
#[derive(Debug)]
pub struct LarkOutboundAdapter {
    pub app_id: String,
    pub app_secret: String,
    tenant_access_token_cache: RwLock<Option<CachedToken>>,
}

#[derive(Debug, Clone)]
struct CachedToken {
    token: String,
    expires_at: std::time::Instant,
}

impl LarkOutboundAdapter {
    pub fn new(app_id: String, app_secret: String) -> Self {
        Self {
            app_id,
            app_secret,
            tenant_access_token_cache: RwLock::new(None),
        }
    }

    pub fn from_env() -> Result<Self, AdapterError> {
        let app_id = std::env::var("LARK_APP_ID")
            .map_err(|_| AdapterError::ConfigError("LARK_APP_ID not set".to_string()))?;
        let app_secret = std::env::var("LARK_APP_SECRET")
            .map_err(|_| AdapterError::ConfigError("LARK_APP_SECRET not set".to_string()))?;
        Ok(Self::new(app_id, app_secret))
    }

    /// Get tenant access token, refreshing if expired.
    fn get_tenant_access_token(&self) -> Result<String, AdapterError> {
        // Check cache first
        {
            let cache = self.tenant_access_token_cache.read().unwrap();
            if let Some(ref cached) = *cache {
                if cached.expires_at > std::time::Instant::now() {
                    return Ok(cached.token.clone());
                }
            }
        }

        // Fetch new token
        let url = "https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal";
        let request_body = serde_json::json!({
            "app_id": self.app_id,
            "app_secret": self.app_secret,
        });

        let client = reqwest::blocking::Client::new();
        let response: LarkTokenResponse = client
            .post(url)
            .json(&request_body)
            .send()
            .map_err(|e| AdapterError::SendError(format!("token request failed: {}", e)))?
            .json()
            .map_err(|e| AdapterError::SendError(format!("token parse failed: {}", e)))?;

        if response.code != 0 {
            return Err(AdapterError::SendError(format!(
                "Lark token error {}: {}",
                response.code,
                response.msg.unwrap_or_default()
            )));
        }

        let token = response
            .tenant_access_token
            .ok_or_else(|| AdapterError::SendError("no tenant_access_token".to_string()))?;

        // Cache with 110 minute expiry (tokens last 2 hours)
        let expires_at = std::time::Instant::now() + std::time::Duration::from_secs(110 * 60);
        {
            let mut cache = self.tenant_access_token_cache.write().unwrap();
            *cache = Some(CachedToken {
                token: token.clone(),
                expires_at,
            });
        }

        Ok(token)
    }
}

impl OutboundAdapter for LarkOutboundAdapter {
    fn send(&self, message: &OutboundMessage) -> Result<SendResult, AdapterError> {
        let token = self.get_tenant_access_token()?;

        let receive_id = message
            .to
            .first()
            .ok_or_else(|| AdapterError::ConfigError("no recipient specified".to_string()))?;

        // Determine receive_id_type based on format
        // ou_xxx = open_id (user), oc_xxx = chat_id (group)
        let receive_id_type = if receive_id.starts_with("ou_") {
            "open_id"
        } else if receive_id.starts_with("oc_") {
            "chat_id"
        } else {
            "open_id" // default
        };

        let text = if message.text_body.is_empty() {
            message.html_body.clone()
        } else {
            message.text_body.clone()
        };

        let content = serde_json::json!({ "text": text });

        let url = format!(
            "https://open.feishu.cn/open-apis/im/v1/messages?receive_id_type={}",
            receive_id_type
        );

        let request_body = serde_json::json!({
            "receive_id": receive_id,
            "msg_type": "text",
            "content": content.to_string(),
        });

        let client = reqwest::blocking::Client::new();
        let response: LarkSendResponse = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&request_body)
            .send()
            .map_err(|e| AdapterError::SendError(format!("send request failed: {}", e)))?
            .json()
            .map_err(|e| AdapterError::SendError(format!("response parse failed: {}", e)))?;

        if response.code != 0 {
            return Ok(SendResult {
                success: false,
                message_id: String::new(),
                submitted_at: String::new(),
                error: Some(format!(
                    "Lark error {}: {}",
                    response.code,
                    response.msg.unwrap_or_default()
                )),
            });
        }

        let message_id = response.data.and_then(|d| d.message_id).unwrap_or_default();

        info!("sent Lark message to {}", receive_id);

        Ok(SendResult {
            success: true,
            message_id,
            submitted_at: chrono::Utc::now().to_rfc3339(),
            error: None,
        })
    }

    fn channel(&self) -> Channel {
        Channel::Lark
    }
}

// ============================================================================
// Lark Webhook Payload Types
// ============================================================================

/// Top-level webhook payload from Lark.
#[derive(Debug, Deserialize)]
pub struct LarkWebhookPayload {
    pub schema: Option<String>,
    pub header: Option<LarkEventHeader>,
    pub event: Option<LarkEvent>,
    /// URL verification challenge
    pub challenge: Option<String>,
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    /// Verification token (for URL verification)
    pub token: Option<String>,
    /// Encrypted payload (when encryption is enabled in Lark console)
    pub encrypt: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LarkEventHeader {
    pub event_id: Option<String>,
    pub event_type: Option<String>,
    pub create_time: Option<String>,
    pub token: Option<String>,
    pub app_id: Option<String>,
    pub tenant_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LarkEvent {
    pub sender: Option<LarkSender>,
    pub message: Option<LarkMessage>,
}

#[derive(Debug, Deserialize)]
pub struct LarkSender {
    pub sender_id: Option<LarkSenderId>,
    pub sender_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LarkSenderId {
    pub union_id: Option<String>,
    pub user_id: Option<String>,
    pub open_id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LarkMessage {
    pub message_id: String,
    pub root_id: Option<String>,
    pub parent_id: Option<String>,
    pub create_time: Option<String>,
    pub chat_id: String,
    pub chat_type: Option<String>,
    pub message_type: Option<String>,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct LarkMessageContent {
    pub text: String,
}

// ============================================================================
// Lark API Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct LarkTokenResponse {
    code: i32,
    msg: Option<String>,
    tenant_access_token: Option<String>,
    expire: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct LarkSendResponse {
    code: i32,
    msg: Option<String>,
    data: Option<LarkSendData>,
}

#[derive(Debug, Deserialize)]
struct LarkSendData {
    message_id: Option<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_text_message() {
        let json = r#"{
            "schema": "2.0",
            "header": {
                "event_id": "evt123",
                "event_type": "im.message.receive_v1",
                "app_id": "cli_xxx",
                "tenant_key": "tenant_xxx"
            },
            "event": {
                "sender": {
                    "sender_id": {
                        "open_id": "ou_xxx",
                        "user_id": "user123"
                    },
                    "sender_type": "user"
                },
                "message": {
                    "message_id": "msg123",
                    "chat_id": "oc_xxx",
                    "chat_type": "p2p",
                    "message_type": "text",
                    "content": "{\"text\": \"Hello from Lark!\"}"
                }
            }
        }"#;

        let adapter = LarkInboundAdapter::new();
        let message = adapter.parse(json.as_bytes()).unwrap();

        assert_eq!(message.channel, Channel::Lark);
        assert_eq!(message.sender, "ou_xxx");
        assert_eq!(message.text_body, Some("Hello from Lark!".to_string()));
        assert_eq!(message.metadata.lark_chat_id, Some("oc_xxx".to_string()));
        assert_eq!(message.metadata.lark_open_id, Some("ou_xxx".to_string()));
    }

    #[test]
    fn parse_message_with_metadata() {
        let json = r#"{
            "schema": "2.0",
            "header": {
                "event_id": "evt456",
                "event_type": "im.message.receive_v1",
                "app_id": "cli_app123",
                "tenant_key": "tenant_key456"
            },
            "event": {
                "sender": {
                    "sender_id": {
                        "open_id": "ou_sender",
                        "name": "Test User"
                    },
                    "sender_type": "user"
                },
                "message": {
                    "message_id": "msg_id_789",
                    "chat_id": "oc_chat123",
                    "chat_type": "group",
                    "message_type": "text",
                    "content": "{\"text\": \"Test message\"}"
                }
            }
        }"#;

        let adapter = LarkInboundAdapter::new();
        let message = adapter.parse(json.as_bytes()).unwrap();

        assert_eq!(message.metadata.lark_app_id, Some("cli_app123".to_string()));
        assert_eq!(
            message.metadata.lark_tenant_key,
            Some("tenant_key456".to_string())
        );
        assert_eq!(
            message.metadata.lark_message_id,
            Some("msg_id_789".to_string())
        );
        assert_eq!(message.sender_name, Some("Test User".to_string()));
    }

    #[test]
    fn thread_id_format() {
        let json = r#"{
            "schema": "2.0",
            "header": {},
            "event": {
                "sender": {
                    "sender_id": {
                        "open_id": "ou_user123"
                    }
                },
                "message": {
                    "message_id": "msg1",
                    "chat_id": "oc_chat456",
                    "content": "{\"text\": \"Hi\"}"
                }
            }
        }"#;

        let adapter = LarkInboundAdapter::new();
        let message = adapter.parse(json.as_bytes()).unwrap();
        assert_eq!(message.thread_id, "lark:oc_chat456:ou_user123");
    }

    #[test]
    fn reply_to_is_sender() {
        let json = r#"{
            "schema": "2.0",
            "header": {},
            "event": {
                "sender": {
                    "sender_id": {
                        "open_id": "ou_sender_id"
                    }
                },
                "message": {
                    "message_id": "msg1",
                    "chat_id": "oc_chat",
                    "content": "{\"text\": \"Hello\"}"
                }
            }
        }"#;

        let adapter = LarkInboundAdapter::new();
        let message = adapter.parse(json.as_bytes()).unwrap();
        assert_eq!(message.reply_to, vec!["ou_sender_id".to_string()]);
    }

    #[test]
    fn adapter_channel_is_lark() {
        let adapter = LarkInboundAdapter::new();
        assert_eq!(adapter.channel(), Channel::Lark);
    }

    #[test]
    fn reject_missing_event() {
        let json = r#"{
            "schema": "2.0",
            "header": {}
        }"#;

        let adapter = LarkInboundAdapter::new();
        let result = adapter.parse(json.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn reject_missing_message() {
        let json = r#"{
            "schema": "2.0",
            "header": {},
            "event": {
                "sender": {
                    "sender_id": {
                        "open_id": "ou_xxx"
                    }
                }
            }
        }"#;

        let adapter = LarkInboundAdapter::new();
        let result = adapter.parse(json.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn reject_invalid_json() {
        let invalid_json = b"not valid json";
        let adapter = LarkInboundAdapter::new();
        let result = adapter.parse(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn raw_payload_preserved() {
        let json = r#"{
            "schema": "2.0",
            "header": {},
            "event": {
                "sender": {
                    "sender_id": {
                        "open_id": "ou_xxx"
                    }
                },
                "message": {
                    "message_id": "msg1",
                    "chat_id": "oc_chat",
                    "content": "{\"text\": \"Test\"}"
                }
            }
        }"#;

        let adapter = LarkInboundAdapter::new();
        let message = adapter.parse(json.as_bytes()).unwrap();
        assert_eq!(message.raw_payload, json.as_bytes());
    }

    // ==================== Encryption/Decryption Tests ====================

    /// Helper to encrypt payload using the same AES-256-CBC scheme Lark uses
    fn encrypt_payload(plaintext: &[u8], encrypt_key: &str) -> String {
        use aes::cipher::{BlockEncryptMut, KeyIvInit};
        use rand::RngCore;

        // Derive AES key from encrypt_key using SHA256
        let key = Sha256::digest(encrypt_key.as_bytes());

        // Generate random IV
        let mut iv = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut iv);

        // Encrypt using AES-256-CBC with PKCS7 padding
        type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;

        // Calculate padded length (PKCS7 padding)
        let block_size = 16;
        let padding_len = block_size - (plaintext.len() % block_size);
        let padded_len = plaintext.len() + padding_len;

        // Create buffer with plaintext + padding
        let mut buf = vec![0u8; padded_len];
        buf[..plaintext.len()].copy_from_slice(plaintext);
        for byte in buf[plaintext.len()..].iter_mut() {
            *byte = padding_len as u8;
        }

        // Encrypt in place
        Aes256CbcEnc::new(key.as_slice().into(), (&iv).into())
            .encrypt_padded_mut::<aes::cipher::block_padding::NoPadding>(&mut buf, padded_len)
            .unwrap();

        // Combine IV + ciphertext and base64 encode
        let mut combined = iv.to_vec();
        combined.extend(&buf);
        BASE64.encode(&combined)
    }

    #[test]
    fn decrypt_payload_roundtrip() {
        let plaintext = r#"{"schema":"2.0","header":{},"event":{"message":{"chat_id":"oc_test"}}}"#;
        let encrypt_key = "test_encrypt_key_12345";

        let encrypted = encrypt_payload(plaintext.as_bytes(), encrypt_key);
        let decrypted = LarkInboundAdapter::decrypt_payload(&encrypted, encrypt_key).unwrap();

        assert_eq!(decrypted, plaintext.as_bytes());
    }

    #[test]
    fn decrypt_payload_produces_valid_json() {
        let inner_json = r#"{
            "schema": "2.0",
            "header": {
                "event_id": "evt_encrypted",
                "event_type": "im.message.receive_v1",
                "app_id": "cli_xxx",
                "tenant_key": "tenant_xxx"
            },
            "event": {
                "sender": {
                    "sender_id": {
                        "open_id": "ou_encrypted_user"
                    }
                },
                "message": {
                    "message_id": "msg_encrypted",
                    "chat_id": "oc_encrypted_chat",
                    "content": "{\"text\": \"Encrypted message!\"}"
                }
            }
        }"#;
        let encrypt_key = "XVWeZ5UsNFHLcbgXpAFeogbSA4ET8QPQ";

        let encrypted = encrypt_payload(inner_json.as_bytes(), encrypt_key);
        let decrypted = LarkInboundAdapter::decrypt_payload(&encrypted, encrypt_key).unwrap();

        // Verify decrypted content is valid JSON with expected structure
        let payload: LarkWebhookPayload = serde_json::from_slice(&decrypted).unwrap();
        assert!(payload.event.is_some());
        let event = payload.event.unwrap();
        assert!(event.message.is_some());
        let message = event.message.unwrap();
        assert_eq!(message.chat_id, "oc_encrypted_chat");
    }

    #[test]
    fn parse_encrypted_payload_end_to_end() {
        let inner_json = r#"{
            "schema": "2.0",
            "header": {
                "event_id": "evt_e2e",
                "app_id": "cli_e2e"
            },
            "event": {
                "sender": {
                    "sender_id": {
                        "open_id": "ou_e2e_user",
                        "name": "E2E Test User"
                    }
                },
                "message": {
                    "message_id": "msg_e2e",
                    "chat_id": "oc_e2e_chat",
                    "content": "{\"text\": \"End to end encrypted!\"}"
                }
            }
        }"#;
        let encrypt_key = "test_key_for_e2e_test";

        // Create encrypted wrapper payload
        let encrypted = encrypt_payload(inner_json.as_bytes(), encrypt_key);
        let wrapper_json = format!(r#"{{"encrypt":"{}"}}"#, encrypted);

        // Set env var for the test
        std::env::set_var("LARK_ENCRYPT_KEY", encrypt_key);

        let adapter = LarkInboundAdapter::new();
        let message = adapter.parse(wrapper_json.as_bytes()).unwrap();

        assert_eq!(message.channel, Channel::Lark);
        assert_eq!(message.sender, "ou_e2e_user");
        assert_eq!(message.text_body, Some("End to end encrypted!".to_string()));
        assert_eq!(
            message.metadata.lark_chat_id,
            Some("oc_e2e_chat".to_string())
        );
        assert_eq!(message.sender_name, Some("E2E Test User".to_string()));

        // Clean up env var
        std::env::remove_var("LARK_ENCRYPT_KEY");
    }

    #[test]
    fn decrypt_fails_with_wrong_key() {
        let plaintext = b"test payload";
        let correct_key = "correct_key";
        let wrong_key = "wrong_key";

        let encrypted = encrypt_payload(plaintext, correct_key);
        let result = LarkInboundAdapter::decrypt_payload(&encrypted, wrong_key);

        assert!(result.is_err());
    }

    #[test]
    fn decrypt_fails_with_invalid_base64() {
        let result = LarkInboundAdapter::decrypt_payload("not_valid_base64!!!", "any_key");
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_fails_with_short_data() {
        // Less than 16 bytes (IV size)
        let short_data = BASE64.encode(b"short");
        let result = LarkInboundAdapter::decrypt_payload(&short_data, "any_key");
        assert!(result.is_err());
    }

    // ==================== Outbound Adapter Tests ====================

    #[test]
    fn outbound_adapter_channel_is_lark() {
        let adapter = LarkOutboundAdapter::new("app123".to_string(), "secret".to_string());
        assert_eq!(adapter.channel(), Channel::Lark);
    }

    #[test]
    fn outbound_adapter_caches_token() {
        let adapter = LarkOutboundAdapter::new("app123".to_string(), "secret".to_string());
        // Initially no cached token
        {
            let cache = adapter.tenant_access_token_cache.read().unwrap();
            assert!(cache.is_none());
        }
    }
}
