# Project Selector Screen Implementation Plan

**Date**: 2025-12-06
**Status**: PLANNING
**Estimate**: 4-6 hours
**Priority**: HIGH (User Feedback - Blocking Feature Usage)

---

## Overview

Add an intermediate "Project Selector" screen to the Project Configs feature that allows users to:
1. View ALL projects from `~/.claude.json`
2. See status of each project (has config, has local settings, no config)
3. Create, import, or manage configs for each project
4. Navigate to project config editor

**Current Problem:**
- `ProjectConfigsPage` shows empty state with no way to create configs
- User cannot SELECT which project to create config for
- Missing connection between Claude projects list and project configs

**User Request (Vietnamese):**
"Should add an intermediate screen to select project config or old ccmate config, including project configs in `~/.claude.json`"

---

## Requirements Analysis

### Functional Requirements

1. **Display All Projects**: List all projects from `~/.claude.json` via `useClaudeProjects()`
2. **Show Config Status**: For each project, display one of:
   - Has CC Mate config (in `~/.ccconfig/project-configs/`)
   - Has local `.claude/settings.json` (can import)
   - No config (can create from global)
3. **Actions Per Project**:
   - Create config (from active global)
   - Import config (from local settings)
   - Edit existing config
   - Activate config
   - Delete config
4. **Empty States**: Handle no projects, no configs gracefully
5. **Search/Filter**: Allow filtering projects by name/path

### Non-Functional Requirements

1. **Performance**: Handle 100+ projects smoothly
2. **Consistency**: Follow existing UI patterns (ConfigSwitcherPage grid)
3. **Accessibility**: Keyboard navigation, screen reader support
4. **i18n**: All strings translatable

---

## Data Architecture

### Data Sources

```typescript
// Source 1: All Claude projects (from ~/.claude.json)
const { data: claudeProjects } = useClaudeProjects();
// Returns: ProjectConfig[] where ProjectConfig = { path: string; config: Record<string, any> }

// Source 2: Existing CC Mate project configs
const { data: projectConfigs } = useProjectConfigs();
// Returns: ProjectConfigStore[] with id, projectPath, title, settings, etc.

// Source 3: Active context (which config is active)
const { data: activeContext } = useActiveContext();
// Returns: { type: 'global' | 'project', id: string, projectPath: string | null }
```

### Merged Data Model

```typescript
interface ProjectWithStatus {
  // From Claude projects
  path: string;
  claudeConfig: Record<string, any>; // Original config from ~/.claude.json

  // Computed status
  status: 'has_ccmate_config' | 'has_local_settings' | 'no_config';

  // From CC Mate (if exists)
  ccmateConfig?: ProjectConfigStore;

  // Local settings check (lazy loaded)
  hasLocalSettings?: boolean;
  localSettingsContent?: unknown;

  // Computed
  isActive: boolean;
  displayName: string; // Last folder segment or full path
}
```

### Data Flow Diagram

```
                                   +-----------------------+
                                   |   ~/.claude.json      |
                                   |   (Claude Projects)   |
                                   +-----------+-----------+
                                               |
                                    useClaudeProjects()
                                               |
                                               v
+---------------------------+      +-----------+-----------+
| ~/.ccconfig/project-      |      |                       |
| configs/*.json            +----->|   ProjectSelector     |
| (CC Mate Project Configs) |      |   Component           |
+---------------------------+      |                       |
         ^                         |   Merges data into    |
         |                         |   ProjectWithStatus[] |
  useProjectConfigs()              |                       |
                                   +-----------+-----------+
                                               |
                                               v
                            +------------------+------------------+
                            |                                     |
              +-------------+-------------+        +--------------+--------------+
              |  For each project:        |        |  User Actions:              |
              |  - Check hasLocalSettings |        |  - Create (autoCreate)      |
              |  - Match with ccmateConfig|        |  - Import (importLocal)     |
              |  - Compute isActive       |        |  - Edit (navigate)          |
              +---------------------------+        |  - Activate (activate)      |
                                                   |  - Delete (delete)          |
                                                   +-----------------------------+
```

---

## UI/UX Design

