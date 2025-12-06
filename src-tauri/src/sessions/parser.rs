use super::types::{Session, SessionMessage};
use crate::models::normalize_model_name;
use std::fs;
use std::path::PathBuf;

/// Parse JSONL session file
pub fn parse_session_file(file_path: &PathBuf) -> Result<Vec<SessionMessage>, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read session file: {}", e))?;

    let mut messages = Vec::new();
    let total_lines = content.lines().count();

    println!("📖 Parsing {} lines from {:?}", total_lines, file_path.file_name());

    for (line_num, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<SessionMessage>(line) {
            Ok(message) => {
                println!("  ✅ Line {}: type={:?}", line_num + 1, message.msg_type);
                messages.push(message);
            },
            Err(e) => {
                eprintln!(
                    "  ❌ Line {}: Parse error: {}",
                    line_num + 1,
                    e
                );
                // Show first 100 chars of problematic line
                let preview = if line.len() > 100 {
                    format!("{}...", &line[..100])
                } else {
                    line.to_string()
                };
                eprintln!("     Content: {}", preview);
                // Continue parsing other lines instead of failing completely
            }
        }
    }

    println!("📊 Parsed {}/{} messages successfully", messages.len(), total_lines);
    Ok(messages)
}

/// Extract session metadata from messages
pub fn extract_session_metadata(
    messages: &[SessionMessage],
    file_path: &PathBuf,
) -> Result<Session, String> {
    println!("🔍 Extracting metadata from {} messages", messages.len());

    // Filter out "Other" type messages (queue-operation, etc.)
    let valid_messages: Vec<&SessionMessage> = messages
        .iter()
        .filter(|m| m.msg_type != super::types::MessageType::Other)
        .collect();

    println!("📝 Valid messages (excluding 'Other'): {}", valid_messages.len());

    if valid_messages.is_empty() {
        return Err("Session has no valid messages".to_string());
    }

    let first_message = valid_messages[0];
    let last_message = valid_messages[valid_messages.len() - 1];

    // Extract title from first user message
    let title = valid_messages
        .iter()
        .find(|m| m.msg_type == super::types::MessageType::User)
        .and_then(|m| m.get_text_content())
        .unwrap_or_else(|| "Untitled Session".to_string());

    // Truncate title to reasonable length
    let title = if title.len() > 100 {
        format!("{}...", &title[..97])
    } else {
        title
    };

    // Extract project path from cwd
    let project_path = first_message
        .cwd
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());

    // Find last model used (from assistant messages) and normalize it
    let model = valid_messages
        .iter()
        .rev()
        .find(|m| m.msg_type == super::types::MessageType::Assistant)
        .and_then(|m| m.model.clone())
        .map(|m| normalize_model_name(&m)); // Normalize old model names

    Ok(Session {
        id: first_message.session_id.clone(),
        project_path,
        title,
        created_at: first_message.timestamp.clone(),
        updated_at: last_message.timestamp.clone(),
        message_count: valid_messages.len(),
        model,
        file_path: file_path.to_string_lossy().to_string(),
    })
}

/// Parse session file and extract metadata
pub fn parse_session(file_path: &PathBuf) -> Result<Session, String> {
    let messages = parse_session_file(file_path)?;
    extract_session_metadata(&messages, file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_lines() {
        let messages = vec![];
        let path = PathBuf::from("/tmp/test.jsonl");
        let result = extract_session_metadata(&messages, &path);
        assert!(result.is_err());
    }
}
