use std::fs;
use std::path::PathBuf;
use crate::models::normalize_model_name;

/// Migrate old model names in session JSONL files
pub fn migrate_session_models(session_dir: &PathBuf) -> Result<usize, String> {
    if !session_dir.exists() {
        return Ok(0);
    }

    let mut migrated_count = 0;

    let entries = fs::read_dir(session_dir)
        .map_err(|e| format!("Failed to read session directory: {}", e))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();

        // Only process .jsonl files
        if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
            continue;
        }

        // Skip agent sessions
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with("agent-") {
                continue;
            }
        }

        if migrate_session_file(&path)? {
            migrated_count += 1;
        }
    }

    Ok(migrated_count)
}

/// Migrate a single session file
fn migrate_session_file(file_path: &PathBuf) -> Result<bool, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let mut modified = false;
    let mut new_lines = Vec::new();

    for line in content.lines() {
        if line.trim().is_empty() {
            new_lines.push(line.to_string());
            continue;
        }

        // Try to parse as JSON to check for model field
        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(mut json) => {
                if let Some(obj) = json.as_object_mut() {
                    // Check if this line has a model field
                    let model_to_update = obj.get("model")
                        .and_then(|v| v.as_str())
                        .map(|s| {
                            let normalized = normalize_model_name(s);
                            if normalized != s {
                                Some((s.to_string(), normalized))
                            } else {
                                None
                            }
                        })
                        .flatten();

                    if let Some((old, new)) = model_to_update {
                        obj.insert("model".to_string(), serde_json::Value::String(new.clone()));
                        modified = true;
                        println!("  🔄 Migrated: {} -> {}", old, new);
                    }

                    // Also check message.model for nested model references
                    if let Some(message) = obj.get_mut("message") {
                        if let Some(msg_obj) = message.as_object_mut() {
                            let nested_model_to_update = msg_obj.get("model")
                                .and_then(|v| v.as_str())
                                .map(|s| {
                                    let normalized = normalize_model_name(s);
                                    if normalized != s {
                                        Some((s.to_string(), normalized))
                                    } else {
                                        None
                                    }
                                })
                                .flatten();

                            if let Some((old, new)) = nested_model_to_update {
                                msg_obj.insert("model".to_string(), serde_json::Value::String(new.clone()));
                                modified = true;
                                println!("  🔄 Migrated (nested): {} -> {}", old, new);
                            }
                        }
                    }
                }

                // Serialize back to JSON (compact format, no pretty print)
                let json_str = serde_json::to_string(&json)
                    .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
                new_lines.push(json_str);
            }
            Err(_) => {
                // Not valid JSON, keep line as-is
                new_lines.push(line.to_string());
            }
        }
    }

    if modified {
        // Write back to file
        let new_content = new_lines.join("\n");
        fs::write(file_path, new_content)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        println!("✅ Migrated: {:?}", file_path.file_name());
        Ok(true)
    } else {
        Ok(false)
    }
}
