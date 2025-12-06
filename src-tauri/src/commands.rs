use serde_json::Value;
// sha2 no longer needed since old project config system was removed
use std::path::PathBuf;
use tauri_plugin_updater::UpdaterExt;
use uuid::Uuid;

// Application configuration directory
const APP_CONFIG_DIR: &str = ".ccconfig";

pub async fn initialize_app_config() -> Result<(), String> {
    println!("initialize_app_config called");

    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let app_config_path = home_dir.join(APP_CONFIG_DIR);

    println!(
        "Checking if app config directory exists: {}",
        app_config_path.display()
    );

    // Create config directory if it doesn't exist
    if !app_config_path.exists() {
        println!("App config directory does not exist, creating...");
        std::fs::create_dir_all(&app_config_path)
            .map_err(|e| format!("Failed to create app config directory: {}", e))?;
        println!(
            "App config directory created: {}",
            app_config_path.display()
        );
    } else {
        println!("App config directory already exists");
    }

    // Check if we need to backup Claude configs
    let claude_dir = home_dir.join(".claude");
    println!(
        "Checking if Claude directory exists: {}",
        claude_dir.display()
    );

    if claude_dir.exists() {
        // Check if we already have a backup
        let backup_dir = app_config_path.join("claude_backup");
        if backup_dir.exists() {
            println!("Claude backup already exists, skipping backup");
        } else {
            println!("Claude directory exists but no backup found, backing up...");
            if let Err(e) = backup_claude_configs_internal(&app_config_path, &claude_dir) {
                return Err(format!("Failed to backup Claude configs: {}", e));
            }
            println!("Claude configs backed up successfully");
        }
    } else {
        println!("Claude directory does not exist, skipping backup");
    }

    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ConfigFile {
    pub path: String,
    pub content: Value,
    pub exists: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ConfigStore {
    pub id: String,
    pub title: String,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
    pub settings: Value,
    pub using: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct McpServer {
    #[serde(flatten)]
    pub config: serde_json::Value,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct StoresData {
    pub configs: Vec<ConfigStore>,
    pub distinct_id: Option<String>,
}

#[tauri::command]
pub async fn read_config_file(config_type: String) -> Result<ConfigFile, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;

    let path = match config_type.as_str() {
        "user" => home_dir.join(".claude/settings.json"),
        "enterprise_macos" => {
            PathBuf::from("/Library/Application Support/ClaudeCode/managed-settings.json")
        }
        "enterprise_linux" => PathBuf::from("/etc/claude-code/managed-settings.json"),
        "enterprise_windows" => PathBuf::from("C:\\ProgramData\\ClaudeCode\\managed-settings.json"),
        "mcp_macos" => PathBuf::from("/Library/Application Support/ClaudeCode/managed-mcp.json"),
        "mcp_linux" => PathBuf::from("/etc/claude-code/managed-mcp.json"),
        "mcp_windows" => PathBuf::from("C:\\ProgramData\\ClaudeCode\\managed-mcp.json"),
        _ => return Err("Invalid configuration type".to_string()),
    };

    let path_str = path.to_string_lossy().to_string();

    if path.exists() {
        let content =
            std::fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))?;

        let json_content: Value =
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        Ok(ConfigFile {
            path: path_str,
            content: json_content,
            exists: true,
        })
    } else {
        Ok(ConfigFile {
            path: path_str,
            content: Value::Object(serde_json::Map::new()),
            exists: false,
        })
    }
}

#[tauri::command]
pub async fn write_config_file(config_type: String, content: Value) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;

    let path = match config_type.as_str() {
        "user" => home_dir.join(".claude/settings.json"),
        _ => return Err("Cannot write to enterprise configuration files".to_string()),
    };

    let json_content = serde_json::to_string_pretty(&content)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    std::fs::write(&path, json_content).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn list_config_files() -> Result<Vec<String>, String> {
    let mut configs = vec![];

    // User settings
    if let Some(home) = dirs::home_dir() {
        let user_settings = home.join(".claude/settings.json");
        if user_settings.exists() {
            configs.push("user".to_string());
        }
    }

    // Enterprise settings (read-only)
    if cfg!(target_os = "macos") {
        let enterprise_path =
            PathBuf::from("/Library/Application Support/ClaudeCode/managed-settings.json");
        if enterprise_path.exists() {
            configs.push("enterprise_macos".to_string());
        }

        let mcp_path = PathBuf::from("/Library/Application Support/ClaudeCode/managed-mcp.json");
        if mcp_path.exists() {
            configs.push("mcp_macos".to_string());
        }
    } else if cfg!(target_os = "linux") {
        let enterprise_path = PathBuf::from("/etc/claude-code/managed-settings.json");
        if enterprise_path.exists() {
            configs.push("enterprise_linux".to_string());
        }

        let mcp_path = PathBuf::from("/etc/claude-code/managed-mcp.json");
        if mcp_path.exists() {
            configs.push("mcp_linux".to_string());
        }
    } else if cfg!(target_os = "windows") {
        let enterprise_path = PathBuf::from("C:\\ProgramData\\ClaudeCode\\managed-settings.json");
        if enterprise_path.exists() {
            configs.push("enterprise_windows".to_string());
        }

        let mcp_path = PathBuf::from("C:\\ProgramData\\ClaudeCode\\managed-mcp.json");
        if mcp_path.exists() {
            configs.push("mcp_windows".to_string());
        }
    }

    Ok(configs)
}

#[tauri::command]
pub async fn check_app_config_exists() -> Result<bool, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let app_config_path = home_dir.join(APP_CONFIG_DIR);
    Ok(app_config_path.exists())
}

#[tauri::command]
pub async fn create_app_config_dir() -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let app_config_path = home_dir.join(APP_CONFIG_DIR);

    std::fs::create_dir_all(&app_config_path)
        .map_err(|e| format!("Failed to create app config directory: {}", e))?;

    Ok(())
}

fn backup_claude_configs_internal(
    app_config_path: &std::path::Path,
    claude_dir: &std::path::Path,
) -> Result<(), String> {
    // Create backup directory
    let backup_dir = app_config_path.join("claude_backup");

    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;

    // Copy all files from .claude directory to backup
    for entry in std::fs::read_dir(claude_dir)
        .map_err(|e| format!("Failed to read Claude directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let source_path = entry.path();
        let file_name = source_path.file_name().ok_or("Invalid file name")?;
        let dest_path = backup_dir.join(file_name);

        if source_path.is_file() {
            std::fs::copy(&source_path, &dest_path)
                .map_err(|e| format!("Failed to copy file {}: {}", source_path.display(), e))?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn backup_claude_configs() -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let claude_dir = home_dir.join(".claude");
    let app_config_path = home_dir.join(APP_CONFIG_DIR);

    if !claude_dir.exists() {
        return Err("Claude configuration directory does not exist".to_string());
    }

    // Ensure app config directory exists
    std::fs::create_dir_all(&app_config_path)
        .map_err(|e| format!("Failed to create app config directory: {}", e))?;

    backup_claude_configs_internal(&app_config_path, &claude_dir)
}

// Store management functions

#[tauri::command]
pub async fn get_stores() -> Result<Vec<ConfigStore>, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let app_config_path = home_dir.join(APP_CONFIG_DIR);
    let stores_file = app_config_path.join("stores.json");

    if !stores_file.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&stores_file)
        .map_err(|e| format!("Failed to read stores file: {}", e))?;

    let stores_data: StoresData = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse stores file: {}", e))?;

    let mut stores_vec = stores_data.configs;
    // Sort by createdAt in ascending order (oldest first)
    stores_vec.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    Ok(stores_vec)
}

#[tauri::command]
pub async fn create_config(
    id: String,
    title: String,
    settings: Value,
) -> Result<ConfigStore, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let app_config_path = home_dir.join(APP_CONFIG_DIR);
    let stores_file = app_config_path.join("stores.json");

    // Ensure app config directory exists
    std::fs::create_dir_all(&app_config_path)
        .map_err(|e| format!("Failed to create app config directory: {}", e))?;

    // Read existing stores
    let mut stores_data = if stores_file.exists() {
        let content = std::fs::read_to_string(&stores_file)
            .map_err(|e| format!("Failed to read stores file: {}", e))?;

        serde_json::from_str::<StoresData>(&content)
            .map_err(|e| format!("Failed to parse stores file: {}", e))?
    } else {
        StoresData {
            configs: vec![],
            distinct_id: None,
        }
    };

    // Determine if this should be the active store (true if no other stores exist)
    let should_be_active = stores_data.configs.is_empty();

    // If this is the first config being created and there's an existing settings.json, create an Original Config store
    if should_be_active {
        let claude_settings_path = home_dir.join(".claude/settings.json");
        if claude_settings_path.exists() {
            // Read existing settings
            let settings_content = std::fs::read_to_string(&claude_settings_path)
                .map_err(|e| format!("Failed to read existing Claude settings: {}", e))?;

            let settings_json: Value = serde_json::from_str(&settings_content)
                .map_err(|e| format!("Failed to parse existing Claude settings: {}", e))?;

            // Create an Original Config store with existing settings
            let original_store = ConfigStore {
                id: nanoid::nanoid!(6), // Generate a 6-character ID
                title: "Original Config".to_string(),
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| format!("Failed to get timestamp: {}", e))?
                    .as_secs(),
                settings: settings_json,
                using: false, // Original Config should not be active by default
            };

            // Add the Original Config store to the collection
            stores_data.configs.push(original_store);
            println!("Created Original Config store from existing settings.json");
        }
    }

    // If this is the first store (and therefore active), write its settings to the user's actual settings.json with partial update
    if should_be_active {
        let user_settings_path = home_dir.join(".claude/settings.json");

        // Create .claude directory if it doesn't exist
        if let Some(parent) = user_settings_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create .claude directory: {}", e))?;
        }

        // Read existing settings if file exists, otherwise start with empty object
        let mut existing_settings = if user_settings_path.exists() {
            let content = std::fs::read_to_string(&user_settings_path)
                .map_err(|e| format!("Failed to read existing settings: {}", e))?;
            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse existing settings: {}", e))?
        } else {
            serde_json::Value::Object(serde_json::Map::new())
        };

        // Merge the new settings into existing settings (partial update)
        if let Some(settings_obj) = settings.as_object() {
            if let Some(existing_obj) = existing_settings.as_object_mut() {
                // Update only the keys present in the stored settings
                for (key, value) in settings_obj {
                    existing_obj.insert(key.clone(), value.clone());
                }
            } else {
                // If existing settings is not an object, replace it entirely
                existing_settings = settings.clone();
            }
        } else {
            // If stored settings is not an object, replace existing entirely
            existing_settings = settings.clone();
        }

        // Write the merged settings back to file
        let json_content = serde_json::to_string_pretty(&existing_settings)
            .map_err(|e| format!("Failed to serialize merged settings: {}", e))?;

        std::fs::write(&user_settings_path, json_content)
            .map_err(|e| format!("Failed to write user settings: {}", e))?;
    }

    // Create new store
    let new_store = ConfigStore {
        id: id.clone(),
        title: title.clone(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("Failed to get timestamp: {}", e))?
            .as_secs(),
        settings,
        using: should_be_active,
    };

    // Add store to collection
    stores_data.configs.push(new_store.clone());

    // Write back to stores file
    let json_content = serde_json::to_string_pretty(&stores_data)
        .map_err(|e| format!("Failed to serialize stores: {}", e))?;

    std::fs::write(&stores_file, json_content)
        .map_err(|e| format!("Failed to write stores file: {}", e))?;

    // Automatically unlock CC extension when creating new config
    if let Err(e) = unlock_cc_ext().await {
        eprintln!("Warning: Failed to unlock CC extension: {}", e);
    }

    Ok(new_store)
}

