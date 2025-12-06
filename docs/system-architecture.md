# CC Foundation - System Architecture

**Last Updated**: 2025-12-06
**Version**: 1.0
**Phase**: Phase 1 (Backend Foundation) Complete

---

## 1. Architecture Overview

CC Foundation follows a classic desktop application architecture with a clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                    React Frontend (TypeScript)               │
│  • Configuration UI                                          │
│  • Project management                                        │
│  • Context switching                                         │
│  • Analytics dashboard                                       │
└────────────────────┬────────────────────────────────────────┘
                     │ Tauri IPC
┌────────────────────▼────────────────────────────────────────┐
│         Tauri Bridge (Bidirectional Communication)           │
│  • Commands: Frontend → Backend                              │
│  • Events: Backend → Frontend                                │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│              Rust Backend (Tauri Commands)                   │
│  • File I/O operations                                       │
│  • Configuration management                                  │
│  • Context switching logic                                   │
│  • Path canonicalization & hashing                           │
│  • Settings merging                                          │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│           File System & External Services                    │
│  • ~/.ccconfig/ (app config)                                 │
│  • ~/.claude/ (Claude Code config)                           │
│  • /Library|/etc|C:\ProgramData (Enterprise)                 │
│  • PostHog (Analytics)                                       │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Frontend Architecture

### 2.1 Directory Structure

```
src/
├── main.tsx                  # React app entry point, React Query setup
├── App.tsx                   # Main app component (router setup)
├── components/
│   └── ui/                   # shadcn/ui imported components
├── lib/
│   ├── query.ts              # React Query hooks & Tauri API functions (876 lines, 46 hooks)
│   └── utils.ts              # Utility functions (formatting, validation, etc.)
└── pages/                    # Route page components
```

### 2.2 React Query Integration

All API calls to Tauri commands are wrapped in React Query for:

-   Server state management
-   Caching and deduplication
-   Automatic retry logic
-   Background refetching
-   Loading/error state handling

**Frontend Data Layer Stats** (Phase 2):

-   Total Hooks: 46 custom React Query hooks
-   Total Lines: 876 lines
-   Interfaces: ProjectConfigStore, ActiveContext, and 10+ supporting types
-   Patterns: Query (read) and Mutation (write) patterns

**Hook Categories**:

-   Global Config Management: 8 hooks (useStores, useCreateConfig, useUpdateConfig, etc.)
-   Project Config Management: 11 hooks (useProjectConfigs, useCreateProjectConfig, useActivateProjectConfig, etc.)
-   MCP Server Management: 5 hooks (useGlobalMcpServers, useUpdateGlobalMcpServer, etc.)
-   Memory & Commands: 6 hooks (useClaudeMemory, useClaudeCommands, useClaudeAgents, etc.)
-   Config File Operations: 3 hooks (useConfigFiles, useConfigFile, useWriteConfigFile)
-   Notifications & Misc: 13 hooks (useNotificationSettings, useCheckForUpdates, useProjectUsageFiles, etc.)

**Example Pattern**:

```typescript
// src/lib/query.ts
export function useProjectConfigs() {
	return useQuery({
		queryKey: ["project-configs"],
		queryFn: () => invoke<ProjectConfigStore[]>("get_project_configs"),
	});
}

// In component
const { data: configs, isLoading } = useProjectConfigs();
```

### 2.3 Form Handling

-   React Hook Form for form state management
-   Zod for schema validation
-   Validation happens before sending to backend
-   JSON structure validation performed on both frontend and backend

---

## 3. Backend Architecture (Rust)

### 3.1 File Organization

```
src-tauri/
├── src/
│   ├── main.rs              # Tauri app entry point
│   ├── lib.rs               # App setup, handler registration (258 lines)
│   ├── commands.rs          # All Tauri commands (2922 lines)
│   └── hook_server.rs       # Hook server implementation
├── Cargo.toml               # Dependencies
└── tauri.conf.json          # Tauri configuration
```

### 3.2 Commands Module (commands.rs)

**Line Count**: 2922 lines

**Logical Sections**:

1. **Imports & Constants** (lines 1-10)

    - Dependencies: serde, sha2, PathBuf, Tauri utilities
    - APP_CONFIG_DIR constant: `.ccconfig`

2. **App Initialization** (lines 12-60)

    - `initialize_app_config()` - Creates app directory, backs up Claude configs

3. **Data Structures** (lines 62-97)

    - `ConfigFile` - Configuration file representation
    - `ConfigStore` - Named configuration with metadata
    - `McpServer` - MCP server configuration
    - `StoresData` - Global storage structure
    - `NotificationSettings` - Notification preferences

4. **Config File Operations** (lines 98-500)

    - `read_config_file(config_type)` - Read by type (user, enterprise, etc.)
    - `write_config_file()` - Write configuration
    - `list_config_files()` - List all configs
    - Supports user, enterprise, MCP configurations

