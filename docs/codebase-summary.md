# CC Mate - Codebase Summary

**Generated**: 2025-12-06
**Version**: 1.0
**Status**: Phase 1 Backend Foundation Complete

---

## 1. Project Overview

CC Mate is a modern desktop application built with **Tauri 2**, **React 19**, and **Rust** for managing Claude Code configuration files. It allows developers to create, organize, and switch between multiple configurations across different scopes (global, enterprise, and per-project).

**Key Metrics**:
- Total Files: 189
- Total Code: 318,303 tokens (~1.2M characters)
- Largest File: src-tauri/src/commands.rs (2,922 lines, 23,597 tokens)
- Languages: TypeScript, Rust, JSON, Markdown

---

## 2. Repository Structure

### Root Level Files

```
ccmate/
├── README.md                    # Project README with features & setup
├── CONTRIBUTING.md              # Contributing guidelines
├── package.json                 # Frontend dependencies (pnpm)
├── pnpm-lock.yaml               # Dependency lock file
├── tsconfig.json                # TypeScript configuration
├── vite.config.ts               # Vite build configuration
├── tailwind.config.ts           # Tailwind CSS configuration
├── index.html                   # HTML entry point
├── repomix-output.xml           # Full codebase repomix dump (generated)
└── CLAUDE.md                    # Project instructions for Claude Code
```

### Frontend Directory (src/)

```
src/
├── main.tsx                     # React entry point, React Query setup
├── App.tsx                      # Main router and layout
├── assets/
│   └── react.svg
├── components/
│   ├── ui/                      # shadcn/ui components (auto-generated)
│   │   ├── button.tsx
│   │   ├── card.tsx
│   │   ├── dialog.tsx
│   │   ├── input.tsx
│   │   ├── textarea.tsx
│   │   ├── tabs.tsx
│   │   ├── select.tsx
│   │   ├── drawer.tsx
│   │   ├── dropdown-menu.tsx
│   │   ├── alert.tsx
│   │   ├── separator.tsx
│   │   └── ... (20+ UI components)
│   ├── ActivityGrid.tsx         # Activity/analytics grid
│   ├── ErrorBoundary.tsx        # Error boundary wrapper
│   ├── Layout.tsx               # App layout with sidebar/header
│   ├── RouteWrapper.tsx         # Route wrapper with auth
│   ├── QueryErrorFallback.tsx   # React Query error fallback
│   ├── GLMDialog.tsx            # GLM AI dialog
│   ├── KimiDialog.tsx           # Kimi AI dialog
│   ├── MiniMaxDialog.tsx        # MiniMax AI dialog
│   ├── GLMBanner.tsx            # GLM banner notification
│   └── theme-provider.tsx       # Theme context provider
├── lib/
│   ├── query.ts                 # React Query hooks & Tauri API
│   └── utils.ts                 # Utility functions
└── pages/
    ├── ConfigEditorPage.tsx     # Config editor UI
    ├── GlobalConfigsPage.tsx    # Global configs list
    ├── ProjectConfigsPage.tsx   # Project configs list
    ├── NotFoundPage.tsx         # 404 page
    ├── SettingsPage.tsx         # Settings page
    └── ... (other pages)
```

**Frontend Stats**:
- Framework: React 19 with TypeScript
- State Management: React Query (@tanstack/react-query)
- Form Handling: React Hook Form + Zod
- Styling: Tailwind CSS v4
- UI Library: shadcn/ui
- Build Tool: Vite
- Total Files: ~50 TypeScript/TSX files

### Backend Directory (src-tauri/)

```
src-tauri/
├── src/
│   ├── main.rs                  # Tauri app entry point
│   ├── lib.rs                   # App initialization & handler setup
│   ├── commands.rs              # All Tauri commands (2,922 lines)
│   ├── hook_server.rs           # HTTP hook server implementation
│   └── tray.rs                  # System tray menu
├── Cargo.toml                   # Rust dependencies
├── tauri.conf.json              # Tauri configuration
├── build.rs                     # Build script
├── icons/                       # App icons (multiple formats)
│   ├── icon.png
│   ├── icon.icns
│   └── ... (icon assets)
└── capabilities/                # Tauri security scopes
    ├── desktop.json
    └── mobile.json
```

**Backend Stats**:
- Language: Rust
- Framework: Tauri v2
- Async Runtime: Tokio
- Main Module: commands.rs (2,922 lines)
- Total Lines: ~3,500 lines Rust code

