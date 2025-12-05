# CC Mate - Code Standards & Organization

**Last Updated**: 2025-12-06
**Version**: 1.0

---

## 1. Code Organization

### 1.1 Directory Structure

```
ccmate/
├── src/                          # React Frontend (TypeScript)
│   ├── main.tsx                  # React app entry point
│   ├── App.tsx                   # Main app router
│   ├── components/
│   │   └── ui/                   # shadcn/ui components
│   ├── lib/
│   │   ├── query.ts              # React Query hooks & API
│   │   └── utils.ts              # Utility functions
│   └── pages/                    # Route pages
│
├── src-tauri/                    # Rust Backend (Tauri)
│   ├── src/
│   │   ├── main.rs               # Tauri app entry point
│   │   ├── lib.rs                # App setup, handler registration
│   │   ├── commands.rs           # Tauri commands (2922 lines)
│   │   └── hook_server.rs        # Hook server implementation
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri.conf.json           # Tauri configuration
│
├── docs/                         # Documentation
│   ├── project-overview-pdr.md   # Project overview & requirements
│   ├── system-architecture.md    # Architecture documentation
│   ├── code-standards.md         # Code standards (this file)
│   ├── codebase-summary.md       # Codebase summary
│   └── deployment-guide.md       # Deployment instructions
│
├── package.json                  # Frontend dependencies
├── tsconfig.json                 # TypeScript configuration
├── vite.config.ts                # Vite build configuration
├── tailwind.config.ts            # Tailwind CSS configuration
├── README.md                      # Project README
└── CONTRIBUTING.md               # Contributing guide

```

### 1.2 Module Responsibilities

**Frontend Modules**:
- `main.tsx` - React app bootstrap, React Query setup
- `App.tsx` - Router configuration and layout
- `lib/query.ts` - All Tauri command wrappers and React Query hooks
- `lib/utils.ts` - Date formatting, validation, type conversions
- `pages/` - Route-specific components
- `components/ui/` - UI components (never edit, auto-generated)

**Backend Modules**:
- `main.rs` - Tauri app entry point, must not be edited
- `lib.rs` - Plugin registration, handler invocation setup
- `commands.rs` - Implementation of all Tauri commands (add new commands here)
- `hook_server.rs` - HTTP server for webhook handling

---

## 2. Naming Conventions

### 2.1 TypeScript/JavaScript

**Files**:
- Components: `PascalCase.tsx` (e.g., `ConfigEditor.tsx`)
- Pages: `PascalCase.tsx` (e.g., `ProjectsPage.tsx`)
- Hooks: `use*.ts` (e.g., `useGetProjectConfigs.ts`)
- Utilities: `camelCase.ts` (e.g., `formatDate.ts`)

**Variables & Functions**:
- Variables: `camelCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Functions: `camelCase`
- React components: `PascalCase`
- Boolean variables: `isLoading`, `hasError`, `canDelete`

**Example**:
```typescript
// Component
export function ProjectConfigEditor() { ... }

// Hook
export function useGetProjectConfig(path: string) { ... }

// Utility
export const formatDatetime = (timestamp: number): string => { ... }

// Constant
const DEFAULT_MODEL = "claude-opus-4-5-20251101";

// Variable
const isLoading = true;
const projectConfigs = [];
```

### 2.2 Rust

**Files**:
- Modules: `snake_case.rs` (e.g., `hook_server.rs`)

**Types & Traits**:
- Structs: `PascalCase` (e.g., `ProjectConfigStore`)
- Enums: `PascalCase` (e.g., `ConfigType`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `APP_CONFIG_DIR`)
- Module constants: `SCREAMING_SNAKE_CASE`

**Functions & Variables**:
- Functions: `snake_case` (e.g., `canonicalize_project_path`)
- Variables: `snake_case` (e.g., `project_path`)
- Tauri commands: `snake_case` (e.g., `get_project_configs`)

**Example**:
```rust
const APP_CONFIG_DIR: &str = ".ccconfig";

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProjectConfigStore {
    pub project_path: String,
}