5. **Store Management** (lines 500-1000)

    - `get_stores()` - Retrieve all global configs
    - `get_store(id)` - Get specific config by ID
    - `create_config()` - Create new named config
    - `update_config()` - Update existing config
    - `delete_config()` - Remove config
    - `set_using_config()` - Mark config as active
    - `reset_to_original_config()` - Revert to backup

6. **MCP Server Management** (lines 1000-1200)

    - `get_global_mcp_servers()` - Retrieve MCP servers
    - `update_global_mcp_server()` - Create/update MCP server
    - `delete_global_mcp_server()` - Remove MCP server
    - `check_mcp_server_exists()` - Verify MCP server

7. **Project Management** (lines 1200-1550)

    - `read_claude_projects()` - Read .claude.json projects
    - `read_claude_config_file()` - Read global .claude.json
    - `write_claude_config_file()` - Write global .claude.json

8. **Analytics & Tracking** (lines 1550-1650)

    - `track()` - Send events to PostHog
    - Includes device info, app version, distinct ID

9. **Hook Management** (lines 1650-1950)

    - `update_claude_code_hook()` - Update hook configuration
    - `add_claude_code_hook()` - Add hook
    - `remove_claude_code_hook()` - Remove hook
    - Helper functions for hook updates

10. **Command Management** (lines 1990-2060)

    - `read_claude_commands()` - List custom commands
    - `write_claude_command()` - Create/update command
    - `delete_claude_command()` - Remove command

11. **Agent Management** (lines 2062-2140)

    - `read_claude_agents()` - List agents
    - `write_claude_agent()` - Create/update agent
    - `delete_claude_agent()` - Remove agent

12. **Per-Project Configuration (Phase 1)** (lines 2145-2922)
    - Data structures, helpers, and commands for project config management

### 3.3 Per-Project Configuration Implementation

#### Data Structures (lines 2145-2186)

**ProjectConfigStore** (lines 2150-2166):

```rust
pub struct ProjectConfigStore {
    pub project_path: String,           // Original project path
    pub canonical_path: String,         // Resolved symlinks
    pub id: String,                     // 6-char nanoid
    pub title: String,                  // User-friendly name
    pub created_at: u64,                // Unix timestamp
    pub last_used_at: u64,              // Unix timestamp
    pub settings: Value,                // JSON settings
    pub inherit_from_global: bool,      // Inheritance flag
    pub parent_global_config_id: Option<String>, // Parent ref
}
```

**ActiveContext** (lines 2170-2176):

```rust
pub struct ActiveContext {
    pub context_type: String,           // "global" or "project"
    pub id: String,                     // Config ID
    pub project_path: Option<String>,   // Project path if context_type="project"
}
```

**EnhancedStoresData** (lines 2180-2186):

```rust
pub struct EnhancedStoresData {
    pub configs: Vec<ConfigStore>,
    pub distinct_id: Option<String>,
    pub notification: Option<NotificationSettings>,
    pub active_context: Option<ActiveContext>,
}
```

#### Helper Functions (lines 2190-2370)

**Path Operations**:

-   `canonicalize_project_path(path)` - Resolve symlinks and normalize
-   `hash_project_path(path)` - SHA256 hash (first 16 chars)
-   `get_project_configs_dir()` - ~/.ccconfig/project-configs

**File I/O**:

-   `read_project_config_file(project_path)` - Load from disk
-   `write_project_config_file(config)` - Save to disk

**Config Merging** (lines 2254-2316):

```
merge_settings(global, project) -> Value
├── Deep merge objects recursively
├── Special handling for "permissions" key
│   ├── permissions.deny: union arrays
│   ├── permissions.allow: union arrays
│   └── Other permission fields: project overrides
└── Project values override global for all other keys
```

**Enterprise Settings** (lines 2318-2354):

-   macOS: `/Library/Application Support/ClaudeCode/managed-*.json`
-   Linux: `/etc/claude-code/managed-*.json`
-   Windows: `C:\ProgramData\ClaudeCode\managed-*.json`

**Context Management** (lines 2371-2410):

-   `read_active_context()` - Load from stores.json
-   `write_active_context(context)` - Save to stores.json

#### Tauri Commands (lines 2470-2900+)

1. **get_project_configs()** → Vec<ProjectConfigStore>

    - Lists all project configurations
    - Sorted by last_used_at (most recent first)

2. **get_project_config(project_path)** → Option<ProjectConfigStore>

    - Retrieve configuration for specific project

3. **create_project_config(project_path, title, settings, parent_id)** → ProjectConfigStore

    - Create new project configuration
    - Validates path doesn't already exist
    - Generates 6-char ID and timestamps
    - Stores with hash-based filename

