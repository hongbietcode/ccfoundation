# CC Mate - Project Overview & Product Development Requirements

**Last Updated**: 2025-12-06
**Version**: 1.0
**Status**: Phase 1 (Backend Foundation) Complete, Phase 2 (Frontend) In Progress

---

## Executive Summary

CC Mate is a modern desktop application for managing Claude Code configuration files across multiple contexts (global, enterprise, and per-project). Built with Tauri 2, React 19, and Rust, it provides an intuitive UI to configure, organize, and switch between different Claude Code configurations without manual file editing.

**Current Milestone**: Phase 1 Backend Foundation - Per-project configuration storage and management system complete.

---

## 1. Project Overview

### 1.1 Purpose & Vision

Enable developers to seamlessly manage Claude Code configurations across:
- **Global scope**: User-level settings applicable across all projects
- **Enterprise scope**: System-managed settings for organization compliance
- **Per-project scope**: Project-specific configurations that override global settings

### 1.2 Core Features

- **Multi-Configuration Support**: Switch between multiple named Claude Code configurations
- **Per-Project Configuration**: Project-specific settings with inheritance from global configs
- **MCP Server Management**: Configure and manage Model Context Protocol servers
- **Agent Management**: Manage Claude Code agents and their settings
- **Global Commands**: Define organization-wide slash commands
- **CLAUDE.md Integration**: Read and write global CLAUDE.md memory files
- **Configuration Merging**: Deep merge project settings with global settings (union arrays for permissions)
- **Enterprise Managed Settings**: Read-only enterprise-managed configurations
- **Auto-import**: Automatically detect and import existing `.claude/settings.json` from projects
- **Context Switching**: Switch between global and project configurations
- **Usage Analytics**: Track Claude Code usage patterns
- **Configuration Backup**: Automatic backup of Claude configs on first run

### 1.3 Target Users

- Individual developers using Claude Code
- Development teams managing multiple projects
- Enterprise organizations with managed Claude Code deployments

---

## 2. Phase 1: Backend Foundation (Complete)

### 2.1 Completed Work

**Added Dependencies**:
- `sha2 = "0.10"` - SHA256 hashing for project path canonicalization

**New Data Structures**:
- `ProjectConfigStore` - Per-project configuration with inheritance support
- `ActiveContext` - Track whether global or project configuration is active
- `EnhancedStoresData` - Extended stores.json with context management

**Implemented Commands (17 new Tauri commands)**:

1. **get_project_configs()** - List all project configurations
2. **get_project_config(project_path)** - Retrieve configuration for specific project
3. **create_project_config(project_path, title, settings, parent_id)** - Create new project configuration
4. **update_project_config(project_path, title, settings)** - Update existing project configuration
5. **delete_project_config(project_path)** - Remove project configuration
6. **activate_project_config(project_path)** - Switch to project context and apply merged settings
7. **get_active_context()** - Get current active context (global or project)
8. **switch_to_global_context(store_id)** - Switch back to global configuration
9. **auto_create_project_config(project_path)** - Create project config from active global config
10. **get_active_merged_config()** - Get merged configuration for current context
11. **check_project_local_settings(project_path)** - Check for `.claude/settings.json` in project
12. **import_project_local_settings(project_path)** - Import local settings as project configuration
13. **update_project_config_path(old_path, new_path)** - Re-link configuration after project rename/move
14. **add_project_to_tracking(project_path)** - Add project to tracking list
15. **validate_project_path(project_path)** - Validate project path exists and is accessible
16. **get_managed_settings()** - Get enterprise-managed settings (read-only)
17. **get_managed_mcp_servers()** - Get enterprise-managed MCP servers (read-only)

**Core Infrastructure**:
- Path canonicalization (resolve symlinks, normalize)
- SHA256 hashing for secure project identification
- Deep merge logic with special handling for permission arrays
- Enterprise managed settings detection (macOS, Linux, Windows)
- Active context persistence in stores.json

### 2.2 Architecture Details

**Storage Location**:
- Project configs: `~/.ccconfig/project-configs/{project-hash}.json`
- Active context: `~/.ccconfig/stores.json` (activeContext field)

**Project Config Structure**:
```json
{
  "projectPath": "/Users/me/projects/my-app",
  "canonicalPath": "/Users/me/projects/my-app",
  "id": "abc123",
  "title": "My App Config",
  "createdAt": 1234567890,
  "lastUsedAt": 1234567890,
  "settings": {
    "model": "claude-opus-4-5-20251101",
    "permissions": {
      "defaultMode": "default",
      "deny": ["Read(/.env)"]
    }
  },
  "inheritFromGlobal": true,
  "parentGlobalConfigId": "vtTZmS"
}
```

**Active Context Structure**:
```json
{
  "activeContext": {
    "type": "global",
    "id": "config-id",
    "projectPath": null
  }
}
```

### 2.3 Key Implementation Details

**Path Handling**:
- Canonical path computed from actual file system (resolves symlinks)
- SHA256 hash of canonical path (first 16 chars) for config file naming
- Prevents duplicate configs for same project with different path formats

