use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

/// Type alias for running Claude processes map
pub type RunningProcesses = Arc<Mutex<HashMap<String, Child>>>;

/// Initialize running processes map
pub fn init_running_processes() -> RunningProcesses {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Stream event from Claude CLI
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StreamEvent {
    SessionIdUpdated { temp_id: String, real_id: String },
    MessageStart { message_id: String },
    ContentDelta { message_id: String, delta: String },
    MessageComplete { message_id: String, content: String },
    Error { error: String },
}

/// Resume a Claude Code session
pub async fn resume_session(
    app: AppHandle,
    session_id: String,
    message: String,
    project_path: String,
    processes: RunningProcesses,
) -> Result<(), String> {
    println!("🔄 resume_session: session_id={}", session_id);

    // Check if Claude CLI is installed
    if !super::discovery::check_claude_installed() {
        return Err("Claude CLI is not installed. Please install it first.".to_string());
    }

    // Validate project path
    let project_path_buf = std::path::PathBuf::from(&project_path);
    if !project_path_buf.exists() {
        return Err(format!("Project path does not exist: {}", project_path));
    }

    let canonical_path = project_path_buf
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize path: {}", e))?;

    // Build Claude CLI command (no model override - use Claude default)
    let mut cmd = Command::new("claude");
    cmd.arg("--resume")
        .arg(&session_id)
        .arg("-p")
        .arg(&message)
        .arg("--output-format")
        .arg("stream-json")
        .arg("--include-partial-messages")
        .arg("--verbose")
        .current_dir(&canonical_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    println!(
        "📝 Command: claude --resume {} -p <message> --output-format stream-json --verbose",
        session_id
    );

    // Spawn process
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn Claude CLI: {}", e))?;

    println!("✅ Process spawned successfully");

    // Take stdout and stderr
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            let _ = child.kill().await;
            return Err("Failed to get stdout".to_string());
        }
    };

    let stderr = child.stderr.take();

    // Store process
    {
        let mut procs = processes.lock().await;
        procs.insert(session_id.clone(), child);
    }

    // Spawn task to read stderr
    if let Some(stderr) = stderr {
        let session_id_for_stderr = session_id.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                eprintln!("❌ Claude stderr [{}]: {}", session_id_for_stderr, line);
            }
        });
    }

    // Spawn task to read output
    let app_clone = app.clone();
    let session_id_clone = session_id.clone();
    let processes_clone = processes.clone();

    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let mut current_message_id = String::new();
        let mut accumulated_content = String::new();

        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() {
                continue;
            }

            println!("📖 Stream: {}", if line.len() > 200 { &line[..200] } else { &line });

            // Parse JSON line
            match serde_json::from_str::<JsonValue>(&line) {
                Ok(json) => {
                    if let Some(event) = parse_stream_event(
                        &json,
                        &mut current_message_id,
                        &mut accumulated_content,
                    ) {
                        println!("📤 Emitting event: {:?}", event);
                        let _ = app_clone.emit(&format!("session-stream:{}", session_id_clone), event);
                    } else {
                        println!("⏭️  No event parsed from JSON");
                    }
                },
                Err(e) => {
                    eprintln!("❌ Failed to parse JSON: {}", e);
                }
            }
        }

        println!("📭 Stream ended for session: {}", session_id_clone);

        // Remove from running processes
        let mut procs = processes_clone.lock().await;
        procs.remove(&session_id_clone);
    });

    Ok(())
}

/// Parse stream event from Claude CLI JSON output
fn parse_stream_event(
    json: &JsonValue,
    current_message_id: &mut String,
    accumulated_content: &mut String,
) -> Option<StreamEvent> {
    let top_type = json.get("type")?.as_str()?;

    match top_type {
        // Standard stream format (without --verbose)
        "stream_event" => {
            let event = json.get("event")?;
            let event_type = event.get("type")?.as_str()?;

            match event_type {
                "message_start" => {
                    let msg_id = event.get("message")?.get("id")?.as_str()?.to_string();
                    *current_message_id = msg_id.clone();
                    *accumulated_content = String::new();
                    Some(StreamEvent::MessageStart { message_id: msg_id })
                }
                "content_block_delta" => {
                    let delta = event.get("delta")?.get("text")?.as_str()?.to_string();
                    *accumulated_content += &delta;
                    Some(StreamEvent::ContentDelta {
                        message_id: current_message_id.clone(),
                        delta,
                    })
                }
                "message_stop" => Some(StreamEvent::MessageComplete {
                    message_id: current_message_id.clone(),
                    content: accumulated_content.clone(),
                }),
                _ => None,
            }
        }
        // Verbose format: system initialization
        "system" => {
            if let Some(session_id) = json.get("session_id").and_then(|v| v.as_str()) {
                println!("🔧 System init for session: {}", session_id);
            }
            None
        }
        // Verbose format: assistant message
        "assistant" => {
            if let Some(message) = json.get("message") {
                let msg_id = message.get("id")?.as_str()?.to_string();

                // Start message
                if current_message_id.is_empty() {
                    *current_message_id = msg_id.clone();
                    *accumulated_content = String::new();
                    println!("🎬 Assistant message started: {}", msg_id);
                }

                // Extract content from message
                if let Some(content_array) = message.get("content").and_then(|v| v.as_array()) {
                    for content_item in content_array {
                        if let Some(text) = content_item.get("text").and_then(|v| v.as_str()) {
                            println!("📝 Extracted content ({} chars)", text.len());
                            *accumulated_content = text.to_string();

                            // Emit content delta for streaming display
                            return Some(StreamEvent::ContentDelta {
                                message_id: msg_id.clone(),
                                delta: text.to_string(),
                            });
                        }
                    }
                }
            }
            None
        }
        // Verbose format: result/completion
        "result" => {
            if !current_message_id.is_empty() {
                println!("✅ Result received, completing message: {}", current_message_id);
                let event = Some(StreamEvent::MessageComplete {
                    message_id: current_message_id.clone(),
                    content: accumulated_content.clone(),
                });
                *current_message_id = String::new();
                *accumulated_content = String::new();
                event
            } else {
                None
            }
        }
        // Verbose format: error
        "error" => {
            if let Some(error_msg) = json.get("error").and_then(|v| v.as_str()) {
                Some(StreamEvent::Error {
                    error: error_msg.to_string(),
                })
            } else {
                Some(StreamEvent::Error {
                    error: "Unknown error".to_string(),
                })
            }
        }
        _ => None,
    }
}

