use serde::{Deserialize, Serialize};

/// Session metadata extracted from JSONL file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    /// Session ID (UUID from sessionId field in messages)
    pub id: String,
    /// Project path (from cwd field)
    pub project_path: String,
    /// First message content (used as title)
    pub title: String,
    /// Timestamp of first message
    pub created_at: String,
    /// Timestamp of last message
    pub updated_at: String,
    /// Total number of messages in session
    pub message_count: usize,
    /// Last model used
    pub model: Option<String>,
    /// Session file path
    pub file_path: String,
}

/// Message from Claude Code session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMessage {
    /// Parent message UUID (null for first message)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_uuid: Option<String>,
    /// Message UUID (not present in queue-operation messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    /// Session ID
    pub session_id: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Message type
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    /// Message content (not present in queue-operation messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<MessageContent>,
    /// Working directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Claude Code version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Is sidechain message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_sidechain: Option<bool>,
    /// API message ID (assistant only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Model used (assistant only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Token usage (assistant only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Message type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum MessageType {
    User,
    Assistant,
    Summary,
    #[serde(other)]
    Other,
}

/// Message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContent {
    /// Message role
    pub role: String,
    /// Content - can be string or array of content blocks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
}

/// Helper to extract text content from message
impl SessionMessage {
    pub fn get_text_content(&self) -> Option<String> {
        // Return None if message field is not present
        let message = self.message.as_ref()?;

        if let Some(content) = &message.content {
            // Handle string content
            if let Some(text) = content.as_str() {
                return Some(text.to_string());
            }

            // Handle array of content blocks
            if let Some(arr) = content.as_array() {
                let mut texts = Vec::new();
                for block in arr {
                    if let Some(obj) = block.as_object() {
                        if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                            texts.push(text);
                        }
                    }
                }
                if !texts.is_empty() {
                    return Some(texts.join("\n"));
                }
            }
        }
        None
    }
}