4. **update_project_config(project_path, title, settings)** → ProjectConfigStore

    - Update existing project config
    - If currently active: apply merged settings to ~/.claude/settings.json
    - Merge with parent global config if inherit_from_global=true

5. **delete_project_config(project_path)** → ()

    - Remove project configuration
    - Clear active context if this was the active project

6. **activate_project_config(project_path)** → ()

    - Switch to project context
    - Update last_used_at timestamp
    - Merge settings with parent global config
    - Write merged settings to ~/.claude/settings.json
    - Update active_context in stores.json

7. **get_active_context()** → Option<ActiveContext>

    - Get current active context (global/project)

8. **switch_to_global_context(store_id)** → ()

    - Switch to global configuration
    - Set global config as "using"
    - Update active_context in stores.json

9. **auto_create_project_config(project_path)** → ProjectConfigStore

    - Auto-create from current active global config
    - Returns existing if already created
    - Extracts project name from path
    - Sets inherit_from_global=true

10. **get_active_merged_config()** → Value

    - Get final merged config for active context
    - Project: merge with parent global if inherit_from_global=true
    - Global: return config settings
    - None: return default empty object

11. **check_project_local_settings(project_path)** → Option<Value>

    - Check for <project>/.claude/settings.json
    - Returns content if exists

12. **import_project_local_settings(project_path)** → ProjectConfigStore

    - Import from project's local .claude/settings.json
    - Creates new project config
    - Sets inherit_from_global=false (imported keeps its settings)
    - Marks as "(Imported)" in title

13. **update_project_config_path(old_path, new_path)** → ProjectConfigStore

    - Re-link configuration after project rename/move
    - Deletes old config file (old hash)
    - Creates new with updated path
    - Updates active context if was active

14. **add_project_to_tracking(project_path)** → ()

    - Add project to tracking list
    - May trigger auto-config creation

15. **validate_project_path(project_path)** → bool

    - Verify project path exists and is accessible
    - Returns true if valid

16. **get_managed_settings()** → Option<Value>

    - Get enterprise-managed settings (read-only)
    - Checks OS-specific paths

17. **get_managed_mcp_servers()** → Option<Value>
    - Get enterprise-managed MCP servers (read-only)

---

## 4. Chat Interface Module (Phase 1)

### 4.0 Chat Module Organization

**File Structure**:

```
src-tauri/src/chat/
├── mod.rs              # Module exports and initialization
├── session.rs          # Data structures (ChatSession, ChatMessage, ChatConfig)
├── storage.rs          # File-based persistence operations
├── claude_cli.rs       # CLI spawning, stream parsing, process management
├── commands.rs         # 9 Tauri commands for chat operations
└── tests.rs            # 27 unit tests

src/lib/chat-query.ts   # React Query hooks for frontend
```

**Integration Points**:

-   `src-tauri/src/lib.rs`: Initializes `StreamProcesses` state and registers chat commands
-   `src-tauri/Cargo.toml`: Adds tokio (async), uuid (session IDs), tempfile (testing)
-   `src/lib/chat-query.ts`: Wraps all chat commands in React Query hooks

### 4.1 Data Structures

**ChatSession** (session.rs):

```rust
pub struct ChatSession {
    pub id: String,                    // UUID v4
    pub project_path: String,          // Absolute project path
    pub title: String,                 // User-friendly name
    pub created_at: u64,               // Unix timestamp
    pub updated_at: u64,               // Unix timestamp
    pub message_count: usize,          // Number of messages in session
}
```

**ChatMessage** (session.rs):

```rust
pub struct ChatMessage {
    pub id: String,                    // UUID v4
    pub session_id: String,            // Parent session ID
    pub role: MessageRole,             // User|Assistant|System|Tool
    pub content: String,               // Message text
    pub timestamp: u64,                // Unix timestamp
    pub tool_use: Option<ToolUse>,     // Optional tool invocation data
    pub metadata: Option<Value>,       // Optional metadata (streaming, etc)
}

pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}
```

**ToolUse** (session.rs):

```rust
pub struct ToolUse {
    pub tool_name: String,             // Name of tool (e.g., "bash")
    pub input: Value,                  // Tool input/parameters
    pub output: Option<String>,        // Optional tool output
}
```

**ChatConfig** (session.rs):

```rust
pub struct ChatConfig {
    pub model: String,                 // sonnet|opus|haiku (validated)
    pub permission_mode: PermissionMode,  // default|acceptEdits|bypassPermissions|plan
    pub max_tokens: Option<u32>,       // Optional token limit
    pub temperature: Option<f32>,      // Optional temperature setting
}

pub enum PermissionMode {
    Default,
    AcceptEdits,
    BypassPermissions,
    Plan,
}
```

### 4.2 Storage Module (storage.rs)