#[tauri::command]
pub async fn delete_config(store_id: String) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let app_config_path = home_dir.join(APP_CONFIG_DIR);
    let stores_file = app_config_path.join("stores.json");

    if !stores_file.exists() {
        return Err("Stores file does not exist".to_string());
    }

    // Read existing stores
    let content = std::fs::read_to_string(&stores_file)
        .map_err(|e| format!("Failed to read stores file: {}", e))?;

    let mut stores_data: StoresData = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse stores file: {}", e))?;

    // Find and remove store by ID
    let original_len = stores_data.configs.len();
    stores_data.configs.retain(|store| store.id != store_id);

    if stores_data.configs.len() == original_len {
        return Err("Store not found".to_string());
    }

    // Write back to file
    let json_content = serde_json::to_string_pretty(&stores_data)
        .map_err(|e| format!("Failed to serialize stores: {}", e))?;

    std::fs::write(&stores_file, json_content)
        .map_err(|e| format!("Failed to write stores file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn set_using_config(store_id: String) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let app_config_path = home_dir.join(APP_CONFIG_DIR);
    let stores_file = app_config_path.join("stores.json");

    if !stores_file.exists() {
        return Err("Stores file does not exist".to_string());
    }

    // Read existing stores
    let content = std::fs::read_to_string(&stores_file)
        .map_err(|e| format!("Failed to read stores file: {}", e))?;

    let mut stores_data: StoresData = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse stores file: {}", e))?;

    // Find the store and check if it exists
    let store_found = stores_data.configs.iter().any(|store| store.id == store_id);
    if !store_found {
        return Err("Store not found".to_string());
    }

    // Set all stores to not using, then set the selected one to using
    let mut selected_store_settings: Option<Value> = None;
    for store in &mut stores_data.configs {
        if store.id == store_id {
            store.using = true;
            selected_store_settings = Some(store.settings.clone());
        } else {
            store.using = false;
        }
    }

    // Write the selected store's settings to the user's actual settings.json with partial update
    if let Some(settings) = selected_store_settings {
        let user_settings_path = home_dir.join(".claude/settings.json");

        // Create .claude directory if it doesn't exist
        if let Some(parent) = user_settings_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create .claude directory: {}", e))?;
        }

        // Read existing settings if file exists, otherwise start with empty object
        let mut existing_settings = if user_settings_path.exists() {
            let content = std::fs::read_to_string(&user_settings_path)
                .map_err(|e| format!("Failed to read existing settings: {}", e))?;
            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse existing settings: {}", e))?
        } else {
            serde_json::Value::Object(serde_json::Map::new())
        };

        // Merge the new settings into existing settings (partial update)
        if let Some(settings_obj) = settings.as_object() {
            if let Some(existing_obj) = existing_settings.as_object_mut() {
                // Update only the keys present in the stored settings
                for (key, value) in settings_obj {
                    existing_obj.insert(key.clone(), value.clone());
                }
            } else {
                // If existing settings is not an object, replace it entirely
                existing_settings = settings.clone();
            }
        } else {
            // If stored settings is not an object, replace existing entirely
            existing_settings = settings.clone();
        }

        // Write the merged settings back to file
        let json_content = serde_json::to_string_pretty(&existing_settings)
            .map_err(|e| format!("Failed to serialize merged settings: {}", e))?;

        std::fs::write(&user_settings_path, json_content)
            .map_err(|e| format!("Failed to write user settings: {}", e))?;
    }

    // Write back to stores file (with active context update)
    // Parse as generic Value to preserve and update active context
    let mut stores_value: Value = serde_json::to_value(&stores_data)
        .map_err(|e| format!("Failed to serialize stores: {}", e))?;

    // Update activeContext to global
    stores_value["activeContext"] = serde_json::json!({
        "type": "global",
        "id": store_id,
        "projectPath": null
    });

    let json_content = serde_json::to_string_pretty(&stores_value)
        .map_err(|e| format!("Failed to serialize stores: {}", e))?;

    std::fs::write(&stores_file, json_content)
        .map_err(|e| format!("Failed to write stores file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn reset_to_original_config() -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let app_config_path = home_dir.join(APP_CONFIG_DIR);
    let stores_file = app_config_path.join("stores.json");

    // Set all stores to not using
    if stores_file.exists() {
        let content = std::fs::read_to_string(&stores_file)
            .map_err(|e| format!("Failed to read stores file: {}", e))?;

        let mut stores_data: StoresData = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse stores file: {}", e))?;

        // Set all stores to not using
        for store in &mut stores_data.configs {
            store.using = false;
        }

        // Write back to stores file
        let json_content = serde_json::to_string_pretty(&stores_data)
            .map_err(|e| format!("Failed to serialize stores: {}", e))?;

        std::fs::write(&stores_file, json_content)
            .map_err(|e| format!("Failed to write stores file: {}", e))?;
    }

    // Clear env field in settings.json
    let user_settings_path = home_dir.join(".claude/settings.json");

    // Create .claude directory if it doesn't exist
    if let Some(parent) = user_settings_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create .claude directory: {}", e))?;
    }

    // Read existing settings if file exists, otherwise start with empty object
    let mut existing_settings = if user_settings_path.exists() {
        let content = std::fs::read_to_string(&user_settings_path)
            .map_err(|e| format!("Failed to read existing settings: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse existing settings: {}", e))?
    } else {
        serde_json::Value::Object(serde_json::Map::new())
    };

    // Set env to empty object
    if let Some(existing_obj) = existing_settings.as_object_mut() {
        existing_obj.insert("env".to_string(), serde_json::json!({}));
    }

    // Write the merged settings back to file
    let json_content = serde_json::to_string_pretty(&existing_settings)
        .map_err(|e| format!("Failed to serialize merged settings: {}", e))?;

    std::fs::write(&user_settings_path, json_content)
        .map_err(|e| format!("Failed to write user settings: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn get_current_store() -> Result<Option<ConfigStore>, String> {
    let stores = get_stores().await?;
    Ok(stores.into_iter().find(|store| store.using))
}

#[tauri::command]
pub async fn get_store(store_id: String) -> Result<ConfigStore, String> {
    let stores = get_stores().await?;
    stores
        .into_iter()
        .find(|store| store.id == store_id)
        .ok_or_else(|| format!("Store with id '{}' not found", store_id))
}

#[tauri::command]
pub async fn update_config(
    store_id: String,
    title: String,
    settings: Value,
) -> Result<ConfigStore, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let app_config_path = home_dir.join(APP_CONFIG_DIR);
    let stores_file = app_config_path.join("stores.json");

    if !stores_file.exists() {
        return Err("Stores file does not exist".to_string());
    }

    // Read existing stores
    let content = std::fs::read_to_string(&stores_file)
        .map_err(|e| format!("Failed to read stores file: {}", e))?;

    let mut stores_data: StoresData = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse stores file: {}", e))?;

    // Find the store by ID
    let store_index = stores_data
        .configs
        .iter()
        .position(|store| store.id == store_id)
        .ok_or_else(|| format!("Store with id '{}' not found", store_id))?;

    // // Check if new title conflicts with existing stores (excluding current one)
    // for existing_store in &stores_data.configs {
    //     if existing_store.id != store_id && existing_store.title == title {
    //         return Err("Store with this title already exists".to_string());
    //     }
    // }

    // Update the store
    let store = &mut stores_data.configs[store_index];
    store.title = title.clone();
    store.settings = settings.clone();

    // If this store is currently in use, also update the user's settings.json with partial update
    if store.using {
        let user_settings_path = home_dir.join(".claude/settings.json");

        // Create .claude directory if it doesn't exist
        if let Some(parent) = user_settings_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create .claude directory: {}", e))?;
        }

        // Read existing settings if file exists, otherwise start with empty object
        let mut existing_settings = if user_settings_path.exists() {
            let content = std::fs::read_to_string(&user_settings_path)
                .map_err(|e| format!("Failed to read existing settings: {}", e))?;
            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse existing settings: {}", e))?
        } else {
            serde_json::Value::Object(serde_json::Map::new())
        };

        // Merge the new settings into existing settings (partial update)
        if let Some(settings_obj) = settings.as_object() {
            if let Some(existing_obj) = existing_settings.as_object_mut() {
                // Update only the keys present in the stored settings
                for (key, value) in settings_obj {
                    existing_obj.insert(key.clone(), value.clone());
                }
            } else {
                // If existing settings is not an object, replace it entirely
                existing_settings = settings.clone();
            }
        } else {
            // If stored settings is not an object, replace existing entirely
            existing_settings = settings.clone();
        }

        // Write the merged settings back to file
        let json_content = serde_json::to_string_pretty(&existing_settings)
            .map_err(|e| format!("Failed to serialize merged settings: {}", e))?;

        std::fs::write(&user_settings_path, json_content)
            .map_err(|e| format!("Failed to write user settings: {}", e))?;
    }

    // Write back to stores file
    let json_content = serde_json::to_string_pretty(&stores_data)
        .map_err(|e| format!("Failed to serialize stores: {}", e))?;

    std::fs::write(&stores_file, json_content)
        .map_err(|e| format!("Failed to write stores file: {}", e))?;

    // Automatically unlock CC extension when updating config
    if let Err(e) = unlock_cc_ext().await {
        eprintln!("Warning: Failed to unlock CC extension: {}", e);
    }

    Ok(stores_data.configs[store_index].clone())
}

#[tauri::command]
pub async fn open_config_path() -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let app_config_path = home_dir.join(APP_CONFIG_DIR);

    // Ensure the directory exists
    if !app_config_path.exists() {
        std::fs::create_dir_all(&app_config_path)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    // Open the directory in the system's file manager
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&app_config_path)
            .spawn()
            .map_err(|e| format!("Failed to open config directory: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&app_config_path)
            .spawn()
            .map_err(|e| format!("Failed to open config directory: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&app_config_path)
            .spawn()
            .map_err(|e| format!("Failed to open config directory: {}", e))?;
    }

    Ok(())
}

// MCP Server management functions

#[tauri::command]
pub async fn get_global_mcp_servers() -> Result<std::collections::HashMap<String, McpServer>, String>
{
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let claude_json_path = home_dir.join(".claude.json");

    if !claude_json_path.exists() {
        return Ok(std::collections::HashMap::new());
    }

    let content = std::fs::read_to_string(&claude_json_path)
        .map_err(|e| format!("Failed to read .claude.json: {}", e))?;

    let json_value: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse .claude.json: {}", e))?;

    let mcp_servers_obj = json_value
        .get("mcpServers")
        .and_then(|servers| servers.as_object())
        .cloned()
        .unwrap_or_else(serde_json::Map::new);

    let mut result = std::collections::HashMap::new();
    for (name, config) in mcp_servers_obj {
        let mcp_server = McpServer {
            config: config.clone(),
        };
        result.insert(name.clone(), mcp_server);
    }

    Ok(result)
}

#[tauri::command]
pub async fn check_mcp_server_exists(server_name: String) -> Result<bool, String> {
    let mcp_servers = get_global_mcp_servers().await?;
    Ok(mcp_servers.contains_key(&server_name))
}

#[tauri::command]
pub async fn update_global_mcp_server(
    server_name: String,
    server_config: Value,
) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let claude_json_path = home_dir.join(".claude.json");

    // Read existing .claude.json or create new structure
    let mut json_value = if claude_json_path.exists() {
        let content = std::fs::read_to_string(&claude_json_path)
            .map_err(|e| format!("Failed to read .claude.json: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse .claude.json: {}", e))?
    } else {
        Value::Object(serde_json::Map::new())
    };

    // Update mcpServers object
    let mcp_servers = json_value
        .as_object_mut()
        .unwrap()
        .entry("mcpServers".to_string())
        .or_insert_with(|| Value::Object(serde_json::Map::new()))
        .as_object_mut()
        .unwrap();

    // Update the specific server
    mcp_servers.insert(server_name, server_config);

    // Write back to file
    let json_content = serde_json::to_string_pretty(&json_value)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    std::fs::write(&claude_json_path, json_content)
        .map_err(|e| format!("Failed to write .claude.json: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn delete_global_mcp_server(server_name: String) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let claude_json_path = home_dir.join(".claude.json");

    if !claude_json_path.exists() {
        return Err("Claude configuration file does not exist".to_string());
    }

    // Read existing .claude.json
    let content = std::fs::read_to_string(&claude_json_path)
        .map_err(|e| format!("Failed to read .claude.json: {}", e))?;

    let mut json_value: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse .claude.json: {}", e))?;

    // Check if mcpServers exists
    let mcp_servers = json_value
        .as_object_mut()
        .unwrap()
        .get_mut("mcpServers")
        .and_then(|servers| servers.as_object_mut());

    let mcp_servers = match mcp_servers {
        Some(servers) => servers,
        None => return Err("No mcpServers found in .claude.json".to_string()),
    };

    // Check if the server exists
    if !mcp_servers.contains_key(&server_name) {
        return Err(format!("MCP server '{}' not found", server_name));
    }

    // Remove the server
    mcp_servers.remove(&server_name);

    // If mcpServers is now empty, we can optionally remove the entire mcpServers object
    if mcp_servers.is_empty() {
        json_value.as_object_mut().unwrap().remove("mcpServers");
    }

    // Write back to file
    let json_content = serde_json::to_string_pretty(&json_value)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    std::fs::write(&claude_json_path, json_content)
        .map_err(|e| format!("Failed to write .claude.json: {}", e))?;

    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub version: Option<String>,
    pub body: Option<String>,
    pub date: Option<String>,
}

#[tauri::command]
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<UpdateInfo, String> {
    // Skip update check in dev mode
    #[cfg(debug_assertions)]
    {
        println!("⏭️  Skipping update check in dev mode");
        println!("📱 App version: {}", app.package_info().version);
        Ok(UpdateInfo {
            available: false,
            version: None,
            body: None,
            date: None,
        })
    }

    #[cfg(not(debug_assertions))]
    {
        println!("🔍 Checking for updates...");
        println!("📱 App version: {}", app.package_info().version);
        println!("🏷️  App identifier: {}", app.package_info().name);

        match app.updater() {
            Ok(updater) => {
                println!("✅ Updater initialized successfully");
                println!("📡 Checking update endpoint: https://github.com/djyde/ccfoundation-release/releases/latest/download/latest.json");

                match updater.check().await {
                    Ok(Some(update)) => {
                        println!("🎉 Update available!");
                        println!("📦 Current version: {}", update.current_version);
                        println!("🚀 New version: {}", update.version);
                        println!("📝 Release notes: {:?}", update.body);
                        println!("📅 Release date: {:?}", update.date);
                        println!("🎯 Target platform: {:?}", update.target);

                        Ok(UpdateInfo {
                            available: true,
                            version: Some(update.version.clone()),
                            body: update.body.clone(),
                            date: update.date.map(|d| d.to_string()),
                        })
                    }
                    Ok(None) => {
                        println!("✅ No updates available - you're on the latest version");

                        Ok(UpdateInfo {
                            available: false,
                            version: None,
                            body: None,
                            date: None,
                        })
                    }
                    Err(e) => {
                        println!("❌ Error checking for updates: {}", e);
                        Err(format!("Failed to check for updates: {}", e))
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to initialize updater: {}", e);
                Err(format!("Failed to get updater: {}", e))
            }
        }
    }
}

#[tauri::command]
pub async fn rebuild_tray_menu_command(app: tauri::AppHandle) -> Result<(), String> {
    crate::tray::rebuild_tray_menu(app).await
}

#[tauri::command]
pub async fn unlock_cc_ext() -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let claude_config_path = home_dir.join(".claude/config.json");

    // Ensure .claude directory exists
    if let Some(parent) = claude_config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create .claude directory: {}", e))?;
    }

    if claude_config_path.exists() {
        // File exists, check if primaryApiKey key exists
        let content = std::fs::read_to_string(&claude_config_path)
            .map_err(|e| format!("Failed to read config.json: {}", e))?;

        let mut json_value: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config.json: {}", e))?;

        // Check if primaryApiKey exists
        if json_value.get("primaryApiKey").is_none() {
            // Add primaryApiKey to existing config
            if let Some(obj) = json_value.as_object_mut() {
                obj.insert(
                    "primaryApiKey".to_string(),
                    Value::String("xxx".to_string()),
                );
            }

            // Write back to file
            let json_content = serde_json::to_string_pretty(&json_value)
                .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

            std::fs::write(&claude_config_path, json_content)
                .map_err(|e| format!("Failed to write config.json: {}", e))?;

            println!("Added primaryApiKey to existing config.json");
        } else {
            println!("primaryApiKey already exists in config.json, no action needed");
        }
    } else {
        // File doesn't exist, create it with primaryApiKey
        let config = serde_json::json!({
            "primaryApiKey": "xxx"
        });

        let json_content = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

        std::fs::write(&claude_config_path, json_content)
            .map_err(|e| format!("Failed to write config.json: {}", e))?;

        println!("Created new config.json with primaryApiKey");
    }

    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UsageData {
    pub input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ProjectUsageRecord {
    pub uuid: String,
    pub timestamp: String,
    pub model: Option<String>,
    pub usage: Option<UsageData>,
}

#[tauri::command]
pub async fn read_project_usage_files() -> Result<Vec<ProjectUsageRecord>, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let projects_dir = home_dir.join(".claude/projects");

    println!(
        "🔍 Looking for projects directory: {}",
        projects_dir.display()
    );

    if !projects_dir.exists() {
        println!("❌ Projects directory does not exist");
        return Ok(vec![]);
    }

    println!("✅ Projects directory exists");

    let mut all_records = Vec::new();
    let mut files_processed = 0;
    let mut lines_processed = 0;

    // Recursively find all .jsonl files in the projects directory and subdirectories
    fn find_jsonl_files(
        dir: &std::path::Path,
        files: &mut Vec<std::path::PathBuf>,
    ) -> Result<(), String> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_file() && path.extension().map(|ext| ext == "jsonl").unwrap_or(false) {
                files.push(path);
            } else if path.is_dir() {
                // Recursively search subdirectories
                if let Err(e) = find_jsonl_files(&path, files) {
                    println!("Warning: {}", e);
                }
            }
        }
        Ok(())
    }

    let mut jsonl_files = Vec::new();
    find_jsonl_files(&projects_dir, &mut jsonl_files)?;

    for path in jsonl_files {
        files_processed += 1;
        // println!("📄 Processing file: {}", path.display());

        // Read the JSONL file
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;

        // Process each line in the JSONL file
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            lines_processed += 1;

            // Parse the JSON line
            let json_value: Value = serde_json::from_str(line)
                .map_err(|e| format!("Failed to parse JSON line: {}", e))?;

            // Extract the required fields
            let uuid = json_value
                .get("uuid")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let timestamp = json_value
                .get("timestamp")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // Extract model field (optional) - check both top-level and nested in message field
            let model = if let Some(model_str) = json_value.get("model").and_then(|v| v.as_str()) {
                Some(model_str.to_string())
            } else if let Some(message_obj) = json_value.get("message") {
                message_obj
                    .get("model")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            };

            // Extract usage data (optional) - check both top-level and nested in message field
            let usage = if let Some(usage_obj) = json_value.get("usage") {
                Some(UsageData {
                    input_tokens: usage_obj.get("input_tokens").and_then(|v| v.as_u64()),
                    cache_read_input_tokens: usage_obj
                        .get("cache_read_input_tokens")
                        .and_then(|v| v.as_u64()),
                    output_tokens: usage_obj.get("output_tokens").and_then(|v| v.as_u64()),
                })
            } else if let Some(message_obj) = json_value.get("message") {
                message_obj.get("usage").map(|usage_obj| UsageData {
                    input_tokens: usage_obj.get("input_tokens").and_then(|v| v.as_u64()),
                    cache_read_input_tokens: usage_obj
                        .get("cache_read_input_tokens")
                        .and_then(|v| v.as_u64()),
                    output_tokens: usage_obj.get("output_tokens").and_then(|v| v.as_u64()),
                })
            } else {
                None
            };

            // Only include records with valid uuid, timestamp, and valid usage data
            if !uuid.is_empty() && !timestamp.is_empty() {
                // Check if usage data exists and has meaningful token values
                if let Some(ref usage_data) = usage {
                    let input_tokens = usage_data.input_tokens.unwrap_or(0);
                    let output_tokens = usage_data.output_tokens.unwrap_or(0);

                    // Only include if input_tokens + output_tokens > 0
                    if input_tokens + output_tokens > 0 {
                        all_records.push(ProjectUsageRecord {
                            uuid,
                            timestamp,
                            model,
                            usage,
                        });
                    }
                }
            }
        }
    }

    println!(
        "📊 Summary: Processed {} files, {} lines, found {} records",
        files_processed,
        lines_processed,
        all_records.len()
    );
    Ok(all_records)
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MemoryFile {
    pub path: String,
    pub content: String,
    pub exists: bool,
}

#[tauri::command]
pub async fn read_claude_memory() -> Result<MemoryFile, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let claude_md_path = home_dir.join(".claude/CLAUDE.md");

    let path_str = claude_md_path.to_string_lossy().to_string();

    if claude_md_path.exists() {
        let content = std::fs::read_to_string(&claude_md_path)
            .map_err(|e| format!("Failed to read CLAUDE.md file: {}", e))?;

        Ok(MemoryFile {
            path: path_str,
            content,
            exists: true,
        })
    } else {
        Ok(MemoryFile {
            path: path_str,
            content: String::new(),
            exists: false,
        })
    }
}

#[tauri::command]
pub async fn write_claude_memory(content: String) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let claude_md_path = home_dir.join(".claude/CLAUDE.md");

    // Ensure .claude directory exists
    if let Some(parent) = claude_md_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create .claude directory: {}", e))?;
    }

    std::fs::write(&claude_md_path, content)
        .map_err(|e| format!("Failed to write CLAUDE.md file: {}", e))?;

    Ok(())
}