### Screen Layout

```
+----------------------------------------------------------------+
| Project Configs                                    [+ Add New]  |
| Select a project to manage its Claude Code configuration        |
+----------------------------------------------------------------+
| [Search projects...]                          [Filter: All v]   |
+----------------------------------------------------------------+
|                                                                 |
| +---------------------------+ +---------------------------+     |
| | /Users/me/project-a       | | /Users/me/project-b       |     |
| | project-a                 | | project-b                 |     |
| | +---------------------+   | | +---------------------+   |     |
| | |  CC Mate Config     |   | |  Local Settings      |   |     |
| | |  Based on: Work     |   | |  .claude/settings.json|   |     |
| | +---------------------+   | | +---------------------+   |     |
| | [Edit] [Activate] [...]   | | [Import] [Create] [...]   |     |
| +---------------------------+ +---------------------------+     |
|                                                                 |
| +---------------------------+ +---------------------------+     |
| | /Users/me/project-c       | | /Users/me/project-d       |     |
| | project-c           [*]   | | project-d                 |     |
| | +---------------------+   | | +---------------------+   |     |
| | |  CC Mate Config     |   | |  No Config            |   |     |
| | |  Active             |   | |                       |   |     |
| | +---------------------+   | | +---------------------+   |     |
| | [Edit] [Deactivate] [...] | | [Create from Global]      |     |
| +---------------------------+ +---------------------------+     |
|                                                                 |
+----------------------------------------------------------------+
| Showing 4 of 12 projects                                        |
+----------------------------------------------------------------+
```

### Status Badges

| Status | Badge | Color | Icon |
|--------|-------|-------|------|
| Has CC Mate Config | "CC Mate Config" | Primary/Blue | CheckCircle2 |
| Has Local Settings | "Local Settings" | Warning/Yellow | FileJson |
| No Config | "No Config" | Muted/Gray | CircleDashed |
| Active | Star overlay | Gold | Star |

### Card Actions Matrix

| Status | Primary Action | Secondary Actions |
|--------|----------------|-------------------|
| Has CC Mate Config | Edit | Activate, Delete, View Merged |
| Has CC Mate Config (Active) | Edit | Deactivate (switch to global), Delete |
| Has Local Settings | Import | Create from Global, View Local |
| No Config | Create from Global | - |

### Empty States

**No Projects Found:**
```
+----------------------------------------------------------------+
|                                                                 |
|                    [FolderOpen Icon]                            |
|                                                                 |
|                   No Projects Found                             |
|                                                                 |
|    There are no Claude projects configured. Projects appear     |
|    here when you use Claude Code in different project folders.  |
|                                                                 |
|                   [Open Project Folder]                         |
|                                                                 |
+----------------------------------------------------------------+
```

**All Projects Have No Config:**
```
+----------------------------------------------------------------+
|                                                                 |
|                     [Sparkles Icon]                             |
|                                                                 |
|            Get Started with Project Configs                     |
|                                                                 |
|    Create per-project configurations to customize Claude Code   |
|    behavior for each of your projects.                          |
|                                                                 |
|    Select a project below to create its first config.           |
|                                                                 |
+----------------------------------------------------------------+
```

---

## Component Architecture

### File Structure

```
src/pages/
  ProjectConfigsPage.tsx        # Main page (UPDATE - add Project Selector)
  ProjectConfigEditor.tsx       # Editor page (EXISTS)

src/components/project-configs/
  ProjectCard.tsx               # Individual project card (NEW)
  ProjectStatusBadge.tsx        # Status indicator component (NEW)
  ProjectActionsMenu.tsx        # Dropdown menu for actions (NEW)
  ProjectSearchFilter.tsx       # Search and filter controls (NEW)

src/lib/
  query.ts                      # React Query hooks (EXISTS - may need updates)
  project-utils.ts              # Helper functions for project data (NEW)
```

### Component Hierarchy

