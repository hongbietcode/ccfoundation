# Continue Chat After Task Completion - Implementation Plan

**Date**: 2025-12-06
**Status**: Planning
**Author**: Claude (Research & Planning Agent)
**Priority**: High
**Estimated Effort**: 8-12 hours

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Current State Analysis](#2-current-state-analysis)
3. [Architecture Decision](#3-architecture-decision)
4. [Implementation Details](#4-implementation-details)
5. [UX Flow](#5-ux-flow)
6. [Code Changes](#6-code-changes)
7. [Testing Strategy](#7-testing-strategy)
8. [Edge Cases](#8-edge-cases)
9. [Implementation Phases](#9-implementation-phases)
10. [TODO Checklist](#10-todo-checklist)

---

## 1. Executive Summary

### Objective

Enable users to continue chatting with Claude after a task reaches "Completed" status. Currently, once a task completes, the UI shows "Task is completed. Cannot send messages." This feature removes that limitation.

### User Story

> As a user, I want to continue asking follow-up questions or requesting modifications after Claude completes my initial task, so that I can iterate on the work without creating a new task.

### Key Requirements

1. Allow sending messages when task status is `completed`
2. Task should transition from `Completed` -> `Running` when user sends a follow-up message
3. Maintain conversation history continuity
4. Clear UI indication that user can continue the conversation
5. Respect concurrent task limits when resuming

---

## 2. Current State Analysis

### 2.1 Task Status Flow (Current)

```
Pending -> Running -> Completed (terminal)
                  -> Failed (terminal)
                  -> Cancelled (terminal)
        -> Paused -> Running -> ...
```

**Problem**: `Completed` is currently a terminal state with no valid transitions out.

### 2.2 Backend Code Analysis

**File**: `/Users/huutri/code/ccmate/src-tauri/src/tasks/task.rs`

```rust
pub enum TaskStatus {
    Pending,
    Running,
    Paused,
    Completed,  // Currently terminal
    Failed,
    Cancelled,
}
```

**File**: `/Users/huutri/code/ccmate/src-tauri/src/tasks/manager.rs`

The `start_task_execution` function (line 78-214):
- Checks if task is already running (line 91-93)
- Does NOT check for `Completed` status - meaning it will allow starting
- Sets status to `Running` (line 100)
- Spawns Claude CLI process

The `resume_task` function (line 274-288):
- Only allows resuming from `Paused` status (line 282-284)
- Does NOT allow resuming from `Completed`

### 2.3 Frontend Code Analysis

**File**: `/Users/huutri/code/ccmate/src/components/tasks/TaskDetail.tsx`

```typescript
// Line 145-148
const canSendMessage =
    task.status === "pending" || task.status === "paused";
```

This explicitly excludes `completed` status from allowing message sending.

**UI Display (Line 248-258)**:
```typescript
{isRunning ? (
    <div className="flex items-center justify-center gap-2">
        <Loader2 className="h-4 w-4 animate-spin" />
        Task is running...
    </div>
) : (
    `Task is ${task.status}. Cannot send messages.`
)}
```

### 2.4 React Query Hooks

**File**: `/Users/huutri/code/ccmate/src/lib/tasks-query.ts`

- `useStartTask`: Used for `pending` tasks
- `useResumeTask`: Used for `paused` tasks (requires message)
- No hook for continuing `completed` tasks

---

## 3. Architecture Decision

### 3.1 Options Considered

| Option | Description | Pros | Cons |
|--------|-------------|------|------|
| A. Reuse `task_resume` | Modify resume to accept `completed` status | Minimal changes | Semantic confusion (resume vs continue) |
| B. New `task_continue` command | Create dedicated command for continuing completed tasks | Clear semantics, explicit | More code |
| C. Allow `task_start` for completed | Modify start to work with completed tasks | Reuses existing code | Confusing - "start" implies fresh |
| D. No status change | Keep as `completed`, just allow messages | Simple | Status doesn't reflect actual state |

### 3.2 Recommended Approach: Option B - New `task_continue` Command

**Rationale**:
1. **Clear Semantics**: "Continue" accurately describes resuming a completed conversation
2. **Explicit State Machine**: Makes transition `Completed -> Running` explicit
3. **Audit Trail**: Can track how many times a task was continued
4. **Future Extensibility**: Can add continuation-specific logic (e.g., context summarization)

### 3.3 State Machine Update

```
                         +---> Completed ---+
                         |                  |
                         |    [continue]    |
                         |        v         |
Pending -> Running ------+---> Running -----+
   |          |                   ^
   |          |                   |
   |          +---> Paused -------+ [resume]
   |          |
   |          +---> Failed
   |          +---> Cancelled
```

**New Transition**: `Completed -> Running` (via `task_continue` command)

---

## 4. Implementation Details

### 4.1 Backend Changes

#### 4.1.1 New Command: `task_continue`

**File**: `/Users/huutri/code/ccmate/src-tauri/src/tasks/commands.rs`

```rust
/// Continue a completed task with a new message
#[tauri::command]
pub async fn task_continue(
    app: AppHandle,
    task_id: String,
    message: String,
    processes: State<'_, RunningProcesses>,
) -> Result<(), String> {
    println!("Continue task_continue: task_id={}, message={}", task_id, message);
    manager::continue_task(app, task_id, message, processes.inner().clone()).await
}
```

#### 4.1.2 New Manager Function: `continue_task`

**File**: `/Users/huutri/code/ccmate/src-tauri/src/tasks/manager.rs`

```rust
/// Continue a completed task with a new message
pub async fn continue_task(
    app: AppHandle,
    task_id: String,
    message: String,
    processes: RunningProcesses,
) -> Result<(), String> {
    let (task, _) = storage::load_task(&task_id)?;

    // Only allow continuing completed tasks
    if task.status != TaskStatus::Completed {
        return Err(format!(
            "Can only continue completed tasks. Current status: {:?}",
            task.status
        ));
    }

    // Check concurrent task limit
    let active_count = storage::get_active_tasks_count(&task.project_path)?;
    if active_count >= MAX_CONCURRENT_TASKS {
        return Err(format!(
            "Maximum concurrent tasks limit reached ({}/{}). Please wait for tasks to complete.",
            active_count, MAX_CONCURRENT_TASKS
        ));
    }

    // Start execution with the new message
    start_task_execution(app, task_id, message, processes).await
}
```

#### 4.1.3 Update Task Status Handling

**File**: `/Users/huutri/code/ccmate/src-tauri/src/tasks/task.rs`

Update `set_status` method to handle continuation:

```rust
pub fn set_status(&mut self, status: TaskStatus) {
    let previous_status = self.status.clone();
    self.status = status.clone();
    self.updated_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    match status {
        TaskStatus::Running => {
            if self.started_at.is_none() {
                self.started_at = Some(self.updated_at);
            }
            // Clear completed_at when continuing from completed
            if previous_status == TaskStatus::Completed {
                self.completed_at = None;
            }
        }
        TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => {
            self.completed_at = Some(self.updated_at);
        }
        _ => {}
    }
}
```

#### 4.1.4 Register New Command

**File**: `/Users/huutri/code/ccmate/src-tauri/src/lib.rs`

Add `task_continue` to the command registration:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    tasks::commands::task_continue,
])
```

### 4.2 Frontend Changes

#### 4.2.1 New React Query Hook

**File**: `/Users/huutri/code/ccmate/src/lib/tasks-query.ts`

```typescript
// Continue a completed task
export function useContinueTask() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: async ({
            taskId,
            message,
        }: {
            taskId: string;
            message: string;
        }) => {
            await invoke("task_continue", { taskId, message });
        },
        onSuccess: (_, variables) => {
            queryClient.invalidateQueries({
                queryKey: taskKeys.detail(variables.taskId),
            });
        },
    });
}
```

#### 4.2.2 Update TaskDetail Component

**File**: `/Users/huutri/code/ccmate/src/components/tasks/TaskDetail.tsx`

**Import the new hook**:
```typescript
import {
    // ... existing imports
    useContinueTask,
} from "@/lib/tasks-query";
```

**Add hook usage**:
```typescript
const continueTask = useContinueTask();
```

**Update canSendMessage logic**:
```typescript
const canSendMessage =
    task.status === "pending" ||
    task.status === "paused" ||
    task.status === "completed";  // NEW: Allow completed tasks
```

**Update handleSendMessage function**:
```typescript
const handleSendMessage = async (message: string) => {
    if (!task) return;

    try {
        if (task.status === "pending") {
            await startTask.mutateAsync({ taskId, message });
        } else if (task.status === "paused") {
            await resumeTask.mutateAsync({ taskId, message });
        } else if (task.status === "completed") {
            // NEW: Handle completed task continuation
            await continueTask.mutateAsync({ taskId, message });
        }
    } catch (error) {
        console.error("Failed to send message:", error);
    }
};
```

**Update placeholder text**:
```typescript
<TaskInput
    onSend={handleSendMessage}
    disabled={startTask.isPending || resumeTask.isPending || continueTask.isPending}
    placeholder={
        task.status === "pending"
            ? "Send a message to start the task..."
            : task.status === "paused"
            ? "Send a message to resume the task..."
            : task.status === "completed"
            ? "Send a message to continue the conversation..."  // NEW
            : "Send a message..."
    }
/>
```

**Update the "cannot send messages" display** (remove completed from this condition):
```typescript
{/* Input */}
<div className="p-4 border-t">
    {canSendMessage ? (
        <TaskInput
            onSend={handleSendMessage}
            disabled={startTask.isPending || resumeTask.isPending || continueTask.isPending}
            placeholder={
                task.status === "pending"
                    ? "Send a message to start the task..."
                    : task.status === "paused"
                    ? "Send a message to resume the task..."
                    : "Send a message to continue..."
            }
        />
    ) : (
        <div className="text-center text-sm text-muted-foreground py-4">
            {isRunning ? (
                <div className="flex items-center justify-center gap-2">
                    <Loader2 className="h-4 w-4 animate-spin" />
                    Task is running...
                </div>
            ) : (
                // Only show for failed/cancelled
                `Task is ${task.status}. Cannot send messages.`
            )}
        </div>
    )}
</div>
```

---

## 5. UX Flow

### 5.1 User Flow Diagram

```
+------------------------------------------+
|           Task Detail View               |
|                                          |
|  [Title: Implement Login Feature]        |
|  Status: [Completed Badge]               |
|                                          |
|  +------------------------------------+  |
|  |  User: Create a login form         |  |
|  +------------------------------------+  |
|  |  Claude: Done! Here's the login... |  |
|  +------------------------------------+  |
|                                          |
|  +------------------------------------+  |
|  | Send a message to continue...     |  |
|  |                              [Send]|  |
|  +------------------------------------+  |
+------------------------------------------+

User types: "Can you add form validation?"
Clicks Send

+------------------------------------------+
|           Task Detail View               |
|                                          |
|  [Title: Implement Login Feature]        |
|  Status: [Running Badge] <-- Changed!    |
|                                          |
|  +------------------------------------+  |
|  |  User: Create a login form         |  |
|  +------------------------------------+  |
|  |  Claude: Done! Here's the login... |  |
|  +------------------------------------+  |
|  |  User: Can you add form validation?|  |
|  +------------------------------------+  |
|  |  Claude: [Streaming response...]   |  |
|  +------------------------------------+  |
|                                          |
|  +------------------------------------+  |
|  | Task is running...            [---]|  |
|  +------------------------------------+  |
+------------------------------------------+
```

### 5.2 Visual Indicators

| State | Badge Color | Input State | Placeholder Text |
|-------|-------------|-------------|------------------|
| Pending | Gray | Enabled | "Send a message to start the task..." |
| Running | Blue (animated) | Disabled | "Task is running..." |
| Paused | Yellow | Enabled | "Send a message to resume the task..." |
| **Completed** | **Green** | **Enabled** | **"Send a message to continue..."** |
| Failed | Red | Disabled | "Task is failed. Cannot send messages." |
| Cancelled | Gray | Disabled | "Task is cancelled. Cannot send messages." |

### 5.3 Conversation Continuity

The conversation continues seamlessly:
- All previous messages remain visible
- New user message appears immediately
- Claude's response streams below
- Task status updates to "Running" then back to "Completed"

---

## 6. Code Changes

### 6.1 Files to Modify

| File | Type | Changes |
|------|------|---------|
| `src-tauri/src/tasks/commands.rs` | Backend | Add `task_continue` command |
| `src-tauri/src/tasks/manager.rs` | Backend | Add `continue_task` function |
| `src-tauri/src/tasks/task.rs` | Backend | Update `set_status` for continuation |
| `src-tauri/src/lib.rs` | Backend | Register `task_continue` command |
| `src/lib/tasks-query.ts` | Frontend | Add `useContinueTask` hook |
| `src/components/tasks/TaskDetail.tsx` | Frontend | Update input logic for completed status |

### 6.2 Lines of Code Estimate

| Component | New/Modified Lines |
|-----------|-------------------|
| `task_continue` command | ~15 |
| `continue_task` function | ~30 |
| `set_status` update | ~10 |
| Command registration | ~2 |
| `useContinueTask` hook | ~20 |
| TaskDetail updates | ~25 |
| **Total** | **~102 lines** |

### 6.3 Complete Code Diff

#### Backend: `src-tauri/src/tasks/commands.rs`

Add after `task_resume` function (around line 103):

```rust
/// Continue a completed task with a new message
#[tauri::command]
pub async fn task_continue(
    app: AppHandle,
    task_id: String,
    message: String,
    processes: State<'_, RunningProcesses>,
) -> Result<(), String> {
    println!(">>  task_continue: task_id={}", task_id);
    manager::continue_task(app, task_id, message, processes.inner().clone()).await
}
```

#### Backend: `src-tauri/src/tasks/manager.rs`

Add after `resume_task` function (around line 288):

```rust
/// Continue a completed task with a new message
pub async fn continue_task(
    app: AppHandle,
    task_id: String,
    message: String,
    processes: RunningProcesses,
) -> Result<(), String> {
    let (task, _) = storage::load_task(&task_id)?;

    // Only allow continuing completed tasks
    if task.status != TaskStatus::Completed {
        return Err(format!(
            "Can only continue completed tasks. Current status: {:?}",
            task.status
        ));
    }

    // Check concurrent task limit
    let active_count = storage::get_active_tasks_count(&task.project_path)?;
    if active_count >= MAX_CONCURRENT_TASKS {
        return Err(format!(
            "Maximum concurrent tasks limit reached ({}/{}). Please wait for tasks to complete.",
            active_count, MAX_CONCURRENT_TASKS
        ));
    }

    // Start execution with the new message (reuse existing logic)
    start_task_execution(app, task_id, message, processes).await
}
```

#### Backend: `src-tauri/src/tasks/task.rs`

Update `set_status` method (around line 105):

```rust
/// Update task status and timestamp
pub fn set_status(&mut self, status: TaskStatus) {
    let previous_status = self.status.clone();
    self.status = status.clone();
    self.updated_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    match status {
        TaskStatus::Running => {
            if self.started_at.is_none() {
                self.started_at = Some(self.updated_at);
            }
            // Clear completed_at when continuing from completed state
            if previous_status == TaskStatus::Completed {
                self.completed_at = None;
            }
        }
        TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => {
            self.completed_at = Some(self.updated_at);
        }
        _ => {}
    }
}
```

#### Frontend: `src/lib/tasks-query.ts`

Add after `useResumeTask` function (around line 236):

```typescript
// Continue a completed task
export function useContinueTask() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: async ({
            taskId,
            message,
        }: {
            taskId: string;
            message: string;
        }) => {
            await invoke("task_continue", { taskId, message });
        },
        onSuccess: (_, variables) => {
            queryClient.invalidateQueries({
                queryKey: taskKeys.detail(variables.taskId),
            });
        },
    });
}
```

#### Frontend: `src/components/tasks/TaskDetail.tsx`

Multiple changes needed:

1. **Update imports** (line ~10):
```typescript
import {
    useTask,
    useTaskMessages,
    useStartTask,
    usePauseTask,
    useResumeTask,
    useContinueTask,  // ADD THIS
    useCancelTask,
    useDeleteTask,
    useTaskStream,
    useSaveAssistantMessage,
    type StreamEvent,
} from "@/lib/tasks-query";
```

2. **Add hook** (after line 61):
```typescript
const continueTask = useContinueTask();
```

3. **Update canSendMessage** (line 146-147):
```typescript
const canSendMessage =
    task.status === "pending" || task.status === "paused" || task.status === "completed";
```

4. **Update handleSendMessage** (line 93-107):
```typescript
const handleSendMessage = async (message: string) => {
    if (!task) return;

    try {
        if (task.status === "pending") {
            await startTask.mutateAsync({ taskId, message });
        } else if (task.status === "paused") {
            await resumeTask.mutateAsync({ taskId, message });
        } else if (task.status === "completed") {
            await continueTask.mutateAsync({ taskId, message });
        }
    } catch (error) {
        console.error("Failed to send message:", error);
    }
};
```

5. **Update TaskInput disabled prop and placeholder** (line 237-246):
```typescript
{canSendMessage ? (
    <TaskInput
        onSend={handleSendMessage}
        disabled={startTask.isPending || resumeTask.isPending || continueTask.isPending}
        placeholder={
            task.status === "pending"
                ? "Send a message to start the task..."
                : task.status === "paused"
                ? "Send a message to resume the task..."
                : "Send a message to continue..."
        }
    />
) : (
    // ... rest unchanged
)}
```

---

## 7. Testing Strategy

### 7.1 Unit Tests (Rust)

**File**: `src-tauri/src/tasks/tests.rs` (add to existing)

```rust
#[cfg(test)]
mod continue_task_tests {
    use super::*;

    #[test]
    fn test_set_status_clears_completed_at_on_continue() {
        let mut task = Task::new(
            "test-id".to_string(),
            "/test/path".to_string(),
            "Test".to_string(),
            None,
            None,
        );

        // Complete the task
        task.set_status(TaskStatus::Completed);
        assert!(task.completed_at.is_some());

        // Continue (set to running)
        task.set_status(TaskStatus::Running);
        assert!(task.completed_at.is_none());
    }

    #[tokio::test]
    async fn test_continue_task_validates_status() {
        // Test that only completed tasks can be continued
        // Setup: Create a running task
        // Act: Try to continue it
        // Assert: Error with appropriate message
    }

    #[tokio::test]
    async fn test_continue_task_respects_concurrent_limit() {
        // Test that concurrent limit is checked
        // Setup: Fill up concurrent task slots
        // Act: Try to continue a completed task
        // Assert: Error about concurrent limit
    }
}
```

### 7.2 Integration Tests

```typescript
// tests/task-continue.spec.ts
describe("Continue Completed Task", () => {
    it("should allow continuing a completed task", async () => {
        // 1. Create and complete a task
        // 2. Verify status is "completed"
        // 3. Send a continuation message
        // 4. Verify status changes to "running"
        // 5. Wait for completion
        // 6. Verify new messages in history
    });

    it("should reject continuing non-completed tasks", async () => {
        // 1. Create a pending task
        // 2. Try to continue it
        // 3. Expect error
    });

    it("should respect concurrent task limits", async () => {
        // 1. Start 5 tasks
        // 2. Complete one task
        // 3. Try to continue the completed task
        // 4. Expect error about concurrent limit
    });

    it("should maintain message history", async () => {
        // 1. Create task with initial conversation
        // 2. Complete task
        // 3. Continue with new message
        // 4. Verify all messages preserved
    });
});
```

### 7.3 Manual Testing Checklist

- [ ] Create a new task and complete it
- [ ] Verify "Send a message to continue..." placeholder appears
- [ ] Send a continuation message
- [ ] Verify status badge changes to "Running"
- [ ] Verify streaming response works
- [ ] Verify status returns to "Completed" after response
- [ ] Verify all messages are preserved in history
- [ ] Test continuing multiple times in succession
- [ ] Test with concurrent task limit reached
- [ ] Test UI responsiveness during continuation

---

## 8. Edge Cases

### 8.1 Concurrent Limit Reached

**Scenario**: User tries to continue a completed task when 5 tasks are already running.

**Handling**:
- Backend returns error: "Maximum concurrent tasks limit reached (5/5)"
- Frontend displays error toast
- Task remains in "Completed" status
- User can retry when a slot opens

### 8.2 Multiple Continuations

**Scenario**: User continues a task multiple times.

**Handling**:
- Each continuation adds to the message history
- `completed_at` is cleared on each continuation
- `completed_at` is set again when response completes
- No limit on number of continuations

### 8.3 App Crash During Continuation

**Scenario**: App crashes while continuing a completed task.

**Handling**:
- Task remains in "Running" status (orphaned)
- On next app start, orphaned processes are cleaned up
- User may need to manually update status or delete task

### 8.4 Claude CLI Error During Continuation

**Scenario**: Claude CLI returns an error during continuation.

**Handling**:
- Stream ends with error event
- Task status set to "Failed"
- Error message stored in task
- User can try continuing again (if task is not terminal)

### 8.5 Message History Size

**Scenario**: Very long conversation with many continuations.

**Handling**:
- Current implementation has no limit
- Future consideration: Context window management
- For now: Warn user if message count exceeds threshold

---

## 9. Implementation Phases

### Phase 1: Backend Implementation (3-4 hours)

**Tasks**:
1. Add `continue_task` function to `manager.rs`
2. Add `task_continue` command to `commands.rs`
3. Update `set_status` in `task.rs` to handle continuation
4. Register command in `lib.rs`
5. Add unit tests

**Deliverable**: Backend supports `task_continue` command

### Phase 2: Frontend Implementation (2-3 hours)

**Tasks**:
1. Add `useContinueTask` hook to `tasks-query.ts`
2. Update `TaskDetail.tsx` to use new hook
3. Update `canSendMessage` logic
4. Update placeholder text
5. Test UI flow

**Deliverable**: Frontend allows continuing completed tasks

### Phase 3: Testing & Polish (2-3 hours)

**Tasks**:
1. Manual testing of all scenarios
2. Add integration tests
3. Test edge cases
4. Fix any bugs found
5. Update documentation

**Deliverable**: Feature is production-ready

### Total Estimated Time: 8-12 hours

---

## 10. TODO Checklist

### Backend Tasks

- [ ] Add `continue_task` function in `src-tauri/src/tasks/manager.rs`
- [ ] Add `task_continue` command in `src-tauri/src/tasks/commands.rs`
- [ ] Update `set_status` method in `src-tauri/src/tasks/task.rs`
- [ ] Register `task_continue` in `src-tauri/src/lib.rs`
- [ ] Add unit tests for continuation logic
- [ ] Run `cargo test` to verify

### Frontend Tasks

- [ ] Add `useContinueTask` hook in `src/lib/tasks-query.ts`
- [ ] Update imports in `src/components/tasks/TaskDetail.tsx`
- [ ] Add `continueTask` hook usage in TaskDetail
- [ ] Update `canSendMessage` to include "completed"
- [ ] Update `handleSendMessage` to call `continueTask`
- [ ] Update `disabled` prop to include `continueTask.isPending`
- [ ] Update placeholder text for completed status
- [ ] Run `pnpm tsc --noEmit` to verify

### Testing Tasks

- [ ] Test continuing a completed task
- [ ] Test message history preservation
- [ ] Test multiple continuations
- [ ] Test concurrent limit enforcement
- [ ] Test error handling
- [ ] Verify streaming works correctly
- [ ] Test UI indicators (status badge, placeholder)

### Documentation Tasks

- [ ] Update CLAUDE.md if needed
- [ ] Add inline code comments
- [ ] Update any relevant documentation

---

## Open Questions

1. **Should we add a "continuation count" field to track how many times a task was continued?**
   - Recommendation: Yes, useful for analytics/UX. Add `continuation_count: usize` to Task struct.

2. **Should there be a visual indicator showing the task was continued?**
   - Recommendation: Consider adding a subtle indicator or timestamp showing "Continued at X"

3. **Should we allow continuing Failed/Cancelled tasks?**
   - Recommendation: No for now. These are true terminal states. User should create a new task.

---

**Document Version**: 1.0
**Created**: 2025-12-06
**Last Updated**: 2025-12-06