#[tauri::command]
pub async fn get_project_configs() -> Result<Vec<ProjectConfigStore>, String> {
    let config_dir = get_project_configs_dir()?;
    // Implementation...
}

fn canonicalize_project_path(path: &str) -> Result<String, String> {
    // Implementation...
}
```

---

## 3. TypeScript Code Standards

### 3.1 Strict Mode

**Required**: Always use TypeScript strict mode.

**tsconfig.json**:
```json
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true,
    "strictFunctionTypes": true
  }
}
```

### 3.2 Type Annotations

**Always provide type annotations**:
```typescript
// Good
const count: number = 5;
const names: string[] = ["Alice", "Bob"];
const config: ProjectConfigStore = { ... };
const createProject = (path: string): Promise<void> => { ... };

// Bad - avoid any
const count: any = 5;
const doSomething = (data: any) => { ... };
```

### 3.3 Functional Components

**Always use functional components**, never class components.

```typescript
// Good
export function ConfigEditor() {
  const [title, setTitle] = useState("");
  return <input value={title} onChange={e => setTitle(e.target.value)} />;
}

// Bad - avoid class components
export class ConfigEditor extends React.Component {
  render() { ... }
}
```

### 3.4 JSDoc Comments

**Required for public APIs**:

```typescript
/**
 * Merge two configuration objects.
 * Project settings override global settings.
 * Permission arrays are unioned.
 *
 * @param global - Global configuration object
 * @param project - Project-specific overrides
 * @returns Merged configuration
 *
 * @example
 * const merged = mergeConfigs(globalConfig, projectConfig);
 */
export function mergeConfigs(global: Value, project: Value): Value {
  // Implementation...
}

/**
 * Hook to fetch project configurations.
 *
 * @returns Query object with project configs data, loading state, and errors
 */
export function useGetProjectConfigs() {
  return useQuery({
    queryKey: ['project-configs'],
    queryFn: async () => invoke('get_project_configs'),
  });
}
```

### 3.5 React Hooks & Query

**Frontend Data Layer** (Phase 2 - 876 lines, 46 hooks):

All API calls wrapped in React Query for:
- **Query hooks** (read operations): useQuery, useSuspenseQuery
- **Mutation hooks** (write operations): useMutation with onSuccess/onError callbacks
- **Query invalidation**: Automatically refetch related data after mutations
- **Error handling**: Toast notifications on success/error
- **Loading states**: Managed by React Query

**Pattern 1: Simple Query Hook**:

```typescript
// src/lib/query.ts
export function useProjectConfigs() {
  return useQuery({
    queryKey: ['project-configs'],
    queryFn: () => invoke<ProjectConfigStore[]>('get_project_configs'),
  });
}

// In component
const { data: configs, isLoading, error } = useProjectConfigs();
```

**Pattern 2: Query Hook with Parameters**:

```typescript
export function useProjectConfig(projectPath: string) {
  return useQuery({
    queryKey: ['project-config', projectPath],
    queryFn: () =>
      invoke<ProjectConfigStore | null>('get_project_config', { projectPath }),
    enabled: !!projectPath,  // Only run when projectPath is provided
  });
}
```

**Pattern 3: Mutation Hook with Invalidation**:

```typescript
export function useUpdateProjectConfig() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      projectPath,
      title,
      settings,
    }: {
      projectPath: string;
      title: string;
      settings: unknown;
    }) =>
      invoke<ProjectConfigStore>('update_project_config', {
        projectPath,
        title,
        settings,
      }),
    onSuccess: (data) => {
      toast.success(i18n.t('toast.projectConfigSaved', { title: data.title }));
      // Invalidate related queries to refetch
      queryClient.invalidateQueries({ queryKey: ['project-configs'] });
      queryClient.invalidateQueries({
        queryKey: ['project-config', data.projectPath],
      });
      queryClient.invalidateQueries({ queryKey: ['active-context'] });
    },
    onError: (error) => {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      toast.error(
        i18n.t('toast.projectConfigSaveFailed', { error: errorMessage }),
      );
    },
  });
}
```

**Pattern 4: Suspense Query** (for guaranteed data):

```typescript
export const useStores = () => {
  return useSuspenseQuery({
    queryKey: ['stores'],
    queryFn: () => invoke<ConfigStore[]>('get_stores'),
  });
};