```
ProjectConfigsPage
├── Header
│   ├── Title + Description
│   └── Add New Button (folder picker)
├── ProjectSearchFilter
│   ├── Search Input
│   └── Filter Dropdown (All/Has Config/No Config/Has Local)
├── ProjectGrid
│   └── ProjectCard (x N)
│       ├── ProjectStatusBadge
│       ├── Project Info (name, path)
│       └── ProjectActionsMenu
│           ├── Edit
│           ├── Activate/Deactivate
│           ├── Import (if has local)
│           ├── Create (if no config)
│           └── Delete (if has config)
└── Footer (project count)
```

### State Management

```typescript
// Page-level state
const [searchQuery, setSearchQuery] = useState('');
const [statusFilter, setStatusFilter] = useState<'all' | 'has_config' | 'no_config' | 'has_local'>('all');

// Derived data
const projectsWithStatus = useMemo(() => {
  return mergeProjectData(claudeProjects, projectConfigs, activeContext);
}, [claudeProjects, projectConfigs, activeContext]);

const filteredProjects = useMemo(() => {
  return filterProjects(projectsWithStatus, searchQuery, statusFilter);
}, [projectsWithStatus, searchQuery, statusFilter]);
```

---

## Implementation Details

### 1. ProjectCard Component

**File**: `src/components/project-configs/ProjectCard.tsx`

```typescript
interface ProjectCardProps {
  project: ProjectWithStatus;
  onEdit: (path: string) => void;
  onActivate: (path: string) => void;
  onDeactivate: () => void;
  onImport: (path: string) => void;
  onCreate: (path: string) => void;
  onDelete: (path: string) => void;
}

export function ProjectCard({ project, ...handlers }: ProjectCardProps) {
  const { t } = useTranslation();

  return (
    <Card className={cn(
      "p-4 flex flex-col gap-3 transition-colors",
      project.isActive && "border-primary border-2 bg-primary/5"
    )}>
      {/* Header: Name + Active indicator */}
      <div className="flex items-start justify-between">
        <div className="flex-1 min-w-0">
          <h4 className="font-medium flex items-center gap-2">
            <FolderIcon className="h-4 w-4 shrink-0" />
            <span className="truncate">{project.displayName}</span>
            {project.isActive && (
              <Star className="h-4 w-4 text-yellow-500 fill-yellow-500" />
            )}
          </h4>
          <p className="text-xs text-muted-foreground truncate mt-1" title={project.path}>
            {project.path}
          </p>
        </div>
      </div>

      {/* Status Badge */}
      <ProjectStatusBadge status={project.status} config={project.ccmateConfig} />

      {/* Actions */}
      <div className="flex items-center gap-2 pt-2 mt-auto">
        {/* Primary action based on status */}
        {project.status === 'has_ccmate_config' && (
          <Button size="sm" variant="outline" onClick={() => handlers.onEdit(project.path)}>
            Edit
          </Button>
        )}
        {project.status === 'has_local_settings' && (
          <Button size="sm" variant="default" onClick={() => handlers.onImport(project.path)}>
            <Import className="h-4 w-4 mr-1" />
            Import
          </Button>
        )}
        {project.status === 'no_config' && (
          <Button size="sm" variant="default" onClick={() => handlers.onCreate(project.path)}>
            <Plus className="h-4 w-4 mr-1" />
            Create
          </Button>
        )}

        {/* Secondary actions in dropdown */}
        <ProjectActionsMenu project={project} {...handlers} />
      </div>
    </Card>
  );
}
```

### 2. ProjectStatusBadge Component

**File**: `src/components/project-configs/ProjectStatusBadge.tsx`

```typescript
interface ProjectStatusBadgeProps {
  status: ProjectWithStatus['status'];
  config?: ProjectConfigStore;
}

export function ProjectStatusBadge({ status, config }: ProjectStatusBadgeProps) {
  return (
    <div className={cn(
      "flex items-center gap-2 px-3 py-2 rounded-md text-sm",
      status === 'has_ccmate_config' && "bg-primary/10 text-primary",
      status === 'has_local_settings' && "bg-yellow-500/10 text-yellow-600",
      status === 'no_config' && "bg-muted text-muted-foreground"
    )}>
      {status === 'has_ccmate_config' && (
        <>
          <CheckCircle2 className="h-4 w-4" />
          <div>
            <span className="font-medium">CC Mate Config</span>
            {config?.inheritFromGlobal && (
              <span className="text-xs ml-2 opacity-70">Inheriting from global</span>
            )}
          </div>
        </>
      )}
      {status === 'has_local_settings' && (
        <>
          <FileJson className="h-4 w-4" />
          <span>Local .claude/settings.json detected</span>
        </>
      )}
      {status === 'no_config' && (
        <>
          <CircleDashed className="h-4 w-4" />
          <span>No configuration</span>
        </>
      )}
    </div>
  );
}
```

