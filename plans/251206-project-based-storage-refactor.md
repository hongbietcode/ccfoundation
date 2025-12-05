# CC Mate: Project-Based Storage Refactoring Plan

**Date**: 2025-12-06
**Status**: Planning
**Author**: Claude (Research Agent)
**Estimate**: 5-7 days implementation

---

## 1. Executive Summary

This plan refactors the CC Mate per-project configuration system from **centralized storage** (`~/.ccconfig/project-configs/{hash}.json`) to **project-based storage** following Claude Code conventions (`PROJECT_DIR/.claude/`).

### Key Changes
- **Remove**: Centralized storage at `~/.ccconfig/project-configs/`
- **Remove**: SHA256 hashing logic
- **Add**: Direct read/write to `PROJECT_DIR/.claude/` structure
- **Preserve**: Existing Project Selector UI (Phase 3.5 complete)

### Target Structure
```
PROJECT_DIR/
├── .claude/
│   ├── settings.json      # Model, env, permissions
│   ├── agents/            # Project-specific agents
│   │   └── *.md
│   └── commands/          # Project-specific commands
│       └── *.md
└── .mcp.json              # Project MCP servers
```

---

## 2. Architecture Changes

### 2.1 Storage Location Changes

| Component | Before (Current) | After (Target) |
|-----------|------------------|----------------|
| Project settings | `~/.ccconfig/project-configs/{hash}.json` | `PROJECT_DIR/.claude/settings.json` |
| Project agents | N/A (global only) | `PROJECT_DIR/.claude/agents/*.md` |
| Project commands | N/A (global only) | `PROJECT_DIR/.claude/commands/*.md` |
| Project MCP | N/A | `PROJECT_DIR/.mcp.json` |
| Project metadata | Inside `{hash}.json` | `~/.ccconfig/project-registry.json` |

### 2.2 Data Structure Changes

**REMOVE: `ProjectConfigStore` (centralized)**
```rust
// Current - TO BE REMOVED
pub struct ProjectConfigStore {
    pub project_path: String,
    pub canonical_path: String,
    pub id: String,
    pub title: String,
    pub created_at: u64,
    pub last_used_at: u64,
    pub settings: Value,
    pub inherit_from_global: bool,
    pub parent_global_config_id: Option<String>,
}
```

**ADD: `ProjectSettings` (lightweight metadata)**
```rust
// New - project-based approach
pub struct ProjectSettings {
    pub path: String,               // Project path
    pub exists: bool,               // Whether .claude/ exists
    pub settings: Option<Value>,    // Contents of .claude/settings.json
    pub has_agents: bool,           // Whether .claude/agents/ exists
    pub has_commands: bool,         // Whether .claude/commands/ exists
    pub has_mcp: bool,              // Whether .mcp.json exists
}
```

**ADD: `ProjectRegistryEntry` (for tracking)**
```rust
// Registry for project metadata (not config content)
pub struct ProjectRegistryEntry {
    pub project_path: String,
    pub title: String,              // User-friendly name
    pub last_used_at: u64,
    pub inherit_from_global: bool,  // Merge behavior flag
    pub parent_global_config_id: Option<String>,
}
```

**UPDATE: `~/.ccconfig/project-registry.json`**
```json
{
  "projects": {
    "/Users/me/projects/my-app": {
      "title": "My App",
      "lastUsedAt": 1733472000,
      "inheritFromGlobal": true,
      "parentGlobalConfigId": "vtTZmS"
    }
  }
}
```

### 2.3 Removed Functionality

| Item | Location | Reason |
|------|----------|--------|
| `hash_project_path()` | `commands.rs` L2200-2206 | No centralized storage |
| `get_project_configs_dir()` | `commands.rs` L2208-2212 | No centralized directory |
| `read_project_config_file()` | `commands.rs` L2214-2231 | Direct file read instead |
| `write_project_config_file()` | `commands.rs` L2233-2251 | Direct file write instead |
| SHA256 hashing | `commands.rs` L4 | Not needed |

### 2.4 New Functionality

