use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;

/// Active stream processes
pub type StreamProcesses = Arc<Mutex<HashMap<String, tokio::process::Child>>>;

/// Check if Claude CLI is installed
pub async fn check_claude_installed() -> Result<bool, String> {
    let output = Command::new("which")
        .arg("claude")
        .output()
        .await
        .map_err(|e| format!("Failed to check Claude CLI: {}", e))?;

    Ok(output.status.success())
}

/// Stream event payloads
#[derive(Debug, serde::Serialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StreamEvent {
    MessageStart {
        message_id: String,
    },
    ContentDelta {
        message_id: String,
        delta: String,
    },
    MessageComplete {
        message_id: String,
        content: String,
    },
    ToolUse {
        message_id: String,
        tool_name: String,
        input: serde_json::Value,
    },
    ToolResult {
        message_id: String,
        tool_name: String,
        output: String,
    },
    Error {
        error: String,
    },
}

/// Validate model parameter (no longer needed - normalization handles this)
fn validate_model(_model: &str) -> Result<(), String> {
    // Model validation is now handled by normalization in the models module
    // Unknown models are passed through unchanged
    Ok(())
}

/// Validate project path exists and is absolute
fn validate_project_path(path: &str) -> Result<std::path::PathBuf, String> {
    let path_buf = std::path::PathBuf::from(path);

    // Must be absolute path
    if !path_buf.is_absolute() {
        return Err("Project path must be absolute".to_string());
    }

    // Path must exist
    if !path_buf.exists() {
        return Err(format!("Project path does not exist: {}", path));
    }

    // Canonicalize to prevent traversal
    let canonical = path_buf
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize path: {}", e))?;

    Ok(canonical)
}

/// Spawn Claude CLI and stream responses
pub async fn spawn_claude_stream(
    app: AppHandle,
    session_id: String,
    message: String,
    project_path: String,
    model: String,
    processes: StreamProcesses,
) -> Result<(), String> {
    println!("🚀 spawn_claude_stream: session={}, model={}, path={}", session_id, model, project_path);

    // Normalize model name to full API identifier
    let normalized_model = crate::models::normalize_model_name(&model);
    println!("🔄 Model normalized: {} -> {}", model, normalized_model);

    // Validate inputs
    validate_model(&normalized_model)?;
    println!("✅ Model validated: {}", normalized_model);

    let canonical_path = validate_project_path(&project_path)?;
    println!("✅ Project path validated: {:?}", canonical_path);

    // Build CLI command
    let mut cmd = Command::new("claude");
    cmd.arg("-p") // Print mode
        .arg("--verbose") // Required for stream-json format
        .arg("--output-format")
        .arg("stream-json")
        .arg("--include-partial-messages") // Enable streaming chunks
        .arg("--model")
        .arg(&normalized_model)
        .arg(&message) // Pass message as argument
        .current_dir(&canonical_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    println!("📝 Command built: claude -p --verbose --output-format stream-json --include-partial-messages --model {} <message>", normalized_model);

    // Spawn process
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn Claude CLI: {}", e))?;

    println!("✅ Process spawned successfully");

    // Take stdout before storing process (no early return after this point)
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            // Kill child if stdout not available
            let _ = child.kill().await;
            return Err("Failed to get stdout".to_string());
        }
    };

    // Also capture stderr for debugging
    let stderr = child.stderr.take();

    // Store process for cancellation - guaranteed cleanup from this point
    {
        let mut procs = processes.lock().await;
        procs.insert(session_id.clone(), child);
    }

    // Read and parse stdout line by line
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    let event_name = format!("chat-stream:{}", session_id);
    println!("📡 Event name: {}", event_name);
    let mut current_message_id = String::new();
    let mut accumulated_content = String::new();

    println!("📖 Reading Claude CLI output...");
    while let Ok(Some(line)) = lines.next_line().await {
        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        println!("📝 Received line: {}", line);

        // Parse JSON line
        let json_value: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("❌ Failed to parse JSON line: {} - Error: {}", line, e);
                continue;
            }
        };

        // Determine message type and emit appropriate event
        let event = parse_claude_message(&json_value, &mut current_message_id, &mut accumulated_content);

        if let Some(evt) = event {
            println!("📤 Emitting event: {:?}", evt);
            app.emit(&event_name, evt)
                .map_err(|e| format!("Failed to emit event: {}", e))?;
        }
    }

    println!("📚 Finished reading output");

    // Emit final message complete event
    if !current_message_id.is_empty() && !accumulated_content.is_empty() {
        app.emit(
            &event_name,
            StreamEvent::MessageComplete {
                message_id: current_message_id.clone(),
                content: accumulated_content.clone(),
            },
        )
        .ok();
    }

    // Clean up process
    {
        let mut procs = processes.lock().await;
        if let Some(mut child) = procs.remove(&session_id) {
            let _ = child.wait().await;
        }
    }

    Ok(())
}

/// Parse Claude CLI JSON message and convert to stream event
fn parse_claude_message(
    json: &serde_json::Value,
    current_message_id: &mut String,
    accumulated_content: &mut String,
) -> Option<StreamEvent> {
    // Check top-level type
    let top_type = json.get("type")?.as_str()?;

    // Handle stream_event wrapper
    if top_type == "stream_event" {
        let event = json.get("event")?;
        let event_type = event.get("type")?.as_str()?;

        match event_type {
            "message_start" => {
                let msg_id = event
                    .get("message")?
                    .get("id")?
                    .as_str()?
                    .to_string();
                *current_message_id = msg_id.clone();
                *accumulated_content = String::new();
                Some(StreamEvent::MessageStart { message_id: msg_id })
            }
            "content_block_delta" => {
                let delta = event
                    .get("delta")?
                    .get("text")?
                    .as_str()?
                    .to_string();
                *accumulated_content += &delta;
                Some(StreamEvent::ContentDelta {
                    message_id: current_message_id.clone(),
                    delta,
                })
            }
            "message_stop" => {
                Some(StreamEvent::MessageComplete {
                    message_id: current_message_id.clone(),
                    content: accumulated_content.clone(),
                })
            }
            _ => None,
        }
    } else {
        // Skip non-stream events (system, assistant, result, etc.)
        None
    }
}

/// Cancel streaming for a session
pub async fn cancel_stream(session_id: &str, processes: StreamProcesses) -> Result<(), String> {
    let mut procs = processes.lock().await;
    if let Some(mut child) = procs.remove(session_id) {
        child
            .kill()
            .await
            .map_err(|e| format!("Failed to kill process: {}", e))?;
    }
    Ok(())
}