**Configuration Merging**:
- Deep merge of global and project settings
- Special handling for permissions arrays: union of deny and allow
- Project settings override global settings for all other fields
- Recursive merge for nested objects

**Enterprise Managed Settings**:
- macOS: `/Library/Application Support/ClaudeCode/managed-settings.json`
- Linux: `/etc/claude-code/managed-settings.json`
- Windows: `C:\ProgramData\ClaudeCode\managed-settings.json`
- Read-only, detected and returned separately

---

## 3. Phase 2: Frontend Implementation (In Progress)

### 3.1 Planned Components

- **Project Config Tab**: UI for managing per-project configurations
- **Config Selector**: Switch between global and project contexts
- **Project List**: Display all tracked projects with their configs
- **Context Indicator**: Show current active context (global/project)
- **Config Inheritance UI**: Visual indication of inherited vs. project-specific settings
- **Local Settings Import**: Detect and import existing `.claude/settings.json`

### 3.2 User Workflows

**Workflow 1**: Create project-specific configuration
1. Open project in CC Mate
2. System auto-detects project or user adds it manually
3. User creates project config (can inherit from active global config)
4. User customizes settings (model, permissions, env vars)
5. User activates project config
6. CC Mate applies merged settings to `~/.claude/settings.json`

**Workflow 2**: Import existing local settings
1. User opens project with existing `.claude/settings.json`
2. CC Mate detects local settings
3. User clicks "Import to CC Mate"
4. System creates project configuration from local settings
5. Local settings can be optionally removed after import

**Workflow 3**: Switch between contexts
1. User switches between projects
2. CC Mate shows project-specific config in UI
3. User can quickly switch back to global context
4. All settings automatically updated in `~/.claude/settings.json`

---

## 4. Technical Architecture

### 4.1 Tech Stack

**Frontend**:
- React 19 with TypeScript (strict mode)
- React Router for navigation
- React Hook Form + Zod for form validation
- React Query (@tanstack/react-query) for server state management
- Tailwind CSS v4 via @tailwindcss/vite
- shadcn/ui components
- Vite as build tool

**Backend**:
- Tauri v2 framework
- Rust with async/await (tokio runtime)
- serde/serde_json for serialization
- SHA256 hashing (sha2 crate)
- File I/O with standard Rust stdlib

**Build & Package**:
- pnpm for package management
- Tauri CLI for desktop app building
- Cross-platform support (macOS, Windows, Linux)

### 4.2 Application Structure

**Frontend**:
```
src/
├── main.tsx              # React app entry point
├── components/
│   └── ui/              # shadcn/ui components
├── lib/
│   ├── query.ts         # React Query hooks & API functions
│   └── utils.ts         # Utility functions
└── pages/               # Route pages
```

**Backend (Rust)**:
```
src-tauri/
├── src/
│   ├── main.rs          # Tauri app entry point
│   ├── lib.rs           # App setup & plugin configuration
│   ├── commands.rs      # All Tauri commands (2900+ lines)
│   └── hook_server.rs   # Hook server implementation
└── Cargo.toml           # Rust dependencies
```

### 4.3 Data Flow

1. **Reading Config**:
   - Frontend calls React Query hook → Tauri command → File system read
   - Returns ConfigFile, ProjectConfigStore, or EnhancedStoresData

2. **Updating Config**:
   - Frontend form submission → Validation → Tauri command → File write
   - Project config updates may apply merged settings to `~/.claude/settings.json`

3. **Context Switching**:
   - User action → activate_project_config/switch_to_global_context
   - Tauri merges settings and updates stores.json
   - Writes final merged config to `~/.claude/settings.json`

4. **Enterprise Settings**:
   - Application reads managed settings from OS-specific paths
   - Merge with global settings if applicable
   - Display as read-only in UI

---

## 5. File Organization

### 5.1 Configuration Files

**User-managed**:
- `~/.claude/settings.json` - Global Claude Code settings (updated by CC Mate)
- `~/.claude/CLAUDE.md` - Global memory file
- `~/.claude/commands/*.md` - Global custom commands
- `~/.claude/agents/*.md` - Global agent definitions
- `<project>/.claude/settings.json` - Project-specific settings (optional)

**CC Mate-managed**:
- `~/.ccconfig/stores.json` - Global configs, active context, notification settings
- `~/.ccconfig/project-configs/*.json` - Per-project configurations (hashed by path)
- `~/.ccconfig/claude_backup/` - Initial backup of Claude configs

**Enterprise-managed (read-only)**:
- macOS: `/Library/Application Support/ClaudeCode/managed-settings.json`
- Linux: `/etc/claude-code/managed-settings.json`
- Windows: `C:\ProgramData\ClaudeCode\managed-settings.json`

### 5.2 Project Files

- `src-tauri/Cargo.toml` - Rust dependencies
- `src-tauri/src/commands.rs` - Main Tauri commands (2922 lines)
- `src-tauri/src/lib.rs` - App initialization (handler registration)
- `package.json` - Frontend dependencies and scripts
- `vite.config.ts` - Vite configuration

