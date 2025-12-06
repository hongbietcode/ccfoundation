# Remove Model Selection/Override Feature from Sessions System

**Date**: 2025-12-06
**Status**: Draft
**Priority**: Medium

---

## 1. Overview

This plan details the removal of the model selection/override feature from the sessions system. After implementation, sessions will always use Claude CLI's default model behavior (either system-wide settings or Claude CLI's built-in default).

### Goals

1. Remove model selection dropdown from "New Session" dialog
2. Remove model override when resuming sessions
3. Simplify API by removing model parameters from commands
4. **Keep read-only model display** for historical sessions (shows which model was used)
5. **Retain model parsing** from JSONL files for statistics and display

### Non-Goals

- Do NOT remove the model mapping system (`src-tauri/src/models/`) - it may be used elsewhere
- Do NOT migrate/modify existing session files - the model field is historical data
- Do NOT remove model-related imports that are still needed for display

---

## 2. Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Model Display | **Keep read-only** | Historical data is useful for cost estimation and auditing |
| Default Behavior | **Let Claude CLI decide** | No `--model` flag = Claude uses its default from config or built-in |
| Session Metadata | **Keep parsing model from JSONL** | Required for SessionStats cost calculation |
| Migration | **No migration needed** | Model field in JSONL is historical data, not configuration |
| Model Filtering | **Remove from UI** | Filter by model only makes sense when users can set models |

---

## 3. Files to Modify

### 3.1 Frontend Files

#### `src/components/sessions/SessionList.tsx`

**Changes:**

1. Remove `useModels` import
2. Remove `newModel` state variable
3. Remove model dropdown from "New Session" dialog (lines 211-225)
4. Remove `model: newModel` from `handleCreateSession`
5. Remove model-related filter dropdown items ("Sonnet", "Opus", "Haiku")
6. Remove `filterModel` state and filtering logic
7. Remove "Sort by Model" option from sort dropdown

**Affected Lines:** 4, 52-54, 59, 66-68, 80-81, 94-97, 211-225, 268-270, 282-296

#### `src/components/sessions/SessionDetail.tsx`

**Changes:**

1. Remove `model: session?.model` from `handleSend` function (line 115)
2. Keep model display in header (lines 253-258) - **read-only**
3. Keep model reference in `exportToMarkdown` (line 157) - **read-only**

**Affected Lines:** 107, 115

#### `src/lib/sessions-query.ts`

**Changes:**

1. Remove `model` parameter from `useResumeSession` mutation
2. Remove `model` parameter from `useCreateSession` mutation
3. Keep `Session` interface with optional `model` field - **read-only**
4. Keep `SessionMessage` interface with optional `model` field - **read-only**

**Affected Lines:** 110-127, 177-201

#### `src/components/sessions/SessionCard.tsx`

**No changes required** - Model display is read-only and should remain.

#### `src/components/sessions/SessionStats.tsx`

**No changes required** - Uses `session.model` for cost estimation (read-only).

---

### 3.2 Backend Files

#### `src-tauri/src/sessions/commands.rs`

**Changes:**

1. Remove `model` parameter from `session_resume` command (line 89)
2. Remove `model` parameter from `session_create` command (line 143)
3. Update function calls to pass `None` instead of `model`

**Before:**
```rust
#[tauri::command]
pub async fn session_resume(
    app: AppHandle,
    session_id: String,
    message: String,
    project_path: String,
    model: Option<String>,  // REMOVE THIS
    processes: State<'_, RunningProcesses>,
) -> Result<(), String>
```

**After:**
```rust
#[tauri::command]
pub async fn session_resume(
    app: AppHandle,
    session_id: String,
    message: String,
    project_path: String,
    processes: State<'_, RunningProcesses>,
) -> Result<(), String>
```

**Affected Lines:** 82-103, 137-156

#### `src-tauri/src/sessions/resume.rs`

**Changes:**

1. Remove `model` parameter from `resume_session` function signature (line 35)
2. Remove model normalization call (line 56)
3. Remove `--model` flag from Claude CLI command (lines 72-84)
4. Remove `model` parameter from `create_session` function signature (line 280)
5. Remove model normalization call (line 301)
6. Remove `--model` flag from Claude CLI command (lines 315-321)

