# Claude Code Configuration System - Key Findings Summary

**Critical Update for CC Mate Per-Project Configuration Plan**

---

## 🚨 Critical Finding: MCP Servers Are NOT in settings.json

The plan at `/Users/huutri/code/ccmate/plans/251205-per-project-config.md` **must be updated** regarding MCP server handling.

### Current Plan Assumption (INCORRECT)
```json
// Plan assumes this works:
{
  "settings": {
    "model": "claude-opus-4-5-20251101",
    "permissions": {...},
    "mcpServers": { ... }  // ❌ THIS IS WRONG
  }
}
```

### Reality (CORRECT)
MCP servers are **NOT** configured in `settings.json` at all. They have dedicated storage:

| Scope | Location | Type | Control |
|-------|----------|------|---------|
| **User** | `~/.claude.json` → `mcpServers` | User-scoped | CLI: `claude mcp add` |
| **Project** | `.mcp.json` (project root) | Team-shared | Version control |
| **Enterprise** | `managed-mcp.json` | Admin-managed | System admin |

### Why Separation?
- MCP servers are integrations, not configuration options
- Different scope requirements
- Different management methods (CLI vs files)
- Different precedence logic

### Recommendation for CC Mate
**Phase 1**: Skip MCP per-project configuration
- Focus on settings.json only
- Add note: "MCP servers managed separately via `.mcp.json`"

**Phase 2**: Add separate MCP project management
- Create `.mcp.json` editor
- Manage project MCP servers independently

---

## ✅ Confirmed: Live-Reload Works (v1.0.90+)

Settings changes take effect **immediately** without restart.

### Verified Behavior
- ✅ Model changes: Apply immediately
- ✅ Permissions: Apply immediately
- ✅ Env vars: Likely apply immediately (needs test)
- ✅ Status line: Apply immediately
- ⚠️ Hooks: May NOT reload (security exception?)
- ⚠️ Env vars: May require next session (unclear)

### Implication for CC Mate
**UI/UX Consideration**: Most changes are live, but:
- Consider adding "Restart Claude Code" option in UI
- Document which settings need restart
- Test env var behavior specifically

---

## ✅ Confirmed: Per-Project Settings Are Fully Supported

Claude Code natively supports:
- `.claude/settings.json` - Team-shared, version-controlled
- `.claude/settings.local.json` - Personal, git-ignored
- Automatic git-ignore configuration
- Deep merging with project override

### No Custom Implementation Needed For
- Loading project settings (Claude Code does this automatically)
- Merging with user settings (Claude Code handles)
- Git-ignore setup (Claude Code auto-creates)

### CC Mate Still Needs To Handle
- ✅ UI for editing project settings
- ✅ Display active context (global vs project)
- ✅ Switch between contexts
- ✅ Auto-import from existing `.claude/settings.json`

---

## Settings Precedence (Verified)

From highest to lowest priority:

```
1. Enterprise managed-settings.json (immutable)
2. Command-line arguments
3. .claude/settings.local.json (project local)
4. .claude/settings.json (project shared)
5. ~/.claude/settings.json (user)
```

### Key Points
- Enterprise settings **cannot be overridden**
- Project local > project shared > user
- Arrays (permissions) are **merged**, not replaced
- Objects are **deep-merged** with later values winning

---

## All Available Settings Fields

**Fully overridable per-project**:
- `env` - Environment variables
- `permissions` - Tool access rules (allow/deny/ask)
- `hooks` - Pre/post-tool commands
- `model` - LLM selection
- `statusLine` - Custom status line
- `outputStyle` - Prompt style
- `enableAllProjectMcpServers` / `enabledMcpjsonServers` - MCP control
- `awsAuthRefresh` / `awsCredentialExport` - AWS config
- `apiKeyHelper` - Auth script
- `cleanupPeriodDays` - Session cleanup
- `includeCoAuthoredBy` - Git byline

**Not overridable** (enterprise/user only):
- `forceLoginMethod` / `forceLoginOrgUUID`
- `useEnterpriseMcpConfigOnly`
- `companyAnnouncements`
- `sandbox` settings

---

## Critical Bugs & Limitations Found

### Docker/Container Issue (#7624)
**Problem**: inotify ENOSPC error can crash on startup in containers with low limits
**Solution**: Support `CLAUDE_DISABLE_CONFIG_WATCH=1` or `--no-config-watch` flags
**Impact on CC Mate**: Document for enterprise deployments