**Storage Location**:

```
~/.ccconfig/chat-sessions/
├── {uuid-session-id}/
│   ├── session.json      # ChatSession metadata
│   └── messages.json     # Vec<ChatMessage>
```

**Key Functions**:

-   `save_session(session, messages)` - Create or update session with messages
-   `load_session(session_id)` - Load session metadata and all messages
-   `list_sessions(project_path)` - List all sessions for a project (sorted by updated_at)
-   `delete_session(session_id)` - Remove session directory
-   `update_session_metadata(session)` - Update only session.json (timestamps, title)

### 4.3 Claude CLI Module (claude_cli.rs)

**Process Spawning**:

```rust
pub type StreamProcesses = Arc<Mutex<HashMap<String, tokio::process::Child>>>;

pub async fn spawn_claude_stream(
    app: AppHandle,
    session_id: String,
    message: String,
    project_path: String,
    model: String,
    processes: StreamProcesses,
) -> Result<(), String>
```

**Command Executed**:

```bash
claude -p --output-format stream-json
# Input: User message sent via stdin
# Output: JSONL stream of response chunks
```

**Stream Format** (JSONL - one object per line):

```json
{"type": "text", "content": "Hello, I can help"}
{"type": "text", "content": " with that!"}
{"type": "tool_use", "name": "bash", "input": {"command": "ls"}}
{"type": "tool_result", "content": "file1.txt\nfile2.txt"}
```

**Process Management**:

-   Tracks running processes by session ID
-   Reads stdout line-by-line via BufReader
-   Emits Tauri events for each chunk: `chat-stream:{sessionId}`
-   Cleans up process on completion or cancellation

**Validation**:

-   `check_claude_installed()` - Verifies claude CLI in PATH
-   Model whitelist: sonnet, opus, haiku (prevents arbitrary models)
-   Path canonicalization: resolves symlinks
-   UUID session ID format validation

### 4.4 Tauri Commands (commands.rs)

#### Command Reference

1. **chat_check_claude_installed() → Result<bool, String>**

   - Check if Claude CLI is available in PATH
   - Runs `claude --version` silently

2. **chat_create_session(project_path, title?) → Result<ChatSession, String>**

   - Create new session for project
   - Generates UUID, timestamps
   - Saves empty session file
   - Returns created session object
   - Error if project_path not absolute

3. **chat_get_sessions(project_path) → Result<Vec<ChatSession>, String>**

   - List all sessions for a project
   - Sorted by updated_at (most recent first)
   - Error if project not found

4. **chat_get_messages(session_id) → Result<Vec<ChatMessage>, String>**

   - Load all messages from session
   - Returns empty array if session empty
   - Error if session not found

5. **chat_delete_session(session_id) → Result<(), String>**

   - Remove session directory and all contents
   - Silent success if already deleted

6. **chat_send_message(session_id, message, config?) → Result<(), String>**

   - Primary command for user messages
   - Validates input not empty
   - Saves user message to session
   - Spawns Claude CLI stream in background
   - Emits `chat-stream:{sessionId}` events
   - Returns immediately (streaming continues asynchronously)
   - Optional ChatConfig overrides default

7. **chat_cancel_stream(session_id) → Result<(), String>**

   - Kill running stream process for session
   - Cleans up process handle
   - Safe to call if no process running

8. **chat_save_assistant_message(session_id, content) → Result<(), String>**

   - Save complete assistant response to session
   - Updates message_count and updated_at
   - Called after streaming completes
   - Content is full accumulated response

9. **chat_update_session_title(session_id, title) → Result<(), String>**

   - Rename session for UI display
   - Updates session.json
   - Updates updated_at timestamp

#### Error Handling

All commands return `Result<T, String>` with descriptive error messages:

-   "Session not found" - Invalid or deleted session ID
-   "Invalid project path" - Non-absolute or non-existent path
-   "Claude CLI not installed" - Missing claude command
-   "Invalid model" - Model not in whitelist
-   "Failed to parse stream" - JSONL parse error
-   "I/O error" - File system issues

### 4.5 Frontend Integration (chat-query.ts)

**React Query Hooks** (7 hooks):

```typescript
// Query hooks
useCheckClaudeInstalled()        // Check CLI availability
useGetChatSessions(projectPath)  // List sessions
useGetChatMessages(sessionId)    // Load messages

// Mutation hooks
useCreateChatSession()           // Create new session
useDeleteChatSession()           // Remove session
useSendChatMessage()             // Send message + stream
useCancelChatStream()            // Stop streaming
useSaveAssistantMessage()        // Persist response
useUpdateChatSessionTitle()      // Rename session
```

**Event Streaming**:

```typescript
const unsubscribe = listen<string>(
  `chat-stream:${sessionId}`,
  (event) => {
    // Handle streaming content
    const content = event.payload;
    // Update UI in real-time
  }
);
```