**Before:**
```rust
pub async fn resume_session(
    app: AppHandle,
    session_id: String,
    message: String,
    project_path: String,
    model: Option<String>,  // REMOVE THIS
    processes: RunningProcesses,
) -> Result<(), String>
```

**After:**
```rust
pub async fn resume_session(
    app: AppHandle,
    session_id: String,
    message: String,
    project_path: String,
    processes: RunningProcesses,
) -> Result<(), String>
```

**Affected Lines:** 1 (import), 30-84 (`resume_session`), 275-321 (`create_session`)

---

## 4. Implementation Steps

### Phase 1: Backend Changes

- [ ] **Step 1.1**: Modify `src-tauri/src/sessions/resume.rs`
  - Remove `model` parameter from `resume_session` function
  - Remove `normalize_model_option` import if no longer needed
  - Remove model normalization logic
  - Remove `--model` flag from CLI command
  - Simplify logging to not mention model

- [ ] **Step 1.2**: Modify `src-tauri/src/sessions/resume.rs`
  - Remove `model` parameter from `create_session` function
  - Remove model normalization logic
  - Remove `--model` flag from CLI command
  - Simplify logging to not mention model

- [ ] **Step 1.3**: Modify `src-tauri/src/sessions/commands.rs`
  - Remove `model` parameter from `session_resume` Tauri command
  - Update call to `resume_session` to not pass model
  - Remove `model` parameter from `session_create` Tauri command
  - Update call to `create_session` to not pass model

- [ ] **Step 1.4**: Clean up imports in `resume.rs`
  - Remove `use crate::models::normalize_model_option;` if no longer used

### Phase 2: Frontend - Query Layer

- [ ] **Step 2.1**: Modify `src/lib/sessions-query.ts`
  - Remove `model` from `useResumeSession` mutation parameters
  - Remove `model` from invoke call in `useResumeSession`
  - Remove `model` from `useCreateSession` mutation parameters
  - Remove `model` from invoke call in `useCreateSession`

### Phase 3: Frontend - UI Components

- [ ] **Step 3.1**: Modify `src/components/sessions/SessionList.tsx`
  - Remove `useModels` import
  - Remove `newModel` state
  - Remove model selection dropdown from dialog
  - Remove `model: newModel` from create session call
  - Remove `filterModel` state
  - Remove model filter logic
  - Remove "Filter By Model" dropdown items
  - Remove "Sort by Model" option

- [ ] **Step 3.2**: Modify `src/components/sessions/SessionDetail.tsx`
  - Remove `model: session?.model` from `handleSend` call
  - Keep read-only model display in header

### Phase 4: Cleanup

- [ ] **Step 4.1**: Review and remove unused imports
  - Check if `useModels` is used elsewhere before removing from SessionList
  - Verify `model-mapping.ts` exports are still needed

- [ ] **Step 4.2**: Update TypeScript types (if needed)
  - Ensure no type errors after removing model parameters

---

## 5. Code Diffs

### 5.1 `src-tauri/src/sessions/resume.rs`

```diff
-use crate::models::normalize_model_option;
 use serde_json::Value as JsonValue;
 use std::collections::HashMap;
 use std::process::Stdio;
@@ -30,13 +29,10 @@ pub async fn resume_session(
     session_id: String,
     message: String,
     project_path: String,
-    model: Option<String>,
     processes: RunningProcesses,
 ) -> Result<(), String> {
     println!("resume_session: session_id={}", session_id);

@@ -52,9 +48,6 @@ pub async fn resume_session(
         .canonicalize()
         .map_err(|e| format!("Failed to canonicalize path: {}", e))?;

-    // Normalize model name to full API identifier
-    let normalized_model = normalize_model_option(model);
-
     // Build Claude CLI command
     let mut cmd = Command::new("claude");
     cmd.arg("--resume")
@@ -67,17 +60,9 @@ pub async fn resume_session(
         .stdout(Stdio::piped())
         .stderr(Stdio::piped());

-    // Add model override if specified
-    if let Some(model_name) = &normalized_model {
-        cmd.arg("--model").arg(model_name);
-        println!(
-            "Command: claude --resume {} -p <message> --model {} --output-format stream-json --verbose",
-            session_id, model_name
-        );
-    } else {
-        println!(
-            "Command: claude --resume {} -p <message> --output-format stream-json --verbose",
-            session_id
-        );
-    }
+    println!(
+        "Command: claude --resume {} -p <message> --output-format stream-json --verbose",
+        session_id
+    );
```