| Function | Purpose |
|----------|---------|
| `read_project_claude_settings(path)` | Read `PROJECT/.claude/settings.json` |
| `write_project_claude_settings(path, content)` | Write `PROJECT/.claude/settings.json` |
| `read_project_agents(path)` | Read `PROJECT/.claude/agents/*.md` |
| `write_project_agent(path, name, content)` | Write `PROJECT/.claude/agents/{name}.md` |
| `delete_project_agent(path, name)` | Delete project agent |
| `read_project_commands(path)` | Read `PROJECT/.claude/commands/*.md` |
| `write_project_command(path, name, content)` | Write project command |
| `delete_project_command(path, name)` | Delete project command |
| `read_project_mcp(path)` | Read `PROJECT/.mcp.json` |
| `write_project_mcp(path, content)` | Write `PROJECT/.mcp.json` |
| `get_project_registry()` | Get all registered projects |
| `update_project_registry(path, entry)` | Update project metadata |

---

## 3. Backend Refactor (Phase 1 Revisit)

### 3.1 Commands to REMOVE (6 commands)

| # | Command | Line | Reason |
|---|---------|------|--------|
| 1 | `get_project_configs()` | L2469-2500 | Returns centralized configs |
| 2 | `get_project_config(project_path)` | L2503-2508 | Uses hash-based lookup |
| 3 | `create_project_config(...)` | L2511-2545 | Creates in centralized store |
| 4 | `update_project_config(...)` | L2548-2587 | Updates centralized store |
| 5 | `delete_project_config(project_path)` | L2590-2611 | Deletes from centralized |
| 6 | `update_project_config_path(...)` | L2805-2840 | Path relinking for centralized |

### 3.2 Commands to KEEP but MODIFY (11 commands)

| # | Command | Changes Required |
|---|---------|------------------|
| 1 | `activate_project_config(project_path)` | Read from `PROJECT/.claude/settings.json` instead of hash file |
| 2 | `get_active_context()` | No changes (reads from stores.json) |
| 3 | `switch_to_global_context(store_id)` | No changes |
| 4 | `auto_create_project_config(project_path)` | Write to `PROJECT/.claude/settings.json` |
| 5 | `get_active_merged_config()` | Read from project path directly |
| 6 | `check_project_local_settings(project_path)` | Already reads from project path |
| 7 | `import_project_local_settings(project_path)` | Update to write registry only |
| 8 | `add_project_to_tracking(project_path)` | No changes |
| 9 | `validate_project_path(project_path)` | No changes |
| 10 | `get_managed_settings()` | No changes |
| 11 | `get_managed_mcp_servers()` | No changes |

### 3.3 Commands to ADD (12 new commands)

```rust
// Project Settings Management
#[tauri::command]
pub async fn read_project_settings(project_path: String) -> Result<ProjectSettings, String>

#[tauri::command]
pub async fn write_project_settings(project_path: String, settings: Value) -> Result<(), String>

#[tauri::command]
pub async fn init_project_claude_dir(project_path: String) -> Result<(), String>

// Project Agents Management
#[tauri::command]
pub async fn read_project_agents(project_path: String) -> Result<Vec<AgentFile>, String>

#[tauri::command]
pub async fn write_project_agent(project_path: String, agent_name: String, content: String) -> Result<(), String>

#[tauri::command]
pub async fn delete_project_agent(project_path: String, agent_name: String) -> Result<(), String>

// Project Commands Management
#[tauri::command]
pub async fn read_project_commands(project_path: String) -> Result<Vec<CommandFile>, String>

#[tauri::command]
pub async fn write_project_command(project_path: String, command_name: String, content: String) -> Result<(), String>

#[tauri::command]
pub async fn delete_project_command(project_path: String, command_name: String) -> Result<(), String>

// Project MCP Management
#[tauri::command]
pub async fn read_project_mcp(project_path: String) -> Result<Option<Value>, String>

#[tauri::command]
pub async fn write_project_mcp(project_path: String, content: Value) -> Result<(), String>

// Project Registry Management
#[tauri::command]
pub async fn get_project_registry() -> Result<Vec<ProjectRegistryEntry>, String>
```

