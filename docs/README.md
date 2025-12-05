# CC Mate Documentation

**Last Updated**: 2025-12-06
**Status**: Phase 1 Backend Foundation Complete

This directory contains comprehensive documentation for the CC Mate project.

## Documentation Files

### 1. [project-overview-pdr.md](./project-overview-pdr.md) - 444 lines
**Purpose**: Project overview and Product Development Requirements

**Contents**:
- Executive summary of the project
- Core features and target users
- Phase 1 (Backend Foundation) completion details
- Phase 2 (Frontend) planned work
- Technical architecture overview
- File organization reference
- Future enhancements (Phase 3)
- Acceptance criteria and success metrics

**When to read**: Start here to understand what CC Mate does and what's been completed in Phase 1.

---

### 2. [system-architecture.md](./system-architecture.md) - 749 lines
**Purpose**: Detailed technical system architecture documentation

**Contents**:
- High-level architecture diagram
- Frontend architecture (React Query, directory structure)
- Backend architecture (Rust, Tauri, commands organization)
- Per-project configuration implementation details
- Data flow diagrams (creation, activation, merging, context switching)
- Configuration file formats (ProjectConfigStore, ActiveContext, EnhancedStoresData)
- Complete API command reference (35+ commands with signatures)
- Error handling strategies
- Performance considerations
- Security analysis
- Testing strategy
- Deployment process
- Dependency references

**When to read**: Go here for detailed technical understanding of how the system works.

---

### 3. [code-standards.md](./code-standards.md) - 942 lines
**Purpose**: Code style, organization, and development standards

**Contents**:
- Directory structure and module organization
- TypeScript naming conventions
- Rust naming conventions
- TypeScript code standards (strict mode, types, components, hooks, forms, errors)
- Rust code standards (modules, errors, async/await, JSON, file operations, documentation)
- API command structure and naming
- Configuration management patterns
- Testing standards (unit, E2E, coverage)
- Performance guidelines
- Security guidelines
- Documentation guidelines
- Code review checklist
- Development workflow commands

**When to read**: Follow this when writing new code or reviewing pull requests.

---

### 4. [codebase-summary.md](./codebase-summary.md) - 702 lines
**Purpose**: Overview of the entire codebase structure and organization

**Contents**:
- Repository structure (root, frontend, backend, docs, config)
- Core components breakdown (UI components, pages, hooks, commands)
- Top files by size and complexity
- Key features implemented (Phase 1 backend)
- Existing features (pre-Phase 1)
- Technology stack overview
- Development workflow
- Code metrics and statistics
- Security and performance features
- Current status and next steps
- Documentation files reference
- Contributing and maintenance info
- File organization reference

**When to read**: Use this to navigate the codebase and understand what files do what.

---

## Quick Links

### For Different Roles

**New Developers**:
1. Read [project-overview-pdr.md](./project-overview-pdr.md) - Get context
2. Read [codebase-summary.md](./codebase-summary.md) - Understand structure
3. Follow [code-standards.md](./code-standards.md) - Learn standards
4. Reference [system-architecture.md](./system-architecture.md) - Deep dive

**Architects**:
1. Read [project-overview-pdr.md](./project-overview-pdr.md) - Requirements
2. Focus on [system-architecture.md](./system-architecture.md) - Technical design

**Frontend Developers**:
1. [system-architecture.md](./system-architecture.md) - Section 2 (Frontend Architecture)
2. [code-standards.md](./code-standards.md) - Section 3 (TypeScript Standards)
3. [codebase-summary.md](./codebase-summary.md) - Section 3 (Frontend Components)

**Backend Developers**:
1. [system-architecture.md](./system-architecture.md) - Section 3 (Backend Architecture)
2. [code-standards.md](./code-standards.md) - Section 4 (Rust Standards)
3. [codebase-summary.md](./codebase-summary.md) - Section 3 (Backend Commands)

**QA/Testers**:
1. [project-overview-pdr.md](./project-overview-pdr.md) - Features to test
2. [code-standards.md](./code-standards.md) - Section 7 (Testing Standards)
3. [system-architecture.md](./system-architecture.md) - Section 6 & 7 (Data flow & API)

---

## Phase 1 Implementation Summary