**Query Key Structure**:

```typescript
["claude-installed"]                      // CLI check
["chat-sessions", projectPath]           // Sessions list
["chat-messages", sessionId]             // Messages for session
```

### 4.6 Data Flow

```
Frontend                          Backend                          File System
───────────────────────────────────────────────────────────────────────────────

User types message & sends
    │
    ▼
chat_send_message(sessionId, msg)
    │
    ├──────────────────────────────────► Validate input
    │                                   Canonicalize path
    │                                   Load session file
    │
    │                                   Save user message
    │                                   Save to session.json  ────► session.json
    │
    │                                   Spawn process:
    │                                   claude -p --output-format stream-json
    │
    ◄──────────────────────────────────┐
    │                                   │
    │                                   └─ Read JSONL stream line by line
    │                                      Parse {"type": "text", "content": "..."}
    │
Listen for chat-stream:{sessionId}      │
    │                                   │
    ▼◄──────────────────────────────────┘
    │
Update UI with content                  Accumulate full response
    │                                   │
    │                                   └─ Send remaining events
    │                                   │  as stream completes
    │
User calls save when done
    │
    ▼
chat_save_assistant_message(sessionId, fullContent)
    │
    └──────────────────────────────────► Save assistant message
                                        Save to messages.json  ───► messages.json
```

---

## 5. Data Flow Diagrams

### 5.1 Project Config Creation Flow

```
Frontend (React)
    ↓
invoke('create_project_config', {
  project_path: "/path/to/project",
  title: "My Project",
  settings: { ... },
  parent_global_config_id: "abc123" (optional)
})
    ↓
Tauri Backend
    ├─ canonicalize_project_path() → /Users/me/path/to/project
    ├─ hash_project_path() → "a1b2c3d4e5f6g7h8"
    ├─ generate_id() → "xyz789"
    ├─ get_timestamp() → 1234567890
    └─ write_project_config_file(ProjectConfigStore)
         └─ ~/.ccconfig/project-configs/a1b2c3d4e5f6g7h8.json
    ↓
Frontend (success callback)
    └─ Update project configs list
```

### 4.2 Project Activation Flow

```
User clicks "Activate Project"
    ↓
invoke('activate_project_config', { project_path })
    ↓
Tauri Backend
    ├─ read_project_config_file() → ProjectConfigStore
    ├─ update_last_used_at()
    ├─ write_project_config_file()
    ├─ if inherit_from_global:
    │   ├─ get_stores() → find parent by id
    │   └─ merge_settings(parent_settings, project_settings)
    │       → merged_settings
    ├─ apply_config_to_settings(merged_settings)
    │   └─ write ~/.claude/settings.json
    └─ write_active_context({
         type: "project",
         id: "xyz789",
         projectPath: "/path/to/project"
       })
    ↓
stores.json updated with active context
    ↓
Frontend
    └─ activeContext is now "project"
       show merged config in UI
       highlight this project as active
```

### 4.3 Config Merge Flow

```
Global Config:
{
  "model": "claude-opus-4-5-20251101",
  "permissions": {
    "deny": ["Read(/.env)"]
  },
  "env": {
    "API_KEY": "global-key"
  }
}

Project Config:
{
  "model": "claude-sonnet-4-20250514",
  "permissions": {
    "deny": ["Execute(npm)"]
  },
  "env": {
    "API_KEY": "project-key"
  }
}

merge_settings(global, project)
    ↓
Result:
{
  "model": "claude-sonnet-4-20250514",    ← project overrides
  "permissions": {
    "deny": [                              ← union of arrays
      "Read(/.env)",                       (from global)
      "Execute(npm)"                       (from project)
    ]
  },
  "env": {
    "API_KEY": "project-key"               ← project overrides
  }
}
```

### 4.4 Context Switching Flow

```
┌─────────────────────────────────────────────────────────┐
│              Active: Project "my-app"                    │
│  ~/.claude/settings.json contains merged settings       │
│  activeContext.type = "project"                         │
└─────────────────────────────────────────────────────────┘
                        ↓ (user action)
invoke('switch_to_global_context', { store_id: "abc123" })
                        ↓
                  Backend
            ├─ set_using_config("abc123")
            │   └─ update stores.json using=true for this config
            ├─ get_stores() → find by id
            ├─ apply_config_to_settings(config.settings)
            │   └─ write ~/.claude/settings.json
            └─ write_active_context({
                 type: "global",
                 id: "abc123"
               })
                        ↓
┌─────────────────────────────────────────────────────────┐
│              Active: Global "Production"                 │
│  ~/.claude/settings.json contains global settings       │
│  activeContext.type = "global"                          │
└─────────────────────────────────────────────────────────┘
```

---

## 4.5 Frontend Hooks API Reference