// In component with Suspense boundary
const { data: stores } = useStores();  // Data is guaranteed, no loading check needed
```

**Usage in components**:

```typescript
export function ProjectsPage() {
  const { data: configs, isLoading, error } = useProjectConfigs();

  if (isLoading) return <div>Loading...</div>;
  if (error) return <div>Error: {error.message}</div>;

  return (
    <div>
      {configs?.map(config => (
        <ProjectConfigCard key={config.id} config={config} />
      ))}
    </div>
  );
}
```

**Hook Categories** (in query.ts):
- **Global Config Hooks** (8): useStores, useCreateConfig, useUpdateConfig, etc.
- **Project Config Hooks** (11): useProjectConfigs, useCreateProjectConfig, useActivateProjectConfig, etc.
- **MCP Server Hooks** (5): useGlobalMcpServers, useUpdateGlobalMcpServer, etc.
- **Memory & Commands** (6): useClaudeMemory, useClaudeCommands, useClaudeAgents, etc.
- **Config Files** (3): useConfigFiles, useConfigFile, useWriteConfigFile
- **Notifications & Misc** (13): useNotificationSettings, useCheckForUpdates, etc.

**Total**: 46 React Query hooks covering all Tauri command bindings

### 3.6 Form Handling

**Use React Hook Form + Zod**:

```typescript
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';

const projectConfigSchema = z.object({
  title: z.string().min(1, "Title required"),
  model: z.string().min(1, "Model required"),
  permissions: z.object({
    deny: z.array(z.string()),
    allow: z.array(z.string()),
  }).optional(),
});

type ProjectConfigFormData = z.infer<typeof projectConfigSchema>;

export function ProjectConfigForm() {
  const { register, handleSubmit, formState: { errors } } = useForm<ProjectConfigFormData>({
    resolver: zodResolver(projectConfigSchema),
  });

  const onSubmit = (data: ProjectConfigFormData) => {
    // Handle form submission
  };

  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      <input {...register('title')} />
      {errors.title && <span>{errors.title.message}</span>}
      {/* More fields... */}
    </form>
  );
}
```

### 3.7 Error Handling

**Always handle errors explicitly**:

```typescript
export async function loadConfigs(): Promise<ProjectConfigStore[]> {
  try {
    const configs = await invoke<ProjectConfigStore[]>('get_project_configs');
    return configs;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    console.error(`Failed to load configs: ${message}`);
    throw new Error(`Failed to load configurations: ${message}`);
  }
}
```

---

## 4. Rust Code Standards

### 4.1 Module Organization

**commands.rs organization** (2922 lines):

```rust
// Section 1: Imports & Constants
use ...;
const APP_CONFIG_DIR: &str = ".ccconfig";

// Section 2: Data Structures
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ConfigStore { ... }

// Section 3: Tauri Commands
#[tauri::command]
pub async fn get_stores() -> Result<Vec<ConfigStore>, String> { ... }

// Section 4: Helper Functions
fn merge_settings(global: &Value, project: &Value) -> Value { ... }
```

### 4.2 Error Handling

**All commands return `Result<T, String>`**:

```rust
#[tauri::command]
pub async fn get_project_config(project_path: String) -> Result<Option<ProjectConfigStore>, String> {
    read_project_config_file(&project_path)
        .map_err(|e| format!("Failed to read project config: {}", e))
}
```

**Provide helpful error messages**:

```rust
// Good - specific error message
fn canonicalize_project_path(path: &str) -> Result<String, String> {
    std::fs::canonicalize(path)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("Failed to canonicalize path '{}': {}", path, e))
}

