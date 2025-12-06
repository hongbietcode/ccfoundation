use super::claude_cli::{check_claude_installed, spawn_claude_stream, cancel_stream, StreamProcesses};
use super::session::{ChatConfig, ChatMessage, ChatSession, MessageRole};
use super::storage;
use tauri::{AppHandle, State};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

/// Check if Claude CLI is installed
#[tauri::command]
pub async fn chat_check_claude_installed() -> Result<bool, String> {
    check_claude_installed().await
}

/// Create a new chat session
#[tauri::command]
pub async fn chat_create_session(
    project_path: String,
    title: Option<String>,
) -> Result<ChatSession, String> {
    let session_title = title.unwrap_or_else(|| "New Chat".to_string());
    let session = ChatSession::new(project_path, session_title);

    // Save empty session
    storage::save_session(&session, &[])?;

    Ok(session)
}

/// Get all sessions for a project
#[tauri::command]
pub async fn chat_get_sessions(project_path: String) -> Result<Vec<ChatSession>, String> {
    storage::list_sessions(&project_path)
}

/// Get messages for a session
#[tauri::command]
pub async fn chat_get_messages(session_id: String) -> Result<Vec<ChatMessage>, String> {
    let (_, messages) = storage::load_session(&session_id)?;
    Ok(messages)
}

/// Delete a session
#[tauri::command]
pub async fn chat_delete_session(session_id: String) -> Result<(), String> {
    storage::delete_session(&session_id)
}

/// Send a message and start streaming response
#[tauri::command]
pub async fn chat_send_message(
    app: AppHandle,
    session_id: String,
    message: String,
    config: Option<ChatConfig>,
    processes: State<'_, StreamProcesses>,
) -> Result<(), String> {
    println!("🔵 chat_send_message called: session_id={}, message={}", session_id, message);

    // Load session
    let (mut session, mut messages) = storage::load_session(&session_id)?;
    println!("📦 Loaded session with {} messages", messages.len());

    // Add user message
    let user_message = ChatMessage::new(session_id.clone(), MessageRole::User, message.clone());
    messages.push(user_message);
    println!("➕ Added user message, total messages: {}", messages.len());

    // Update session metadata
    session.message_count = messages.len();
    session.updated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Save messages
    storage::save_session(&session, &messages)?;
    println!("💾 Saved session");

    // Get config
    let chat_config = config.unwrap_or_default();
    println!("⚙️  Config: model={}", chat_config.model);

    // Spawn Claude CLI stream
    let processes_clone = processes.inner().clone();
    println!("🚀 Spawning Claude CLI stream...");
    spawn_claude_stream(
        app,
        session_id.clone(),
        message,
        session.project_path.clone(),
        chat_config.model,
        processes_clone,
    )
    .await?;
    println!("✅ Claude CLI stream spawned for session {}", session_id);

    Ok(())
}

/// Cancel streaming for a session
#[tauri::command]
pub async fn chat_cancel_stream(
    session_id: String,
    processes: State<'_, StreamProcesses>,
) -> Result<(), String> {
    cancel_stream(&session_id, processes.inner().clone()).await
}

/// Save assistant response to session
#[tauri::command]
pub async fn chat_save_assistant_message(
    session_id: String,
    content: String,
) -> Result<(), String> {
    let (mut session, mut messages) = storage::load_session(&session_id)?;

    // Add assistant message
    let assistant_message = ChatMessage::new(session_id.clone(), MessageRole::Assistant, content);
    messages.push(assistant_message);

    // Update session metadata
    session.message_count = messages.len();
    session.updated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    storage::save_session(&session, &messages)?;

    Ok(())
}

/// Update session title
#[tauri::command]
pub async fn chat_update_session_title(
    session_id: String,
    title: String,
) -> Result<(), String> {
    let (mut session, _) = storage::load_session(&session_id)?;
    session.title = title;
    session.updated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    storage::update_session_metadata(&session)
}

/// Initialize stream processes state
pub fn init_stream_processes() -> StreamProcesses {
    Arc::new(Mutex::new(HashMap::new()))
}