### 3. Helper Functions

**File**: `src/lib/project-utils.ts`

```typescript
import type { ProjectConfig, ProjectConfigStore, ActiveContext } from './query';

export interface ProjectWithStatus {
  path: string;
  claudeConfig: Record<string, any>;
  status: 'has_ccmate_config' | 'has_local_settings' | 'no_config';
  ccmateConfig?: ProjectConfigStore;
  isActive: boolean;
  displayName: string;
}

/**
 * Merge data from Claude projects and CC Mate configs
 */
export function mergeProjectData(
  claudeProjects: ProjectConfig[] | undefined,
  projectConfigs: ProjectConfigStore[] | undefined,
  activeContext: ActiveContext | null | undefined
): ProjectWithStatus[] {
  if (!claudeProjects) return [];

  const configMap = new Map<string, ProjectConfigStore>();
  projectConfigs?.forEach(config => {
    configMap.set(config.projectPath, config);
  });

  return claudeProjects.map(project => {
    const ccmateConfig = configMap.get(project.path);
    const isActive = activeContext?.type === 'project' && activeContext?.projectPath === project.path;
    const displayName = project.path.split('/').pop() || project.path;

    // Determine status
    let status: ProjectWithStatus['status'] = 'no_config';
    if (ccmateConfig) {
      status = 'has_ccmate_config';
    }
    // Note: has_local_settings is determined by lazy-loaded check

    return {
      path: project.path,
      claudeConfig: project.config,
      status,
      ccmateConfig,
      isActive,
      displayName
    };
  });
}

/**
 * Filter projects by search query and status
 */
export function filterProjects(
  projects: ProjectWithStatus[],
  searchQuery: string,
  statusFilter: 'all' | 'has_config' | 'no_config' | 'has_local'
): ProjectWithStatus[] {
  return projects.filter(project => {
    // Search filter
    const matchesSearch = !searchQuery ||
      project.path.toLowerCase().includes(searchQuery.toLowerCase()) ||
      project.displayName.toLowerCase().includes(searchQuery.toLowerCase());

    // Status filter
    let matchesStatus = true;
    if (statusFilter === 'has_config') {
      matchesStatus = project.status === 'has_ccmate_config';
    } else if (statusFilter === 'no_config') {
      matchesStatus = project.status === 'no_config';
    } else if (statusFilter === 'has_local') {
      matchesStatus = project.status === 'has_local_settings';
    }

    return matchesSearch && matchesStatus;
  });
}

/**
 * Get display name from project path
 */
export function getProjectDisplayName(path: string): string {
  return path.split('/').pop() || path;
}
```

### 4. Updated ProjectConfigsPage

**File**: `src/pages/ProjectConfigsPage.tsx` (REPLACE EXISTING)

