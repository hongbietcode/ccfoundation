use std::fs;
use std::path::PathBuf;

/// Get Claude Code sessions directory
fn get_sessions_dir() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    Ok(home_dir.join(".claude/projects"))
}

/// Encode project path for session directory name
/// Claude Code uses format: -{path-with-slashes-as-hyphens}
/// Example: /Users/user/project -> -Users-user-project
fn encode_project_path(path: &str) -> String {
    // Simply replace all slashes with hyphens
    // The leading '/' will become '-' automatically
    path.replace('/', "-")
}

/// Decode project path from session directory name
fn decode_project_path(encoded: &str) -> String {
    if encoded.starts_with('-') {
        encoded[1..].replace('-', "/")
    } else {
        encoded.to_string()
    }
}

/// Get session directory for a project
pub fn get_project_sessions_dir(project_path: &str) -> Result<PathBuf, String> {
    let sessions_dir = get_sessions_dir()?;
    let encoded_path = encode_project_path(project_path);
    Ok(sessions_dir.join(encoded_path))
}

/// List all session files for a project
pub fn list_session_files(project_path: &str) -> Result<Vec<PathBuf>, String> {
    let project_sessions_dir = get_project_sessions_dir(project_path)?;

    println!("🔍 Looking for sessions in: {:?}", project_sessions_dir);

    if !project_sessions_dir.exists() {
        println!("⚠️  Directory does not exist: {:?}", project_sessions_dir);
        return Ok(Vec::new());
    }

    let mut session_files = Vec::new();

    let entries = fs::read_dir(&project_sessions_dir)
        .map_err(|e| format!("Failed to read sessions directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        // Only include .jsonl files
        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            // Skip agent sessions (start with "agent-")
            if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                if !file_name.starts_with("agent-") {
                    println!("✅ Found session file: {:?}", file_name);
                    session_files.push(path);
                } else {
                    println!("⏭️  Skipping agent session: {:?}", file_name);
                }
            }
        }
    }

    // Sort by modified time (newest first)
    session_files.sort_by(|a, b| {
        let a_modified = fs::metadata(a).and_then(|m| m.modified()).ok();
        let b_modified = fs::metadata(b).and_then(|m| m.modified()).ok();
        b_modified.cmp(&a_modified)
    });

    println!("📊 Total session files found: {}", session_files.len());
    Ok(session_files)
}

/// Extract session ID from file path
pub fn extract_session_id(file_path: &PathBuf) -> Option<String> {
    file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

/// Check if Claude Code is installed
pub fn check_claude_installed() -> bool {
    std::process::Command::new("claude")
        .arg("--version")
        .output()
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_project_path() {
        assert_eq!(
            encode_project_path("/Users/huutri/code/ccmate"),
            "-Users-huutri-code-ccmate"
        );
    }

    #[test]
    fn test_decode_project_path() {
        assert_eq!(
            decode_project_path("-Users-huutri-code-ccmate"),
            "/Users/huutri/code/ccmate"
        );
    }
}
