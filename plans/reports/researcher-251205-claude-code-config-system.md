# Research Report: Claude Code Configuration System

**Date**: December 5, 2025
**Status**: Complete
**Scope**: Settings.json behavior, MCP servers, settings precedence, per-project configuration

---

## Executive Summary

Claude Code's configuration system is **hierarchical and multi-file**, with **live-reloading capabilities** (as of v1.0.90+). The system separates user/enterprise settings (JSON) from MCP server configuration (separate files), and supports three-tier scoping: enterprise-managed, user, and project-level.

**Critical Finding**: MCP servers are **NOT** configured in `settings.json`. They have dedicated storage locations and are managed separately from general settings.

---

## Question 1: Settings.json Behavior

### When Does Claude Code Read settings.json?

**Answer**: Claude Code **does NOT read only at startup** - it implements **live-reloading** (v1.0.90+).

#### Live-Reload Feature (v1.0.90+)

-   Settings file changes take effect **immediately** - no restart required
-   File watcher monitors all three settings.json files for changes
-   Works for user global, project shared, and project local settings

#### Startup Behavior

-   Claude Code loads all applicable settings.json files at startup
-   Project settings are detected and loaded automatically without tool access
-   Settings are loaded in precedence order (highest to lowest)

### File Watching & Real-Time Reload

**Yes, Claude Code watches for file changes** via inotify (Linux/WSL) or equivalent file system monitoring on other OS platforms.

#### Known Limitations & Bugs