```typescript
import { useState, useMemo } from 'react';
import { ask, open } from "@tauri-apps/plugin-dialog";
import {
  FolderIcon, FolderOpenIcon, PlusIcon, SearchIcon,
  SparklesIcon, Loader2
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ProjectCard } from "@/components/project-configs/ProjectCard";
import {
  mergeProjectData,
  filterProjects,
  type ProjectWithStatus
} from "@/lib/project-utils";
import {
  useActivateProjectConfig,
  useActiveContext,
  useAutoCreateProjectConfig,
  useCheckProjectLocalSettings,
  useClaudeProjects,
  useDeleteProjectConfig,
  useImportProjectLocalSettings,
  useProjectConfigs,
  useSwitchToGlobalContext,
} from "../lib/query";

export function ProjectConfigsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();

  // Data queries
  const { data: claudeProjects, isLoading: isLoadingProjects } = useClaudeProjects();
  const { data: projectConfigs, isLoading: isLoadingConfigs } = useProjectConfigs();
  const { data: activeContext } = useActiveContext();

  // Mutations
  const deleteProjectConfig = useDeleteProjectConfig();
  const activateProjectConfig = useActivateProjectConfig();
  const importProjectLocalSettings = useImportProjectLocalSettings();
  const autoCreateProjectConfig = useAutoCreateProjectConfig();
  const switchToGlobalContext = useSwitchToGlobalContext();

  // Local state
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState<'all' | 'has_config' | 'no_config'>('all');

  // Merge and filter data
  const projectsWithStatus = useMemo(() => {
    return mergeProjectData(claudeProjects, projectConfigs, activeContext);
  }, [claudeProjects, projectConfigs, activeContext]);

  const filteredProjects = useMemo(() => {
    return filterProjects(projectsWithStatus, searchQuery, statusFilter);
  }, [projectsWithStatus, searchQuery, statusFilter]);

  // Handlers
  const handleEdit = (projectPath: string) => {
    navigate(`/project-configs/${encodeURIComponent(projectPath)}`);
  };

  const handleActivate = (projectPath: string) => {
    activateProjectConfig.mutate(projectPath);
  };

  const handleDeactivate = () => {
    // Switch to first global config or original
    switchToGlobalContext.mutate('');
  };

  const handleImport = (projectPath: string) => {
    importProjectLocalSettings.mutate(projectPath);
  };

  const handleCreate = (projectPath: string) => {
    autoCreateProjectConfig.mutate(projectPath, {
      onSuccess: (data) => {
        // Navigate to editor after creation
        navigate(`/project-configs/${encodeURIComponent(data.projectPath)}`);
      }
    });
  };

  const handleDelete = async (projectPath: string, title: string) => {
    const confirmed = await ask(
      t("projectConfigs.deleteConfirm", { name: title }),
      { title: t("projectConfigs.deleteTitle"), kind: "warning" }
    );
    if (confirmed) {
      deleteProjectConfig.mutate(projectPath);
    }
  };

  const handleAddNewProject = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t("projectConfigs.selectProjectFolder")
    });
    if (selected) {
      // Create config for the selected folder
      handleCreate(selected as string);
    }
  };

  // Loading state
  if (isLoadingProjects || isLoadingConfigs) {
    return (
      <div className="">
        <Header onAddNew={handleAddNewProject} />
        <div className="flex items-center justify-center py-16">
          <Loader2 className="h-6 w-6 animate-spin mr-2" />
          <span className="text-muted-foreground">{t("loading")}</span>
        </div>
      </div>
    );
  }

  // Empty state: No Claude projects at all
  if (!claudeProjects || claudeProjects.length === 0) {
    return (
      <div className="">
        <Header onAddNew={handleAddNewProject} />
        <div className="flex flex-col items-center justify-center py-16 text-center">
          <FolderOpenIcon className="h-12 w-12 text-muted-foreground mb-4" />
          <h3 className="text-lg font-semibold mb-2">{t("projectConfigs.noProjects")}</h3>
          <p className="text-sm text-muted-foreground max-w-md mb-4">
            {t("projectConfigs.noProjectsDescription")}
          </p>
          <Button onClick={handleAddNewProject} variant="outline">
            <FolderIcon className="h-4 w-4 mr-2" />
            {t("projectConfigs.addProject")}
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="">
      <Header onAddNew={handleAddNewProject} />

      {/* Search and Filter */}
      <div className="px-4 pt-4 flex gap-3">
        <div className="relative flex-1">
          <SearchIcon className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder={t("projectConfigs.searchPlaceholder")}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-9"
          />
        </div>
        <Select value={statusFilter} onValueChange={(v: any) => setStatusFilter(v)}>
          <SelectTrigger className="w-[180px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t("projectConfigs.filterAll")}</SelectItem>
            <SelectItem value="has_config">{t("projectConfigs.filterHasConfig")}</SelectItem>
            <SelectItem value="no_config">{t("projectConfigs.filterNoConfig")}</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Project Grid */}
      <div className="p-4">
        {filteredProjects.length === 0 ? (
          <Alert>
            <AlertDescription>
              {searchQuery
                ? t("projectConfigs.noSearchResults")
                : t("projectConfigs.noFilterResults")
              }
            </AlertDescription>
          </Alert>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {filteredProjects.map((project) => (
              <ProjectCardWithLocalCheck
                key={project.path}
                project={project}
                onEdit={handleEdit}
                onActivate={handleActivate}
                onDeactivate={handleDeactivate}
                onImport={handleImport}
                onCreate={handleCreate}
                onDelete={handleDelete}
              />
            ))}
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="px-4 py-2 text-sm text-muted-foreground border-t">
        {t("projectConfigs.showingCount", {
          shown: filteredProjects.length,
          total: projectsWithStatus.length
        })}
      </div>
    </div>
  );
}

// Header subcomponent
function Header({ onAddNew }: { onAddNew: () => void }) {
  const { t } = useTranslation();

  return (
    <div
      className="flex items-center p-3 border-b px-3 justify-between sticky top-0 bg-background z-10"
      data-tauri-drag-region
    >
      <div data-tauri-drag-region>
        <h3 className="font-bold" data-tauri-drag-region>
          {t("projectConfigs.title")}
        </h3>
        <p className="text-sm text-muted-foreground" data-tauri-drag-region>
          {t("projectConfigs.description")}
        </p>
      </div>
      <Button variant="outline" size="sm" onClick={onAddNew}>
        <PlusIcon className="h-4 w-4 mr-1" />
        {t("projectConfigs.addNew")}
      </Button>
    </div>
  );
}

// Wrapper component that adds local settings check
function ProjectCardWithLocalCheck({
  project,
  ...handlers
}: {
  project: ProjectWithStatus;
  onEdit: (path: string) => void;
  onActivate: (path: string) => void;
  onDeactivate: () => void;
  onImport: (path: string) => void;
  onCreate: (path: string) => void;
  onDelete: (path: string, title: string) => void;
}) {
  // Only check local settings if no CC Mate config exists
  const shouldCheckLocal = project.status === 'no_config';
  const { data: localSettings } = useCheckProjectLocalSettings(
    shouldCheckLocal ? project.path : ''
  );

  // Update status if local settings found
  const updatedProject: ProjectWithStatus = {
    ...project,
    status: shouldCheckLocal && localSettings
      ? 'has_local_settings'
      : project.status
  };

  return (
    <ProjectCard
      project={updatedProject}
      {...handlers}
      onDelete={(path) => handlers.onDelete(path, project.displayName)}
    />
  );
}
```