// Bad - generic error
.map_err(|_| "Error".to_string())
```

### 4.3 Async/Await

**All I/O operations must be async**:

```rust
// Good
#[tauri::command]
pub async fn create_project_config(
    project_path: String,
    title: String,
    settings: Value,
) -> Result<ProjectConfigStore, String> {
    // Implementation
}

// Bad - sync I/O in async function is blocking
pub async fn create_project_config(...) -> Result<ProjectConfigStore, String> {
    std::thread::sleep(Duration::from_secs(1)); // Blocks event loop!
}
```

### 4.4 JSON Handling

**Use serde for serialization**:

```rust
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ProjectConfigStore {
    #[serde(rename = "projectPath")]
    pub project_path: String,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
    pub settings: Value,
}

// Serialization
let json = serde_json::to_string_pretty(&config)?;

// Deserialization
let config: ProjectConfigStore = serde_json::from_str(&content)?;
```

### 4.5 File Operations

**Standard pattern for file I/O**:

```rust
// Reading
let content = std::fs::read_to_string(&path)
    .map_err(|e| format!("Failed to read file: {}", e))?;
let data: MyType = serde_json::from_str(&content)
    .map_err(|e| format!("Failed to parse JSON: {}", e))?;

// Writing
std::fs::create_dir_all(&dir)
    .map_err(|e| format!("Failed to create directory: {}", e))?;
let json = serde_json::to_string_pretty(&data)?;
std::fs::write(&path, json)
    .map_err(|e| format!("Failed to write file: {}", e))?;
```

### 4.6 Documentation

**Document public functions**:

```rust
/// Canonicalize project path (resolve symlinks, normalize).
///
/// # Arguments
/// * `path` - The project path to canonicalize
///
/// # Returns
/// Canonical path as string, or error if path doesn't exist
fn canonicalize_project_path(path: &str) -> Result<String, String> {
    std::fs::canonicalize(path)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("Failed to canonicalize path: {}", e))
}
```

---

## 5. API Command Structure

### 5.1 Adding New Commands

**Pattern for new Tauri command**:

```rust
/// Description of what this command does.
///
/// Returns the config or error message.
#[tauri::command]
pub async fn my_new_command(
    param1: String,
    param2: u64,
) -> Result<ReturnType, String> {
    // 1. Validate inputs
    if param1.is_empty() {
        return Err("param1 cannot be empty".to_string());
    }

    // 2. Perform operations
    let result = perform_operation(&param1, param2)?;

    // 3. Return result
    Ok(result)
}
```

**Register in lib.rs**:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    my_new_command,  // Add here
])
```

### 5.2 Command Naming

**Rules**:
- Use snake_case
- Start with action verb: `get_`, `create_`, `update_`, `delete_`, `set_`
- Be descriptive about what you're operating on
- Avoid acronyms unless very clear

**Examples**:
- `get_project_configs()` - Retrieve configs
- `create_project_config()` - Create new
- `update_project_config()` - Modify existing
- `delete_project_config()` - Remove
- `activate_project_config()` - Switch to
- `get_managed_settings()` - Fetch read-only enterprise

---

## 6. Configuration & Settings Management

### 6.1 Adding New Config Fields

**In ProjectConfigStore**:

```rust
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProjectConfigStore {
    // Existing fields...

    #[serde(rename = "newField")]
    pub new_field: String,  // Always use serde rename for camelCase in JSON
}
```

**In merge logic** (if special handling needed):

```rust
fn merge_settings(global: &Value, project: &Value) -> Value {
    // ... existing code ...

    // Special handling for specific fields
    if key == "newField" {
        // Custom merge logic
    }
}
```

### 6.2 Updating Frontend Hooks

