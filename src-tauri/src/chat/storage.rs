use super::session::{ChatMessage, ChatSession};
use std::fs;
use std::path::PathBuf;

const CHAT_SESSIONS_DIR: &str = ".ccconfig/chat-sessions";

/// Get chat sessions directory
fn get_sessions_dir() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let sessions_dir = home_dir.join(CHAT_SESSIONS_DIR);

    if !sessions_dir.exists() {
        fs::create_dir_all(&sessions_dir)
            .map_err(|e| format!("Failed to create sessions directory: {}", e))?;
    }

    Ok(sessions_dir)
}

/// Validate session ID format (must be valid UUID)
fn validate_session_id(session_id: &str) -> Result<(), String> {
    // Check for path traversal attempts
    if session_id.contains("..") || session_id.contains('/') || session_id.contains('\\') {
        return Err("Invalid session ID: contains path traversal characters".to_string());
    }

    // Validate UUID format
    uuid::Uuid::parse_str(session_id)
        .map_err(|_| "Invalid session ID: must be a valid UUID".to_string())?;

    Ok(())
}

/// Get session file path
fn get_session_path(session_id: &str) -> Result<PathBuf, String> {
    // Validate session ID first
    validate_session_id(session_id)?;

    let sessions_dir = get_sessions_dir()?;
    Ok(sessions_dir.join(format!("{}.json", session_id)))
}

/// Session storage structure
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SessionStorage {
    session: ChatSession,
    messages: Vec<ChatMessage>,
}

/// Save session and messages
pub fn save_session(session: &ChatSession, messages: &[ChatMessage]) -> Result<(), String> {
    let session_path = get_session_path(&session.id)?;

    let storage = SessionStorage {
        session: session.clone(),
        messages: messages.to_vec(),
    };

    let json = serde_json::to_string_pretty(&storage)
        .map_err(|e| format!("Failed to serialize session: {}", e))?;

    fs::write(&session_path, json)
        .map_err(|e| format!("Failed to write session file: {}", e))?;

    Ok(())
}

/// Load session and messages
pub fn load_session(session_id: &str) -> Result<(ChatSession, Vec<ChatMessage>), String> {
    let session_path = get_session_path(session_id)?;

    if !session_path.exists() {
        return Err(format!("Session not found: {}", session_id));
    }

    let content = fs::read_to_string(&session_path)
        .map_err(|e| format!("Failed to read session file: {}", e))?;

    let storage: SessionStorage = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse session file: {}", e))?;

    Ok((storage.session, storage.messages))
}

/// List all sessions for a project
pub fn list_sessions(project_path: &str) -> Result<Vec<ChatSession>, String> {
    let sessions_dir = get_sessions_dir()?;

    let mut sessions = Vec::new();

    if let Ok(entries) = fs::read_dir(&sessions_dir) {
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".json") {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if let Ok(storage) = serde_json::from_str::<SessionStorage>(&content) {
                            if storage.session.project_path == project_path {
                                sessions.push(storage.session);
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by updated_at descending
    sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    Ok(sessions)
}

/// Delete session
pub fn delete_session(session_id: &str) -> Result<(), String> {
    let session_path = get_session_path(session_id)?;

    if session_path.exists() {
        fs::remove_file(&session_path)
            .map_err(|e| format!("Failed to delete session file: {}", e))?;
    }

    Ok(())
}

/// Update session metadata
pub fn update_session_metadata(session: &ChatSession) -> Result<(), String> {
    let (_, messages) = load_session(&session.id)?;
    save_session(session, &messages)
}
