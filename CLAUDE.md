# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Role & Responsibilities

Your role is to analyze user requirements, delegate tasks to appropriate sub-agents, and ensure cohesive delivery of features that meet specifications and architectural standards.

## Workflows

-   Primary workflow: `./.claude/workflows/primary-workflow.md`
-   Development rules: `./.claude/workflows/development-rules.md`
-   Orchestration protocols: `./.claude/workflows/orchestration-protocol.md`
-   Documentation management: `./.claude/workflows/documentation-management.md`
-   And other workflows: `./.claude/workflows/*`

**IMPORTANT:** Analyze the skills catalog and activate the skills that are needed for the task during the process.
**IMPORTANT:** You must follow strictly the development rules in `./.claude/workflows/development-rules.md` file.
**IMPORTANT:** Before you plan or proceed any implementation, always read the `./README.md` file first to get context.
**IMPORTANT:** Sacrifice grammar for the sake of concision when writing reports.
**IMPORTANT:** In reports, list any unresolved questions at the end, if any.
**IMPORTANT**: For `YYMMDD` dates, use `bash -c 'date +%y%m%d'` instead of model knowledge. Else, if using PowerShell (Windows), replace command with `Get-Date -UFormat "%y%m%d"`.

## Documentation Management

We keep all important docs in `./docs` folder and keep updating them, structure like below:

```
./docs
├── project-overview-pdr.md
├── code-standards.md
├── codebase-summary.md
├── design-guidelines.md
├── deployment-guide.md
├── system-architecture.md
└── project-roadmap.md
```

**IMPORTANT:** _MUST READ_ and _MUST COMPLY_ all _INSTRUCTIONS_ in project `./CLAUDE.md`, especially _WORKFLOWS_ section is _CRITICALLY IMPORTANT_, this rule is _MANDATORY. NON-NEGOTIABLE. NO EXCEPTIONS. MUST REMEMBER AT ALL TIMES!!!_

## Project Overview

This is a Tauri v2 application for managing Claude Code configuration files. It provides a UI to view, edit, and backup various Claude Code configuration files across different locations (user, enterprise).

## Tech Stack

-   **Frontend**: React 19 with TypeScript
-   **Backend**: Rust with Tauri v2
-   **Build Tool**: Vite with React plugin
-   **Styling**: Tailwind CSS v4 via @tailwindcss/vite
-   **UI Components**: shadcn/ui components
-   **Data Fetching**: @tanstack/react-query
-   **Forms**: react-hook-form with @hookform/resolvers and zod
-   **Routing**: react-router-dom
-   **Package Manager**: pnpm (required)

## Development Commands

```bash
# Install dependencies
pnpm install

# Start development server
pnpm tauri dev

# Build for production
pnpm build

# Preview built app
pnpm preview

# Check TypeScript lint error

pnpm tsc --noEmit
```

## Architecture

### Frontend Structure

-   `src/main.tsx` - App entry point with React Query client setup
-   `src/lib/query.ts` - React Query hooks and API functions
-   `src/lib/utils.ts` - Utility functions
-   `src/components/ui/` - shadcn/ui components

### Backend Structure (Rust)

-   `src-tauri/src/main.rs` - Tauri application entry point
-   `src-tauri/src/lib.rs` - Main application setup and plugin configuration
-   `src-tauri/src/commands.rs` - Tauri commands for file operations

### Key Configuration Types

The app handles these configuration file types:

-   `user` - `~/.claude/settings.json`
-   `enterprise_macos/linux/windows` - System-wide managed settings
-   `mcp_macos/linux/windows` - System-wide MCP configurations

### Data Flow

1. React Query hooks in `src/lib/query.ts` call Tauri commands
2. Tauri commands in `src-tauri/src/commands.rs` handle file I/O
3. Frontend displays config content in a textarea with JSON validation
4. Changes are saved back via mutations that invalidate relevant queries

## Code Principles

-   Use functional components and hooks
-   Do not use export default to export components
-   Place React Query/mutation logic in `src/lib/query.ts` by default
-   Write Tauri commands in `src-tauri/src/commands.rs` with well-designed names
-   Do not separate components into smaller files unless explicitly requested
-   Use `pnpm tsc --noEmit` instead of `pnpm tauri dev` to check TypeScript lint errors after modifying frontend code

## Important Notes

-   The app automatically backs up existing Claude configs on first run to `~/.ccconfig/claude_backup/`
-   Enterprise config files are read-only
-   All file operations use async/await patterns
-   JSON validation is performed client-side before saving
-   DO NOT use --yes for shadcn/ui components installation

## Use exa by Default

Always use exa when I need code generation, library installation, setup or configuration steps, or library/API documentation. This means you should automatically use the exa MCP tools get library docs without me having to explicitly ask.
