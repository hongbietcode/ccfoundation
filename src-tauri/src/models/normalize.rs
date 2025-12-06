use super::config::get_alias_map;

/// Normalize a model name (alias or short name) to its full API identifier
///
/// Examples:
/// - "sonnet" -> "claude-sonnet-4-5-20250929"
/// - "claude-opus-4" -> "claude-opus-4-5-20251101"
/// - "claude-sonnet-4-5-20250929" -> "claude-sonnet-4-5-20250929" (unchanged)
/// - "unknown-model" -> "unknown-model" (passthrough)
pub fn normalize_model_name(model_name: &str) -> String {
    let alias_map = get_alias_map();

    // Case-insensitive lookup
    let key = model_name.to_lowercase();

    match alias_map.get(&key) {
        Some(full_id) => {
            if full_id != model_name {
                println!("🔄 Normalized model: {} -> {}", model_name, full_id);
            }
            full_id.clone()
        }
        None => {
            // Passthrough: return unchanged for unknown models
            println!("⚠️  Unknown model name (passthrough): {}", model_name);
            model_name.to_string()
        }
    }
}

/// Normalize an optional model name
pub fn normalize_model_option(model: Option<String>) -> Option<String> {
    model.map(|m| normalize_model_name(&m))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_alias() {
        assert_eq!(
            normalize_model_name("sonnet"),
            "claude-sonnet-4-5-20250929"
        );
        assert_eq!(
            normalize_model_name("opus"),
            "claude-opus-4-5-20251101"
        );
    }

    #[test]
    fn test_normalize_short_name() {
        assert_eq!(
            normalize_model_name("claude-sonnet-4"),
            "claude-sonnet-4-5-20250929"
        );
    }

    #[test]
    fn test_normalize_full_id() {
        // Full ID should return unchanged
        assert_eq!(
            normalize_model_name("claude-sonnet-4-5-20250929"),
            "claude-sonnet-4-5-20250929"
        );
    }

    #[test]
    fn test_normalize_case_insensitive() {
        assert_eq!(
            normalize_model_name("SONNET"),
            "claude-sonnet-4-5-20250929"
        );
        assert_eq!(
            normalize_model_name("Claude-Sonnet-4"),
            "claude-sonnet-4-5-20250929"
        );
    }

    #[test]
    fn test_normalize_unknown() {
        // Unknown models should passthrough unchanged
        assert_eq!(
            normalize_model_name("unknown-model-xyz"),
            "unknown-model-xyz"
        );
    }
}