Similar changes for `create_session` function.

### 5.2 `src-tauri/src/sessions/commands.rs`

```diff
@@ -82,12 +82,10 @@ pub async fn session_resume(
     app: AppHandle,
     session_id: String,
     message: String,
     project_path: String,
-    model: Option<String>,
     processes: State<'_, RunningProcesses>,
 ) -> Result<(), String> {
     println!("session_resume: session_id={}", session_id);

     resume_session(
         app,
         session_id,
         message,
         project_path,
-        model,
         processes.inner().clone(),
     )
     .await
@@ -137,12 +135,10 @@ pub async fn session_create(
     app: AppHandle,
     message: String,
     project_path: String,
-    model: Option<String>,
     processes: State<'_, RunningProcesses>,
 ) -> Result<String, String> {
     println!("session_create: project_path={}", project_path);

     create_session(
         app,
         message,
         project_path,
-        model,
         processes.inner().clone(),
     )
     .await
```

### 5.3 `src/lib/sessions-query.ts`

```diff
@@ -105,16 +105,12 @@ export function useResumeSession() {
         mutationFn: async ({
             sessionId,
             message,
             projectPath,
-            model,
         }: {
             sessionId: string;
             message: string;
             projectPath: string;
-            model?: string;
         }) => {
-            // Model normalization now happens on backend
             await invoke("session_resume", {
                 sessionId,
                 message,
                 projectPath,
-                model,
             });
         },
@@ -174,14 +170,11 @@ export function useCreateSession() {

     return useMutation({
         mutationFn: async ({
             message,
             projectPath,
-            model,
         }: {
             message: string;
             projectPath: string;
-            model?: string;
         }) => {
             const sessionId = await invoke<string>("session_create", {
                 message,
                 projectPath,
-                model,
             });
             return sessionId;
         },
```

### 5.4 `src/components/sessions/SessionList.tsx`

```diff
@@ -1,8 +1,7 @@
 import { useState } from "react";
 import { useParams } from "react-router-dom";
-import { useSessions, useCreateSession, useDeleteSession } from "@/lib/sessions-query";
-import { useModels } from "@/lib/model-mapping";
+import { useSessions, useCreateSession, useDeleteSession } from "@/lib/sessions-query";
 import { SessionCard } from "./SessionCard";
 import { Input } from "@/components/ui/input";
 import { Button } from "@/components/ui/button";
-import { Search, Plus, Loader2, ArrowUpDown, Filter, Sparkles, Trash2, CheckSquare } from "lucide-react";
+import { Search, Plus, Loader2, ArrowUpDown, Sparkles, Trash2, CheckSquare } from "lucide-react";
@@ -49,13 +48,9 @@ export function SessionList({
     const [searchQuery, setSearchQuery] = useState("");
     const [dialogOpen, setDialogOpen] = useState(false);
     const [newMessage, setNewMessage] = useState("");
-    const [newModel, setNewModel] = useState<string | undefined>();
-    const [sortBy, setSortBy] = useState<"date-new" | "date-old" | "messages" | "model">("date-new");
-    const [filterModel, setFilterModel] = useState<string | "all">("all");
+    const [sortBy, setSortBy] = useState<"date-new" | "date-old" | "messages">("date-new");
     const [selectionMode, setSelectionMode] = useState(false);
     const [selectedSessions, setSelectedSessions] = useState<Set<string>>(new Set());

     const { data: sessions = [], isLoading } = useSessions(decodedPath);
-    const { data: models = [] } = useModels();
     const createSession = useCreateSession();
     const deleteSession = useDeleteSession();

     // Filter sessions
     let filteredSessions = sessions.filter((session) => {
-        const matchesSearch = session.title.toLowerCase().includes(searchQuery.toLowerCase());
-        const matchesModel = filterModel === "all" ||
-            (session.model && session.model.toLowerCase().includes(filterModel.toLowerCase()));
-        return matchesSearch && matchesModel;
+        return session.title.toLowerCase().includes(searchQuery.toLowerCase());
     });

     // Sort sessions
     filteredSessions = [...filteredSessions].sort((a, b) => {
         switch (sortBy) {
             case "date-new":
                 return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime();
             case "date-old":
                 return new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
             case "messages":
                 return b.messageCount - a.messageCount;
-            case "model":
-                return (a.model || "").localeCompare(b.model || "");
             default:
                 return 0;
         }
     });

     const handleCreateSession = async () => {
         if (!newMessage.trim()) return;

         try {
             const sessionId = await createSession.mutateAsync({
                 message: newMessage,
                 projectPath: decodedPath,
-                model: newModel,
             });
             setNewMessage("");
-            setNewModel(undefined);
             setDialogOpen(false);
             // Auto-select the new session
```