---

## i18n Translations

**File**: `src/i18n/locales/en.json` (Add keys)

```json
{
  "projectConfigs": {
    "title": "Project Configs",
    "description": "Select a project to manage its Claude Code configuration",
    "addNew": "Add Project",
    "addProject": "Add Project Folder",
    "selectProjectFolder": "Select Project Folder",
    "searchPlaceholder": "Search projects...",
    "filterAll": "All Projects",
    "filterHasConfig": "Has Config",
    "filterNoConfig": "No Config",
    "noProjects": "No Projects Found",
    "noProjectsDescription": "There are no Claude projects configured. Projects appear here when you use Claude Code in different project folders.",
    "noSearchResults": "No projects match your search",
    "noFilterResults": "No projects match the selected filter",
    "showingCount": "Showing {{shown}} of {{total}} projects",
    "deleteTitle": "Delete Project Config",
    "deleteConfirm": "Are you sure you want to delete the config for \"{{name}}\"? This cannot be undone.",
    "statusHasConfig": "CC Mate Config",
    "statusHasLocal": "Local Settings Detected",
    "statusNoConfig": "No Configuration",
    "statusInheriting": "Inheriting from global",
    "actionEdit": "Edit",
    "actionActivate": "Activate",
    "actionDeactivate": "Use Global Config",
    "actionImport": "Import",
    "actionCreate": "Create Config",
    "actionDelete": "Delete",
    "actionViewLocal": "View Local Settings"
  }
}
```