### 3.4 Helper Functions to REMOVE

```rust
// REMOVE these from commands.rs
fn hash_project_path(path: &str) -> Result<String, String>    // L2200-2206
fn get_project_configs_dir() -> Result<PathBuf, String>       // L2208-2212
fn read_project_config_file(project_path: &str) -> ...        // L2214-2231
fn write_project_config_file(config: &ProjectConfigStore)     // L2233-2251
```

### 3.5 Helper Functions to ADD

```rust
/// Get project's .claude directory path
fn get_project_claude_dir(project_path: &str) -> PathBuf {
    PathBuf::from(project_path).join(".claude")
}

/// Get project's settings.json path
fn get_project_settings_path(project_path: &str) -> PathBuf {
    get_project_claude_dir(project_path).join("settings.json")
}

/// Get project's .mcp.json path
fn get_project_mcp_path(project_path: &str) -> PathBuf {
    PathBuf::from(project_path).join(".mcp.json")
}

/// Get project registry file path
fn get_project_registry_path() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    Ok(home_dir.join(APP_CONFIG_DIR).join("project-registry.json"))
}

/// Read project registry
fn read_project_registry() -> Result<HashMap<String, ProjectRegistryEntry>, String>

/// Write project registry entry
fn write_project_registry_entry(path: &str, entry: &ProjectRegistryEntry) -> Result<(), String>
```

### 3.6 Dependencies to REMOVE

**File**: `src-tauri/Cargo.toml`

```toml
# REMOVE this dependency
sha2 = "0.10"
```

---

## 4. Data Layer Updates (Phase 2 Revisit)

### 4.1 TypeScript Interfaces to UPDATE

**File**: `src/lib/query.ts`

```typescript
// REMOVE or DEPRECATE
export interface ProjectConfigStore {
  projectPath: string;
  canonicalPath: string;
  id: string;
  title: string;
  createdAt: number;
  lastUsedAt: number;
  settings: unknown;
  inheritFromGlobal: boolean;
  parentGlobalConfigId: string | null;
}

// ADD
export interface ProjectSettings {
  path: string;
  exists: boolean;
  settings: unknown | null;
  hasAgents: boolean;
  hasCommands: boolean;
  hasMcp: boolean;
}

export interface ProjectRegistryEntry {
  projectPath: string;
  title: string;
  lastUsedAt: number;
  inheritFromGlobal: boolean;
  parentGlobalConfigId: string | null;
}
```

### 4.2 React Query Hooks to REMOVE

```typescript
// REMOVE these hooks (L641-867)
export const useProjectConfigs = () => {...}           // L641-646
export const useProjectConfig = (projectPath) => {...} // L648-655
export const useCreateProjectConfig = () => {...}      // L657-691
export const useUpdateProjectConfig = () => {...}      // L693-727
export const useDeleteProjectConfig = () => {...}      // L729-748
```

### 4.3 React Query Hooks to ADD

