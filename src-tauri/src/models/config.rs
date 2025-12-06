use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Model configuration loaded from model-mapping.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub version: String,
    #[serde(rename = "defaultModel")]
    pub default_model: String,
    pub models: Vec<ModelInfo>,
}

/// Information about a Claude model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub family: String,
    #[serde(rename = "releaseDate")]
    pub release_date: String,
    pub aliases: Vec<String>,
}

/// Cached model configuration
static MODEL_CONFIG: OnceLock<ModelConfig> = OnceLock::new();

/// Cached alias -> model ID mapping
static ALIAS_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();

/// Load model configuration from embedded JSON file
pub fn load_config() -> &'static ModelConfig {
    MODEL_CONFIG.get_or_init(|| {
        // Load embedded config file at compile time
        let config_str = include_str!("../../resources/model-mapping.json");

        match serde_json::from_str::<ModelConfig>(config_str) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("⚠️  Failed to parse model config: {}", e);
                eprintln!("   Falling back to empty config");
                // Return minimal config
                ModelConfig {
                    version: "1.0".to_string(),
                    default_model: "claude-sonnet-4-5-20250929".to_string(),
                    models: vec![],
                }
            }
        }
    })
}

/// Get or build the alias -> model ID mapping
pub fn get_alias_map() -> &'static HashMap<String, String> {
    ALIAS_MAP.get_or_init(|| {
        let config = load_config();
        let mut map = HashMap::new();

        for model in &config.models {
            // Map the model ID to itself (for exact matches)
            map.insert(model.id.to_lowercase(), model.id.clone());

            // Map all aliases to the model ID
            for alias in &model.aliases {
                map.insert(alias.to_lowercase(), model.id.clone());
            }
        }

        println!("📋 Loaded {} model aliases", map.len());
        map
    })
}

/// Get list of all models for frontend
pub fn get_all_models() -> Vec<ModelInfo> {
    load_config().models.clone()
}

/// Get default model ID
pub fn get_default_model() -> String {
    load_config().default_model.clone()
}