---

## Step-by-Step Implementation Tasks

### Phase 1: Create Helper Utilities (30 min)

- [ ] Create `src/lib/project-utils.ts`
- [ ] Implement `mergeProjectData()` function
- [ ] Implement `filterProjects()` function
- [ ] Implement `getProjectDisplayName()` function
- [ ] Add TypeScript types/interfaces

### Phase 2: Create UI Components (2 hours)

- [ ] Create `src/components/project-configs/` directory
- [ ] Implement `ProjectStatusBadge.tsx`
- [ ] Implement `ProjectCard.tsx`
- [ ] Implement `ProjectActionsMenu.tsx` (dropdown with actions)
- [ ] Test components in isolation

### Phase 3: Update ProjectConfigsPage (1.5 hours)

- [ ] Add search and filter state
- [ ] Integrate data merging logic
- [ ] Implement all action handlers
- [ ] Add loading states
- [ ] Add empty states (no projects, no results)
- [ ] Add project count footer

### Phase 4: Add i18n Translations (30 min)

- [ ] Add English translations to `en.json`
- [ ] Add Vietnamese translations to `vi.json` (if exists)
- [ ] Update all hardcoded strings to use `t()`

### Phase 5: Testing & Polish (1 hour)

- [ ] Test with 0 projects
- [ ] Test with projects but no configs
- [ ] Test with mixed states (some with config, some without)
- [ ] Test search functionality
- [ ] Test filter functionality
- [ ] Test all action buttons
- [ ] Test navigation to editor
- [ ] Test active indicator updates
- [ ] Verify no TypeScript errors (`pnpm tsc --noEmit`)

---

## Testing Checklist

### Functional Tests

- [ ] All Claude projects from `~/.claude.json` are displayed
- [ ] Projects with CC Mate config show correct status
- [ ] Projects with local `.claude/settings.json` show "Import" option
- [ ] Projects without any config show "Create" option
- [ ] Search filters projects by name and path
- [ ] Status filter works correctly
- [ ] "Create" action creates config and navigates to editor
- [ ] "Import" action imports local settings
- [ ] "Edit" action navigates to correct editor page
- [ ] "Activate" action activates the project config
- [ ] "Delete" action shows confirmation and deletes
- [ ] Active project has visual indicator (star + border)
- [ ] Empty states render correctly

### Edge Cases

- [ ] Very long project paths truncate properly
- [ ] Special characters in project names work
- [ ] 50+ projects render without performance issues
- [ ] Rapid filter changes don't cause errors
- [ ] Network errors show appropriate messages

### Integration Tests

- [ ] Created config appears in list immediately
- [ ] Deleted config disappears from list immediately
- [ ] Imported config appears with correct status
- [ ] Active context updates when activating project
- [ ] Navigation to editor works with URL-encoded paths

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Performance with many projects | Medium | Medium | Virtualized list if needed |
| Local settings check API N+1 calls | High | Medium | Lazy loading, cache results |
| Path encoding issues | Medium | High | Use encodeURIComponent consistently |
| Race conditions on mutations | Low | Medium | Optimistic updates with rollback |

---

## Future Enhancements (Not in Scope)

1. **Bulk Actions**: Select multiple projects, apply same config
2. **Config Templates**: Predefined configs for common project types
3. **Project Grouping**: Group by folder, workspace, etc.
4. **Drag & Drop**: Reorder projects or drop folders to add
5. **Recent Projects**: Show recently used projects at top

---

## Dependencies

### Existing Components (Reuse)

- `Card` from `@/components/ui/card`
- `Button` from `@/components/ui/button`
- `Input` from `@/components/ui/input`
- `Select` from `@/components/ui/select`
- `Alert` from `@/components/ui/alert`
- `DropdownMenu` from `@/components/ui/dropdown-menu`

### New Icons Needed

- `SearchIcon` from lucide-react
- `SparklesIcon` from lucide-react
- `CircleDashed` from lucide-react
- `FileJson` from lucide-react
- `Star` from lucide-react
- `Import` from lucide-react

---

## End of Plan