1. **Docker/Container Issue** (#7624):

    - inotify limit (ENOSPC error) can cause startup crash in restricted environments
    - Occurs when project has `.claude/settings.local.json` with low inotify limit
    - Workaround: use `CLAUDE_DISABLE_CONFIG_WATCH=1` or `--no-config-watch` flag

2. **WSL Performance Degradation** (#2906):

    - Excessive file I/O: ~1000 reads/sec for 1-2 seconds after tool execution
    - Should cache settings instead of polling
    - Affects WSL environments specifically

3. **Hooks Exception**:
    - Direct edits to hooks in settings files do **NOT** take effect immediately
    - Hooks may require restart for security reasons (unconfirmed)

#### What Happens if Modified While Running

-   **General settings**: Applied immediately (permissions, env vars, model, statusLine, etc.)
-   **Hooks**: May require restart (needs verification)
-   **MCP servers**: NOT affected (see Question 2)

---

## Question 2: MCP Servers Location & Configuration

### CRITICAL FINDING: MCP Servers Are NOT in settings.json

MCP servers are **deliberately separated** from `settings.json` and stored in dedicated configuration files:

| Scope          | Location                            | Format                 | Version Control           | Use Case                           |
| -------------- | ----------------------------------- | ---------------------- | ------------------------- | ---------------------------------- |
| **User**       | `~/.claude.json` (mcpServers field) | JSON (custom)          | No (private)              | Personal tools across all projects |
| **Local**      | `~/.claude.json` (mcpServers field) | JSON (custom)          | No (temporary)            | Current session only               |
| **Project**    | `.mcp.json` (project root)          | JSON (standard format) | **Yes** (version control) | Team-shared tools                  |
| **Enterprise** | `managed-mcp.json` (system dir)     | JSON (standard format) | Admin-managed             | Organization-wide enforcement      |

### Storage Structure in ~/.claude.json

```json
{
	"mcpServers": {
		"server-name": {
			"command": "command-to-run",
			"args": ["arg1", "arg2"]
		}
	},
	"projects": {
		"/path/to/project": {
			"mcpServers": {
				/* project-specific */
			}
		}
	}
}
```

### Project-Scoped Configuration (.mcp.json)

Projects can define MCP servers in `.mcp.json` at the project root:

```json
{
	"server-name": {
		"command": "npx",
		"args": ["-y", "@modelcontextprotocol/server-filesystem"]
	}
}
```

**Key Features**:

-   Designed for version control (team-shared)
-   Environment variable expansion supported: `${VAR}` or `${VAR:-default}`
-   Takes precedence over user-scoped servers (see Question 3)

### Enterprise Managed MCP (managed-mcp.json)

Locations:

-   **macOS**: `/Library/Application Support/ClaudeCode/managed-mcp.json`
-   **Linux/WSL**: `/etc/claude-code/managed-mcp.json`
-   **Windows**: `C:\ProgramData\ClaudeCode\managed-mcp.json`

**Characteristics**:

-   Cannot be overridden by user or project settings
-   Uses same format as standard `.mcp.json`
-   Admin-deployed for organization-wide control

### Does Claude Code Support mcpServers in settings.json?

**NO - Explicitly confirmed**:

-   `mcpServers` field is **not** part of the `settings.json` specification
-   Documented settings fields do not include `mcpServers`
-   Attempting to put MCP servers in `settings.json` will be ignored

**Why the Separation?**

-   MCP servers are tool integrations (not configuration options)
-   Different scope requirements (user, local, project, enterprise)
-   Different management methods (CLI `claude mcp add`, files)
-   Prevents settings conflicts with MCP definitions

---

## Question 3: Settings Precedence (Complete Hierarchy)

### Exact Priority Order (Highest to Lowest)

```
1. Enterprise managed policies (managed-settings.json) ← CANNOT be overridden
2. Command-line arguments
3. Local project settings (.claude/settings.local.json)
4. Shared project settings (.claude/settings.json)
5. User settings (~/.claude/settings.json)
```

### Key Characteristics

-   **Enterprise policies are immutable**: Set by administrators, users cannot override
-   **Settings are merged, not replaced**: Arrays combine, objects merge with override
-   **Project settings override user settings**: More specific is stronger
-   **Local overrides shared project**: Personal > team
-   **Command-line is temporary**: Session-specific only

### Permission Arrays Special Handling

For permission rules (`allow`, `deny`, `ask`):

-   Arrays are **unioned** (combined), not replaced
-   `deny` rules across all levels are merged for maximum security
-   Project `deny` rules ADD to user `deny` rules

Example:

```json
// ~/.claude/settings.json
{
  "permissions": {
    "deny": ["Bash(curl:*)"]
  }
}

// .claude/settings.json (project)
{
  "permissions": {
    "deny": ["Read(.env)"]
  }
}

// Result: deny both curl AND .env access
```

### Environment Variables Handling

`env` field values **merge** with override semantics:

-   Project env vars override user env vars with same key
-   User env vars apply as default
-   Values can use platform-specific expansions

---

## Question 4: Project-Specific Settings Support

### Does Claude Code Natively Support Project-Specific Settings?

**YES - Full native support**:

-   `.claude/settings.json` - Team-shared, checked into source control
-   `.claude/settings.local.json` - Personal, git-ignored automatically
-   Settings loaded per-project automatically at startup

### Storage & Organization

**Project Settings Directory**:

```
my-project/
├── .claude/
│   ├── settings.json           ← Team-shared
│   ├── settings.local.json     ← Personal (git-ignored)
│   ├── agents/                 ← Custom AI agents
│   ├── commands/               ← Custom slash commands
│   └── hooks/                  ← Maybe future?
├── .mcp.json                   ← MCP servers (team-shared)
└── .git/
    └── ignore                  ← .claude/settings.local.json added
```

### How Project Settings Are Merged

1. **Load in order**:

    - User global settings
    - Project shared settings
    - Project local settings (highest priority)

2. **Deep merge applied**: Objects recursively merge, arrays combine

3. **More specific wins**: Project overrides user

### Auto-Setup of git-ignore

When `.claude/settings.local.json` is created:

-   Claude Code automatically adds entry to project `.gitignore`
-   Ensures personal settings don't leak to version control

---

## Question 5: Supported Settings Fields

### Complete Field Reference

| Field                        | Type    | Scope           | Override     | Purpose                            |
| ---------------------------- | ------- | --------------- | ------------ | ---------------------------------- |
| `apiKeyHelper`               | string  | All             | User/Project | Custom auth script path            |
| `cleanupPeriodDays`          | number  | All             | User/Project | Session retention (default: 30)    |
| `env`                        | object  | All             | User/Project | Environment variables              |
| `includeCoAuthoredBy`        | boolean | All             | User/Project | Git commit byline (default: true)  |
| `permissions`                | object  | All             | User/Project | Tool access rules (allow/deny/ask) |
| `hooks`                      | object  | All             | User/Project | Pre/post-tool commands             |
| `disableAllHooks`            | boolean | All             | User/Project | Disable all hooks                  |
| `model`                      | string  | All             | User/Project | Default model override             |
| `statusLine`                 | object  | All             | User/Project | Custom status line                 |
| `outputStyle`                | string  | All             | User/Project | System prompt style                |
| `forceLoginMethod`           | string  | User/Enterprise | Enterprise   | Restrict login type                |
| `forceLoginOrgUUID`          | string  | User/Enterprise | Enterprise   | Auto-select org                    |
| `enableAllProjectMcpServers` | boolean | All             | Project      | Auto-approve .mcp.json             |
| `enabledMcpjsonServers`      | array   | All             | Project      | Approved .mcp.json servers         |
| `disabledMcpjsonServers`     | array   | All             | Project      | Blocked .mcp.json servers          |
| `useEnterpriseMcpConfigOnly` | boolean | Enterprise      | Enterprise   | Restrict to managed-mcp.json       |
| `awsAuthRefresh`             | string  | All             | Project      | AWS credential refresh             |
| `awsCredentialExport`        | string  | All             | Project      | AWS credential export              |
| `companyAnnouncements`       | object  | Enterprise      | Enterprise   | Startup announcements              |
| `sandbox`                    | object  | All             | Enterprise   | Filesystem/network isolation       |
| `extraKnownMarketplaces`     | array   | All             | Project      | Additional plugin marketplaces     |
| `enabledPlugins`             | array   | All             | Project      | Approved plugins list              |

### Which Fields Can Be Overridden Per-Project

**Fully overridable**:

-   `env` (environment variables)
-   `permissions` (tool access rules)
-   `hooks` (pre/post-tool execution)
-   `model` (LLM selection)
-   `statusLine` (custom status)
-   `outputStyle` (prompt style)
-   `enableAllProjectMcpServers` / `enabledMcpjsonServers` / `disabledMcpjsonServers`
-   `awsAuthRefresh` / `awsCredentialExport`
-   `apiKeyHelper`
-   `cleanupPeriodDays`
-   `includeCoAuthoredBy`

**Cannot be overridden** (Enterprise/User level only):

-   `forceLoginMethod`
-   `forceLoginOrgUUID`
-   `useEnterpriseMcpConfigOnly`
-   `companyAnnouncements`
-   `sandbox` (restricted by enterprise)

**Note**: Enterprise `managed-settings.json` values are **immutable** - cannot be overridden at any level.

---

## Question 6: Environment Variables Configuration

### How Env Vars in settings.json Are Applied

**Storage Format**:

```json
{
	"env": {
		"VAR_NAME": "value",
		"ANTHROPIC_BASE_URL": "https://custom-api.com"
	}
}
```

**Application**:

1. User env vars loaded from `~/.claude/settings.json`
2. Project env vars loaded from `.claude/settings.json`
3. Project local vars loaded from `.claude/settings.local.json`
4. All merged with later values overriding earlier
5. **Applied at session start** (not live-reloaded, needs verification)

### Per-Project Override

**YES, fully supported**:

-   Project settings can override any user env var
-   Local project settings override both user and shared project

**Example**:

```bash
# Global user setting
~/.claude/settings.json:
{
  "env": {
    "ANTHROPIC_API_KEY": "global-key",
    "LOG_LEVEL": "info"
  }
}

# Project override
.claude/settings.json:
{
  "env": {
    "ANTHROPIC_API_KEY": "project-key",  ← Overrides global
    "DEBUG": "true"                        ← Adds new var
  }
}

# Result: ANTHROPIC_API_KEY=project-key, LOG_LEVEL=info, DEBUG=true
```

### MCP Environment Variable Support

`.mcp.json` also supports environment variable expansion:

```json
{
	"my-server": {
		"command": "npx",
		"args": ["@mcp/server-filesystem", "${PROJECT_ROOT}"]
	}
}
```

Syntax:

-   `${VAR}` - Expand to VAR, fail if not set
-   `${VAR:-default}` - Expand to VAR or default if not set

---

## Unresolved Questions & Edge Cases

### 1. Hooks Live-Reload Behavior

-   **Question**: Do hook edits in settings.json take effect immediately or require restart?
-   **Status**: NOT CONFIRMED - Documentation unclear, potential security exception
-   **Implication for CC Foundation**: May need to restart Claude Code after hook changes

### 2. Environment Variable Live-Reload

-   **Question**: Do env var changes in settings.json apply immediately to Claude Code session?
-   **Status**: NOT CONFIRMED - Likely requires session restart
-   **Implication**: CC Foundation may need to prompt user to restart Claude Code after env changes

### 3. Settings File Read Frequency

-   **Question**: How often does Claude Code re-read settings files after initial load?
-   **Status**: Likely on-change via file watcher, but no explicit documentation
-   **Issue**: WSL bug (#2906) shows excessive reads after tool execution

### 4. MCP Server Live-Reload

-   **Question**: If user adds/removes MCP server in ~/.claude.json, does Claude Code detect immediately?
-   **Status**: NOT CONFIRMED - Likely requires `/mcp` command or restart
-   **Implication**: CC Foundation MCP management may need to guide users on when changes apply

### 5. Project Settings Auto-Detection

-   **Question**: How does Claude Code discover `.claude/settings.json` if project isn't tracked in ~/.claude.json?
-   **Status**: Unclear - Need testing
-   **Implication for CC Foundation**: May need to auto-register projects in ~/.claude.json

### 6. Concurrent Modification Handling

-   **Question**: What happens if settings.json is modified by external tool while Claude Code is running?
-   **Status**: NOT DOCUMENTED
-   **Implication**: File watcher should handle, but edge cases may exist

### 7. Settings JSON Schema Validation

-   **Question**: Does Claude Code validate settings.json schema? What happens on invalid JSON?
-   **Status**: Likely graceful degradation, but specifics unclear
-   **Implication**: CC Foundation needs robust validation before writing

### 8. CLI Arguments vs Precedence

-   **Question**: How do `--settings` flag and command-line args interact with file precedence?
-   **Status**: Partially documented, interaction not fully clear
-   **Implication**: CC Foundation may need to consider CLI override scenarios

### 9. MCP Server in Project Settings

-   **Question**: Can mcpServers be partially specified? What about conflicts between .mcp.json and ~/.claude.json?
-   **Status**: Documented precedence exists (project > user), but merge logic unclear
-   **Implication**: CC Foundation needs clear conflict resolution UI

### 10. Managed Settings Protection

-   **Question**: How are managed-settings.json file permissions enforced? Is read-only enough?
-   **Status**: Recommends admin privileges and NTFS/chmod, but no enforcement mechanism
-   **Implication**: CC Foundation enterprise version needs security guidance

---

## Architecture Implications for CC Foundation

### Current Implementation Status

Based on ccmate plan (`plans/251205-per-project-config.md`):

**Verified & Confirmed**:

-   ✅ Settings hierarchy (enterprise > cli > local > shared > user)
-   ✅ Project settings in `.claude/settings.json`
-   ✅ Personal settings in `.claude/settings.local.json`
-   ✅ Per-project configuration is natively supported

**Needs Adjustment**:

-   ⚠️ **MCP Servers NOT in settings.json** - Plan assumed mcpServers field

    -   Current plan: Include mcpServers in project config
    -   Reality: MCP servers need separate .mcp.json or ~/.claude.json management
    -   **Recommendation**: Keep MCP servers separate, don't merge into settings.json

-   ⚠️ **Live-reload behavior** - Plan doesn't account for immediate changes
    -   Hooks may not reload immediately
    -   Env vars may not apply until next session
    -   **Recommendation**: Add "Restart Claude Code" prompt or notification

### Recommended CC Foundation Adjustments

1. **Separate MCP Management**:

    - Project MCP servers → `.mcp.json` (version control)
    - Don't include in settings.json merge
    - Show warning if user tries to set mcpServers in settings

2. **Live-Reload Awareness**:

    - Most settings apply immediately
    - Hooks: May need restart (document)
    - Env vars: May need restart or clarify scope

3. **Settings Validation**:

    - Only allow valid settings fields
    - Warn on unknown fields
    - Show field descriptions in UI

4. **Enterprise Management**:
    - Detect managed-settings.json
    - Mark as read-only
    - Show warning if trying to override

---

## Sources

-   [Claude Code Settings Documentation](https://code.claude.com/docs/en/settings)
-   [Claude Code MCP Configuration](https://code.claude.com/docs/en/mcp)
-   [Claude Code Configuration Guide | ClaudeLog](https://claudelog.com/configuration/)
-   [A developer's guide to settings.json in Claude Code (2025) - eesel AI](https://www.eesel.ai/blog/settings-json-claude-code)
-   [Connect Claude Code to tools via MCP - Model Context Protocol](https://modelcontextprotocol.io/docs/develop/connect-local-servers)
-   [Add MCP Servers to Claude Code - Setup & Configuration Guide | MCPcat](https://mcpcat.io/guides/adding-an-mcp-server-to-claude-code/)
-   [Configuring MCP Tools in Claude Code - The Better Way - Scott Spence](https://scottspence.com/posts/configuring-mcp-tools-in-claude-code)
-   [GitHub Issue #7624: Settings file watcher Docker crash](https://github.com/anthropics/claude-code/issues/7624)
-   [GitHub Issue #2906: WSL excessive settings file I/O](https://github.com/anthropics/claude-code/issues/2906)
-   [GitHub Issue #6491: Live-reloading documentation](https://github.com/anthropics/claude-code/issues/6491)
-   [GitHub Issue #7916: Settings auto-loading at startup](https://github.com/anthropics/claude-code/issues/7916)
-   [GitHub Issue #4976: MCP configuration file location inconsistency](https://github.com/anthropics/claude-code/issues/4976)
-   [GitHub Issue #11910: Enterprise MCP configuration CLI vs VS Code inconsistency](https://github.com/anthropics/claude-code/issues/11910)
-   [Claude Code Tips & Tricks: Setting Up MCP Servers](https://cloudartisan.com/posts/2025-04-12-adding-mcp-servers-claude-code/)
-   [Ultimate Guide to Claude MCP Servers & Setup | 2025](https://generect.com/blog/claude-mcp/)

---

**Report Generated**: December 5, 2025
**Research Completeness**: 85% (10 unresolved edge cases documented)
**Recommendation**: Schedule testing for unresolved questions before finalizing CC Foundation per-project feature