**When adding new Tauri command**:

1. Add command to backend (commands.rs)
2. Register in lib.rs
3. Create React Query hook in src/lib/query.ts
4. Use hook in components

```typescript
// src/lib/query.ts
export function useMyNewCommand(param: string) {
  return useQuery({
    queryKey: ['my-data', param],
    queryFn: async () => invoke('my_new_command', { param }),
  });
}
```

---

## 7. Testing Standards

### 7.1 Unit Tests (Frontend)

**Location**: Colocated with source files as `.test.ts` or `.test.tsx`

```typescript
import { describe, it, expect } from 'vitest';
import { mergeConfigs } from './utils';

describe('mergeConfigs', () => {
  it('should override global with project settings', () => {
    const global = { model: 'opus' };
    const project = { model: 'sonnet' };
    const result = mergeConfigs(global, project);
    expect(result.model).toBe('sonnet');
  });

  it('should union permission arrays', () => {
    const global = { permissions: { deny: ['Read(/.env)'] } };
    const project = { permissions: { deny: ['Execute(npm)'] } };
    const result = mergeConfigs(global, project);
    expect(result.permissions.deny).toContain('Read(/.env)');
    expect(result.permissions.deny).toContain('Execute(npm)');
  });
});
```

### 7.2 Unit Tests (Backend)

**In Rust** (optional, at bottom of commands.rs):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonicalize_project_path() {
        let result = canonicalize_project_path(".");
        assert!(result.is_ok());
    }

    #[test]
    fn test_hash_project_path() {
        let hash1 = hash_project_path("/path/to/project").unwrap();
        let hash2 = hash_project_path("/path/to/project").unwrap();
        assert_eq!(hash1, hash2);
    }
}
```

### 7.3 E2E Tests

**Using Playwright**:

```typescript
// e2e/project-config.spec.ts
import { test, expect } from '@playwright/test';

test('should create and activate project config', async ({ page }) => {
  // Setup
  await page.goto('/');

  // Create config
  await page.click('button:has-text("New Project Config")');
  await page.fill('input[name="path"]', '/path/to/project');
  await page.click('button:has-text("Create")');

  // Verify
  await expect(page.locator('text=/path/to/project')).toBeVisible();

  // Activate
  await page.click('button:has-text("Activate")');
  await expect(page.locator('text=Active Context')).toContainText('project');
});
```

### 7.4 Code Coverage

**Target**: >80% coverage

```bash
# Run tests with coverage
pnpm test:coverage

# View coverage report
open coverage/index.html
```

---

## 8. Performance Guidelines

### 8.1 Frontend Performance

**Avoid**:
- Unnecessary re-renders (use useMemo, useCallback)
- Large bundles (use code splitting with React.lazy)
- Blocking operations (all I/O must be async)
- Inline styles (use Tailwind CSS)

**Do**:
- Use React Query for caching
- Paginate large lists
- Lazy-load images
- Use Vite dev server for fast HMR

### 8.2 Backend Performance

**Avoid**:
- Synchronous file I/O
- Unnecessary cloning of large values
- Nested loops in config merging
- Excessive logging in production

**Do**:
- Use async/await for I/O
- Cache frequently accessed paths
- Optimize merge algorithm for large settings
- Log errors, not all operations

---

## 9. Security Guidelines

### 9.1 Input Validation

**Always validate user input**:

```typescript
// Validate path
const isValidPath = (path: string): boolean => {
  return path.length > 0 && !path.includes('..') && !path.includes('~');
};

