# Implementation Adjustments Based on Configuration System Research

**For**: `/Users/huutri/code/ccmate/plans/251205-per-project-config.md`
**Date**: December 5, 2025
**Status**: Critical updates required before Phase 1

---

## Change 1: Exclude MCP Servers from Project Settings

### Current Plan (Lines 85-89)

**ISSUE**: Merge strategy assumes `mcpServers` in settings.json

```rust
// CURRENT - INCORRECT
fn merge_settings(global: &Value, project: &Value) -> Value {
  // ...
  - `mcpServers`: Merge (project servers + global servers)
  - `permissions.deny`: Union arrays (maximum security)
  - `env`: Project vars override global vars
}
```

### Corrected Plan

```rust
// CORRECTED - mcpServers handled separately
fn merge_settings(global: &Value, project: &Value) -> Value {
  // Merge all fields EXCEPT mcpServers
  // mcpServers are managed via:
  //   - ~/.claude.json (user scope)
  //   - .mcp.json (project scope)
  //   - managed-mcp.json (enterprise scope)
  // NOT in settings.json

  // Current merge logic applies to:
  // - permissions.deny: Union arrays
  // - env: Project overrides user
  // - model, hooks, statusLine, etc.: Project overrides user
}
```

### Code Changes Required

**File**: `src-tauri/src/commands.rs`

```rust
// Add documentation to merge_settings function
/// Deep merge settings with project override.
///
/// NOTE: MCP servers are NOT merged here. They are managed separately via:
/// - ~/.claude.json (user/local scopes)
/// - .mcp.json (project scope, version control)
/// - managed-mcp.json (enterprise scope)
///
/// Merge behavior:
/// - Arrays (permissions): Union for maximum security
/// - Objects: Deep merge with project overriding user
/// - Scalars: Project overrides user
fn merge_settings(global: &Value, project: &Value) -> Value {
    // Implementation stays same, but add exclusion:
    // Don't process "mcpServers" field if present
}
```

### UI Changes Required

**File**: `src/pages/ProjectConfigEditor.tsx`

Add warning banner:

```typescript
export function ProjectConfigEditor() {
	return (
		<>
			<div className="bg-blue-100 border border-blue-400 text-blue-700 px-4 py-3 rounded">
				<strong>Note:</strong> MCP servers are managed separately in <code>.mcp.json</code>. See <a href="#mcp-management">MCP Management</a>{" "}
				section.
			</div>
			{/* ... rest of form ... */}
		</>
	);
}
```

---

## Change 2: Add "Restart Claude Code" Notification for Certain Settings

### Issue

Some settings changes may not apply immediately:

-   Hooks (unconfirmed)
-   Environment variables (unconfirmed)
-   Potentially others

### Solution

**File**: `src/lib/query.ts`

Add flag to mutation:

```typescript
export const useUpdateProjectConfig = () => {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: (params: { projectPath: string; title: string; settings: unknown }) =>
			invoke<ProjectConfigStore>("update_project_config", params),
		onSuccess: (data, variables) => {
			queryClient.invalidateQueries({ queryKey: ["project-configs"] });
			queryClient.invalidateQueries({ queryKey: ["project-config", variables.projectPath] });
			queryClient.invalidateQueries({ queryKey: ["active-context"] });

			// NEW: Detect if changes need restart
			const needsRestart = detectSettingsNeedRestart(variables.settings as Record<string, unknown>);

			if (needsRestart) {
				toast.warning("Some changes require restarting Claude Code to take effect", {
					action: {
						label: "Docs",
						onClick: () => openExternalLink("https://docs.claude.com/..."),
					},
				});
			} else {
				toast.success("Project config updated");
			}
		},
	});
};

// Helper function
function detectSettingsNeedRestart(settings: Record<string, unknown>): boolean {
	const fieldsNeedingRestart = ["hooks", "env"]; // Based on research

	return fieldsNeedingRestart.some((field) => field in settings);
}
```

### Alternative: Always Show Option

**File**: `src/pages/ProjectConfigEditor.tsx`

```typescript
<Button onClick={() => shell.open("claude://")} variant="outline">
	Restart Claude Code
</Button>
```

---

## Change 3: Add Settings Field Validation

### Issue

Current plan doesn't validate which fields are supported

### Solution

**File**: `src-tauri/src/commands.rs`