---

## 6. Development Workflow

### 6.1 Setup

```bash
# Install dependencies
pnpm install

# Development server
pnpm tauri dev

# Build for production
pnpm build
```

### 6.2 Code Standards

- TypeScript strict mode enforced
- Functional components in React (no class components)
- JSDoc comments for public APIs
- Place React Query logic in `src/lib/query.ts`
- Place Tauri commands in `src-tauri/src/commands.rs`
- Naming convention: camelCase for JS/TS, snake_case for Rust
- Use `pnpm tsc --noEmit` for type checking

### 6.3 Testing & Quality

- Unit tests with Jest
- E2E tests with Playwright
- Target >80% code coverage
- Type checking before commits
- ESLint for code quality

---

## 7. Key Design Decisions

### 7.1 Per-Project Configuration

**Why hash-based file naming?**
- Prevents name collisions when projects at different paths have same name
- Supports path changes/project moves (update mapping)
- Secure identification without exposing full paths in filenames

**Why deep merge with special permission handling?**
- Maximum security: union of deny/allow lists ensures strictest permissions apply
- Flexibility: project can relax restrictions if global allows broader scope
- Prevents accidental permission escalation

**Why separate project configs directory?**
- Centralized storage makes backup/migration easier
- Isolated from global configs for clear separation of concerns
- Easier to implement UI filtering (global vs. project)

### 7.2 Active Context

**Why track in stores.json?**
- Single source of truth for application state
- Persists across application restarts
- Simpler than maintaining separate context file

**Why include project path in context?**
- Allows quick re-activation without re-hashing
- Enables "recent projects" functionality
- Better UX: show which project is active

### 7.3 Enterprise Managed Settings

**Why separate paths by OS?**
- Follows system conventions for managed settings
- Respects OS-level security and access control
- Aligns with enterprise deployment patterns

---

## 8. Future Enhancements

### Phase 3 Potential Features

- **Project Groups**: Organize projects into groups with shared settings
- **Configuration Profiles**: Save multiple config variations per project
- **Team Sharing**: Export/import configurations for team collaboration
- **AI-Assisted Setup**: Suggest settings based on project type/language
- **Performance Optimization**: Cache merged configs, lazy load project list
- **Mobile Support**: Mobile app for quick config viewing
- **Conflict Resolution**: Handle conflicts when local and CC Mate configs diverge
- **Audit Logging**: Track configuration changes with timestamps and diffs

---

## 9. Acceptance Criteria (Phase 1)

- [x] Project config storage implemented
- [x] Path canonicalization and hashing working
- [x] Deep merge logic with permission array union
- [x] All 17 Tauri commands implemented and tested
- [x] Active context persistence in stores.json
- [x] Auto-import from `.claude/settings.json`
- [x] Enterprise managed settings detection
- [x] Configuration activation applies merged settings

---

## 10. Success Metrics

- Backend commands tested and working correctly
- No breaking changes to existing global config functionality
- Project config storage properly isolated from global configs
- Enterprise settings correctly detected and handled
- Path canonicalization prevents duplicate configurations

---

## Appendix A: File Paths Reference

### macOS
```
~/.ccconfig/                          # App config directory
~/.ccconfig/stores.json               # Global configs & context
~/.ccconfig/project-configs/          # Project configurations
~/.ccconfig/claude_backup/            # Initial backup
~/.claude/settings.json               # Claude Code global settings
~/.claude/CLAUDE.md                   # Global memory
~/.claude/commands/                   # Custom commands
~/.claude/agents/                     # Agent definitions
/Library/Application Support/ClaudeCode/managed-settings.json  # Enterprise
```

### Linux
```
~/.ccconfig/                          # App config directory
~/.ccconfig/stores.json               # Global configs & context
~/.ccconfig/project-configs/          # Project configurations
~/.ccconfig/claude_backup/            # Initial backup
~/.claude/settings.json               # Claude Code global settings
~/.claude/CLAUDE.md                   # Global memory
~/.claude/commands/                   # Custom commands
~/.claude/agents/                     # Agent definitions
/etc/claude-code/managed-settings.json  # Enterprise
```

### Windows
```
%USERPROFILE%\.ccconfig\                              # App config directory
%USERPROFILE%\.ccconfig\stores.json                   # Global configs & context
%USERPROFILE%\.ccconfig\project-configs\              # Project configurations
%USERPROFILE%\.ccconfig\claude_backup\                # Initial backup
%USERPROFILE%\.claude\settings.json                   # Claude Code global settings
%USERPROFILE%\.claude\CLAUDE.md                       # Global memory
%USERPROFILE%\.claude\commands\                       # Custom commands
%USERPROFILE%\.claude\agents\                         # Agent definitions
C:\ProgramData\ClaudeCode\managed-settings.json       # Enterprise
```

---

## Appendix B: API Command Reference

See `system-architecture.md` for detailed API documentation of all Tauri commands.