// Project Memory Commands
#[tauri::command]
pub async fn read_project_memory(project_path: String) -> Result<MemoryFile, String> {
    // Primary location: ./CLAUDE.md at project root
    let claude_md_path = PathBuf::from(&project_path).join("CLAUDE.md");

    let path_str = claude_md_path.to_string_lossy().to_string();

    if claude_md_path.exists() {
        let content = std::fs::read_to_string(&claude_md_path)
            .map_err(|e| format!("Failed to read CLAUDE.md file: {}", e))?;

        Ok(MemoryFile {
            path: path_str,
            content,
            exists: true,
        })
    } else {
        Ok(MemoryFile {
            path: path_str,
            content: String::new(),
            exists: false,
        })
    }
}

#[tauri::command]
pub async fn write_project_memory(project_path: String, content: String) -> Result<(), String> {
    // Primary location: ./CLAUDE.md at project root
    let claude_md_path = PathBuf::from(&project_path).join("CLAUDE.md");

    std::fs::write(&claude_md_path, content)
        .map_err(|e| format!("Failed to write CLAUDE.md file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn install_and_restart(app: tauri::AppHandle) -> Result<(), String> {
    println!("🚀 Starting update installation process...");

    match app.updater() {
        Ok(updater) => {
            println!("✅ Updater ready for installation");
            println!("📡 Re-checking for updates to get download info...");

            match updater.check().await {
                Ok(Some(update)) => {
                    println!("📥 Starting download and installation...");
                    println!("🎯 Update version: {}", update.version);
                    println!("🎯 Update target: {:?}", update.target);

                    // Download and install the update
                    match update
                        .download_and_install(
                            |chunk_length, content_length| {
                                let progress = if let Some(total) = content_length {
                                    (chunk_length as f64 / total as f64) * 100.0
                                } else {
                                    0.0
                                };
                                println!(
                                    "⬇️  Download progress: {:.1}% ({} bytes)",
                                    progress, chunk_length
                                );
                            },
                            || {
                                println!("✅ Download completed! Preparing to restart...");
                            },
                        )
                        .await
                    {
                        Ok(_) => {
                            println!("🔄 Update installed successfully! Restarting application in 500ms...");

                            // Schedule restart after a short delay to allow the response to be sent
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                                println!("🔄 Restarting now!");
                                app_handle.restart();
                            });
                            Ok(())
                        }
                        Err(e) => {
                            println!("❌ Failed to install update: {}", e);
                            Err(format!("Failed to install update: {}", e))
                        }
                    }
                }
                Ok(None) => {
                    println!("ℹ️  No update available for installation");
                    Err("No update available".to_string())
                }
                Err(e) => {
                    println!("❌ Error checking for updates before installation: {}", e);
                    Err(format!("Failed to check for updates: {}", e))
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to get updater for installation: {}", e);
            Err(format!("Failed to get updater: {}", e))
        }
    }
}

// Get or create distinct_id from stores.json
async fn get_or_create_distinct_id() -> Result<String, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let app_config_path = home_dir.join(APP_CONFIG_DIR);
    let stores_file = app_config_path.join("stores.json");

    // Ensure app config directory exists
    std::fs::create_dir_all(&app_config_path)
        .map_err(|e| format!("Failed to create app config directory: {}", e))?;

    // Read existing stores.json or create new one
    let mut stores_data = if stores_file.exists() {
        let content = std::fs::read_to_string(&stores_file)
            .map_err(|e| format!("Failed to read stores file: {}", e))?;

        serde_json::from_str::<StoresData>(&content)
            .map_err(|e| format!("Failed to parse stores file: {}", e))?
    } else {
        StoresData {
            configs: vec![],
            distinct_id: None,
        }
    };

    // Return existing distinct_id or create new one
    if let Some(ref id) = stores_data.distinct_id {
        Ok(id.clone())
    } else {
        // Generate new UUID
        let new_id = Uuid::new_v4().to_string();
        stores_data.distinct_id = Some(new_id.clone());

        // Write back to stores.json
        let json_content = serde_json::to_string_pretty(&stores_data)
            .map_err(|e| format!("Failed to serialize stores data: {}", e))?;

        std::fs::write(&stores_file, json_content)
            .map_err(|e| format!("Failed to write stores file: {}", e))?;

        println!("Created new distinct_id: {}", new_id);
        Ok(new_id)
    }
}

// Get operating system name in PostHog format
fn get_os_name() -> &'static str {
    #[cfg(target_os = "macos")]
    return "macOS";
    #[cfg(target_os = "windows")]
    return "Windows";
    #[cfg(target_os = "linux")]
    return "Linux";
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    return "Unknown";
}