### Documentation (docs/)

```
docs/
├── project-overview-pdr.md      # Project overview & PDR
├── system-architecture.md       # Detailed system architecture
├── code-standards.md            # Code standards & guidelines
└── codebase-summary.md          # This file
```

### Configuration & Planning

```
.claude/
├── settings.local.json          # Local Claude Code settings
├── commands/                    # Custom slash commands
│   ├── brainstorm.md
│   ├── code.md
│   ├── publish.md
│   └── release.md
├── agents/                      # Agent definitions
│   ├── brainstormer.md
│   ├── code-reviewer.md
│   ├── database-admin.md
│   ├── debugger.md
│   ├── docs-manager.md
│   ├── fullstack-developer.md
│   ├── git-manager.md
│   ├── planner.md
│   ├── project-manager.md
│   ├── researcher.md
│   ├── tester.md
│   ├── ui-ux-designer.md
│   └── ... (more agents)
├── workflows/                   # Development workflows
│   ├── primary-workflow.md
│   ├── development-rules.md
│   ├── documentation-management.md
│   └── orchestration-protocol.md
└── skills/                      # Skills configuration
    ├── claude-code/
    ├── ui-ux-pro-max/
    └── translate/

plans/
├── 251205-per-project-config.md  # Phase 1 feature plan
└── reports/                      # Analysis reports
    ├── researcher-251205-claude-code-config-system.md
    ├── tester-251205-phase1-backend.md
    ├── code-reviewer-251206-phase1-backend-review.md
    ├── FINDINGS_SUMMARY.md
    └── IMPLEMENTATION_ADJUSTMENTS.md

changelog/
├── 0.2.3.md
├── 0.2.4.md
├── 0.2.5.md
├── ... (version history)
└── 0.3.4.md
```

---

## 3. Core Components

### Frontend Components

**Layout Components**:
- `Layout.tsx` - Main app layout with sidebar/header
- `RouteWrapper.tsx` - Route protection and setup
- `ErrorBoundary.tsx` - Error boundary for error handling
- `QueryErrorFallback.tsx` - React Query error UI

**Pages**:
- `ConfigEditorPage.tsx` - Configuration editor (~26K chars, 7,434 tokens)
- `GlobalConfigsPage.tsx` - Global configurations list
- `ProjectConfigsPage.tsx` - Project configurations list
- `SettingsPage.tsx` - Application settings
- `NotFoundPage.tsx` - 404 page

**Dialog Components**:
- `GLMDialog.tsx` - GLM AI integration
- `KimiDialog.tsx` - Kimi AI integration
- `MiniMaxDialog.tsx` - MiniMax AI integration
- `GLMBanner.tsx` - Notification banner

**UI Components** (shadcn/ui):
- Button, Card, Dialog, Input, Textarea, Tabs, Select
- Dropdown Menu, Alert, Separator, Drawer
- Theme Toggle, Accordion, Popover, Scroll Area
- Command Palette, Skeleton, Sonner (toast)
- Alert Dialog, Button Group, Native Select, Toggle, Tooltip

**React Query Hooks** (lib/query.ts):
- `useGetStores()` - Fetch global configurations
- `useGetProjectConfigs()` - Fetch project configurations
- `useCreateConfig()` - Create global config mutation
- `useCreateProjectConfig()` - Create project config mutation
- `useUpdateProjectConfig()` - Update project config mutation
- `useDeleteProjectConfig()` - Delete project config mutation
- `useActivateProjectConfig()` - Switch to project context
- `useGetActiveContext()` - Get current active context
- `useGetManagedSettings()` - Get enterprise settings
- ... (30+ React Query hooks total)

### Backend Commands (commands.rs - 2,922 lines)

**Configuration Management** (900 lines):
- `get_stores()` - List all global configs
- `get_store(id)` - Get specific config
- `create_config(title, settings)` - Create config
- `update_config(id, title, settings)` - Update config
- `delete_config(id)` - Delete config
- `set_using_config(id)` - Mark as active
- `reset_to_original_config()` - Restore from backup