```rust
// Add validation for settings.json fields
const VALID_SETTINGS_FIELDS: &[&str] = &[
    // All supported fields from official docs
    "apiKeyHelper",
    "cleanupPeriodDays",
    "env",
    "includeCoAuthoredBy",
    "permissions",
    "hooks",
    "disableAllHooks",
    "model",
    "statusLine",
    "outputStyle",
    "forceLoginMethod",
    "forceLoginOrgUUID",
    "enableAllProjectMcpServers",
    "enabledMcpjsonServers",
    "disabledMcpjsonServers",
    "useEnterpriseMcpConfigOnly",
    "awsAuthRefresh",
    "awsCredentialExport",
    "companyAnnouncements",
    "sandbox",
    "enabledPlugins",
    "extraKnownMarketplaces",
];

fn validate_settings(settings: &Value) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if let Some(obj) = settings.as_object() {
        for (key, _) in obj.iter() {
            if !VALID_SETTINGS_FIELDS.contains(&key.as_str()) {
                errors.push(format!("Unknown field: {}", key));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

### Frontend Validation

**File**: `src/lib/query.ts`

```typescript
// Zod schema for settings.json
export const settingsSchema = z
	.object({
		apiKeyHelper: z.string().optional(),
		cleanupPeriodDays: z.number().positive().optional(),
		env: z.record(z.string()).optional(),
		includeCoAuthoredBy: z.boolean().optional(),
		permissions: z
			.object({
				allow: z.array(z.string()).optional(),
				deny: z.array(z.string()).optional(),
				ask: z.array(z.string()).optional(),
			})
			.optional(),
		hooks: z.record(z.unknown()).optional(),
		disableAllHooks: z.boolean().optional(),
		model: z.string().optional(),
		statusLine: z.object({}).optional(),
		outputStyle: z.string().optional(),
		// ... all other fields
	})
	.strict(); // Strict mode: unknown fields cause error
```

---

## Change 4: Detect and Handle Managed Settings

### Issue

Doesn't check for `managed-settings.json` (enterprise)

### Solution

**File**: `src-tauri/src/commands.rs`

```rust
// Add function to detect managed settings
fn check_managed_settings_exists() -> Result<Option<Value>, String> {
    let managed_paths = get_managed_settings_paths();

    for path in managed_paths {
        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read managed settings: {}", e))?;
            let json: Value = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse managed settings: {}", e))?;
            return Ok(Some(json));
        }
    }

    Ok(None)
}

fn get_managed_settings_paths() -> Vec<PathBuf> {
    #[cfg(target_os = "macos")]
    return vec![
        PathBuf::from("/Library/Application Support/ClaudeCode/managed-settings.json")
    ];

    #[cfg(target_os = "linux")]
    return vec![
        PathBuf::from("/etc/claude-code/managed-settings.json")
    ];

    #[cfg(target_os = "windows")]
    return vec![
        PathBuf::from("C:\\ProgramData\\ClaudeCode\\managed-settings.json")
    ];
}

// Add Tauri command
#[tauri::command]
pub async fn get_managed_settings() -> Result<Option<Value>, String> {
    check_managed_settings_exists()
}
```

### Frontend Display

**File**: `src/pages/ProjectConfigEditor.tsx`

```typescript
export function ProjectConfigEditor() {
	const { data: managedSettings } = useQuery({
		queryKey: ["managed-settings"],
		queryFn: () => invoke<unknown>("get_managed_settings"),
	});

	if (managedSettings) {
		return (
			<Alert>
				<AlertCircle className="h-4 w-4" />
				<AlertTitle>Enterprise Managed Settings Active</AlertTitle>
				<AlertDescription>
					Your organization has deployed enterprise managed settings. Some fields cannot be overridden at the project level.
					<a href="...">Learn more</a>
				</AlertDescription>
			</Alert>
		);
	}

	// ... rest of editor
}
```

---

## Change 5: Auto-Import Existing Project Settings

### Current Implementation (Already in Plan)

✅ Already includes `check_project_local_settings` and `import_project_local_settings`

### Enhancement: Verify Path Canonicalization

**File**: `src-tauri/src/commands.rs`

Ensure project paths are canonicalized before comparison:

```rust
// When checking for existing configs, always canonicalize
#[tauri::command]
pub async fn get_project_config(project_path: String) -> Result<Option<ProjectConfigStore>, String> {
    let canonical = canonicalize_project_path(&project_path)?;

    // Look up by canonical path
    let configs_dir = get_project_configs_dir()?;
    let hash = hash_project_path(&canonical)?;

    let config_file = configs_dir.join(format!("{}.json", hash));
    // ... rest of logic
}
```

---

## Change 6: Update Plan Documentation

### Section 9: Claude Code Behavior (Lines 903-948)

**UPDATE** with findings:

```markdown
## 9. Claude Code Behavior - RESEARCH COMPLETED