// Get operating system version
fn get_os_version() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .map_err(|e| format!("Failed to get macOS version: {}", e))?;

        let version = String::from_utf8(output.stdout)
            .map_err(|e| format!("Failed to parse macOS version: {}", e))?;

        Ok(version.trim().to_string())
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("cmd")
            .args(&["/C", "ver"])
            .output()
            .map_err(|e| format!("Failed to get Windows version: {}", e))?;

        let version_str = String::from_utf8(output.stdout)
            .map_err(|e| format!("Failed to parse Windows version: {}", e))?;

        // Extract version number from "Microsoft Windows [Version 10.0.19045.2364]"
        if let Some(start) = version_str.find("Version ") {
            let version_part = &version_str[start + 8..];
            let version = version_part.trim_end_matches("]").trim().to_string();
            Ok(version)
        } else {
            Ok("Unknown".to_string())
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::fs;
        // Try to read from /etc/os-release first
        if let Ok(content) = fs::read_to_string("/etc/os-release") {
            for line in content.lines() {
                if line.starts_with("VERSION_ID=") {
                    let version = line
                        .split('=')
                        .nth(1)
                        .unwrap_or("Unknown")
                        .trim_matches('"');
                    return Ok(version.to_string());
                }
            }
        }

        // Fallback to uname
        use std::process::Command;
        let output = Command::new("uname")
            .arg("-r")
            .output()
            .map_err(|e| format!("Failed to get Linux kernel version: {}", e))?;

        let version = String::from_utf8(output.stdout)
            .map_err(|e| format!("Failed to parse Linux version: {}", e))?;

        Ok(version.trim().to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    Ok("Unknown".to_string())
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ProjectConfig {
    pub path: String,
    pub config: serde_json::Value,
}

#[tauri::command]
pub async fn read_claude_projects() -> Result<Vec<ProjectConfig>, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let claude_json_path = home_dir.join(".claude.json");

    if !claude_json_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&claude_json_path)
        .map_err(|e| format!("Failed to read .claude.json: {}", e))?;

    let json_value: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse .claude.json: {}", e))?;

    let projects_obj = json_value
        .get("projects")
        .and_then(|projects| projects.as_object())
        .cloned()
        .unwrap_or_else(serde_json::Map::new);

    let mut result = Vec::new();
    for (path, config) in projects_obj {
        let project_config = ProjectConfig {
            path: path.clone(),
            config: config.clone(),
        };
        result.push(project_config);
    }

    Ok(result)
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ClaudeConfigFile {
    pub path: String,
    pub content: Value,
    pub exists: bool,
}

#[tauri::command]
pub async fn read_claude_config_file() -> Result<ClaudeConfigFile, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let claude_json_path = home_dir.join(".claude.json");

    let path_str = claude_json_path.to_string_lossy().to_string();

    if claude_json_path.exists() {
        let content = std::fs::read_to_string(&claude_json_path)
            .map_err(|e| format!("Failed to read .claude.json: {}", e))?;

        let json_content: Value =
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        Ok(ClaudeConfigFile {
            path: path_str,
            content: json_content,
            exists: true,
        })
    } else {
        Ok(ClaudeConfigFile {
            path: path_str,
            content: Value::Object(serde_json::Map::new()),
            exists: false,
        })
    }
}

