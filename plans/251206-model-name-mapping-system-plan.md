# Model Name Mapping System - Implementation Plan

**Date**: 2025-12-06
**Status**: Draft
**Priority**: Medium
**Estimated Effort**: 4-6 hours

## Executive Summary

This plan outlines the implementation of a configurable model name mapping system that maps short model aliases (e.g., "sonnet", "opus") to full API model identifiers (e.g., "claude-sonnet-4-5-20250929"). The system will load mappings from a JSON configuration file bundled with the application, making them accessible to both the Rust backend and TypeScript frontend.

---

## Table of Contents

1. [Problem Statement](#problem-statement)
2. [Requirements Analysis](#requirements-analysis)
3. [Architecture Decision Records](#architecture-decision-records)
4. [Config File Specification](#config-file-specification)
5. [Rust Implementation](#rust-implementation)
6. [TypeScript Implementation](#typescript-implementation)
7. [Integration Points](#integration-points)
8. [Migration Strategy](#migration-strategy)
9. [Error Handling](#error-handling)
10. [Testing Strategy](#testing-strategy)
11. [Implementation Checklist](#implementation-checklist)

---

## Problem Statement

### Current State

The application has hardcoded model name mappings in `src/lib/sessions-query.ts` (lines 106-122):

```typescript
export function normalizeModelName(model: string | undefined): string | undefined {
    if (!model) return undefined;

    const modelMap: Record<string, string> = {
        "sonnet": "claude-sonnet-4-5-20250929",
        "claude-sonnet-4": "claude-sonnet-4-5-20250929",
        "claude-sonnet-4-5": "claude-sonnet-4-5-20250929",
        "opus": "claude-opus-4-5-20251101",
        "claude-opus-4": "claude-opus-4-5-20251101",
        "claude-opus-4-5": "claude-opus-4-5-20251101",
        "haiku": "claude-3-5-haiku-20241022",
        "claude-haiku-3": "claude-3-5-haiku-20241022",
        "claude-haiku-3-5": "claude-3-5-haiku-20241022",
    };

    return modelMap[model] || model;
}
```

### Issues

1. **Duplication Risk**: Mapping may need to be replicated in Rust backend
2. **Maintenance Burden**: Code changes required when new models are released
3. **No Single Source of Truth**: Risk of inconsistency between frontend and backend
4. **Compile-Time Updates Only**: Cannot update mappings without rebuilding

### Desired State

- Single JSON configuration file as source of truth
- Accessible from both Rust and TypeScript
- Easy to update without deep code changes
- Graceful fallback when model not found in mapping

---

## Requirements Analysis

### Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| FR1 | Load model mappings from JSON config file | Must |
| FR2 | Rust backend can normalize model names | Must |
| FR3 | TypeScript frontend can normalize model names | Must |
| FR4 | Passthrough unknown model names unchanged | Must |
| FR5 | Handle missing config file gracefully | Should |
| FR6 | Support multiple aliases per model | Should |

### Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR1 | Config loads in < 10ms | Should |
| NFR2 | Zero runtime overhead for repeated lookups | Should |
| NFR3 | Config file editable in external editor | Could |

---

## Architecture Decision Records

### ADR-1: Config File Format

**Decision**: Use JSON format

**Rationale**:
- Already used throughout the project (tauri.conf.json, settings.json)
- Native parsing in both Rust (serde_json) and TypeScript
- No additional dependencies required
- Easy to validate with JSON Schema

**Alternatives Considered**:
- TOML: Common in Rust, but requires additional dependency for TypeScript
- YAML: More readable, but requires js-yaml dependency

### ADR-2: Config File Location

**Decision**: Bundle config file at `src-tauri/resources/model-mapping.json` and access via Tauri resource system

**Rationale**:
- Tauri v2 resource bundling provides cross-platform access
- File included in application bundle automatically
- Accessible from both Rust and frontend via `$RESOURCE` path
- Follows Tauri best practices

**Alternatives Considered**:
- `include_str!` compile-time: Fast but requires recompilation for changes
- User config directory: Would require initial copy/creation logic
- Separate TypeScript/Rust configs: Violates single source of truth

### ADR-3: Loading Strategy

**Decision**: Compile-time embedding with `include_str!` in Rust, Tauri command for frontend

**Rationale**:
- Rust side: `include_str!` guarantees availability and zero runtime I/O
- Frontend: Fetches once on app startup, caches in React Query or module state
- Best of both worlds: reliability + frontend flexibility

**Implementation**: Use `include_str!` for Rust backend and expose a Tauri command for frontend to fetch the same config.

### ADR-4: Fallback Behavior

**Decision**: Passthrough mode - return input unchanged if not in mapping

**Rationale**:
- Existing behavior in current implementation
- Allows direct use of full model names
- Future-proofs against new models before config update
- No breaking changes for users

---

## Config File Specification

### Location

```
src-tauri/resources/model-mapping.json
```

### Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Model Mapping Configuration",
  "type": "object",
  "required": ["version", "models"],
  "properties": {
    "version": {
      "type": "string",
      "description": "Config schema version for future migrations"
    },
    "defaultModel": {
      "type": "string",
      "description": "Default model identifier when none specified"
    },
    "models": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "aliases"],
        "properties": {
          "id": {
            "type": "string",
            "description": "Full API model identifier"
          },
          "aliases": {
            "type": "array",
            "items": { "type": "string" },
            "description": "Short names that map to this model"
          },
          "displayName": {
            "type": "string",
            "description": "Human-readable name for UI display"
          },
          "family": {
            "type": "string",
            "enum": ["sonnet", "opus", "haiku"],
            "description": "Model family for grouping"
          }
        }
      }
    }
  }
}
```

### Example Config

```json
{
  "version": "1.0",
  "defaultModel": "claude-sonnet-4-5-20250929",
  "models": [
    {
      "id": "claude-sonnet-4-5-20250929",
      "displayName": "Claude Sonnet 4.5",
      "family": "sonnet",
      "aliases": [
        "sonnet",
        "claude-sonnet-4",
        "claude-sonnet-4-5"
      ]
    },
    {
      "id": "claude-opus-4-5-20251101",
      "displayName": "Claude Opus 4.5",
      "family": "opus",
      "aliases": [
        "opus",
        "claude-opus-4",
        "claude-opus-4-5"
      ]
    },
    {
      "id": "claude-3-5-haiku-20241022",
      "displayName": "Claude Haiku 3.5",
      "family": "haiku",
      "aliases": [
        "haiku",
        "claude-haiku-3",
        "claude-haiku-3-5"
      ]
    }
  ]
}
```

---

## Rust Implementation

### File Structure

```
src-tauri/
├── resources/
│   └── model-mapping.json        # Config file (NEW)
├── src/
│   ├── lib.rs                    # Register new commands
│   ├── models/                   # New module (NEW)
│   │   ├── mod.rs
│   │   ├── config.rs             # Config types and loading
│   │   └── normalize.rs          # Normalization logic
│   └── sessions/
│       └── resume.rs             # Update to use normalize
```

### Types (src/models/config.rs)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Model mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMappingConfig {
    pub version: String,
    #[serde(rename = "defaultModel")]
    pub default_model: Option<String>,
    pub models: Vec<ModelEntry>,
}

/// Single model entry with aliases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub family: Option<String>,
    pub aliases: Vec<String>,
}

/// Precomputed alias-to-id lookup map
static MODEL_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();

/// Load and parse the model mapping config
pub fn get_model_map() -> &'static HashMap<String, String> {
    MODEL_MAP.get_or_init(|| {
        // Include the config file at compile time
        let config_str = include_str!("../../resources/model-mapping.json");

        match serde_json::from_str::<ModelMappingConfig>(config_str) {
            Ok(config) => {
                let mut map = HashMap::new();
                for model in config.models {
                    for alias in model.aliases {
                        map.insert(alias.to_lowercase(), model.id.clone());
                    }
                    // Also map the full ID to itself for consistency
                    map.insert(model.id.to_lowercase(), model.id.clone());
                }
                map
            }
            Err(e) => {
                eprintln!("Failed to parse model mapping config: {}", e);
                HashMap::new()
            }
        }
    })
}

/// Get the full config (for frontend access)
pub fn get_model_config() -> Result<ModelMappingConfig, String> {
    let config_str = include_str!("../../resources/model-mapping.json");
    serde_json::from_str(config_str)
        .map_err(|e| format!("Failed to parse model mapping config: {}", e))
}
```

### Normalization (src/models/normalize.rs)

```rust
use super::config::get_model_map;

/// Normalize a model name to its full API identifier
///
/// # Arguments
/// * `model` - The model name or alias to normalize
///
/// # Returns
/// The full API model identifier, or the input unchanged if not found
///
/// # Example
/// ```
/// assert_eq!(normalize_model_name("sonnet"), "claude-sonnet-4-5-20250929");
/// assert_eq!(normalize_model_name("unknown-model"), "unknown-model");
/// ```
pub fn normalize_model_name(model: &str) -> String {
    let map = get_model_map();
    let normalized_key = model.to_lowercase();

    map.get(&normalized_key)
        .cloned()
        .unwrap_or_else(|| model.to_string())
}

/// Normalize an optional model name
pub fn normalize_model_name_opt(model: Option<&str>) -> Option<String> {
    model.map(normalize_model_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_known_alias() {
        assert_eq!(
            normalize_model_name("sonnet"),
            "claude-sonnet-4-5-20250929"
        );
    }

    #[test]
    fn test_normalize_unknown_passthrough() {
        assert_eq!(
            normalize_model_name("unknown-model"),
            "unknown-model"
        );
    }

    #[test]
    fn test_normalize_case_insensitive() {
        assert_eq!(
            normalize_model_name("SONNET"),
            "claude-sonnet-4-5-20250929"
        );
    }

    #[test]
    fn test_normalize_full_id() {
        assert_eq!(
            normalize_model_name("claude-sonnet-4-5-20250929"),
            "claude-sonnet-4-5-20250929"
        );
    }
}
```

### Module Entry (src/models/mod.rs)

```rust
mod config;
mod normalize;

pub use config::{get_model_config, ModelMappingConfig, ModelEntry};
pub use normalize::{normalize_model_name, normalize_model_name_opt};
```

### Tauri Command (src/commands.rs)

Add to existing commands file:

```rust
use crate::models;

/// Get the model mapping configuration
#[tauri::command]
pub async fn get_model_mapping() -> Result<models::ModelMappingConfig, String> {
    models::get_model_config()
}
```

### Update lib.rs

```rust
mod models;  // Add this line

// In invoke_handler, add:
get_model_mapping,
```

### Update resume.rs

```rust
use crate::models::normalize_model_name_opt;

// In resume_session function, before building command:
let normalized_model = normalize_model_name_opt(model.as_deref());

// Use normalized_model instead of model
if let Some(model_name) = &normalized_model {
    cmd.arg("--model").arg(model_name);
}
```

---

## TypeScript Implementation

### File Structure

```
src/
├── lib/
│   ├── model-mapping.ts          # Model mapping utilities (NEW)
│   └── sessions-query.ts         # Update to use model-mapping
```

### Model Mapping Module (src/lib/model-mapping.ts)

```typescript
import { invoke } from "@tauri-apps/api/core";

/** Model entry from config */
export interface ModelEntry {
  id: string;
  displayName?: string;
  family?: "sonnet" | "opus" | "haiku";
  aliases: string[];
}

/** Full model mapping configuration */
export interface ModelMappingConfig {
  version: string;
  defaultModel?: string;
  models: ModelEntry[];
}

/** Cached model mapping */
let cachedConfig: ModelMappingConfig | null = null;
let aliasMap: Map<string, string> | null = null;

/**
 * Fetch the model mapping configuration from backend
 * Caches the result for subsequent calls
 */
export async function getModelMapping(): Promise<ModelMappingConfig> {
  if (cachedConfig) {
    return cachedConfig;
  }

  try {
    cachedConfig = await invoke<ModelMappingConfig>("get_model_mapping");
    return cachedConfig;
  } catch (error) {
    console.error("Failed to load model mapping:", error);
    // Return empty config on error
    return { version: "0.0", models: [] };
  }
}

/**
 * Build the alias-to-id lookup map from config
 */
async function getAliasMap(): Promise<Map<string, string>> {
  if (aliasMap) {
    return aliasMap;
  }

  const config = await getModelMapping();
  aliasMap = new Map();

  for (const model of config.models) {
    for (const alias of model.aliases) {
      aliasMap.set(alias.toLowerCase(), model.id);
    }
    // Also map the full ID to itself
    aliasMap.set(model.id.toLowerCase(), model.id);
  }

  return aliasMap;
}

/**
 * Normalize a model name to its full API identifier
 * Returns the input unchanged if not found in mapping
 *
 * @param model - The model name or alias to normalize
 * @returns The full API model identifier
 */
export async function normalizeModelName(
  model: string | undefined
): Promise<string | undefined> {
  if (!model) return undefined;

  const map = await getAliasMap();
  return map.get(model.toLowerCase()) ?? model;
}

/**
 * Synchronous version using cached data
 * Returns input unchanged if cache not populated
 * Call getModelMapping() first to ensure cache is populated
 */
export function normalizeModelNameSync(
  model: string | undefined
): string | undefined {
  if (!model) return undefined;
  if (!aliasMap) return model;
  return aliasMap.get(model.toLowerCase()) ?? model;
}

/**
 * Get display name for a model
 */
export async function getModelDisplayName(modelId: string): Promise<string> {
  const config = await getModelMapping();
  const entry = config.models.find(m => m.id === modelId);
  return entry?.displayName ?? modelId;
}

/**
 * Get all available models for UI selection
 */
export async function getAvailableModels(): Promise<ModelEntry[]> {
  const config = await getModelMapping();
  return config.models;
}

/**
 * Get the default model identifier
 */
export async function getDefaultModel(): Promise<string | undefined> {
  const config = await getModelMapping();
  return config.defaultModel;
}

/**
 * Clear the cache (useful for hot-reload scenarios)
 */
export function clearModelMappingCache(): void {
  cachedConfig = null;
  aliasMap = null;
}
```

### Update sessions-query.ts

```typescript
// Replace the hardcoded normalizeModelName with import
import { normalizeModelNameSync, getModelMapping } from "./model-mapping";

// Remove the old hardcoded function (lines 106-122)

// Update useResumeSession mutation
export function useResumeSession() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      sessionId,
      message,
      projectPath,
      model,
    }: {
      sessionId: string;
      message: string;
      projectPath: string;
      model?: string;
    }) => {
      // Model normalization now happens in Rust backend
      // Frontend just passes the alias, backend normalizes
      await invoke("session_resume", {
        sessionId,
        message,
        projectPath,
        model,  // Pass as-is, Rust normalizes
      });
    },
    // ... rest unchanged
  });
}
```

### React Query Hook (Optional Enhancement)

```typescript
import { useQuery } from "@tanstack/react-query";
import { getModelMapping, ModelMappingConfig } from "./model-mapping";

/** React Query hook for model mapping */
export function useModelMapping() {
  return useQuery({
    queryKey: ["model-mapping"],
    queryFn: getModelMapping,
    staleTime: Infinity,  // Config doesn't change at runtime
    gcTime: Infinity,
  });
}
```

---

## Integration Points

### Where Model Normalization is Used

| Location | File | Action Required |
|----------|------|-----------------|
| Resume session | `src-tauri/src/sessions/resume.rs:69-74` | Call `normalize_model_name_opt()` |
| Create session | `src-tauri/src/sessions/resume.rs:309-311` | Call `normalize_model_name_opt()` |
| Frontend resume | `src/lib/sessions-query.ts:141` | Remove, let backend handle |
| Frontend create | `src/lib/sessions-query.ts:208` | Remove, let backend handle |

### Recommended Approach

**Normalize in Rust backend only**. This ensures:
- Single point of normalization (backend)
- Frontend passes user input unchanged
- Simpler frontend code
- Backend always has correct model name for Claude CLI

---

## Migration Strategy

### Phase 1: Create Infrastructure (Non-Breaking)

1. Create `src-tauri/resources/model-mapping.json`
2. Create `src-tauri/src/models/` module
3. Add `get_model_mapping` Tauri command
4. Register command in `lib.rs`
5. Create `src/lib/model-mapping.ts`

### Phase 2: Backend Integration

1. Update `resume_session()` to normalize model names
2. Update `create_session()` to normalize model names
3. Test that Claude CLI receives correct model names

### Phase 3: Frontend Cleanup

1. Remove hardcoded `normalizeModelName()` from `sessions-query.ts`
2. Remove frontend normalization calls (backend handles it)
3. Add `useModelMapping()` hook for UI features (model selector, etc.)

### Phase 4: Testing & Verification

1. Test all model aliases work correctly
2. Test unknown model passthrough
3. Test frontend config access
4. Verify no regressions in session creation/resume

---

## Error Handling

### Scenario: Config Parse Error

**Rust**:
```rust
// In get_model_map(), log error and return empty map
Err(e) => {
    eprintln!("Failed to parse model mapping config: {}", e);
    HashMap::new()
}
```
**Result**: All model names pass through unchanged. Logs error for debugging.

### Scenario: Model Not Found in Mapping

**Rust**:
```rust
map.get(&normalized_key)
    .cloned()
    .unwrap_or_else(|| model.to_string())
```
**Result**: Input returned unchanged. Allows direct use of full model IDs.

### Scenario: Frontend Cache Miss

**TypeScript**:
```typescript
if (!aliasMap) return model;  // Passthrough if cache not ready
```
**Result**: Returns input unchanged, no error.

---

## Testing Strategy

### Unit Tests (Rust)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let config = get_model_config().expect("Config should parse");
        assert!(!config.models.is_empty());
    }

    #[test]
    fn test_normalize_sonnet() {
        assert_eq!(normalize_model_name("sonnet"), "claude-sonnet-4-5-20250929");
    }

    #[test]
    fn test_normalize_opus() {
        assert_eq!(normalize_model_name("opus"), "claude-opus-4-5-20251101");
    }

    #[test]
    fn test_normalize_haiku() {
        assert_eq!(normalize_model_name("haiku"), "claude-3-5-haiku-20241022");
    }

    #[test]
    fn test_normalize_unknown_passthrough() {
        assert_eq!(normalize_model_name("unknown-model-123"), "unknown-model-123");
    }

    #[test]
    fn test_normalize_case_insensitive() {
        assert_eq!(normalize_model_name("SONNET"), "claude-sonnet-4-5-20250929");
        assert_eq!(normalize_model_name("Opus"), "claude-opus-4-5-20251101");
    }

    #[test]
    fn test_normalize_full_id_passthrough() {
        assert_eq!(
            normalize_model_name("claude-sonnet-4-5-20250929"),
            "claude-sonnet-4-5-20250929"
        );
    }
}
```

### Integration Tests

1. **Session Resume with Alias**:
   - Resume session with model="sonnet"
   - Verify Claude CLI receives "--model claude-sonnet-4-5-20250929"

2. **Session Create with Alias**:
   - Create session with model="opus"
   - Verify Claude CLI receives "--model claude-opus-4-5-20251101"

3. **Frontend Config Access**:
   - Call `get_model_mapping` from frontend
   - Verify correct config structure returned

### Manual Testing Checklist

- [ ] App starts without errors
- [ ] `get_model_mapping` command returns config
- [ ] Resume session with "sonnet" works
- [ ] Resume session with "claude-sonnet-4-5-20250929" works
- [ ] Resume session with unknown model passes through
- [ ] Create session with aliases works
- [ ] Model selector UI shows correct options (if implemented)

---

## Implementation Checklist

### Backend Tasks

- [ ] Create `src-tauri/resources/` directory
- [ ] Create `src-tauri/resources/model-mapping.json` with initial config
- [ ] Create `src-tauri/src/models/mod.rs`
- [ ] Create `src-tauri/src/models/config.rs` with types and loading
- [ ] Create `src-tauri/src/models/normalize.rs` with normalization logic
- [ ] Add `mod models;` to `src-tauri/src/lib.rs`
- [ ] Add `get_model_mapping` command to `src-tauri/src/commands.rs`
- [ ] Register `get_model_mapping` in invoke_handler
- [ ] Update `resume_session()` to normalize model names
- [ ] Update `create_session()` to normalize model names
- [ ] Write unit tests for Rust normalization
- [ ] Run `cargo test` to verify

### Frontend Tasks

- [ ] Create `src/lib/model-mapping.ts`
- [ ] Add `ModelEntry` and `ModelMappingConfig` types
- [ ] Implement `getModelMapping()` with caching
- [ ] Implement `normalizeModelName()` async function
- [ ] Implement `normalizeModelNameSync()` for cached access
- [ ] Remove hardcoded `normalizeModelName()` from `sessions-query.ts`
- [ ] Remove frontend normalization calls (now handled by backend)
- [ ] Run `pnpm tsc --noEmit` to verify types

### Documentation Tasks

- [ ] Update README if model configuration is user-facing
- [ ] Document config file format in code comments
- [ ] Add JSDoc to TypeScript functions

### Testing Tasks

- [ ] Run Rust unit tests
- [ ] Manual test: Resume session with "sonnet"
- [ ] Manual test: Resume session with "opus"
- [ ] Manual test: Resume session with full model ID
- [ ] Manual test: Resume session with unknown model
- [ ] Manual test: Create new session with alias
- [ ] Verify no TypeScript errors

---

## References

- [Tauri v2 Embedding Additional Files](https://v2.tauri.app/develop/resources/)
- [Rust include_str! macro](https://doc.rust-lang.org/std/macro.include_str.html)
- [serde_json documentation](https://docs.rs/serde_json/latest/serde_json/)

---

## Appendix: File Diff Summary

### New Files

| File | Purpose |
|------|---------|
| `src-tauri/resources/model-mapping.json` | Model mapping configuration |
| `src-tauri/src/models/mod.rs` | Module entry point |
| `src-tauri/src/models/config.rs` | Config types and loading |
| `src-tauri/src/models/normalize.rs` | Normalization logic |
| `src/lib/model-mapping.ts` | Frontend model mapping utilities |

### Modified Files

| File | Changes |
|------|---------|
| `src-tauri/src/lib.rs` | Add `mod models;`, register command |
| `src-tauri/src/commands.rs` | Add `get_model_mapping` command |
| `src-tauri/src/sessions/resume.rs` | Call normalize before CLI |
| `src/lib/sessions-query.ts` | Remove hardcoded function, cleanup |
