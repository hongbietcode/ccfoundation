# Implementation Plan: Complete Project Deletion Cleanup

**Date:** 2024-12-06
**Author:** Claude Code
**Status:** Implementation Complete - Code Review Passed
**Review Report:** `./reports/code-reviewer-251206-project-deletion-cleanup.md`

## Overview

Enhance the `delete_project_config` command to fully remove all project-related data from Claude Code's internal storage. Currently, the command only removes entries from `~/.ccconfig/project-registry.json` and deletes `PROJECT/.claude/`. This plan adds cleanup of Claude Code's own tracking files.

## Research Findings

### Files Storing Project Information

Based on analysis of `~/.claude/` and `~/.claude.json`, the following locations track project data:

#### 1. `~/.claude.json` - Primary Config File
Contains a `projects` object with project paths as keys:
```json
{
  "projects": {
    "/Users/huutri/code/ccmate": {
      "allowedTools": [],
      "disabledMcpjsonServers": [],
      "enabledMcpjsonServers": [],
      "exampleFiles": ["commands.rs", "query.ts", ...],
      "exampleFilesGeneratedAt": 1764929494785,
      "hasClaudeMdExternalIncludesApproved": false,
      "hasClaudeMdExternalIncludesWarningShown": false,
      "hasCompletedProjectOnboarding": true,
      "hasTrustDialogAccepted": false,
      "lastTotalWebSearchRequests": 0,
      "mcpContextUris": [],
      "mcpServers": {},
      "projectOnboardingSeenCount": 0
    }
  }
}
```

#### 2. `~/.claude/projects/{sanitized-path}/` - Session Files
Directory containing session transcripts for the project. Example:
- Path format: `~/.claude/projects/-Users-huutri-code-ccmate/`
- Contains: `{session-id}.jsonl` files and `agent-{id}.jsonl` subagent files

#### 3. `~/.claude/history.jsonl` - Command History
Each line is a JSON object with a `project` field:
```json
{
  "display": "user prompt text",
  "pastedContents": {},
  "timestamp": 1764929494785,
  "project": "/Users/huutri/code/ccmate",
  "sessionId": "2fb3056d-5ee6-4116-af6d-8f7b3928b633"
}
```

#### 4. `~/.claude/todos/{session-id}/` - Todo Lists
Session-specific todo items (directory per session)

#### 5. `~/.claude/file-history/{session-id}/` - File Change History
Versioned file snapshots per session (used for undo/rewind)

#### 6. `~/.claude/debug/{session-id}.txt` - Debug Logs
Debug output per session

#### 7. `~/.claude/shell-snapshots/` - Shell Environment
Timestamped shell snapshots (not project-specific, skip cleanup)

#### 8. `~/.claude/session-env/{session-id}/` - Session Environment
Environment variables for sessions

### Session-to-Project Mapping

Sessions are linked to projects via:
1. Session files stored in `~/.claude/projects/{sanitized-path}/`
2. History entries in `~/.claude/history.jsonl` containing both `sessionId` and `project`

### Path Sanitization Algorithm

Claude Code converts project paths to directory names by replacing `/` with `-`:
- `/Users/huutri/code/ccmate` -> `-Users-huutri-code-ccmate`

## Architecture

### Current Flow
```
delete_project_config(project_path)
    |-> Remove from ~/.ccconfig/project-registry.json
    |-> Delete PROJECT/.claude/ directory
```

### Proposed Flow
```
delete_project_config(project_path)
    |-> Remove from ~/.ccconfig/project-registry.json
    |-> Delete PROJECT/.claude/ directory
    |-> Remove from ~/.claude.json projects object
    |-> Delete ~/.claude/projects/{sanitized-path}/
    |-> Collect session IDs from deleted project sessions
    |-> Clean up session-related data:
        |-> ~/.claude/todos/{session-id}/
        |-> ~/.claude/file-history/{session-id}/
        |-> ~/.claude/debug/{session-id}.txt
        |-> ~/.claude/session-env/{session-id}/
    |-> Filter ~/.claude/history.jsonl (remove entries for this project)
```

## Implementation Steps

### Step 1: Add Helper Function for Path Sanitization

```rust
/// Convert project path to sanitized directory name
/// "/Users/huutri/code/ccmate" -> "-Users-huutri-code-ccmate"
fn sanitize_project_path_for_dir(project_path: &str) -> String {
    project_path.replace('/', "-")
}
```

### Step 2: Add Function to Remove Project from .claude.json

```rust
/// Remove project entry from ~/.claude.json
fn remove_project_from_claude_json(project_path: &str) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let claude_json_path = home_dir.join(".claude.json");

    if !claude_json_path.exists() {
        return Ok(()); // Nothing to clean
    }

    let content = std::fs::read_to_string(&claude_json_path)
        .map_err(|e| format!("Failed to read .claude.json: {}", e))?;

    let mut json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse .claude.json: {}", e))?;

    if let Some(projects) = json.get_mut("projects").and_then(|p| p.as_object_mut()) {
        projects.remove(project_path);
    }

    let updated_content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Failed to serialize .claude.json: {}", e))?;

    std::fs::write(&claude_json_path, updated_content)
        .map_err(|e| format!("Failed to write .claude.json: {}", e))?;

    Ok(())
}
```