```typescript
// Project Settings
export const useProjectSettings = (projectPath: string) => {
  return useQuery({
    queryKey: ["project-settings", projectPath],
    queryFn: () => invoke<ProjectSettings>("read_project_settings", { projectPath }),
    enabled: !!projectPath,
  });
};

export const useWriteProjectSettings = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ projectPath, settings }: { projectPath: string; settings: unknown }) =>
      invoke<void>("write_project_settings", { projectPath, settings }),
    onSuccess: (_, variables) => {
      toast.success(i18n.t("toast.projectSettingsSaved"));
      queryClient.invalidateQueries({ queryKey: ["project-settings", variables.projectPath] });
    },
  });
};

export const useInitProjectClaudeDir = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (projectPath: string) =>
      invoke<void>("init_project_claude_dir", { projectPath }),
    onSuccess: (_, projectPath) => {
      queryClient.invalidateQueries({ queryKey: ["project-settings", projectPath] });
    },
  });
};

// Project Agents
export const useProjectAgents = (projectPath: string) => {
  return useQuery({
    queryKey: ["project-agents", projectPath],
    queryFn: () => invoke<CommandFile[]>("read_project_agents", { projectPath }),
    enabled: !!projectPath,
  });
};

export const useWriteProjectAgent = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ projectPath, agentName, content }: { projectPath: string; agentName: string; content: string }) =>
      invoke<void>("write_project_agent", { projectPath, agentName, content }),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["project-agents", variables.projectPath] });
    },
  });
};

export const useDeleteProjectAgent = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ projectPath, agentName }: { projectPath: string; agentName: string }) =>
      invoke<void>("delete_project_agent", { projectPath, agentName }),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["project-agents", variables.projectPath] });
    },
  });
};

// Project Commands
export const useProjectCommands = (projectPath: string) => {
  return useQuery({
    queryKey: ["project-commands", projectPath],
    queryFn: () => invoke<CommandFile[]>("read_project_commands", { projectPath }),
    enabled: !!projectPath,
  });
};

export const useWriteProjectCommand = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ projectPath, commandName, content }: { projectPath: string; commandName: string; content: string }) =>
      invoke<void>("write_project_command", { projectPath, commandName, content }),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["project-commands", variables.projectPath] });
    },
  });
};

export const useDeleteProjectCommand = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ projectPath, commandName }: { projectPath: string; commandName: string }) =>
      invoke<void>("delete_project_command", { projectPath, commandName }),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["project-commands", variables.projectPath] });
    },
  });
};

// Project MCP
export const useProjectMcp = (projectPath: string) => {
  return useQuery({
    queryKey: ["project-mcp", projectPath],
    queryFn: () => invoke<Value | null>("read_project_mcp", { projectPath }),
    enabled: !!projectPath,
  });
};

export const useWriteProjectMcp = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ projectPath, content }: { projectPath: string; content: unknown }) =>
      invoke<void>("write_project_mcp", { projectPath, content }),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["project-mcp", variables.projectPath] });
    },
  });
};

// Project Registry
export const useProjectRegistry = () => {
  return useQuery({
    queryKey: ["project-registry"],
    queryFn: () => invoke<ProjectRegistryEntry[]>("get_project_registry"),
  });
};
```

### 4.4 Query Key Updates

| Old Key | New Key |
|---------|---------|
| `["project-configs"]` | `["project-registry"]` |
| `["project-config", path]` | `["project-settings", path]` |
| N/A | `["project-agents", path]` |
| N/A | `["project-commands", path]` |
| N/A | `["project-mcp", path]` |

---

## 5. UI Flow Changes (Phase 3 Updates)

### 5.1 Navigation Flow

```
[User opens app]
        |
        v
[Layout.tsx - Navigation Sidebar]
        |
        +-- [Project Configs] --> ProjectConfigsPage (preserved)
        |         |
        |         v
        |   [Select Project Card]
        |         |
        |         +-- [Has .claude/] --> ProjectConfigEditor (edit settings)
        |         |
        |         +-- [No .claude/] --> [Create] --> init_project_claude_dir()
        |                                              |
        |                                              v
        |                                  ProjectConfigEditor (new)
        |
        +-- [Configurations] --> Main config editor (global)
```

### 5.2 Updated `ProjectConfigsPage.tsx` Logic

**Current behavior**: Reads from centralized `~/.ccconfig/project-configs/`
**New behavior**: Combines Claude projects list + project registry + actual file existence check

```typescript
// Key changes in ProjectConfigsPage.tsx
const { data: claudeProjects } = useClaudeProjects();      // From ~/.claude.json
const { data: projectRegistry } = useProjectRegistry();    // Metadata from registry

// For each project, check if .claude/ exists
const enrichedProjects = useMemo(() => {
  if (!claudeProjects) return [];

  return claudeProjects.map(project => {
    const registryEntry = projectRegistry?.find(r => r.projectPath === project.path);
    return {
      path: project.path,
      name: extractProjectName(project.path),
      title: registryEntry?.title,
      hasClaudeDir: registryEntry !== undefined, // Registry entry = has .claude/
      lastUsedAt: registryEntry?.lastUsedAt,
      status: registryEntry ? "has_config" : "none"
    };
  });
}, [claudeProjects, projectRegistry]);
```