### 9.1 Settings.json Live-Reload (VERIFIED ✅)

**Confirmed**: Claude Code v1.0.90+ implements live-reloading.

-   Settings changes take effect **immediately** without restart
-   File watcher monitors all three settings.json files
-   Exception: Hooks may not reload (unconfirmed, needs testing)

**Implications for CC Foundation**:

-   Most changes apply immediately
-   Consider "Restart Claude Code" prompt for hooks/env changes
-   Document which settings need restart

### 9.2 MCP Servers Location (VERIFIED ✅)

**Critical Finding**: MCP servers are **NOT** in `settings.json`.

Storage locations:

-   User: `~/.claude.json` (mcpServers field)
-   Project: `.mcp.json` (version control)
-   Enterprise: `managed-mcp.json` (system directory)

**Implications for CC Foundation**:

-   Phase 1: Don't include mcpServers in project settings
-   Phase 2: Separate MCP management for `.mcp.json`
-   Add warning in UI about MCP server separation

### 9.3 Settings Precedence (VERIFIED ✅)

**Confirmed**: Exact precedence order:

1. Enterprise managed-settings.json (immutable)
2. Command-line arguments
3. Local project settings (.claude/settings.local.json)
4. Shared project settings (.claude/settings.json)
5. User settings (~/.claude/settings.json)

Arrays merge (union), objects deep-merge.

### 9.4 Enterprise Managed Settings (VERIFIED ✅)

**Confirmed**: Read-only at system level:

-   macOS: `/Library/Application Support/ClaudeCode/managed-settings.json`
-   Linux: `/etc/claude-code/managed-settings.json`
-   Windows: `C:\ProgramData\ClaudeCode\managed-settings.json`

Cannot be overridden by users. Also check for `managed-mcp.json`.

### 9.5 Unresolved Questions (DOCUMENTED)

See FINDINGS_SUMMARY.md for:

-   Env var live-reload behavior
-   Hooks live-reload behavior
-   MCP server detection timing
-   Concurrent modification handling
-   JSON schema validation
```

---

## Change 7: Update Rust Dependencies (If Needed)

**File**: `src-tauri/Cargo.toml`

Current plan includes:

```toml
sha2 = "0.10"
```

**Verify already added**:

-   ✅ serde/serde_json (for JSON handling)
-   ✅ tokio (for async)
-   ✅ dirs (for home_dir)
-   ✅ nanoid (for ID generation)

**Additional if validation added**:

```toml
# For more robust path canonicalization
pathdiff = "0.2"  # Compare canonical paths
```

---

## Summary of Changes

| Change                       | Impact | Files                 | Effort      |
| ---------------------------- | ------ | --------------------- | ----------- |
| Remove mcpServers from merge | High   | commands.rs, UI       | Low         |
| Add restart notification     | Medium | query.ts, UI          | Low-Medium  |
| Add field validation         | Medium | commands.rs, query.ts | Medium      |
| Detect managed settings      | Medium | commands.rs, UI       | Low         |
| Auto-import enhancement      | Low    | commands.rs           | Very Low    |
| Update documentation         | High   | Plan doc              | Low         |
| Testing unresolved items     | High   | N/A (testing)         | Medium-High |

---

## Recommended Implementation Order

1. **Update Plan Doc** (Today) - Document findings
2. **Phase 1a**: Remove mcpServers from merge logic
3. **Phase 1b**: Add settings field validation
4. **Phase 1c**: Detect managed-settings.json
5. **Phase 1d**: Add restart notification (optional)
6. **Phase 2**: Plan MCP management (after Phase 1 complete)

---

## Testing Verification Checklist

Before Phase 1 completion, verify:

-   [ ] Settings.json live-reload works for all fields
-   [ ] Hooks don't reload (or confirm they do)
-   [ ] Env vars behavior (immediate or next session?)
-   [ ] Managed-settings.json detection works
-   [ ] Invalid JSON handling (graceful error)
-   [ ] Path canonicalization prevents duplicates
-   [ ] Concurrent modifications handled safely
-   [ ] MCP `.mcp.json` not affected by settings.json changes

---

**Next Step**: Update `/Users/huutri/code/ccmate/plans/251205-per-project-config.md` with these adjustments