**Project Configuration** (800 lines):
- `get_project_configs()` - List all project configs
- `get_project_config(path)` - Get project config
- `create_project_config(...)` - Create project config
- `update_project_config(...)` - Update project config
- `delete_project_config(path)` - Delete project config
- `activate_project_config(path)` - Switch context
- `get_active_context()` - Get current context
- `switch_to_global_context(id)` - Switch to global
- `auto_create_project_config(path)` - Auto-create from global
- `get_active_merged_config()` - Get merged config
- `check_project_local_settings(path)` - Check local settings
- `import_project_local_settings(path)` - Import from local
- `update_project_config_path(old, new)` - Update path
- `validate_project_path(path)` - Validate path
- `get_managed_settings()` - Get enterprise settings
- `get_managed_mcp_servers()` - Get enterprise MCP

**File Operations** (600 lines):
- `read_config_file(type)` - Read config by type
- `write_config_file(type, content)` - Write config
- `list_config_files()` - List available configs
- `read_claude_config_file()` - Read .claude.json
- `write_claude_config_file(content)` - Write .claude.json

**MCP Server Management** (300 lines):
- `get_global_mcp_servers()` - Get MCP servers
- `update_global_mcp_server(...)` - Create/update MCP
- `delete_global_mcp_server(id)` - Delete MCP
- `check_mcp_server_exists(id)` - Check MCP exists

**Project Management** (200 lines):
- `read_claude_projects()` - Read projects from .claude.json
- `read_project_usage_files()` - Read usage analytics

**Command Management** (150 lines):
- `read_claude_commands()` - List custom commands
- `write_claude_command(name, content)` - Create/update command
- `delete_claude_command(name)` - Delete command

**Agent Management** (150 lines):
- `read_claude_agents()` - List agents
- `write_claude_agent(name, content)` - Create/update agent
- `delete_claude_agent(name)` - Delete agent

**Hook Management** (300 lines):
- `add_claude_code_hook()` - Add hook
- `update_claude_code_hook()` - Update hook
- `remove_claude_code_hook()` - Remove hook
- `get_latest_hook_command()` - Get hook command

**Notifications & Settings** (100 lines):
- `get_notification_settings()` - Get notification prefs
- `update_notification_settings(...)` - Update settings

**Analytics** (100 lines):
- `track(event, properties)` - Send event to PostHog
- `get_or_create_distinct_id()` - Get/create user ID

**Utility** (200 lines):
- `initialize_app_config()` - Initialize app on startup
- `open_config_path()` - Open config directory
- `check_app_config_exists()` - Check if app config exists
- `create_app_config_dir()` - Create app directory
- `backup_claude_configs()` - Backup on first run

**Helper Functions**:
- `canonicalize_project_path(path)` - Resolve symlinks
- `hash_project_path(path)` - SHA256 hash (first 16 chars)
- `merge_settings(global, project)` - Deep merge with special permission handling
- `get_project_configs_dir()` - Get configs directory path
- `read_project_config_file(path)` - Load project config
- `write_project_config_file(config)` - Save project config
- `read_active_context()` - Load active context
- `write_active_context(context)` - Save active context
- `check_project_local_settings_file(path)` - Check local settings
- `get_managed_settings_paths()` - Get enterprise settings paths
- `get_managed_mcp_paths()` - Get enterprise MCP paths

### Data Structures

**Frontend Types** (inferred from commands):
- `ConfigStore` - Global configuration
- `ProjectConfigStore` - Project-specific configuration
- `ActiveContext` - Current active context
- `EnhancedStoresData` - Stores with context
- `ConfigFile` - File representation
- `ClaudeConfigFile` - .claude.json file
- `ProjectConfig` - Project config entry
- `McpServer` - MCP server config
- `NotificationSettings` - Notification preferences
- `CommandFile` - Command file representation
- `AgentFile` - Agent file representation

---

## 4. Top Files by Size & Complexity

### Top 5 Files by Token Count

1. **src-tauri/src/commands.rs** (23,597 tokens, 2,922 lines)
   - All Tauri command implementations
   - Helper functions for config management
   - Per-project config logic

2. **styles.csv** (9,925 tokens, 40,262 chars)
   - UI/UX style guide data (50 styles)
   - Part of ui-ux-pro-max skill

3. **plans/251205-per-project-config.md** (9,471 tokens, 38,020 chars)
   - Phase 1 feature plan and specification
   - Data architecture, API design

4. **typography.csv** (9,115 tokens, 31,878 chars)
   - UI/UX typography data (50+ font pairings)
   - Part of ui-ux-pro-max skill