### WSL Performance Issue (#2906)
**Problem**: ~1000 file reads/sec after tool execution, causes slowdown
**Status**: Unresolved in Claude Code
**Impact on CC Mate**: Acknowledge in docs, may affect WSL users

### Hooks Live-Reload Exception
**Problem**: Hooks may not reload immediately (unconfirmed)
**Status**: Unclear if intentional (security) or bug
**Impact on CC Mate**: Document, test, and possibly prompt for restart

---

## Unresolved Questions (Need Testing)

1. **Do env var changes apply immediately to session?**
   - Likely needs restart or affects new sessions only

2. **Do hooks changes require restart?**
   - Unconfirmed if intentional for security

3. **How often are settings re-read after load?**
   - On-change via file watcher (probable)
   - But WSL bug suggests polling fallback

4. **MCP server detection when .mcp.json added?**
   - Does Claude Code detect immediately?
   - Or need `/mcp` command or restart?

5. **How does Claude Code handle invalid JSON in settings?**
   - Graceful degradation? Error? Skip file?

6. **Concurrent modification handling?**
   - If external tool edits settings.json simultaneously

---

## Plan Update Recommendations

### For Phase 1 (Current)
**CHANGE**: Remove mcpServers from project config settings

**From**:
```rust
pub struct ProjectConfigStore {
  pub settings: Value,  // Currently includes mcpServers
}
```

**To**:
```rust
pub struct ProjectConfigStore {
  pub settings: Value,  // settings.json only (no MCP)
  // MCP servers handled separately via .mcp.json
}
```

### For Phase 2 (Future)
**ADD**: Separate MCP project management
- Read/write `.mcp.json`
- Support environment variable expansion
- Merge with user MCP servers for display

### For UI
**ADD**: Warning/Note in project config form
```
"MCP Servers are configured separately in .mcp.json files.
See MCP management section for project-specific servers."
```

### For Documentation
**ADD**: Settings field reference
- List all supported fields
- Mark which can be overridden per-project
- Note enterprise restrictions
- Include examples

---

## Testing Checklist for Unresolved Questions

- [ ] Test env var changes in settings.json → Verify if applied immediately
- [ ] Test hooks changes in settings.json → Check if live-reload works
- [ ] Test MCP server addition to .mcp.json → Verify detection timing
- [ ] Test invalid JSON in settings.json → Verify error handling
- [ ] Test concurrent modifications → File lock handling
- [ ] Test with managed-settings.json → Verify immutability
- [ ] Test WSL file watcher → Measure performance
- [ ] Test Docker with low inotify → Confirm ENOSPC workaround

---

## File Locations Reference

```
User/Global:
  ~/.claude/settings.json          ← User settings
  ~/.claude.json                   ← Preferences, MCP servers, projects
  ~/.claude/CLAUDE.md              ← User memory file
  ~/.claude/agents/                ← User subagents
  ~/.claude/commands/              ← User slash commands

Project:
  .claude/settings.json            ← Shared project settings
  .claude/settings.local.json      ← Personal project settings
  .mcp.json                        ← Team-shared MCP servers
  .claude/CLAUDE.md                ← Project memory file
  .claude/agents/                  ← Project subagents
  .claude/commands/                ← Project commands

Enterprise:
  /etc/claude-code/managed-settings.json       (Linux/WSL)
  /Library/Application Support/ClaudeCode/managed-settings.json (macOS)
  C:\ProgramData\ClaudeCode\managed-settings.json (Windows)

  /etc/claude-code/managed-mcp.json            (Linux/WSL)
  /Library/Application Support/ClaudeCode/managed-mcp.json (macOS)
  C:\ProgramData\ClaudeCode\managed-mcp.json  (Windows)
```

---

## Recommended CC Mate Focus Areas

**High Priority**:
1. ✅ Project settings (`.claude/settings.json`) editor
2. ✅ Context switching (global ↔ project)
3. ✅ Auto-import from existing project configs
4. ✅ Settings validation & field documentation
5. ✅ Enterprise managed settings detection (read-only)

**Medium Priority**:
1. ⚠️ Live-reload testing & documentation
2. ⚠️ Hooks behavior documentation
3. ⚠️ Settings field help/examples

**Low Priority / Phase 2**:
1. 🔄 Project MCP server management (`.mcp.json`)
2. 🔄 Config history/versioning
3. 🔄 Import/export configs

---

**Report Date**: December 5, 2025
**Sources**: Official Claude Code documentation, GitHub issues, community resources
**Status**: Ready for implementation plan updates