### 5.3 Updated `ProjectConfigEditor.tsx` Logic

**Current behavior**: Uses `useProjectConfig(path)` (centralized)
**New behavior**: Uses `useProjectSettings(path)` (project-based)

```typescript
// Key changes in ProjectConfigEditor.tsx
const { projectPath } = useParams<{ projectPath: string }>();
const decodedPath = projectPath ? decodeURIComponent(projectPath) : "";

// NEW: Read directly from project
const { data: projectSettings, isLoading } = useProjectSettings(decodedPath);
const writeProjectSettings = useWriteProjectSettings();

// Optional: Load project agents/commands/mcp for full context
const { data: projectAgents } = useProjectAgents(decodedPath);
const { data: projectCommands } = useProjectCommands(decodedPath);
const { data: projectMcp } = useProjectMcp(decodedPath);

const onSave = handleSubmit((formValues) => {
  const { configName, inheritFromGlobal, settings } = convertToNestedJSON(formValues);
  writeProjectSettings.mutate({
    projectPath: decodedPath,
    settings: { ...settings, inheritFromGlobal },
  });
});
```

### 5.4 Project-Utils Updates

**File**: `src/lib/project-utils.ts`

```typescript
// UPDATE EnrichedProject interface
export interface EnrichedProject {
  path: string;
  name: string;
  title?: string;                    // From registry
  hasClaudeDir: boolean;             // NEW: .claude/ exists
  hasSettings: boolean;              // NEW: settings.json exists
  hasAgents: boolean;                // NEW: agents/ exists
  hasCommands: boolean;              // NEW: commands/ exists
  hasMcp: boolean;                   // NEW: .mcp.json exists
  lastUsedAt?: number;               // From registry
  status: ProjectStatus;
  claudeConfig?: Record<string, any>;
}

// UPDATE mergeProjectsWithConfigs function
export function mergeProjectsWithConfigs(
  claudeProjects: ProjectConfig[],
  projectRegistry: ProjectRegistryEntry[],
): EnrichedProject[] {
  const registryMap = new Map(projectRegistry.map(r => [r.projectPath, r]));

  return claudeProjects.map(project => {
    const registry = registryMap.get(project.path);
    return {
      path: project.path,
      name: extractProjectName(project.path),
      title: registry?.title,
      hasClaudeDir: registry !== undefined,
      hasSettings: registry !== undefined,
      hasAgents: false, // TODO: Could add to registry
      hasCommands: false,
      hasMcp: false,
      lastUsedAt: registry?.lastUsedAt,
      status: registry ? "has_config" : "none",
      claudeConfig: project.config,
    };
  });
}
```

---

## 6. Implementation Steps

### Step 1: Backend - Add New Commands (Day 1-2)

1. **Add new structs** in `commands.rs`:
   - `ProjectSettings`
   - `ProjectRegistryEntry`

2. **Add helper functions**:
   - `get_project_claude_dir()`
   - `get_project_settings_path()`
   - `get_project_mcp_path()`
   - `get_project_registry_path()`
   - `read_project_registry()`
   - `write_project_registry_entry()`

3. **Add new commands** (12 total):
   - Project settings: `read_project_settings`, `write_project_settings`, `init_project_claude_dir`
   - Project agents: `read_project_agents`, `write_project_agent`, `delete_project_agent`
   - Project commands: `read_project_commands`, `write_project_command`, `delete_project_command`
   - Project MCP: `read_project_mcp`, `write_project_mcp`
   - Registry: `get_project_registry`

4. **Register commands** in `lib.rs`

### Step 2: Backend - Modify Existing Commands (Day 2-3)

1. **Update `activate_project_config`**:
   - Read from `PROJECT/.claude/settings.json`
   - Update registry `lastUsedAt`
   - Apply merged settings