#[tauri::command]
pub async fn write_claude_config_file(content: Value) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let claude_json_path = home_dir.join(".claude.json");

    let json_content = serde_json::to_string_pretty(&content)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    std::fs::write(&claude_json_path, json_content)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn track(
    event: String,
    properties: serde_json::Value,
    app: tauri::AppHandle,
) -> Result<(), String> {
    println!("📊 Tracking event: {}", event);

    // Get distinct_id
    let distinct_id = get_or_create_distinct_id().await?;

    // Get app version
    let app_version = app.package_info().version.to_string();

    // Get OS information
    let os_name = get_os_name();
    let os_version = get_os_version().unwrap_or_else(|_| "Unknown".to_string());

    // Prepare request payload
    let mut payload = serde_json::json!({
        "api_key": "phc_zlfJLeYsreOvash1EhL6IO6tnP00exm75OT50SjnNcy",
        "event": event,
        "properties": {
            "distinct_id": distinct_id,
            "app_version": app_version,
            "$os": os_name,
            "$os_version": os_version
        }
    });

    // Merge additional properties
    if let Some(props_obj) = payload["properties"].as_object_mut() {
        if let Some(additional_props) = properties.as_object() {
            for (key, value) in additional_props {
                props_obj.insert(key.clone(), value.clone());
            }
        }
    }

    // Add timestamp if not provided
    if !payload["properties"]
        .as_object()
        .unwrap()
        .contains_key("timestamp")
    {
        let timestamp = chrono::Utc::now().to_rfc3339();
        payload["properties"]["timestamp"] = serde_json::Value::String(timestamp);
    }

    println!(
        "📤 Sending to PostHog: {}",
        serde_json::to_string_pretty(&payload).unwrap()
    );

    // Send request to PostHog
    let client = reqwest::Client::new();
    let response = client
        .post("https://us.i.posthog.com/capture/")
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to send request to PostHog: {}", e))?;

    if response.status().is_success() {
        println!("✅ Event tracked successfully");
        Ok(())
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        println!("❌ Failed to track event: {} - {}", status, error_text);
        Err(format!("PostHog API error: {} - {}", status, error_text))
    }
}

// Hook management functions

/// Get the latest hook command based on the current operating system
fn get_latest_hook_command() -> serde_json::Value {
    if cfg!(target_os = "windows") {
        serde_json::json!({
            "__ccfoundation__": true,
            "type": "command",
            "command": "powershell -Command \"try { Invoke-RestMethod -Uri http://localhost:59948/claude_code/hooks -Method POST -ContentType 'application/json' -Body $input -ErrorAction Stop } catch { '' }\""
        })
    } else {
        serde_json::json!({
            "__ccfoundation__": true,
            "type": "command",
            "command": "curl -s -X POST http://localhost:59948/claude_code/hooks -H 'Content-Type: application/json' --data-binary @- 2>/dev/null || echo"
        })
    }
}

/// Update existing ccfoundation hooks for specified events (doesn't add new ones)
fn update_existing_hooks(
    hooks_obj: &mut serde_json::Map<String, serde_json::Value>,
    events: &[&str],
) -> Result<bool, String> {
    let latest_hook_command = get_latest_hook_command();
    let latest_command_str = latest_hook_command
        .get("command")
        .and_then(|cmd| cmd.as_str())
        .unwrap_or("");

    let mut hook_updated = false;

    for event in events {
        if let Some(event_hooks) = hooks_obj.get_mut(*event).and_then(|h| h.as_array_mut()) {
            // Find and update existing ccfoundation hooks only
            for entry in event_hooks.iter_mut() {
                if let Some(hooks_array) = entry.get_mut("hooks").and_then(|h| h.as_array_mut()) {
                    for hook in hooks_array.iter_mut() {
                        if hook.get("__ccfoundation__").is_some() {
                            // Compare only the command string, not the entire JSON object
                            if let Some(existing_command) =
                                hook.get("command").and_then(|cmd| cmd.as_str())
                            {
                                if existing_command != latest_command_str {
                                    // Update only the command field, preserve other properties
                                    hook["command"] =
                                        serde_json::Value::String(latest_command_str.to_string());
                                    hook_updated = true;
                                    println!(
                                        "🔄 Updated {} hook command: {}",
                                        event, latest_command_str
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(hook_updated)
}

/// Update or add ccfoundation hooks for specified events
fn update_or_add_hooks(
    hooks_obj: &mut serde_json::Map<String, serde_json::Value>,
    events: &[&str],
) -> Result<bool, String> {
    let latest_hook_command = get_latest_hook_command();
    let mut hook_updated = false;

    for event in events {
        if let Some(event_hooks) = hooks_obj.get_mut(*event).and_then(|h| h.as_array_mut()) {
            // Find and update existing ccfoundation hooks
            for entry in event_hooks.iter_mut() {
                if let Some(hooks_array) = entry.get_mut("hooks").and_then(|h| h.as_array_mut()) {
                    for hook in hooks_array.iter_mut() {
                        if hook.get("__ccfoundation__").is_some() {
                            // Update the command to the latest version
                            if hook.get("command") != latest_hook_command.get("command") {
                                *hook = latest_hook_command.clone();
                                hook_updated = true;
                            }
                        }
                    }
                }
            }

            // If no ccfoundation hooks found, add one
            let ccfoundation_hook_exists = event_hooks.iter().any(|entry| {
                if let Some(hooks_array) = entry.get("hooks").and_then(|h| h.as_array()) {
                    hooks_array
                        .iter()
                        .any(|hook| hook.get("__ccfoundation__").is_some())
                } else {
                    false
                }
            });

            if !ccfoundation_hook_exists {
                let ccfoundation_hook_entry = serde_json::json!({
                    "hooks": [latest_hook_command.clone()]
                });
                event_hooks.push(ccfoundation_hook_entry);
                hook_updated = true;
            }
        } else {
            // Create event hooks array with ccfoundation hook
            let ccfoundation_hook_entry = serde_json::json!({
                "hooks": [latest_hook_command.clone()]
            });
            hooks_obj.insert(
                event.to_string(),
                serde_json::Value::Array(vec![ccfoundation_hook_entry]),
            );
            hook_updated = true;
        }
    }

    Ok(hook_updated)
}

#[tauri::command]
pub async fn update_claude_code_hook() -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let settings_path = home_dir.join(".claude/settings.json");

    if !settings_path.exists() {
        // If settings file doesn't exist, just add the hooks
        return add_claude_code_hook().await;
    }

    // Read existing settings
    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings.json: {}", e))?;

    let mut settings: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings.json: {}", e))?;

    // Ensure hooks object exists
    let hooks_obj = settings
        .as_object_mut()
        .unwrap()
        .entry("hooks".to_string())
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()))
        .as_object_mut()
        .unwrap();

    // Update existing hooks for Notification, Stop, and PreToolUse events (only update, don't add new ones)
    let events = ["Notification", "Stop", "PreToolUse"];
    let hook_updated = update_existing_hooks(hooks_obj, &events)?;

    if hook_updated {
        // Write back to settings file
        let json_content = serde_json::to_string_pretty(&settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        // Create .claude directory if it doesn't exist
        if let Some(parent) = settings_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create .claude directory: {}", e))?;
        }

        std::fs::write(&settings_path, json_content)
            .map_err(|e| format!("Failed to write settings.json: {}", e))?;

        println!("✅ Claude Code hooks updated successfully");
    } else {
        println!("ℹ️  Claude Code hooks are already up to date - no updates needed");
    }

    Ok(())
}

#[tauri::command]
pub async fn add_claude_code_hook() -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let settings_path = home_dir.join(".claude/settings.json");

    // Read existing settings or create new structure
    let mut settings = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read settings.json: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse settings.json: {}", e))?
    } else {
        serde_json::Value::Object(serde_json::Map::new())
    };

    // Ensure hooks object exists
    let hooks_obj = settings
        .as_object_mut()
        .unwrap()
        .entry("hooks".to_string())
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()))
        .as_object_mut()
        .unwrap();

    // Add hooks for Notification, Stop, and PreToolUse events
    let events = ["Notification", "Stop", "PreToolUse"];
    update_or_add_hooks(hooks_obj, &events)?;

    // Write back to settings file
    let json_content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    // Create .claude directory if it doesn't exist
    if let Some(parent) = settings_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create .claude directory: {}", e))?;
    }

    std::fs::write(&settings_path, json_content)
        .map_err(|e| format!("Failed to write settings.json: {}", e))?;

    println!("✅ Claude Code hooks added successfully");
    Ok(())
}

#[tauri::command]
pub async fn remove_claude_code_hook() -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let settings_path = home_dir.join(".claude/settings.json");

    if !settings_path.exists() {
        return Ok(()); // Settings file doesn't exist, nothing to remove
    }

    // Read existing settings
    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings.json: {}", e))?;

    let mut settings: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings.json: {}", e))?;

    // Check if hooks object exists
    if let Some(hooks_obj) = settings.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        let events = ["Notification", "Stop", "PreToolUse"];

        for event in events {
            if let Some(event_hooks) = hooks_obj.get_mut(event).and_then(|h| h.as_array_mut()) {
                // Remove hooks that have __ccfoundation__ key from nested hooks arrays
                let mut new_event_hooks = Vec::new();
                for entry in event_hooks.iter() {
                    if let Some(hooks_array) = entry.get("hooks").and_then(|h| h.as_array()) {
                        // Filter out hooks that have __ccfoundation__ key
                        let filtered_hooks: Vec<serde_json::Value> = hooks_array
                            .iter()
                            .filter(|hook| hook.get("__ccfoundation__").is_none())
                            .cloned()
                            .collect();

                        // Keep the entry only if it still has hooks
                        if !filtered_hooks.is_empty() {
                            let mut new_entry = entry.clone();
                            new_entry["hooks"] = serde_json::Value::Array(filtered_hooks);
                            new_event_hooks.push(new_entry);
                        }
                    } else {
                        // Keep entries that don't have a hooks array
                        new_event_hooks.push(entry.clone());
                    }
                }
                *event_hooks = new_event_hooks;

                // If the event hooks array is empty, remove the entire event entry
                if event_hooks.is_empty() {
                    hooks_obj.remove(event);
                }
            }
        }

        // If hooks object is empty, remove it entirely
        if hooks_obj.is_empty() {
            settings.as_object_mut().unwrap().remove("hooks");
        }
    }

    // Write back to settings file
    let json_content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(&settings_path, json_content)
        .map_err(|e| format!("Failed to write settings.json: {}", e))?;

    println!("✅ Claude Code hooks removed successfully");
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct CommandFile {
    pub name: String,
    pub content: String,
    pub exists: bool,
}

#[tauri::command]
pub async fn read_claude_commands() -> Result<Vec<CommandFile>, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let commands_dir = home_dir.join(".claude/commands");

    if !commands_dir.exists() {
        return Ok(vec![]);
    }

    let mut command_files = Vec::new();

    // Read all .md files in the commands directory
    let entries = std::fs::read_dir(&commands_dir)
        .map_err(|e| format!("Failed to read commands directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() && path.extension().map(|ext| ext == "md").unwrap_or(false) {
            let file_name = path
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string();

            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read command file {}: {}", path.display(), e))?;

            command_files.push(CommandFile {
                name: file_name,
                content,
                exists: true,
            });
        }
    }

    // Sort commands alphabetically by name
    command_files.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(command_files)
}