### Global Configuration Hooks

| Hook                         | Type     | Input                      | Output              | Purpose                 |
| ---------------------------- | -------- | -------------------------- | ------------------- | ----------------------- |
| `useStores()`                | Query    | -                          | Vec<ConfigStore>    | List all global configs |
| `useStore(storeId)`          | Query    | String                     | ConfigStore         | Get specific config     |
| `useCurrentStore()`          | Query    | -                          | ConfigStore \| null | Get active config       |
| `useCreateConfig()`          | Mutation | {title, settings}          | ConfigStore         | Create new config       |
| `useUpdateConfig()`          | Mutation | {storeId, title, settings} | ConfigStore         | Update config           |
| `useDeleteConfig()`          | Mutation | storeId                    | void                | Delete config           |
| `useSetUsingConfig()`        | Mutation | storeId                    | void                | Mark as active          |
| `useResetToOriginalConfig()` | Mutation | -                          | void                | Restore from backup     |

### Project Configuration Hooks

| Hook                             | Type     | Input                             | Output                     | Purpose                  |
| -------------------------------- | -------- | --------------------------------- | -------------------------- | ------------------------ |
| `useProjectConfigs()`            | Query    | -                                 | Vec<ProjectConfigStore>    | List all project configs |
| `useProjectConfig(path)`         | Query    | String                            | ProjectConfigStore \| null | Get project config       |
| `useCreateProjectConfig()`       | Mutation | {path, title, settings, parentId} | ProjectConfigStore         | Create project config    |
| `useUpdateProjectConfig()`       | Mutation | {path, title, settings}           | ProjectConfigStore         | Update project config    |
| `useDeleteProjectConfig()`       | Mutation | path                              | void                       | Delete project config    |
| `useActivateProjectConfig()`     | Mutation | path                              | void                       | Switch to project        |
| `useAutoCreateProjectConfig()`   | Mutation | path                              | ProjectConfigStore         | Auto-create from global  |
| `useActiveContext()`             | Query    | -                                 | ActiveContext \| null      | Get current context      |
| `useSwitchToGlobalContext()`     | Mutation | storeId                           | void                       | Switch to global         |
| `useActiveMergedConfig()`        | Query    | -                                 | unknown                    | Get merged config        |
| `useCheckProjectLocalSettings()` | Query    | path                              | unknown \| null            | Check local settings     |

### MCP Server Hooks

| Hook                         | Type     | Input          | Output                    | Purpose           |
| ---------------------------- | -------- | -------------- | ------------------------- | ----------------- |
| `useGlobalMcpServers()`      | Query    | -              | Record<string, McpServer> | Get MCP servers   |
| `useUpdateGlobalMcpServer()` | Mutation | {name, config} | void                      | Create/update MCP |
| `useAddGlobalMcpServer()`    | Mutation | {name, config} | void                      | Add MCP server    |
| `useDeleteGlobalMcpServer()` | Mutation | name           | void                      | Delete MCP        |
| `useCheckMcpServerExists()`  | Query    | name           | bool                      | Check if exists   |

### Memory, Commands, & Agents Hooks

| Hook                       | Type     | Input           | Output           | Purpose               |
| -------------------------- | -------- | --------------- | ---------------- | --------------------- |
| `useClaudeMemory()`        | Query    | -               | MemoryFile       | Read CLAUDE.md        |
| `useWriteClaudeMemory()`   | Mutation | content         | void             | Write CLAUDE.md       |
| `useClaudeCommands()`      | Query    | -               | Vec<CommandFile> | List commands         |
| `useWriteClaudeCommand()`  | Mutation | {name, content} | void             | Create/update command |
| `useDeleteClaudeCommand()` | Mutation | name            | void             | Delete command        |
| `useClaudeAgents()`        | Query    | -               | Vec<CommandFile> | List agents           |
| `useWriteClaudeAgent()`    | Mutation | {name, content} | void             | Create/update agent   |
| `useDeleteClaudeAgent()`   | Mutation | name            | void             | Delete agent          |

### Config File Operations Hooks

| Hook                       | Type     | Input           | Output          | Purpose               |
| -------------------------- | -------- | --------------- | --------------- | --------------------- |
| `useConfigFiles()`         | Query    | -               | Vec<ConfigType> | List all config types |
| `useConfigFile(type)`      | Query    | ConfigType      | ConfigFile      | Get config file       |
| `useWriteConfigFile()`     | Mutation | {type, content} | void            | Write config file     |
| `useBackupClaudeConfigs()` | Mutation | -               | void            | Backup configs        |

### Project & Analytics Hooks

