use axum::{extract::Json, http::StatusCode, response::IntoResponse, routing::post, Router};
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::Arc;
use tauri_plugin_notification::NotificationExt;
use tower_http::cors::{Any, CorsLayer};

// Hook event data structure
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct HookEvent {
    pub session_id: String,
    pub transcript_path: String,
    pub cwd: String,
    pub hook_event_name: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

// Hook server functions

pub async fn start_hook_server(app_handle: tauri::AppHandle) -> Result<(), String> {
    let app = create_hook_app(app_handle);

    let addr = SocketAddr::from(([127, 0, 0, 1], 59948));
    println!("🚀 Starting hook server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind to address {}: {}", addr, e))?;

    println!("✅ Hook server listening on http://localhost:59948");

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Failed to start server: {}", e))?;

    Ok(())
}

fn create_hook_app(app_handle: tauri::AppHandle) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let shared_handle = Arc::new(app_handle);

    Router::new()
        .route(
            "/claude_code/hooks",
            post(move |payload| handle_hook_event(payload, shared_handle.clone())),
        )
        .layer(cors)
}

async fn handle_hook_event(
    Json(payload): Json<HookEvent>,
    app_handle: Arc<tauri::AppHandle>,
) -> impl IntoResponse {
    println!("📥 Received hook event: {}", payload.hook_event_name);
    println!(
        "📄 Hook data: {}",
        serde_json::to_string_pretty(&payload)
            .unwrap_or_else(|_| "Failed to serialize".to_string())
    );

    // Check notification settings before sending notification
    if let Ok(Some(settings)) = crate::commands::get_notification_settings().await {
        if settings.enable && settings.enabled_hooks.contains(&payload.hook_event_name) {
            // Send notification based on the hook event
            send_hook_notification(&payload, &app_handle).await;
        } else {
            println!(
                "🔕 Hook '{}' is not enabled in notification settings, skipping notification",
                payload.hook_event_name
            );
        }
    } else {
        println!("⚠️ Could not get notification settings, defaulting to sending notification");
        // Send notification based on the hook event (fallback behavior)
        send_hook_notification(&payload, &app_handle).await;
    }

    (StatusCode::OK, "Hook received")
}

// Send notification based on hook event type
async fn send_hook_notification(event: &HookEvent, app_handle: &tauri::AppHandle) {
    let title = "Claude Code";
    let description = match event.hook_event_name.as_str() {
        "Stop" => "Task completed successfully".to_string(),
        "PreToolUse" => {
            if let Some(tool_name) = event.extra.get("tool_name").and_then(|v| v.as_str()) {
                format!("🔨 Using {} tool", tool_name)
            } else {
                "A tool is going to be used".to_string()
            }
        }
        "Notification" => {
            if let Some(message) = event.extra.get("message").and_then(|v| v.as_str()) {
                format!("✅ {}", message)
            } else {
                "Received notification".to_string()
            }
        }
        _ => "Hook event received".to_string(),
    };

    // Send notification using Tauri notification plugin
    match app_handle
        .notification()
        .builder()
        .title(title)
        .body(&description)
        .show()
    {
        Ok(_) => {
            println!("🔔 Sent Tauri notification: {} - {}", title, description);
        }
        Err(e) => {
            eprintln!("Failed to send Tauri notification: {}", e);
        }
    }
}