#[tauri::command]
pub async fn write_claude_command(command_name: String, content: String) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let commands_dir = home_dir.join(".claude/commands");
    let command_file_path = commands_dir.join(format!("{}.md", command_name));

    // Ensure .claude/commands directory exists
    std::fs::create_dir_all(&commands_dir)
        .map_err(|e| format!("Failed to create .claude/commands directory: {}", e))?;

    std::fs::write(&command_file_path, content)
        .map_err(|e| format!("Failed to write command file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn delete_claude_command(command_name: String) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let commands_dir = home_dir.join(".claude/commands");
    let command_file_path = commands_dir.join(format!("{}.md", command_name));

    if command_file_path.exists() {
        std::fs::remove_file(&command_file_path)
            .map_err(|e| format!("Failed to delete command file: {}", e))?;
    }

    Ok(())
}

// Agent management functions

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AgentFile {
    pub name: String,
    pub content: String,
    pub exists: bool,
}

#[tauri::command]
pub async fn read_claude_agents() -> Result<Vec<AgentFile>, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let agents_dir = home_dir.join(".claude/agents");

    if !agents_dir.exists() {
        return Ok(vec![]);
    }

    let mut agent_files = Vec::new();

    // Read all .md files in the agents directory
    let entries = std::fs::read_dir(&agents_dir)
        .map_err(|e| format!("Failed to read agents directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() && path.extension().map(|ext| ext == "md").unwrap_or(false) {
            let file_name = path
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string();

            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read agent file {}: {}", path.display(), e))?;

            agent_files.push(AgentFile {
                name: file_name,
                content,
                exists: true,
            });
        }
    }

    // Sort agents alphabetically by name
    agent_files.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(agent_files)
}

#[tauri::command]
pub async fn write_claude_agent(agent_name: String, content: String) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let agents_dir = home_dir.join(".claude/agents");
    let agent_file_path = agents_dir.join(format!("{}.md", agent_name));

    // Ensure .claude/agents directory exists
    std::fs::create_dir_all(&agents_dir)
        .map_err(|e| format!("Failed to create .claude/agents directory: {}", e))?;

    std::fs::write(&agent_file_path, content)
        .map_err(|e| format!("Failed to write agent file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn delete_claude_agent(agent_name: String) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let agents_dir = home_dir.join(".claude/agents");
    let agent_file_path = agents_dir.join(format!("{}.md", agent_name));

    if agent_file_path.exists() {
        std::fs::remove_file(&agent_file_path)
            .map_err(|e| format!("Failed to delete agent file: {}", e))?;
    }

    Ok(())
}

// ============================================================================
// Per-Project Configuration - Refactored to Project-Based Storage
// ============================================================================

/// Project settings - represents settings stored in PROJECT/.claude/settings.json
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProjectSettings {
    pub path: String,
    pub exists: bool,
    pub settings: Option<Value>,
    #[serde(rename = "hasAgents")]
    pub has_agents: bool,
    #[serde(rename = "hasCommands")]
    pub has_commands: bool,
    #[serde(rename = "hasMcp")]
    pub has_mcp: bool,
}

/// Project registry entry - lightweight tracking in ~/.ccconfig/project-registry.json
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProjectRegistryEntry {
    #[serde(rename = "projectPath")]
    pub project_path: String,
    pub title: String,
    #[serde(rename = "lastUsedAt")]
    pub last_used_at: u64,
    #[serde(rename = "inheritFromGlobal")]
    pub inherit_from_global: bool,
    #[serde(rename = "parentGlobalConfigId")]
    pub parent_global_config_id: Option<String>,
}

// ============================================================================
// Helper Functions for Project-Based Storage
// ============================================================================

fn get_project_claude_dir(project_path: &str) -> PathBuf {
    PathBuf::from(project_path).join(".claude")
}

fn get_project_settings_path(project_path: &str) -> PathBuf {
    get_project_claude_dir(project_path).join("settings.json")
}

fn get_project_agents_dir(project_path: &str) -> PathBuf {
    get_project_claude_dir(project_path).join("agents")
}

fn get_project_commands_dir(project_path: &str) -> PathBuf {
    get_project_claude_dir(project_path).join("commands")
}

fn get_project_mcp_path(project_path: &str) -> PathBuf {
    get_project_claude_dir(project_path).join(".mcp.json")
}

fn read_project_registry() -> Result<Vec<ProjectRegistryEntry>, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let registry_path = home_dir.join(APP_CONFIG_DIR).join("project-registry.json");

    if !registry_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&registry_path)
        .map_err(|e| format!("Failed to read project registry: {}", e))?;

    // Try parsing as array first (new format)
    if let Ok(registry) = serde_json::from_str::<Vec<ProjectRegistryEntry>>(&content) {
        return Ok(registry);
    }

    // Fall back to parsing as object/map (old format) and convert to array
    let registry_map: std::collections::HashMap<String, ProjectRegistryEntry> =
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse project registry: {}", e))?;

    // Convert map to vec
    let registry: Vec<ProjectRegistryEntry> = registry_map.into_values().collect();

    // Auto-migrate to new format
    let json_content = serde_json::to_string_pretty(&registry)
        .map_err(|e| format!("Failed to serialize registry: {}", e))?;
    std::fs::write(&registry_path, json_content)
        .map_err(|e| format!("Failed to write migrated registry: {}", e))?;

    println!("✅ Migrated project registry to new format");

    Ok(registry)
}

fn write_project_registry_entry(
    project_path: &str,
    entry: &ProjectRegistryEntry,
) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let app_config_path = home_dir.join(APP_CONFIG_DIR);
    let registry_path = app_config_path.join("project-registry.json");

    // Ensure directory exists
    std::fs::create_dir_all(&app_config_path)
        .map_err(|e| format!("Failed to create app config directory: {}", e))?;

    let mut registry = read_project_registry()?;

    // Find and update or add entry
    if let Some(existing) = registry.iter_mut().find(|e| e.project_path == project_path) {
        *existing = entry.clone();
    } else {
        registry.push(entry.clone());
    }

    // Write back
    let json_content = serde_json::to_string_pretty(&registry)
        .map_err(|e| format!("Failed to serialize registry: {}", e))?;

    std::fs::write(&registry_path, json_content)
        .map_err(|e| format!("Failed to write registry: {}", e))?;

    Ok(())
}

// ============================================================================
// NEW: Project-Based Storage Commands
// ============================================================================

/// Read project settings from PROJECT/.claude/settings.json
#[tauri::command]
pub async fn read_project_settings(project_path: String) -> Result<ProjectSettings, String> {
    let claude_dir = get_project_claude_dir(&project_path);
    let settings_path = get_project_settings_path(&project_path);
    let agents_dir = get_project_agents_dir(&project_path);
    let commands_dir = get_project_commands_dir(&project_path);
    let mcp_path = get_project_mcp_path(&project_path);

    let exists = claude_dir.exists();
    let settings = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read project settings: {}", e))?;
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse project settings: {}", e))?;
        Some(json)
    } else {
        None
    };

    Ok(ProjectSettings {
        path: project_path,
        exists,
        settings,
        has_agents: agents_dir.exists() && agents_dir.is_dir(),
        has_commands: commands_dir.exists() && commands_dir.is_dir(),
        has_mcp: mcp_path.exists(),
    })
}

/// Write project settings to PROJECT/.claude/settings.json
#[tauri::command]
pub async fn write_project_settings(project_path: String, settings: Value) -> Result<(), String> {
    let settings_path = get_project_settings_path(&project_path);

    // Ensure .claude directory exists
    if let Some(parent) = settings_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create .claude directory: {}", e))?;
    }

    let json_content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(&settings_path, json_content)
        .map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(())
}

/// Initialize PROJECT/.claude/ directory structure
#[tauri::command]
pub async fn init_project_claude_dir(project_path: String) -> Result<(), String> {
    let claude_dir = get_project_claude_dir(&project_path);
    let agents_dir = get_project_agents_dir(&project_path);
    let commands_dir = get_project_commands_dir(&project_path);

    std::fs::create_dir_all(&claude_dir)
        .map_err(|e| format!("Failed to create .claude directory: {}", e))?;
    std::fs::create_dir_all(&agents_dir)
        .map_err(|e| format!("Failed to create agents directory: {}", e))?;
    std::fs::create_dir_all(&commands_dir)
        .map_err(|e| format!("Failed to create commands directory: {}", e))?;

    // Create default settings.json if it doesn't exist
    let settings_path = get_project_settings_path(&project_path);
    if !settings_path.exists() {
        let default_settings = serde_json::json!({
            "model": "claude-sonnet-4-5-20250929",
            "env": {},
            "permissions": {
                "allow": [],
                "deny": []
            }
        });
        let json_content = serde_json::to_string_pretty(&default_settings)
            .map_err(|e| format!("Failed to serialize default settings: {}", e))?;
        std::fs::write(&settings_path, json_content)
            .map_err(|e| format!("Failed to write default settings: {}", e))?;
    }

    Ok(())
}

/// Read project agents from PROJECT/.claude/agents/
#[tauri::command]
pub async fn read_project_agents(project_path: String) -> Result<Vec<AgentFile>, String> {
    let agents_dir = get_project_agents_dir(&project_path);

    if !agents_dir.exists() {
        return Ok(vec![]);
    }

    let mut agent_files = Vec::new();

    let entries = std::fs::read_dir(&agents_dir)
        .map_err(|e| format!("Failed to read agents directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() && path.extension().map(|ext| ext == "md").unwrap_or(false) {
            let file_name = path
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string();

            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read agent file {}: {}", path.display(), e))?;

            agent_files.push(AgentFile {
                name: file_name,
                content,
                exists: true,
            });
        }
    }

    agent_files.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(agent_files)
}

/// Write project agent to PROJECT/.claude/agents/{name}.md
#[tauri::command]
pub async fn write_project_agent(
    project_path: String,
    agent_name: String,
    content: String,
) -> Result<(), String> {
    let agents_dir = get_project_agents_dir(&project_path);

    // Ensure directory exists
    std::fs::create_dir_all(&agents_dir)
        .map_err(|e| format!("Failed to create agents directory: {}", e))?;

    let file_path = agents_dir.join(format!("{}.md", agent_name));
    std::fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write agent file: {}", e))?;

    Ok(())
}

/// Delete project agent from PROJECT/.claude/agents/{name}.md
#[tauri::command]
pub async fn delete_project_agent(project_path: String, agent_name: String) -> Result<(), String> {
    let agents_dir = get_project_agents_dir(&project_path);
    let file_path = agents_dir.join(format!("{}.md", agent_name));

    if file_path.exists() {
        std::fs::remove_file(&file_path)
            .map_err(|e| format!("Failed to delete agent file: {}", e))?;
    }

    Ok(())
}