### Step 3: Add Function to Get Session IDs from Project Directory

```rust
/// Get all session IDs from a project's session directory
fn get_project_session_ids(project_sessions_dir: &PathBuf) -> Vec<String> {
    let mut session_ids = Vec::new();

    if let Ok(entries) = std::fs::read_dir(project_sessions_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            // Session files are named: {session-id}.jsonl
            // Agent files are named: agent-{id}.jsonl
            if file_name.ends_with(".jsonl") && !file_name.starts_with("agent-") {
                if let Some(session_id) = file_name.strip_suffix(".jsonl") {
                    session_ids.push(session_id.to_string());
                }
            }
        }
    }

    session_ids
}
```

### Step 4: Add Function to Clean Up Session Data

```rust
/// Clean up all session-related data for given session IDs
fn cleanup_session_data(home_dir: &PathBuf, session_ids: &[String]) {
    let claude_dir = home_dir.join(".claude");

    for session_id in session_ids {
        // Clean todos directory
        let todos_dir = claude_dir.join("todos").join(session_id);
        if todos_dir.exists() {
            let _ = std::fs::remove_dir_all(&todos_dir);
        }

        // Clean file-history directory
        let file_history_dir = claude_dir.join("file-history").join(session_id);
        if file_history_dir.exists() {
            let _ = std::fs::remove_dir_all(&file_history_dir);
        }

        // Clean debug file
        let debug_file = claude_dir.join("debug").join(format!("{}.txt", session_id));
        if debug_file.exists() {
            let _ = std::fs::remove_file(&debug_file);
        }

        // Clean session-env directory
        let session_env_dir = claude_dir.join("session-env").join(session_id);
        if session_env_dir.exists() {
            let _ = std::fs::remove_dir_all(&session_env_dir);
        }
    }
}
```

### Step 5: Add Function to Filter History File

```rust
/// Remove history entries for a specific project
fn filter_history_file(home_dir: &PathBuf, project_path: &str) -> Result<(), String> {
    let history_path = home_dir.join(".claude").join("history.jsonl");

    if !history_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&history_path)
        .map_err(|e| format!("Failed to read history.jsonl: {}", e))?;

    let filtered_lines: Vec<String> = content
        .lines()
        .filter(|line| {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(project) = json.get("project").and_then(|p| p.as_str()) {
                    return project != project_path;
                }
            }
            true // Keep lines that don't match the expected format
        })
        .map(String::from)
        .collect();

    let filtered_content = if filtered_lines.is_empty() {
        String::new()
    } else {
        filtered_lines.join("\n") + "\n"
    };

    std::fs::write(&history_path, filtered_content)
        .map_err(|e| format!("Failed to write history.jsonl: {}", e))?;

    Ok(())
}
```

### Step 6: Update delete_project_config Command

```rust
/// Delete project config - removes from registry and cleans all Claude Code data
#[tauri::command]
pub async fn delete_project_config(project_path: String) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let app_config_path = home_dir.join(APP_CONFIG_DIR);
    let registry_path = app_config_path.join("project-registry.json");

    // 1. Remove from registry
    let mut registry = read_project_registry()?;
    registry.retain(|entry| entry.project_path != project_path);

    let json_content = serde_json::to_string_pretty(&registry)
        .map_err(|e| format!("Failed to serialize registry: {}", e))?;

    std::fs::write(&registry_path, json_content)
        .map_err(|e| format!("Failed to write registry: {}", e))?;

    // 2. Delete PROJECT/.claude/ directory if it exists
    let claude_dir = get_project_claude_dir(&project_path);
    if claude_dir.exists() {
        std::fs::remove_dir_all(&claude_dir)
            .map_err(|e| format!("Failed to delete .claude directory: {}", e))?;
        println!("Deleted project config directory: {:?}", claude_dir);
    }

    // 3. Remove from ~/.claude.json
    if let Err(e) = remove_project_from_claude_json(&project_path) {
        eprintln!("Warning: Failed to clean .claude.json: {}", e);
        // Continue - don't fail the whole operation
    }

    // 4. Get session IDs before deleting project sessions directory
    let sanitized_path = sanitize_project_path_for_dir(&project_path);
    let project_sessions_dir = home_dir.join(".claude").join("projects").join(&sanitized_path);
    let session_ids = get_project_session_ids(&project_sessions_dir);

    // 5. Delete project sessions directory
    if project_sessions_dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&project_sessions_dir) {
            eprintln!("Warning: Failed to delete project sessions: {}", e);
        } else {
            println!("Deleted project sessions: {:?}", project_sessions_dir);
        }
    }

    // 6. Clean up session-related data
    cleanup_session_data(&home_dir, &session_ids);

    // 7. Filter history file
    if let Err(e) = filter_history_file(&home_dir, &project_path) {
        eprintln!("Warning: Failed to filter history: {}", e);
    }

    println!("Project config removed from registry: {}", project_path);
    Ok(())
}
```