5. **src/pages/ConfigEditorPage.tsx** (7,434 tokens, 26,824 chars)
   - Main configuration editor UI
   - Settings display and modification

### Largest Directories

- `src/` - Frontend code (~150K chars)
- `src-tauri/src/` - Backend code (~120K chars)
- `.claude/skills/ui-ux-pro-max/` - Design data (~100K chars)
- `plans/` - Planning and analysis documents (~100K chars)

---

## 5. Key Features Implemented

### Phase 1: Backend Foundation (Complete)

**Per-Project Configuration**:
- Storage location: `~/.ccconfig/project-configs/{hash}.json`
- Path canonicalization and SHA256 hashing
- 17 new Tauri commands for project config management
- Auto-import from local `.claude/settings.json`

**Configuration Merging**:
- Deep merge of global and project settings
- Special union logic for permission arrays
- Project settings override global settings
- Inheritance tracking

**Context Management**:
- Track active context (global vs. project)
- Persist in stores.json
- Switch contexts with automatic config application

**Enterprise Managed Settings**:
- Detect managed settings from OS-specific paths
- Read-only access
- Display in UI

### Existing Features (Pre-Phase 1)

**Global Configuration Management**:
- Create, read, update, delete global configs
- Mark configs as active
- Auto-backup on first run

**MCP Server Management**:
- Add, update, delete MCP servers
- Support for enterprise managed MCP

**Command Management**:
- Read/write custom slash commands
- Store in `.claude/commands/`

**Agent Management**:
- Read/write agent definitions
- Store in `.claude/agents/`

**CLAUDE.md Integration**:
- Read/write global memory file
- Store in `~/.claude/CLAUDE.md`

**Hook Management**:
- Add/update/remove Claude Code hooks
- Hook server for webhook handling
- Support for Notification, Stop, PreToolUse events

**Analytics**:
- PostHog integration for event tracking
- Usage analytics dashboard
- Distinct ID generation

**Notifications**:
- Notification settings management
- Enable/disable notification hooks

---

## 6. Technology Stack

### Frontend Dependencies

```
react@19                    # UI framework
react-router-dom@^6         # Routing
@tanstack/react-query@^5   # Server state management
react-hook-form@^7         # Form state management
zod@^3                      # Schema validation
tailwindcss@^4             # Utility CSS framework
@tauri-apps/api@^2         # Tauri bindings
shadcn-ui                  # Component library (30+ components)
sonner                     # Toast notifications
clsx                       # Conditional classnames
class-variance-authority   # Component variants
```

### Backend Dependencies

```
tauri@2                    # Desktop framework
tauri-plugin-store@2       # Persistent storage
tauri-plugin-opener@2      # File/URL opener
tauri-plugin-fs@2          # File system
tauri-plugin-dialog@2      # File dialogs
tauri-plugin-notification@2 # Notifications
tauri-plugin-os@2          # OS information
tauri-plugin-updater@2     # Auto-updates

serde, serde_json@1        # Serialization
sha2@0.10                  # SHA256 hashing
dirs@5                     # Home directory
chrono@0.4                 # Date/time
nanoid@0.4                 # Nano ID generation
uuid@1.0                   # UUID generation
tokio@1                    # Async runtime
reqwest@0.11               # HTTP client
axum@0.7                   # Web framework
tower@0.4                  # Middleware
tower-http@0.5             # HTTP middleware (CORS)
```

### Build Tools

```
vite@^5                    # Frontend bundler
typescript@^5              # Type checking
vitest@^1                  # Unit testing
@playwright/test@^1        # E2E testing
tailwindcss@^4             # CSS framework
```

---

## 7. Development Workflow

### Setup

```bash
# Install dependencies
pnpm install

# Start development server
pnpm tauri dev

# Check TypeScript
pnpm tsc --noEmit

# Run tests
pnpm test
```

### Build

```bash
# Build for production
pnpm build

# Output: dist/ (frontend) + src-tauri/target/ (backend)
```

### Release

```bash
# Automatic via GitHub Actions
# Publishes to Homebrew tap (djyde/brew)
# Available at https://randynamic.org/ccmate
```

---

## 8. Code Metrics

### Frontend

- **Total Files**: ~50 TypeScript/TSX files
- **Total Lines**: ~3,000 lines
- **Main Page**: ConfigEditorPage.tsx (~800 lines)
- **React Components**: 30+ UI components
- **React Query Hooks**: 30+ custom hooks
- **Utility Functions**: 15+ utility functions