2. **Update `auto_create_project_config`**:
   - Create `PROJECT/.claude/` directory
   - Write `settings.json` with defaults from global
   - Add entry to registry

3. **Update `import_project_local_settings`**:
   - Already reads from project path (keep as-is)
   - Add entry to registry after import

4. **Update `get_active_merged_config`**:
   - Read from project path directly instead of hash file

### Step 3: Backend - Remove Old Code (Day 3)

1. **Remove SHA256 hashing**:
   - Delete `sha2` from `Cargo.toml`
   - Remove `use sha2::{Digest, Sha256}` import
   - Remove `hash_project_path()` function

2. **Remove centralized storage functions**:
   - `get_project_configs_dir()`
   - `read_project_config_file()`
   - `write_project_config_file()`

3. **Remove old commands**:
   - `get_project_configs()`
   - `get_project_config()`
   - `create_project_config()`
   - `update_project_config()`
   - `delete_project_config()`
   - `update_project_config_path()`

4. **Unregister removed commands** from `lib.rs`

### Step 4: Frontend - Update Data Layer (Day 4)

1. **Update interfaces** in `query.ts`:
   - Add `ProjectSettings`
   - Add `ProjectRegistryEntry`
   - Deprecate `ProjectConfigStore`

2. **Add new hooks** in `query.ts`:
   - `useProjectSettings()`
   - `useWriteProjectSettings()`
   - `useInitProjectClaudeDir()`
   - `useProjectAgents()`
   - `useWriteProjectAgent()`
   - `useDeleteProjectAgent()`
   - `useProjectCommands()`
   - `useWriteProjectCommand()`
   - `useDeleteProjectCommand()`
   - `useProjectMcp()`
   - `useWriteProjectMcp()`
   - `useProjectRegistry()`

3. **Remove old hooks**:
   - `useProjectConfigs()`
   - `useProjectConfig()`
   - `useCreateProjectConfig()`
   - `useUpdateProjectConfig()`
   - `useDeleteProjectConfig()`

### Step 5: Frontend - Update UI Components (Day 5)

1. **Update `project-utils.ts`**:
   - Update `EnrichedProject` interface
   - Update `mergeProjectsWithConfigs()` to use registry

2. **Update `ProjectConfigsPage.tsx`**:
   - Replace `useProjectConfigs()` with `useProjectRegistry()`
   - Update project enrichment logic
   - Update create/delete handlers

3. **Update `ProjectConfigEditor.tsx`**:
   - Replace `useProjectConfig()` with `useProjectSettings()`
   - Replace `useUpdateProjectConfig()` with `useWriteProjectSettings()`
   - Add optional agents/commands/MCP tabs

4. **Update `ProjectCard.tsx`**:
   - Adjust for new `EnrichedProject` interface
   - Update status display

### Step 6: Testing & Validation (Day 6-7)

1. **Unit tests** (Rust):
   - Test new read/write commands
   - Test registry operations
   - Test directory creation

2. **Integration tests**:
   - Create project config from UI
   - Edit project settings
   - Switch between global/project contexts
   - Verify files created in correct locations

3. **Manual testing checklist**:
   - [ ] Create new project config (creates `.claude/settings.json`)
   - [ ] Edit project config (updates file in place)
   - [ ] Delete project config (removes file)
   - [ ] Activate project config (applies settings)
   - [ ] Switch to global context (restores global settings)
   - [ ] Import existing `.claude/settings.json`
   - [ ] Create project agents (writes to `.claude/agents/`)
   - [ ] Create project commands (writes to `.claude/commands/`)
   - [ ] Manage project MCP (writes to `.mcp.json`)

---

## 7. Migration Considerations

### 7.1 Existing Centralized Configs

**Strategy**: One-time migration on first app launch after update.