Remove model dropdown from dialog (lines 211-225):

```diff
                         </div>
-                        <div className="space-y-2">
-                            <Label htmlFor="model">Model (Optional)</Label>
-                            <Select value={newModel} onValueChange={setNewModel}>
-                                <SelectTrigger id="model">
-                                    <SelectValue placeholder="Default model" />
-                                </SelectTrigger>
-                                <SelectContent>
-                                    {models.map((model) => (
-                                        <SelectItem key={model.id} value={model.id}>
-                                            {model.displayName}
-                                        </SelectItem>
-                                    ))}
-                                </SelectContent>
-                            </Select>
-                        </div>
                     </div>
```

Remove sort by model option (line 268-270):

```diff
                             <DropdownMenuItem onClick={() => setSortBy("messages")}>
                                 Message Count
                             </DropdownMenuItem>
-                            <DropdownMenuItem onClick={() => setSortBy("model")}>
-                                Model
-                            </DropdownMenuItem>
```

Remove filter dropdown entirely (lines 274-297):

```diff
-                    <DropdownMenu>
-                        <DropdownMenuTrigger asChild>
-                            <Button variant="outline" size="sm" className="flex-1">
-                                <Filter className="h-4 w-4 mr-2" />
-                                Filter
-                            </Button>
-                        </DropdownMenuTrigger>
-                        <DropdownMenuContent align="end">
-                            <DropdownMenuLabel>Filter By Model</DropdownMenuLabel>
-                            <DropdownMenuSeparator />
-                            <DropdownMenuItem onClick={() => setFilterModel("all")}>
-                                All Models
-                            </DropdownMenuItem>
-                            <DropdownMenuItem onClick={() => setFilterModel("sonnet")}>
-                                Sonnet
-                            </DropdownMenuItem>
-                            <DropdownMenuItem onClick={() => setFilterModel("opus")}>
-                                Opus
-                            </DropdownMenuItem>
-                            <DropdownMenuItem onClick={() => setFilterModel("haiku")}>
-                                Haiku
-                            </DropdownMenuItem>
-                        </DropdownMenuContent>
-                    </DropdownMenu>
```

### 5.5 `src/components/sessions/SessionDetail.tsx`

```diff
@@ -107,8 +107,6 @@ export function SessionDetail({ sessionId }: SessionDetailProps) {
         console.log("Sending message to session:", {
             sessionId,
             message: message.substring(0, 50),
             projectPath: decodedPath,
-            model: session?.model,
         });

         try {
             await resumeSession.mutateAsync({
                 sessionId,
                 message,
                 projectPath: decodedPath,
-                model: session?.model, // Pass session model to override stored model
             });
```

---

## 6. Testing Strategy

### 6.1 Backend Tests

```bash
# Build and check for compile errors
cd src-tauri && cargo build

# Run existing tests
cargo test
```