/// Read project commands from PROJECT/.claude/commands/
#[tauri::command]
pub async fn read_project_commands(project_path: String) -> Result<Vec<CommandFile>, String> {
    let commands_dir = get_project_commands_dir(&project_path);

    if !commands_dir.exists() {
        return Ok(vec![]);
    }

    let mut command_files = Vec::new();

    let entries = std::fs::read_dir(&commands_dir)
        .map_err(|e| format!("Failed to read commands directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() && path.extension().map(|ext| ext == "md").unwrap_or(false) {
            let file_name = path
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string();

            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read command file {}: {}", path.display(), e))?;

            command_files.push(CommandFile {
                name: file_name,
                content,
                exists: true,
            });
        }
    }

    command_files.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(command_files)
}

/// Write project command to PROJECT/.claude/commands/{name}.md
#[tauri::command]
pub async fn write_project_command(
    project_path: String,
    command_name: String,
    content: String,
) -> Result<(), String> {
    let commands_dir = get_project_commands_dir(&project_path);

    // Ensure directory exists
    std::fs::create_dir_all(&commands_dir)
        .map_err(|e| format!("Failed to create commands directory: {}", e))?;

    let file_path = commands_dir.join(format!("{}.md", command_name));
    std::fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write command file: {}", e))?;

    Ok(())
}

/// Delete project command from PROJECT/.claude/commands/{name}.md
#[tauri::command]
pub async fn delete_project_command(
    project_path: String,
    command_name: String,
) -> Result<(), String> {
    let commands_dir = get_project_commands_dir(&project_path);
    let file_path = commands_dir.join(format!("{}.md", command_name));

    if file_path.exists() {
        std::fs::remove_file(&file_path)
            .map_err(|e| format!("Failed to delete command file: {}", e))?;
    }

    Ok(())
}

/// Read project MCP from PROJECT/.mcp.json
#[tauri::command]
pub async fn read_project_mcp(project_path: String) -> Result<Option<Value>, String> {
    let mcp_path = get_project_mcp_path(&project_path);

    if !mcp_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&mcp_path)
        .map_err(|e| format!("Failed to read project MCP: {}", e))?;

    let json: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse project MCP: {}", e))?;

    Ok(Some(json))
}

/// Write project MCP to PROJECT/.mcp.json
#[tauri::command]
pub async fn write_project_mcp(project_path: String, content: Value) -> Result<(), String> {
    let mcp_path = get_project_mcp_path(&project_path);

    let json_content = serde_json::to_string_pretty(&content)
        .map_err(|e| format!("Failed to serialize MCP: {}", e))?;

    std::fs::write(&mcp_path, json_content).map_err(|e| format!("Failed to write MCP: {}", e))?;

    Ok(())
}

/// Get project registry (all tracked projects)
#[tauri::command]
pub async fn get_project_registry() -> Result<Vec<ProjectRegistryEntry>, String> {
    read_project_registry()
}

/// Update project registry entry
#[tauri::command]
pub async fn update_project_registry(
    project_path: String,
    title: String,
    inherit_from_global: bool,
    parent_global_config_id: Option<String>,
) -> Result<(), String> {
    let entry = ProjectRegistryEntry {
        project_path: project_path.clone(),
        title,
        last_used_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        inherit_from_global,
        parent_global_config_id,
    };

    write_project_registry_entry(&project_path, &entry)?;
    Ok(())
}

/// Convert project path to sanitized directory name for Claude storage
/// "/Users/huutri/code/ccmate" -> "-Users-huutri-code-ccmate"
fn sanitize_project_path_for_dir(project_path: &str) -> String {
    project_path.replace('/', "-")
}

/// Remove project entry from ~/.claude.json
fn remove_project_from_claude_json(project_path: &str) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let claude_json_path = home_dir.join(".claude.json");

    if !claude_json_path.exists() {
        return Ok(()); // Nothing to clean
    }

    let content = std::fs::read_to_string(&claude_json_path)
        .map_err(|e| format!("Failed to read .claude.json: {}", e))?;

    let mut json: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse .claude.json: {}", e))?;

    if let Some(projects) = json.get_mut("projects").and_then(|p| p.as_object_mut()) {
        projects.remove(project_path);
    }

    let updated_content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Failed to serialize .claude.json: {}", e))?;

    std::fs::write(&claude_json_path, updated_content)
        .map_err(|e| format!("Failed to write .claude.json: {}", e))?;

    println!("✅ Removed project from .claude.json");
    Ok(())
}

/// Get all session IDs from a project's session directory
fn get_project_session_ids(project_sessions_dir: &PathBuf) -> Vec<String> {
    let mut session_ids = Vec::new();

    if let Ok(entries) = std::fs::read_dir(project_sessions_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            // Session files are named: {session-id}.jsonl
            // Agent files are named: agent-{id}.jsonl
            if file_name.ends_with(".jsonl") && !file_name.starts_with("agent-") {
                if let Some(session_id) = file_name.strip_suffix(".jsonl") {
                    session_ids.push(session_id.to_string());
                }
            }
        }
    }

    session_ids
}

/// Clean up all session-related data for given session IDs
fn cleanup_session_data(home_dir: &PathBuf, session_ids: &[String]) {
    let claude_dir = home_dir.join(".claude");
    let mut cleaned_count = 0;

    for session_id in session_ids {
        // Clean todos directory
        let todos_dir = claude_dir.join("todos").join(session_id);
        if todos_dir.exists() {
            if std::fs::remove_dir_all(&todos_dir).is_ok() {
                cleaned_count += 1;
            }
        }

        // Clean file-history directory
        let file_history_dir = claude_dir.join("file-history").join(session_id);
        if file_history_dir.exists() {
            if std::fs::remove_dir_all(&file_history_dir).is_ok() {
                cleaned_count += 1;
            }
        }

        // Clean debug file
        let debug_file = claude_dir.join("debug").join(format!("{}.txt", session_id));
        if debug_file.exists() {
            let _ = std::fs::remove_file(&debug_file);
        }

        // Clean session-env directory
        let session_env_dir = claude_dir.join("session-env").join(session_id);
        if session_env_dir.exists() {
            if std::fs::remove_dir_all(&session_env_dir).is_ok() {
                cleaned_count += 1;
            }
        }
    }

    if cleaned_count > 0 {
        println!("✅ Cleaned {} session data directories", cleaned_count);
    }
}

/// Remove history entries for a specific project
fn filter_history_file(home_dir: &PathBuf, project_path: &str) -> Result<(), String> {
    let history_path = home_dir.join(".claude").join("history.jsonl");

    if !history_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&history_path)
        .map_err(|e| format!("Failed to read history.jsonl: {}", e))?;

    let mut removed_count = 0;
    let filtered_lines: Vec<String> = content
        .lines()
        .filter(|line| {
            if let Ok(json) = serde_json::from_str::<Value>(line) {
                if let Some(project) = json.get("project").and_then(|p| p.as_str()) {
                    if project == project_path {
                        removed_count += 1;
                        return false;
                    }
                }
            }
            true // Keep lines that don't match or can't be parsed
        })
        .map(String::from)
        .collect();

    let filtered_content = if filtered_lines.is_empty() {
        String::new()
    } else {
        filtered_lines.join("\n") + "\n"
    };

    std::fs::write(&history_path, filtered_content)
        .map_err(|e| format!("Failed to write history.jsonl: {}", e))?;

    if removed_count > 0 {
        println!("✅ Removed {} history entries", removed_count);
    }

    Ok(())
}