```rust
async fn migrate_centralized_configs() -> Result<(), String> {
    let old_dir = get_project_configs_dir()?; // ~/.ccconfig/project-configs/

    if !old_dir.exists() {
        return Ok(()); // Nothing to migrate
    }

    for entry in std::fs::read_dir(&old_dir)? {
        let path = entry?.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            // Read old config
            let content = std::fs::read_to_string(&path)?;
            let old_config: ProjectConfigStore = serde_json::from_str(&content)?;

            // Create new structure in project directory
            let project_claude_dir = PathBuf::from(&old_config.project_path).join(".claude");
            std::fs::create_dir_all(&project_claude_dir)?;

            // Write settings.json
            let settings_path = project_claude_dir.join("settings.json");
            let json = serde_json::to_string_pretty(&old_config.settings)?;
            std::fs::write(&settings_path, json)?;

            // Add to registry
            write_project_registry_entry(&old_config.project_path, &ProjectRegistryEntry {
                project_path: old_config.project_path,
                title: old_config.title,
                last_used_at: old_config.last_used_at,
                inherit_from_global: old_config.inherit_from_global,
                parent_global_config_id: old_config.parent_global_config_id,
            })?;
        }
    }

    // Optionally: Rename old directory as backup
    let backup_dir = old_dir.with_extension("backup");
    std::fs::rename(&old_dir, &backup_dir)?;

    Ok(())
}
```

### 7.2 Backward Compatibility

- **Not supported**: Old centralized configs will be migrated once
- **Fallback**: If migration fails, leave old configs in place with warning

### 7.3 Migration Script Location

Add to `initialize_app_config()` in `commands.rs`:

```rust
pub async fn initialize_app_config() -> Result<(), String> {
    // ... existing initialization code ...

    // NEW: Migrate centralized configs if needed
    if let Err(e) = migrate_centralized_configs().await {
        eprintln!("Warning: Failed to migrate project configs: {}", e);
        // Don't fail initialization, just log warning
    }

    Ok(())
}
```

---

## 8. Testing Plan

### 8.1 Unit Tests (Rust)