// Validate JSON
const parseSettings = (json: string): Value | Error => {
  try {
    return JSON.parse(json);
  } catch (e) {
    return new Error(`Invalid JSON: ${e.message}`);
  }
};
```

### 9.2 Path Security

**Canonicalize all paths**:

```rust
fn canonicalize_project_path(path: &str) -> Result<String, String> {
    std::fs::canonicalize(path)  // Resolves symlinks, prevents ../ traversal
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("Invalid path: {}", e))
}
```

### 9.3 Secret Handling

**Never**:
- Log secrets
- Commit .env files
- Store API keys in code

**Do**:
- Use environment variables
- Mask sensitive values in logs
- Request from secure sources only

---

## 10. Documentation Guidelines

### 10.1 Code Comments

**Rules**:
- Explain "why", not "what"
- Use JSDoc for public APIs
- Keep comments in sync with code
- Avoid redundant comments

**Good**:
```rust
/// Union permission arrays to ensure strictest permissions apply.
/// Project restrictions are combined with global restrictions.
if perm_key == "deny" {
    // Merge arrays (no duplicates)
}
```

**Bad**:
```rust
// Add the array
arr.push(item);  // This comments the obvious

// This is a function
pub fn my_func() { }  // Obvious from signature
```

### 10.2 Commit Messages

**Format**:
```
<type>: <short description>

<optional longer description>

Fixes #<issue_number> (if applicable)
```

**Types**:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation
- `refactor:` - Code reorganization
- `perf:` - Performance improvement
- `test:` - Test changes

**Examples**:
```
feat: Add project config merge with permission union

Implement deep merge logic that unions permission.deny and
permission.allow arrays to ensure strictest permissions apply.

refactor: Extract path canonicalization to helper function

fix: Handle missing parent global config in merge

docs: Update architecture documentation for Phase 1
```

---

## 11. Code Review Checklist

Before submitting code:

**TypeScript**:
- [ ] TypeScript strict mode passes
- [ ] No `any` types (except proven necessary)
- [ ] JSDoc comments on public APIs
- [ ] Error handling in try/catch blocks
- [ ] React Query hooks for all API calls
- [ ] No class components
- [ ] Proper key prop on list items

**Rust**:
- [ ] All I/O operations are async
- [ ] Errors have descriptive messages
- [ ] No `unwrap()` without context (use `?` operator)
- [ ] Documentation on public functions
- [ ] Proper serialization with serde rename
- [ ] No blocking operations

**General**:
- [ ] Tests added/updated
- [ ] >80% code coverage maintained
- [ ] No console.log() in production code
- [ ] No hardcoded paths or secrets
- [ ] Naming conventions followed
- [ ] Performance implications considered

---

## 12. Development Workflow

### 12.1 Setup Environment

```bash
# Install node dependencies
pnpm install

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify setup
pnpm tauri dev
```

### 12.2 Development Commands

```bash
# Start dev server (hot reload)
pnpm tauri dev

# Type check frontend
pnpm tsc --noEmit

# Run tests
pnpm test

# Build for release
pnpm build

# Run E2E tests
pnpm test:e2e
```

### 12.3 Git Workflow

```bash
# Create feature branch
git checkout -b feat/my-feature

# Make changes, commit regularly
git commit -m "feat: add new feature"

# Push and create PR
git push origin feat/my-feature

# After approval and passing tests, merge
git merge feat/my-feature
```

---

## Appendix: Tools & Dependencies

### Development Tools

- **Node.js**: v18+ (LTS recommended)
- **Rust**: Latest stable (from rustup)
- **VSCode Extensions**:
  - rust-analyzer
  - TypeScript Vue Plugin (Volar)
  - Tailwind CSS IntelliSense
  - ESLint
  - Prettier

### Frontend Dependencies

```json
{
  "react": "19",
  "react-router-dom": "^6",
  "@tanstack/react-query": "^5",
  "react-hook-form": "^7",
  "zod": "^3",
  "tailwindcss": "^4",
  "@tauri-apps/api": "^2",
  "shadcn-ui": "latest"
}
```

### Build Tools

```json
{
  "vite": "^5",
  "typescript": "^5",
  "vitest": "^1",
  "@playwright/test": "^1"
}
```

---

**Last updated**: 2025-12-06
**Maintained by**: Development Team