| Hook                         | Type     | Input   | Output                  | Purpose             |
| ---------------------------- | -------- | ------- | ----------------------- | ------------------- |
| `useClaudeProjects()`        | Query    | -       | Vec<ProjectConfig>      | List projects       |
| `useClaudeConfigFile()`      | Query    | -       | ClaudeConfigFile        | Read .claude.json   |
| `useWriteClaudeConfigFile()` | Mutation | content | void                    | Write .claude.json  |
| `useProjectUsageFiles()`     | Query    | -       | Vec<ProjectUsageRecord> | Get usage analytics |
| `useCheckForUpdates()`       | Query    | -       | UpdateInfo              | Check updates       |
| `useInstallAndRestart()`     | Mutation | -       | void                    | Install update      |

### Notification Settings Hooks

| Hook                              | Type     | Input    | Output                       | Purpose         |
| --------------------------------- | -------- | -------- | ---------------------------- | --------------- |
| `useNotificationSettings()`       | Query    | -        | NotificationSettings \| null | Get settings    |
| `useUpdateNotificationSettings()` | Mutation | settings | void                         | Update settings |

### Import & Migration Hooks

| Hook                              | Type     | Input | Output             | Purpose               |
| --------------------------------- | -------- | ----- | ------------------ | --------------------- |
| `useImportProjectLocalSettings()` | Mutation | path  | ProjectConfigStore | Import local settings |

---

## 5. Configuration File Formats

### 5.1 Project Config File

**Location**: `~/.ccconfig/project-configs/{hash}.json`

**Example**:

```json
{
	"projectPath": "/Users/developer/projects/my-app",
	"canonicalPath": "/Users/developer/projects/my-app",
	"id": "a7b2c9",
	"title": "My App - Production",
	"createdAt": 1733505600,
	"lastUsedAt": 1733523400,
	"settings": {
		"model": "claude-opus-4-5-20251101",
		"permissions": {
			"defaultMode": "default",
			"deny": ["Read(/.env.local)", "Execute(npm install)"],
			"allow": ["Read(/src)", "Execute(npm run test)"]
		},
		"env": {
			"ANTHROPIC_API_KEY": "sk-proj-...",
			"DEPLOYMENT_ENV": "production"
		},
		"mcpServers": {}
	},
	"inheritFromGlobal": true,
	"parentGlobalConfigId": "vtTZmS"
}
```

### 5.2 Active Context in stores.json

**Location**: `~/.ccconfig/stores.json`

**Structure**:

```json
{
  "configs": [
    {
      "id": "vtTZmS",
      "title": "Default",
      "createdAt": 1733419200,
      "settings": { ... },
      "using": false
    }
  ],
  "distinct_id": "user-12345",
  "notification": {
    "enable": true,
    "enabledHooks": ["Notification", "Stop"]
  },
  "activeContext": {
    "type": "project",
    "id": "a7b2c9",
    "projectPath": "/Users/developer/projects/my-app"
  }
}
```

### 5.3 Project Local Settings

**Location**: `<project>/.claude/settings.json`

**Structure** (same as global settings):

```json
{
  "model": "claude-sonnet-4-20250514",
  "permissions": { ... },
  "env": { ... },
  "mcpServers": { ... }
}
```

---

## 6. API Command Reference

### 6.1 Global Config Commands

| Command                              | Input                   | Output              | Purpose                  |
| ------------------------------------ | ----------------------- | ------------------- | ------------------------ |
| `get_stores()`                       | -                       | Vec<ConfigStore>    | List all global configs  |
| `get_store(id)`                      | String                  | Option<ConfigStore> | Get specific config      |
| `create_config(title, settings)`     | (String, Value)         | ConfigStore         | Create new global config |
| `update_config(id, title, settings)` | (String, String, Value) | ConfigStore         | Update config            |
| `delete_config(id)`                  | String                  | ()                  | Delete config            |
| `set_using_config(id)`               | String                  | ()                  | Mark as active           |
| `reset_to_original_config()`         | -                       | ()                  | Restore from backup      |

### 6.2 Project Config Commands

| Command                                                   | Input                                   | Output                     | Purpose                  |
| --------------------------------------------------------- | --------------------------------------- | -------------------------- | ------------------------ |
| `get_project_configs()`                                   | -                                       | Vec<ProjectConfigStore>    | List all project configs |
| `get_project_config(path)`                                | String                                  | Option<ProjectConfigStore> | Get specific project     |
| `create_project_config(path, title, settings, parent_id)` | (String, String, Value, Option<String>) | ProjectConfigStore         | Create project config    |
| `update_project_config(path, title, settings)`            | (String, String, Value)                 | ProjectConfigStore         | Update project config    |
| `delete_project_config(path)`                             | String                                  | ()                         | Delete project config    |
| `activate_project_config(path)`                           | String                                  | ()                         | Switch to project        |
| `get_active_context()`                                    | -                                       | Option<ActiveContext>      | Get current context      |
| `switch_to_global_context(id)`                            | String                                  | ()                         | Switch to global         |
| `auto_create_project_config(path)`                        | String                                  | ProjectConfigStore         | Auto-create from global  |
| `get_active_merged_config()`                              | -                                       | Value                      | Get merged config        |
| `check_project_local_settings(path)`                      | String                                  | Option<Value>              | Check local settings     |
| `import_project_local_settings(path)`                     | String                                  | ProjectConfigStore         | Import from local        |
| `update_project_config_path(old, new)`                    | (String, String)                        | ProjectConfigStore         | Update path              |
| `validate_project_path(path)`                             | String                                  | bool                       | Validate path            |