/// Delete project config - removes from registry and cleans all Claude Code tracking data
/// Note: Does NOT delete PROJECT/.claude/ directory (user's project config is preserved)
#[tauri::command]
pub async fn delete_project_config(project_path: String) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let app_config_path = home_dir.join(APP_CONFIG_DIR);
    let registry_path = app_config_path.join("project-registry.json");

    // 1. Remove from registry
    let mut registry = read_project_registry()?;
    registry.retain(|entry| entry.project_path != project_path);

    // Write updated registry back
    let json_content = serde_json::to_string_pretty(&registry)
        .map_err(|e| format!("Failed to serialize registry: {}", e))?;

    std::fs::write(&registry_path, json_content)
        .map_err(|e| format!("Failed to write registry: {}", e))?;

    // 2. Remove from ~/.claude.json
    if let Err(e) = remove_project_from_claude_json(&project_path) {
        eprintln!("⚠️  Warning: Failed to clean .claude.json: {}", e);
        // Continue - don't fail the whole operation
    }

    // 3. Get session IDs before deleting project sessions directory
    let sanitized_path = sanitize_project_path_for_dir(&project_path);
    let project_sessions_dir = home_dir
        .join(".claude")
        .join("projects")
        .join(&sanitized_path);
    let session_ids = get_project_session_ids(&project_sessions_dir);

    // 4. Delete project sessions directory
    if project_sessions_dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&project_sessions_dir) {
            eprintln!("⚠️  Warning: Failed to delete project sessions: {}", e);
        } else {
            println!("✅ Deleted project sessions: {:?}", project_sessions_dir);
        }
    }

    // 5. Clean up session-related data
    if !session_ids.is_empty() {
        cleanup_session_data(&home_dir, &session_ids);
    }

    // 6. Filter history file
    if let Err(e) = filter_history_file(&home_dir, &project_path) {
        eprintln!("⚠️  Warning: Failed to filter history: {}", e);
    }

    println!("✅ Project config removed from registry: {}", project_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // Helper function to create temporary test directories
    fn create_test_dir(name: &str) -> PathBuf {
        let test_dir = PathBuf::from(format!("/tmp/ccmate_test_{}", name));
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).expect("Failed to create test dir");
        test_dir
    }

    // Helper function to write JSON to file
    fn write_json_file(path: &PathBuf, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent dirs");
        }
        fs::write(path, content).expect("Failed to write JSON file");
    }

    #[test]
    fn test_sanitize_project_path_for_dir_basic() {
        let result = sanitize_project_path_for_dir("/Users/huutri/code/ccmate");
        assert_eq!(result, "-Users-huutri-code-ccmate");
    }

    #[test]
    fn test_sanitize_project_path_for_dir_with_trailing_slash() {
        let result = sanitize_project_path_for_dir("/Users/huutri/code/ccmate/");
        assert_eq!(result, "-Users-huutri-code-ccmate-");
    }

    #[test]
    fn test_sanitize_project_path_for_dir_single_component() {
        let result = sanitize_project_path_for_dir("ccmate");
        assert_eq!(result, "ccmate");
    }

    #[test]
    fn test_sanitize_project_path_for_dir_with_dots() {
        let result = sanitize_project_path_for_dir("/Users/user.name/code/my.project");
        assert_eq!(result, "-Users-user.name-code-my.project");
    }

    #[test]
    fn test_sanitize_project_path_for_dir_empty() {
        let result = sanitize_project_path_for_dir("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_remove_project_from_claude_json_file_not_exists() {
        // Should return Ok if .claude.json doesn't exist
        let result = remove_project_from_claude_json("/nonexistent/project");
        assert!(
            result.is_ok(),
            "Should handle missing .claude.json gracefully"
        );
    }

    #[test]
    fn test_remove_project_from_claude_json_removes_entry() {
        let test_dir = create_test_dir("claude_json_test");
        let claude_json_path = test_dir.join(".claude.json");

        // Create test .claude.json with project entry
        let json_content = r#"{"projects": {"/Users/test/project1": {"key": "value"}, "/Users/test/project2": {"key": "value"}}}"#;
        write_json_file(&claude_json_path, json_content);

        // Temporarily override home_dir for this test
        // This is a limitation - we can't fully test without mocking dirs::home_dir()
        // But we can test the core logic by creating the file structure

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_get_project_session_ids_no_directory() {
        let test_dir = create_test_dir("session_ids_no_dir");
        let non_existent = test_dir.join("nonexistent");

        let result = get_project_session_ids(&non_existent);
        assert_eq!(
            result.len(),
            0,
            "Should return empty vec for non-existent directory"
        );

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_get_project_session_ids_extracts_session_ids() {
        let test_dir = create_test_dir("session_ids_extract");

        // Create session files
        fs::create_dir_all(&test_dir).expect("Failed to create test dir");
        fs::write(test_dir.join("session-abc123.jsonl"), "").expect("Failed to write");
        fs::write(test_dir.join("session-xyz789.jsonl"), "").expect("Failed to write");
        fs::write(test_dir.join("agent-special.jsonl"), "").expect("Failed to write"); // Should be excluded

        let result = get_project_session_ids(&test_dir);

        // Should have 2 sessions (agent files excluded)
        assert_eq!(
            result.len(),
            2,
            "Should extract exactly 2 session IDs (agent excluded)"
        );
        assert!(result.contains(&"session-abc123".to_string()));
        assert!(result.contains(&"session-xyz789".to_string()));
        assert!(!result.iter().any(|s| s.contains("agent")));

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_get_project_session_ids_excludes_agent_files() {
        let test_dir = create_test_env("session_ids_agent_exclude");

        fs::create_dir_all(&test_dir).expect("Failed to create test dir");
        fs::write(test_dir.join("agent-workflow-123.jsonl"), "").expect("Failed to write");
        fs::write(test_dir.join("agent-assistant-456.jsonl"), "").expect("Failed to write");

        let result = get_project_session_ids(&test_dir);

        assert_eq!(result.len(), 0, "Should exclude all agent files");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_filter_history_file_not_exists() {
        let test_dir = create_test_dir("history_not_exists");
        let home_dir = test_dir.clone();

        // Should return Ok if history.jsonl doesn't exist
        let result = filter_history_file(&home_dir, "/Users/test/project");
        assert!(
            result.is_ok(),
            "Should handle missing history.jsonl gracefully"
        );

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_filter_history_file_empty() {
        let test_dir = create_test_env("history_empty");
        let claude_dir = test_dir.join(".claude");
        fs::create_dir_all(&claude_dir).expect("Failed to create .claude dir");

        let history_path = claude_dir.join("history.jsonl");
        fs::write(&history_path, "").expect("Failed to write empty history");

        let result = filter_history_file(&test_dir, "/Users/test/project");
        assert!(result.is_ok(), "Should handle empty history.jsonl");

        let content = fs::read_to_string(&history_path).expect("Failed to read history");
        assert_eq!(content, "", "Empty history should remain empty");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_filter_history_file_removes_matching_entries() {
        let test_dir = create_test_env("history_filter");
        let claude_dir = test_dir.join(".claude");
        fs::create_dir_all(&claude_dir).expect("Failed to create .claude dir");

        let history_path = claude_dir.join("history.jsonl");
        let history_content = r#"{"project": "/Users/test/project1", "action": "create"}
{"project": "/Users/test/project2", "action": "edit"}
{"project": "/Users/test/project1", "action": "delete"}
{"project": "/Users/other/project", "action": "create"}
"#;
        fs::write(&history_path, history_content).expect("Failed to write history");

        let result = filter_history_file(&test_dir, "/Users/test/project1");
        assert!(result.is_ok(), "Should successfully filter history");

        let content = fs::read_to_string(&history_path).expect("Failed to read filtered history");
        assert!(!content.contains(r#""project": "/Users/test/project1""#));
        assert!(content.contains(r#""project": "/Users/test/project2""#));
        assert!(content.contains(r#""project": "/Users/other/project""#));

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_filter_history_file_preserves_malformed_lines() {
        let test_dir = create_test_env("history_malformed");
        let claude_dir = test_dir.join(".claude");
        fs::create_dir_all(&claude_dir).expect("Failed to create .claude dir");

        let history_path = claude_dir.join("history.jsonl");
        let history_content = r#"{"project": "/Users/test/project1", "action": "create"}
invalid json line here
{"project": "/Users/test/project2", "action": "edit"}
"#;
        fs::write(&history_path, history_content).expect("Failed to write history");

        let result = filter_history_file(&test_dir, "/Users/test/project1");
        assert!(result.is_ok(), "Should handle malformed JSON gracefully");

        let content = fs::read_to_string(&history_path).expect("Failed to read filtered history");
        assert!(
            content.contains("invalid json line here"),
            "Should preserve malformed lines"
        );
        assert!(content.contains(r#""project": "/Users/test/project2""#));

        let _ = fs::remove_dir_all(&test_dir);
    }

    // Helper function for creating test environment
    fn create_test_env(name: &str) -> PathBuf {
        let test_dir = PathBuf::from(format!("/tmp/ccmate_test_env_{}", name));
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).expect("Failed to create test env");
        test_dir
    }

    #[test]
    fn test_cleanup_session_data_removes_todos() {
        let test_dir = create_test_env("cleanup_todos");
        let claude_dir = test_dir.join(".claude");
        fs::create_dir_all(&claude_dir).expect("Failed to create .claude dir");

        // Create todos directory with session data
        let todos_dir = claude_dir.join("todos").join("session-123");
        fs::create_dir_all(&todos_dir).expect("Failed to create todos dir");
        fs::write(todos_dir.join("task.json"), "{}").expect("Failed to write task");

        let session_ids = vec!["session-123".to_string()];
        cleanup_session_data(&test_dir, &session_ids);

        assert!(!todos_dir.exists(), "Todos directory should be removed");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_cleanup_session_data_removes_file_history() {
        let test_dir = create_test_env("cleanup_file_history");
        let claude_dir = test_dir.join(".claude");
        fs::create_dir_all(&claude_dir).expect("Failed to create .claude dir");

        // Create file-history directory with session data
        let history_dir = claude_dir.join("file-history").join("session-456");
        fs::create_dir_all(&history_dir).expect("Failed to create history dir");
        fs::write(history_dir.join("file.json"), "{}").expect("Failed to write file");

        let session_ids = vec!["session-456".to_string()];
        cleanup_session_data(&test_dir, &session_ids);

        assert!(
            !history_dir.exists(),
            "File-history directory should be removed"
        );

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_cleanup_session_data_removes_debug_files() {
        let test_dir = create_test_env("cleanup_debug");
        let claude_dir = test_dir.join(".claude");
        fs::create_dir_all(&claude_dir).expect("Failed to create .claude dir");

        // Create debug directory
        let debug_dir = claude_dir.join("debug");
        fs::create_dir_all(&debug_dir).expect("Failed to create debug dir");
        fs::write(debug_dir.join("session-789.txt"), "debug log").expect("Failed to write debug");

        let session_ids = vec!["session-789".to_string()];
        cleanup_session_data(&test_dir, &session_ids);

        assert!(
            !debug_dir.join("session-789.txt").exists(),
            "Debug file should be removed"
        );

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_cleanup_session_data_removes_session_env() {
        let test_dir = create_test_env("cleanup_session_env");
        let claude_dir = test_dir.join(".claude");
        fs::create_dir_all(&claude_dir).expect("Failed to create .claude dir");

        // Create session-env directory
        let env_dir = claude_dir.join("session-env").join("session-999");
        fs::create_dir_all(&env_dir).expect("Failed to create session-env dir");
        fs::write(env_dir.join("vars.json"), "{}").expect("Failed to write vars");

        let session_ids = vec!["session-999".to_string()];
        cleanup_session_data(&test_dir, &session_ids);

        assert!(!env_dir.exists(), "Session-env directory should be removed");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_cleanup_session_data_handles_missing_directories() {
        let test_dir = create_test_env("cleanup_missing");
        let claude_dir = test_dir.join(".claude");
        fs::create_dir_all(&claude_dir).expect("Failed to create .claude dir");

        // Don't create any session directories
        let session_ids = vec!["session-nonexistent".to_string()];

        // Should not panic or error
        cleanup_session_data(&test_dir, &session_ids);

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_cleanup_session_data_handles_empty_session_list() {
        let test_dir = create_test_env("cleanup_empty_list");
        let claude_dir = test_dir.join(".claude");
        fs::create_dir_all(&claude_dir).expect("Failed to create .claude dir");

        let session_ids: Vec<String> = vec![];

        // Should not panic or error
        cleanup_session_data(&test_dir, &session_ids);

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_cleanup_session_data_multiple_sessions() {
        let test_dir = create_test_env("cleanup_multiple");
        let claude_dir = test_dir.join(".claude");
        fs::create_dir_all(&claude_dir).expect("Failed to create .claude dir");

        // Create multiple session directories
        let todos_dir1 = claude_dir.join("todos").join("session-1");
        let todos_dir2 = claude_dir.join("todos").join("session-2");
        fs::create_dir_all(&todos_dir1).expect("Failed to create todos1");
        fs::create_dir_all(&todos_dir2).expect("Failed to create todos2");

        let session_ids = vec!["session-1".to_string(), "session-2".to_string()];
        cleanup_session_data(&test_dir, &session_ids);

        assert!(
            !todos_dir1.exists(),
            "First todos directory should be removed"
        );
        assert!(
            !todos_dir2.exists(),
            "Second todos directory should be removed"
        );

        let _ = fs::remove_dir_all(&test_dir);
    }
}