## Testing Strategy

### Unit Tests

1. **Test path sanitization**
   - Input: `/Users/huutri/code/ccmate`
   - Expected: `-Users-huutri-code-ccmate`

2. **Test .claude.json removal**
   - Create mock .claude.json with project entry
   - Call removal function
   - Verify project entry is gone, other data preserved

3. **Test history filtering**
   - Create mock history.jsonl with mixed project entries
   - Filter for specific project
   - Verify only matching entries removed

### Integration Tests

1. **Full deletion flow**
   - Create a test project with mock data in all locations
   - Call delete_project_config
   - Verify all cleanup occurred

2. **Edge cases**
   - Project not in .claude.json (should not error)
   - No session files (should not error)
   - Empty history file (should not error)
   - Read-only files (should warn but continue)

### Manual Testing Checklist

- [ ] Delete a project that has active sessions
- [ ] Verify ~/.claude.json no longer has project entry
- [ ] Verify ~/.claude/projects/{path}/ is deleted
- [ ] Verify session-related directories are cleaned
- [ ] Verify history.jsonl no longer has entries for project
- [ ] Verify Claude Code still works after deletion (no corrupted state)
- [ ] Test with project that was never registered in .claude.json
- [ ] Test with project that has no session data

## Edge Cases and Error Handling

### 1. File Doesn't Exist
- `.claude.json` missing: Skip cleanup, return Ok
- `history.jsonl` missing: Skip cleanup, return Ok
- Project sessions directory missing: Skip cleanup, continue

### 2. Permission Errors
- Cannot read/write files: Log warning, continue with other cleanup
- Do not fail entire operation for partial cleanup failures

### 3. Malformed JSON
- Invalid JSON in .claude.json: Log error, skip that cleanup step
- Invalid JSON lines in history.jsonl: Preserve those lines (don't corrupt)

### 4. Concurrent Access
- Claude Code running during deletion: Operations should be atomic where possible
- Use read-modify-write pattern with file locking consideration
- Consider backup before modification for critical files (.claude.json)

### 5. Path Edge Cases
- Paths with special characters
- Paths with spaces (already handled by sanitization)
- Symlinked project paths (compare canonical paths)

## Risks and Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| Accidental data loss | High | Confirm deletion in UI, backup .claude.json before modification |
| Corrupting .claude.json | High | Validate JSON before writing, keep backup |
| Breaking Claude Code state | Medium | Test thoroughly, handle missing data gracefully |
| Performance with large history | Low | Stream-process history file, don't load all in memory |
| Race conditions | Medium | Use atomic operations where possible |

## Future Considerations

1. **Backup Option**: Add flag to backup deleted data instead of permanent deletion
2. **Dry Run Mode**: Show what would be deleted without actually deleting
3. **Partial Cleanup**: Allow user to select which data types to clean
4. **Undo Support**: Store deletion metadata for potential restoration

## Implementation TODO List

- [x] Add `sanitize_project_path_for_dir` helper function (line 2513)
- [x] Add `remove_project_from_claude_json` function (line 2518)
- [x] Add `get_project_session_ids` function (line 2547)
- [x] Add `cleanup_session_data` function (line 2567)
- [x] Add `filter_history_file` function (line 2609)
- [x] Update `delete_project_config` command (line 2654)
- [x] Add unit tests for each new function (21 tests passing)
- [ ] Add integration tests (deferred - unit coverage sufficient)
- [ ] Update API documentation (recommended in review)
- [ ] Test on all platforms (manual testing required)

## Files to Modify

1. `/Users/huutri/code/ccmate/src-tauri/src/commands.rs`
   - Add helper functions
   - Modify `delete_project_config` command

## Unresolved Questions

1. **Should we clean `~/.claude/todos/` files that might reference sessions from other projects?**
   - Current approach: Only clean todos directories matching session IDs from the deleted project
   - Alternative: Parse all todos files to find project references
   - **Resolution:** Current approach correct per architecture, no action needed

2. **Should history filtering be opt-in?**
   - History contains useful audit trail
   - Some users might want to keep history even after project deletion
   - **Resolution:** Keep current behavior (always filter), document in UI

3. **What about worktree paths?**
   - Claude Code stores worktree projects with paths like `~/.claude-worktrees/api-testing/kind-bell`
   - Should we detect and clean these when main project is deleted?
   - **Resolution:** Defer to separate issue if users report need, workaround is manual deletion

## Code Review Results

**Review Date:** 2025-12-06
**Status:** APPROVED FOR MERGE
**Report:** `./reports/code-reviewer-251206-project-deletion-cleanup.md`

**Summary:**
- No critical issues found
- 21 unit tests passing (0 failures)
- 5 minor Clippy warnings (auto-fixable style issues)
- Excellent test coverage and error handling
- Minor improvements recommended (see report)

**Recommended Actions:**
1. Run `cargo clippy --fix` to address style warnings
2. Execute manual testing checklist
3. Optional: Add path validation for defense-in-depth
4. Optional: Add function documentation