### 6.3 Enterprise Commands

| Command                     | Input | Output        | Purpose                  |
| --------------------------- | ----- | ------------- | ------------------------ |
| `get_managed_settings()`    | -     | Option<Value> | Read enterprise settings |
| `get_managed_mcp_servers()` | -     | Option<Value> | Read enterprise MCP      |

---

## 7. Error Handling

### 7.1 Backend Error Handling

All commands return `Result<T, String>`:

-   Success: `Ok(value)`
-   Error: `Err(error_message)`

**Common Errors**:

-   "Could not find home directory" - System paths unavailable
-   "Failed to read/write file" - File I/O errors with details
-   "Project config not found" - Path not in tracking
-   "Failed to canonicalize path" - Invalid project path
-   "Failed to parse settings" - Malformed JSON

### 7.2 Frontend Error Handling

React Query automatically:

-   Retries failed requests (configurable)
-   Stores error state
-   Allows user to manually retry
-   Displays error messages in UI

---

## 8. Performance Considerations

### 8.1 Optimization Strategies

**Current**:

-   Direct file I/O (simple, no dependencies)
-   Minimal in-memory caching
-   SHA256 hashing for path identification

**Future Optimizations**:

-   Lazy-load project configs on demand
-   Cache merged configurations
-   Batch file I/O operations
-   Optimize deep merge for large settings objects

### 8.2 Scalability

**Design supports**:

-   Hundreds of project configurations (each <10KB file)
-   Rapid context switching (<100ms)
-   Concurrent config operations (file-based locking)

---

## 9. Security Considerations

### 9.1 Path Security

-   Canonicalization prevents path traversal attacks
-   Hash-based filenames don't expose paths
-   Validates project paths exist before processing

### 9.2 Settings Security

-   No secrets stored in code
-   Project configs stored with user file permissions (~/.ccconfig/)
-   Enterprise settings read from OS-protected locations
-   JSON validation prevents injection attacks

### 9.3 Context Security

-   Active context persists in user-only-readable stores.json
-   No auth required (assumes single-user desktop app)
-   Enterprise managed settings are read-only

---

## 10. Testing Strategy

### 10.1 Backend Testing

**Unit Tests** (in rust tests):

-   Path canonicalization
-   SHA256 hashing
-   Config merge logic
-   File I/O operations
-   Error cases

**Integration Tests**:

-   Command execution
-   File system interaction
-   Context switching
-   Settings application

### 10.2 Frontend Testing

**Unit Tests** (Jest):

-   React Query hooks
-   Utility functions
-   Form validation

**E2E Tests** (Playwright):

-   Create/update/delete configs
-   Context switching
-   Settings merging verification
-   Local settings import

---

## 11. Deployment & Distribution

### 11.1 Build Process

```bash
pnpm build
↓
Vite bundles React frontend
↓
Tauri CLI packages Rust backend
↓
Platform-specific binaries
├─ .dmg (macOS)
├─ .exe (Windows)
└─ .AppImage/.deb (Linux)
```

### 11.2 Update Mechanism

-   Built-in updater plugin (tauri-plugin-updater)
-   Checks for updates on startup
-   Notifies user of available versions
-   One-click install and restart

---

## Appendix: Dependencies

### Rust Dependencies (Cargo.toml)

```toml
tauri = "2"                    # Desktop framework
tauri-plugin-store = "2"       # Persistent storage
serde, serde_json = "1"        # Serialization
sha2 = "0.10"                  # SHA256 hashing
dirs = "5"                     # Home directory
chrono = "0.4"                 # Date/time
uuid = "1.0"                   # ID generation
nanoid = "0.4"                 # Nano ID generation
tokio = "1"                    # Async runtime
reqwest = "0.11"               # HTTP client (PostHog)
```

### Node Dependencies (package.json)

```json
react = "19"                   # UI framework
react-router-dom = "^6"        # Routing
@tanstack/react-query = "^5"  # Server state management
react-hook-form = "^7"         # Form state
zod = "^3"                     # Validation
tailwindcss = "^4"             # Styling
@tauri-apps/api = "^2"         # Tauri bindings
shadcn/ui                      # Component library
```
