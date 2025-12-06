use super::discovery::{check_claude_installed, extract_session_id, list_session_files};
use super::migrate::migrate_session_models;
use super::parser::{parse_session, parse_session_file};
use super::resume::{cancel_session, create_session, resume_session, RunningProcesses};
use super::types::{Session, SessionMessage};
use std::path::PathBuf;
use tauri::{AppHandle, State};

/// Check if Claude CLI is installed
#[tauri::command]
pub async fn session_check_claude_installed() -> Result<bool, String> {
    Ok(check_claude_installed())
}

/// List all sessions for a project
#[tauri::command]
pub async fn session_list(project_path: String) -> Result<Vec<Session>, String> {
    println!("📋 session_list: project_path={}", project_path);

    let session_files = list_session_files(&project_path)?;
    println!("🔍 Found {} session files", session_files.len());

    let mut sessions = Vec::new();

    for (idx, file_path) in session_files.iter().enumerate() {
        println!("📄 Parsing file {}/{}: {:?}", idx + 1, session_files.len(), file_path);
        match parse_session(file_path) {
            Ok(session) => {
                println!("✅ Parsed session: {} - {}", session.id, session.title);
                sessions.push(session);
            },
            Err(e) => {
                eprintln!("❌ Failed to parse session {:?}: {}", file_path, e);
                // Continue with other sessions
            }
        }
    }

    println!("✅ Successfully parsed {} sessions", sessions.len());
    Ok(sessions)
}

/// Get single session details
#[tauri::command]
pub async fn session_get(project_path: String, session_id: String) -> Result<Session, String> {
    println!("🔍 session_get: session_id={}", session_id);

    let session_files = list_session_files(&project_path)?;

    for file_path in session_files {
        if let Some(id) = extract_session_id(&file_path) {
            if id == session_id {
                return parse_session(&file_path);
            }
        }
    }

    Err(format!("Session not found: {}", session_id))
}

/// Get messages for a session
#[tauri::command]
pub async fn session_get_messages(
    project_path: String,
    session_id: String,
) -> Result<Vec<SessionMessage>, String> {
    println!("💬 session_get_messages: session_id={}", session_id);

    let session_files = list_session_files(&project_path)?;

    for file_path in session_files {
        if let Some(id) = extract_session_id(&file_path) {
            if id == session_id {
                return parse_session_file(&file_path);
            }
        }
    }

    Err(format!("Session not found: {}", session_id))
}

/// Resume a session with a new message
#[tauri::command]
pub async fn session_resume(
    app: AppHandle,
    session_id: String,
    message: String,
    project_path: String,
    processes: State<'_, RunningProcesses>,
) -> Result<(), String> {
    println!("▶️  session_resume: session_id={}", session_id);

    resume_session(
        app,
        session_id,
        message,
        project_path,
        processes.inner().clone(),
    )
    .await
}

/// Cancel a running session
#[tauri::command]
pub async fn session_cancel(
    session_id: String,
    processes: State<'_, RunningProcesses>,
) -> Result<(), String> {
    println!("🛑 session_cancel: session_id={}", session_id);

    cancel_session(&session_id, processes.inner().clone()).await
}

/// Delete a session file
#[tauri::command]
pub async fn session_delete(project_path: String, session_id: String) -> Result<(), String> {
    println!("🗑️  session_delete: session_id={}", session_id);

    let session_files = list_session_files(&project_path)?;

    for file_path in session_files {
        if let Some(id) = extract_session_id(&file_path) {
            if id == session_id {
                std::fs::remove_file(&file_path)
                    .map_err(|e| format!("Failed to delete session file: {}", e))?;
                println!("✅ Deleted session: {}", session_id);
                return Ok(());
            }
        }
    }

    Err(format!("Session not found: {}", session_id))
}

/// Create a new session
#[tauri::command]
pub async fn session_create(
    app: AppHandle,
    message: String,
    project_path: String,
    processes: State<'_, RunningProcesses>,
) -> Result<String, String> {
    println!("🆕 session_create: project_path={}", project_path);

    create_session(
        app,
        message,
        project_path,
        processes.inner().clone(),
    )
    .await
}

/// Migrate old model names in session files for a project
#[tauri::command]
pub async fn session_migrate_models(project_path: String) -> Result<usize, String> {
    println!("🔄 session_migrate_models: project_path={}", project_path);

    // Encode project path
    let encoded = project_path.replace('/', "-");
    let home_dir = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
    let session_dir = PathBuf::from(home_dir)
        .join(".claude")
        .join("projects")
        .join(encoded);

    println!("📁 Session dir: {:?}", session_dir);

    if !session_dir.exists() {
        println!("⚠️  Session directory does not exist");
        return Ok(0);
    }

    let count = migrate_session_models(&session_dir)?;
    println!("✅ Migrated {} session files", count);

    Ok(count)
}