**File**: `src-tauri/src/commands_test.rs` (new file)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_read_project_settings_nonexistent() {
        let result = read_project_settings("/nonexistent/path".to_string()).await;
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert!(!settings.exists);
        assert!(settings.settings.is_none());
    }

    #[tokio::test]
    async fn test_write_and_read_project_settings() {
        let temp = tempdir().unwrap();
        let project_path = temp.path().to_string_lossy().to_string();

        let settings = serde_json::json!({
            "model": "claude-opus-4-5",
            "env": { "API_KEY": "test" }
        });

        write_project_settings(project_path.clone(), settings.clone()).await.unwrap();

        let result = read_project_settings(project_path).await.unwrap();
        assert!(result.exists);
        assert_eq!(result.settings.unwrap(), settings);
    }

    #[tokio::test]
    async fn test_init_project_claude_dir() {
        let temp = tempdir().unwrap();
        let project_path = temp.path().to_string_lossy().to_string();

        init_project_claude_dir(project_path.clone()).await.unwrap();

        let claude_dir = temp.path().join(".claude");
        assert!(claude_dir.exists());
        assert!(claude_dir.join("agents").exists());
        assert!(claude_dir.join("commands").exists());
    }

    #[tokio::test]
    async fn test_project_registry_operations() {
        // Test registry read/write
    }
}
```

### 8.2 Integration Tests

**Test scenarios**:

1. **Fresh project config creation**:
   - Start with project that has no `.claude/`
   - Click "Create" in Project Selector
   - Verify `.claude/settings.json` created
   - Verify registry entry added

2. **Edit existing project config**:
   - Open project with existing `.claude/settings.json`
   - Modify settings
   - Save
   - Verify file updated in place

3. **Project activation flow**:
   - Activate project config
   - Verify `~/.claude/settings.json` updated with merged settings
   - Verify active context updated

4. **Migration from centralized**:
   - Create test project with old centralized config
   - Run migration
   - Verify new structure created
   - Verify old directory backed up

### 8.3 Manual Testing Checklist

- [ ] **New project setup**
  - [ ] Create `.claude/` in fresh project
  - [ ] Verify directories created (agents/, commands/)
  - [ ] Verify default settings.json created

- [ ] **Project settings CRUD**
  - [ ] Read existing settings
  - [ ] Update settings
  - [ ] Settings persist after app restart

- [ ] **Project agents**
  - [ ] Create new agent
  - [ ] Edit agent content
  - [ ] Delete agent
  - [ ] Verify files in `.claude/agents/`

- [ ] **Project commands**
  - [ ] Create new command
  - [ ] Edit command content
  - [ ] Delete command
  - [ ] Verify files in `.claude/commands/`

- [ ] **Project MCP**
  - [ ] Create `.mcp.json`
  - [ ] Add/remove servers
  - [ ] Verify file content

- [ ] **Context switching**
  - [ ] Activate project context
  - [ ] Switch to global context
  - [ ] Verify correct settings applied

- [ ] **Migration**
  - [ ] Migrate existing centralized configs
  - [ ] Verify data integrity after migration
  - [ ] Verify backup created

---

## 9. Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Data loss during migration | High | Create backup before migration; test thoroughly |
| File permission issues | Medium | Handle errors gracefully; show clear error messages |
| Orphaned registry entries | Low | Add cleanup routine; validate paths on load |
| Breaking existing workflows | Medium | Provide migration path; document changes |
| Performance with large projects | Low | Lazy load project details; use caching |

---

## 10. Open Questions

1. **Registry cleanup**: Should we automatically remove registry entries for deleted projects?
   - **Recommendation**: Yes, validate on load and clean up stale entries

2. **Default inherit behavior**: Should new project configs inherit from global by default?
   - **Recommendation**: Yes, maintain current behavior

3. **Project agents/commands scope**: Should project agents/commands be shown in global views?
   - **Recommendation**: No, keep project-specific content isolated

4. **MCP server handling**: How to handle conflicts between global and project MCP servers?
   - **Recommendation**: Project MCP servers take precedence; show warning on conflict

---

## 11. Files Modified Summary

### Backend (Rust)

| File | Changes |
|------|---------|
| `src-tauri/Cargo.toml` | Remove `sha2` dependency |
| `src-tauri/src/commands.rs` | Add/remove/modify commands |
| `src-tauri/src/lib.rs` | Update command registration |

### Frontend (TypeScript/React)

| File | Changes |
|------|---------|
| `src/lib/query.ts` | Add/remove hooks, update interfaces |
| `src/lib/project-utils.ts` | Update types and functions |
| `src/pages/ProjectConfigsPage.tsx` | Update data fetching |
| `src/pages/ProjectConfigEditor.tsx` | Update to use new hooks |
| `src/components/project-configs/ProjectCard.tsx` | Adjust for new interface |

---

## 12. TODO Checklist

### Phase A: Backend Foundation
- [ ] Add `ProjectSettings` struct
- [ ] Add `ProjectRegistryEntry` struct
- [ ] Add helper functions for paths
- [ ] Add `read_project_settings` command
- [ ] Add `write_project_settings` command
- [ ] Add `init_project_claude_dir` command
- [ ] Add project agents commands (read/write/delete)
- [ ] Add project commands commands (read/write/delete)
- [ ] Add project MCP commands (read/write)
- [ ] Add `get_project_registry` command
- [ ] Register new commands in lib.rs

### Phase B: Backend Migration
- [ ] Update `activate_project_config`
- [ ] Update `auto_create_project_config`
- [ ] Update `import_project_local_settings`
- [ ] Update `get_active_merged_config`
- [ ] Remove SHA256 and old functions
- [ ] Remove old commands
- [ ] Add migration logic

### Phase C: Frontend Updates
- [ ] Add new TypeScript interfaces
- [ ] Add new React Query hooks
- [ ] Remove old hooks
- [ ] Update `project-utils.ts`
- [ ] Update `ProjectConfigsPage.tsx`
- [ ] Update `ProjectConfigEditor.tsx`
- [ ] Update `ProjectCard.tsx`

### Phase D: Testing
- [ ] Write Rust unit tests
- [ ] Test migration scenarios
- [ ] Complete manual testing checklist
- [ ] Test edge cases
- [ ] Performance testing

---

**End of Plan**