**Completed** (2025-12-06):
- Backend data structures for per-project configuration
- 17 new Tauri commands for project config management
- Path canonicalization and SHA256 hashing
- Deep merge logic with permission array union
- Active context persistence in stores.json
- Auto-import from `.claude/settings.json`
- Enterprise managed settings detection
- Configuration activation and context switching

**Key Files Modified**:
- `src-tauri/Cargo.toml` - Added sha2 dependency
- `src-tauri/src/commands.rs` - Added ~790 lines for project config (now 2922 lines total)
- `src-tauri/src/lib.rs` - Registered 17 new commands

**Key Additions**:
- `ProjectConfigStore` struct - Per-project configuration with inheritance
- `ActiveContext` struct - Tracks global vs. project context
- `EnhancedStoresData` struct - Extended stores.json with context
- Helper functions for path hashing, config merging, context management

---

## Navigation Guide

### By Topic

**Configuration Management**:
- [project-overview-pdr.md](./project-overview-pdr.md) - Sections 2-3
- [system-architecture.md](./system-architecture.md) - Sections 3-5

**API & Commands**:
- [system-architecture.md](./system-architecture.md) - Section 6
- [code-standards.md](./code-standards.md) - Section 5

**Code Organization**:
- [codebase-summary.md](./codebase-summary.md) - Section 2-3
- [code-standards.md](./code-standards.md) - Section 1

**Development Setup**:
- [code-standards.md](./code-standards.md) - Section 12
- [codebase-summary.md](./codebase-summary.md) - Section 7

**Frontend Implementation**:
- [system-architecture.md](./system-architecture.md) - Section 2
- [codebase-summary.md](./codebase-summary.md) - Section 3 (Frontend Components)

**Backend Implementation**:
- [system-architecture.md](./system-architecture.md) - Section 3
- [codebase-summary.md](./codebase-summary.md) - Section 3 (Backend Commands)

---

## File Statistics

| File | Lines | Purpose |
|------|-------|---------|
| project-overview-pdr.md | 444 | Project requirements & overview |
| system-architecture.md | 749 | Technical architecture & API |
| code-standards.md | 942 | Code standards & guidelines |
| codebase-summary.md | 702 | Codebase structure & organization |
| **Total** | **2,837** | Comprehensive documentation |

---

## Related Files

**In Repository Root**:
- `README.md` - Project overview (for users)
- `CONTRIBUTING.md` - Contributing guidelines
- `CLAUDE.md` - Claude Code project instructions

**In plans/ Directory**:
- `251205-per-project-config.md` - Phase 1 detailed specification
- `reports/` - Analysis and implementation reports

---

## Version History

- **v1.0** (2025-12-06) - Initial documentation set for Phase 1 completion
  - Comprehensive project overview
  - Detailed system architecture
  - Complete code standards
  - Full codebase summary
  - Generated from repomix output

---

## How to Update Documentation

When making changes to the codebase:

1. **New feature** - Update [project-overview-pdr.md](./project-overview-pdr.md) with new requirements
2. **Architecture change** - Update [system-architecture.md](./system-architecture.md) sections 3-6
3. **New command** - Update [system-architecture.md](./system-architecture.md) section 6 (API reference)
4. **Code organization change** - Update [code-standards.md](./code-standards.md) section 1 and [codebase-summary.md](./codebase-summary.md) section 2
5. **Major refactor** - Regenerate [codebase-summary.md](./codebase-summary.md) with `repomix`

### Update Checklist

- [ ] Update relevant sections in affected docs
- [ ] Verify cross-references between documents
- [ ] Update file statistics if structure changed
- [ ] Ensure code examples match current implementation
- [ ] Update version/date at top of files
- [ ] Add entry to version history section

---

## Support

For questions about:
- **Features**: See [project-overview-pdr.md](./project-overview-pdr.md)
- **Architecture**: See [system-architecture.md](./system-architecture.md)
- **Coding**: See [code-standards.md](./code-standards.md)
- **Files**: See [codebase-summary.md](./codebase-summary.md)

For external help:
- Issues: https://github.com/djyde/ccconfig/issues
- Discussions: https://github.com/djyde/ccconfig/discussions
- Contributing: See CONTRIBUTING.md in repository root

---

**Last Updated**: 2025-12-06
**Maintained by**: Development Team