/// Create a new Claude Code session
pub async fn create_session(
    app: AppHandle,
    message: String,
    project_path: String,
    processes: RunningProcesses,
) -> Result<String, String> {
    println!("🆕 create_session: project_path={}", project_path);

    // Check if Claude CLI is installed
    if !super::discovery::check_claude_installed() {
        return Err("Claude CLI is not installed. Please install it first.".to_string());
    }

    // Validate project path
    let project_path_buf = std::path::PathBuf::from(&project_path);
    if !project_path_buf.exists() {
        return Err(format!("Project path does not exist: {}", project_path));
    }

    let canonical_path = project_path_buf
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize path: {}", e))?;

    // Build Claude CLI command (no model override - use Claude default)
    let mut cmd = Command::new("claude");
    cmd.arg("-p")
        .arg(&message)
        .arg("--output-format")
        .arg("stream-json")
        .arg("--include-partial-messages")
        .arg("--verbose")
        .current_dir(&canonical_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    println!("📝 Command: claude -p <message> --output-format stream-json --verbose");

    // Spawn process
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn Claude CLI: {}", e))?;

    println!("✅ Process spawned successfully");

    // Take stdout and stderr
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            let _ = child.kill().await;
            return Err("Failed to get stdout".to_string());
        }
    };

    let stderr = child.stderr.take();

    // Generate temporary session ID (will be replaced once we get the actual ID from Claude)
    let temp_session_id = format!("temp-{}", uuid::Uuid::new_v4());

    // Store process
    {
        let mut procs = processes.lock().await;
        procs.insert(temp_session_id.clone(), child);
    }

    // Spawn task to read stderr
    if let Some(stderr) = stderr {
        let session_id_for_stderr = temp_session_id.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                eprintln!("❌ Claude stderr [{}]: {}", session_id_for_stderr, line);
            }
        });
    }

    // Spawn task to read output
    let app_clone = app.clone();
    let temp_session_id_clone = temp_session_id.clone();
    let processes_clone = processes.clone();

    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let mut current_message_id = String::new();
        let mut accumulated_content = String::new();
        let mut real_session_id: Option<String> = None;

        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() {
                continue;
            }

            println!("📖 Stream: {}", if line.len() > 200 { &line[..200] } else { &line });

            // Parse JSON line
            match serde_json::from_str::<JsonValue>(&line) {
                Ok(json) => {
                    // Check for system init with real session ID
                    if let Some("system") = json.get("type").and_then(|v| v.as_str()) {
                        if let Some(session_id) = json.get("session_id").and_then(|v| v.as_str()) {
                            println!("🔧 System init - Real session ID detected: {}", session_id);

                            // Update process map from temp ID to real ID
                            {
                                let mut procs = processes_clone.lock().await;
                                if let Some(process) = procs.remove(&temp_session_id_clone) {
                                    procs.insert(session_id.to_string(), process);
                                    println!("✅ Updated process map: {} -> {}", temp_session_id_clone, session_id);
                                }
                            }

                            // Store real session ID for future events
                            real_session_id = Some(session_id.to_string());

                            // Emit session ID updated event to both temp and real channels
                            let update_event = StreamEvent::SessionIdUpdated {
                                temp_id: temp_session_id_clone.clone(),
                                real_id: session_id.to_string(),
                            };
                            let _ = app_clone.emit(&format!("session-stream:{}", temp_session_id_clone), update_event.clone());
                            let _ = app_clone.emit(&format!("session-stream:{}", session_id), update_event);
                        }
                    }

                    // Use real session ID if available, otherwise use temp ID
                    let active_session_id = real_session_id.as_ref().unwrap_or(&temp_session_id_clone);

                    if let Some(event) = parse_stream_event(
                        &json,
                        &mut current_message_id,
                        &mut accumulated_content,
                    ) {
                        println!("📤 Emitting event: {:?}", event);
                        let _ = app_clone.emit(&format!("session-stream:{}", active_session_id), event);
                    } else {
                        println!("⏭️  No event parsed from JSON");
                    }
                },
                Err(e) => {
                    eprintln!("❌ Failed to parse JSON: {}", e);
                }
            }
        }

        let final_session_id = real_session_id.as_ref().unwrap_or(&temp_session_id_clone);
        println!("📭 Stream ended for session: {}", final_session_id);

        // Remove from running processes
        let mut procs = processes_clone.lock().await;
        procs.remove(final_session_id);
    });

    Ok(temp_session_id)
}

/// Cancel/stop a running session
pub async fn cancel_session(
    session_id: &str,
    processes: RunningProcesses,
) -> Result<(), String> {
    let mut procs = processes.lock().await;

    if let Some(mut child) = procs.remove(session_id) {
        child
            .kill()
            .await
            .map_err(|e| format!("Failed to kill process: {}", e))?;
        println!("🛑 Cancelled session: {}", session_id);
        Ok(())
    } else {
        Err("Session is not running".to_string())
    }
}