### 6.2 Frontend Tests

```bash
# TypeScript type checking
pnpm tsc --noEmit

# Run existing tests
pnpm test
```

### 6.3 Manual Testing Checklist

- [ ] Create new session without model selection
- [ ] Verify new session uses Claude CLI default model
- [ ] Resume existing session without model override
- [ ] Verify resumed session uses Claude CLI default model
- [ ] Check model display in SessionCard (should show historical model)
- [ ] Check model display in SessionDetail header (should show historical model)
- [ ] Check SessionStats cost estimation (should use historical model for pricing)
- [ ] Verify export includes model info (read-only)
- [ ] Sort by date works
- [ ] Sort by messages works
- [ ] Search works
- [ ] No console errors

### 6.4 Integration Test

1. Start app: `pnpm tauri dev`
2. Navigate to sessions page
3. Create new session with message "Hello, test session"
4. Wait for response
5. Verify session appears in list
6. Check model shown in SessionCard matches what Claude returned
7. Resume session with another message
8. Verify response uses default model

---

## 7. Rollback Plan

If issues arise, revert the following commits in order:

1. Frontend component changes
2. Query layer changes
3. Backend command changes

All changes are isolated to the model parameter removal, so rollback is straightforward git revert.

---

## 8. Impact Analysis

### What Changes

| Component | Before | After |
|-----------|--------|-------|
| New Session Dialog | Model dropdown | No model selection |
| Resume Session | Passes `session.model` | No model override |
| CLI Command | `--model <name>` flag | No model flag |
| API Signatures | `model: Option<String>` | No model param |

### What Stays the Same

| Component | Behavior |
|-----------|----------|
| SessionCard | Shows historical model (read-only) |
| SessionStats | Uses model for cost estimation |
| Session type | Has optional `model` field |
| JSONL parsing | Extracts model from messages |
| Export | Includes model info |

---

## 9. Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Type errors from parameter removal | Low | Medium | Run `pnpm tsc --noEmit` before merging |
| Users expect model selection | Medium | Low | Document change in release notes |
| Cost estimation wrong without model | Low | Low | Historical model still parsed from JSONL |
| Backend/frontend mismatch | Low | High | Test full create/resume flow manually |

---

## 10. Checklist Summary

### Backend (Rust)

- [ ] Remove `model` param from `resume_session` in `resume.rs`
- [ ] Remove `model` param from `create_session` in `resume.rs`
- [ ] Remove `model` param from `session_resume` command in `commands.rs`
- [ ] Remove `model` param from `session_create` command in `commands.rs`
- [ ] Remove `normalize_model_option` import if unused
- [ ] Run `cargo build` to verify

### Frontend (TypeScript/React)

- [ ] Remove `model` param from `useResumeSession` in `sessions-query.ts`
- [ ] Remove `model` param from `useCreateSession` in `sessions-query.ts`
- [ ] Remove model dropdown from `SessionList.tsx`
- [ ] Remove model filter from `SessionList.tsx`
- [ ] Remove sort-by-model from `SessionList.tsx`
- [ ] Remove model from `handleSend` in `SessionDetail.tsx`
- [ ] Run `pnpm tsc --noEmit` to verify
- [ ] Manual test create and resume flows

---

## 11. Appendix: Files Reference

| File | Purpose | Changes |
|------|---------|---------|
| `src-tauri/src/sessions/resume.rs` | Session create/resume logic | Remove model param |
| `src-tauri/src/sessions/commands.rs` | Tauri command handlers | Remove model param |
| `src-tauri/src/sessions/types.rs` | Type definitions | **No changes** |
| `src/lib/sessions-query.ts` | React Query hooks | Remove model param |
| `src/lib/model-mapping.ts` | Model utilities | **No changes** |
| `src/components/sessions/SessionList.tsx` | Session list UI | Remove model UI |
| `src/components/sessions/SessionDetail.tsx` | Session detail UI | Remove model override |
| `src/components/sessions/SessionCard.tsx` | Session card UI | **No changes** |
| `src/components/sessions/SessionStats.tsx` | Statistics display | **No changes** |
