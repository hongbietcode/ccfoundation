//! Model name mapping and normalization
//!
//! This module provides functionality to map short model names and aliases
//! to their full API identifiers using a configuration file.

pub mod config;
pub mod normalize;

pub use config::{get_all_models, get_default_model, load_config, ModelConfig, ModelInfo};
pub use normalize::{normalize_model_name, normalize_model_option};

use tauri::command;

/// Tauri command to get all available models
#[command]
pub async fn get_models() -> Result<Vec<ModelInfo>, String> {
    Ok(get_all_models())
}

/// Tauri command to get the default model ID
#[command]
pub async fn get_default_model_id() -> Result<String, String> {
    Ok(get_default_model())
}

/// Tauri command to normalize a model name
#[command]
pub async fn normalize_model(model_name: String) -> Result<String, String> {
    Ok(normalize_model_name(&model_name))
}