### Backend

- **Total Files**: 4 Rust files
- **Total Lines**: ~3,500 lines
- **Largest File**: commands.rs (2,922 lines)
- **Total Commands**: 50+ Tauri commands
- **Data Structures**: 12+ main types

### Overall

- **Total Files**: 189 files
- **Total Tokens**: 318,303 tokens
- **Total Characters**: 1,254,083 characters
- **Languages**: TypeScript, Rust, JSON, Markdown, YAML, CSV

---

## 9. Security & Performance

### Security Features

- Path canonicalization to prevent traversal attacks
- SHA256 hashing for secure project identification
- File permission-based access control
- Enterprise managed settings as read-only
- JSON validation before saving
- No secrets in code or logs

### Performance Optimizations

- React Query caching for all API calls
- Lazy-loaded components with React.lazy
- Vite for fast HMR during development
- Async file I/O with non-blocking operations
- Efficient deep merge algorithm
- Indexed queries for rapid lookups

---

## 10. Current Status & Next Steps

### Phase 1: Complete

- [x] Backend data structures and commands
- [x] Path canonicalization and hashing
- [x] Configuration merging logic
- [x] Active context persistence
- [x] Auto-import from local settings
- [x] Enterprise managed settings detection

### Phase 2: In Progress

- [ ] Frontend UI for project configs
- [ ] Project config editor
- [ ] Context switching UI
- [ ] Project list with sorting/filtering
- [ ] Config inheritance visualization

### Phase 3: Planned

- [ ] Configuration profiles (multiple per project)
- [ ] Project groups with shared settings
- [ ] Team configuration sharing
- [ ] AI-assisted config setup
- [ ] Conflict resolution UI
- [ ] Configuration audit logging

---

## 11. Documentation Files

**User Documentation**:
- `README.md` - Project overview and quick start
- `CONTRIBUTING.md` - Contributing guidelines

**Developer Documentation**:
- `docs/project-overview-pdr.md` - Project requirements and overview
- `docs/system-architecture.md` - Detailed technical architecture
- `docs/code-standards.md` - Code style and organization guidelines
- `docs/codebase-summary.md` - This file

**Internal Planning**:
- `plans/251205-per-project-config.md` - Phase 1 specification
- `plans/reports/` - Implementation analysis and findings

**Changelog**:
- `changelog/` - Version history (v0.2.3 through v0.3.4)

---

## 12. Contributing & Maintenance

### Code Review Process

1. Create feature branch
2. Implement changes following code standards
3. Run tests and type checking
4. Create PR with clear description
5. Code review by team
6. Merge after approval

### Maintenance Schedule

- Weekly: Security updates, dependency updates
- Monthly: Performance analysis, documentation review
- Quarterly: Major feature releases
- As needed: Bug fixes, patch releases

### Contact

- Issues: https://github.com/djyde/ccconfig/issues
- Discussions: https://github.com/djyde/ccconfig/discussions
- Contributing: See CONTRIBUTING.md

---

## Appendix A: File Organization Reference

### Source Files Count

```
Frontend:
  ├── pages/ - 10 page components
  ├── components/ - 35+ UI components
  ├── lib/ - 2 modules (query.ts, utils.ts)
  ├── assets/ - 1 SVG
  └── main.tsx + App.tsx - Entry points

Backend:
  ├── commands.rs - Main implementation (2,922 lines)
  ├── lib.rs - Setup (258 lines)
  ├── main.rs - Entry point
  ├── hook_server.rs - Hook server
  ├── tray.rs - System tray
  └── Cargo.toml - Dependencies

Documentation:
  ├── docs/ - 4 markdown files (this summary set)
  ├── plans/ - 1 feature plan + reports
  ├── changelog/ - Version history
  └── specs/ - Technical specifications
```

### Configuration Files

```
TypeScript/Build:
  ├── tsconfig.json
  ├── vite.config.ts
  ├── tailwind.config.ts
  └── package.json

Rust/Tauri:
  ├── Cargo.toml
  ├── tauri.conf.json
  ├── build.rs
  └── capabilities/

Git/CI:
  ├── .gitignore
  ├── .github/workflows/ - GitHub Actions
  └── .claude/ - Claude Code config
```

---

**Generated by Repomix v1.9.2**
**Full output available in: repomix-output.xml**
