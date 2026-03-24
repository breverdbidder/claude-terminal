---
name: claude-terminal-conventions
description: Development conventions and patterns for claude-terminal. TypeScript Vite project with freeform commits.
---

# Claude Terminal Conventions

> Generated from [talayash/claude-terminal](https://github.com/talayash/claude-terminal) on 2026-03-24

## Overview

This skill teaches Claude the development patterns and conventions used in claude-terminal.

## Tech Stack

- **Primary Language**: TypeScript
- **Framework**: Vite
- **Architecture**: type-based module organization
- **Test Location**: separate

## When to Use This Skill

Activate this skill when:
- Making changes to this repository
- Adding new features following established patterns
- Writing tests that match project conventions
- Creating commits with proper message format

## Commit Conventions

Follow these commit message conventions based on 63 analyzed commits.

### Commit Style: Free-form Messages

### Prefixes Used

- `fix`

### Message Guidelines

- Average message length: ~44 characters
- Keep first line concise and descriptive
- Use imperative mood ("Add feature" not "Added feature")


*Commit message example*

```text
docs: Update README for v1.2.0
```

*Commit message example*

```text
Fix: Use correct Rust toolchain action in release workflow
```

*Commit message example*

```text
Add 8 advanced Claude Code features
```

*Commit message example*

```text
Release v1.16.0
```

*Commit message example*

```text
Release v1.15.0
```

*Commit message example*

```text
Release v1.14.0
```

*Commit message example*

```text
Merge pull request #6 from talayash/feature/claude-config-manager
```

*Commit message example*

```text
Replace README screenshots with animated GIF slideshow
```

## Architecture

### Project Structure: Single Package

This project uses **type-based** module organization.

### Source Layout

```
src/
├── components/
├── fonts/
├── hooks/
├── store/
```

### Entry Points

- `src/App.tsx`
- `src/main.tsx`

### Configuration Files

- `.github/workflows/release.yml`
- `package.json`
- `tailwind.config.js`
- `tsconfig.json`
- `vite.config.ts`

### Guidelines

- Group code by type (components, services, utils)
- Keep related functionality in the same type folder
- Avoid circular dependencies between type folders

## Code Style

### Language: TypeScript

### Naming Conventions

| Element | Convention |
|---------|------------|
| Files | camelCase |
| Functions | camelCase |
| Classes | PascalCase |
| Constants | SCREAMING_SNAKE_CASE |

### Import Style: Relative Imports

### Export Style: Named Exports


*Preferred import style*

```typescript
// Use relative imports
import { Button } from '../components/Button'
import { useAuth } from './hooks/useAuth'
```

*Preferred export style*

```typescript
// Use named exports
export function calculateTotal() { ... }
export const TAX_RATE = 0.1
export interface Order { ... }
```

## Error Handling

### Error Handling Style: Error Boundaries

React **Error Boundaries** are used for graceful UI error handling.


## Common Workflows

These workflows were detected from analyzing commit patterns.

### Feature Development

Standard feature implementation workflow

**Frequency**: ~13 times per month

**Steps**:
1. Add feature implementation
2. Add tests for feature
3. Update documentation

**Files typically involved**:
- `src/components/*`
- `src/hooks/*`
- `src/store/*`

**Example commit sequence**:
```
Enable createUpdaterArtifacts for auto-update .sig generation
Update signing public key with password-protected keypair
Fix hardcoded version strings and add shortcuts to Settings
```

### Release Version Bump

Publishes a new release version, updating version numbers, changelog, and documentation.

**Frequency**: ~4 times per month

**Steps**:
1. Update version in package.json and/or Cargo.toml
2. Update changelog.json with new version entry
3. Update README.md and CLAUDE.md with new version and features
4. Update src-tauri/tauri.conf.json with new version and/or signing keys
5. Commit and tag the release

**Files typically involved**:
- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`
- `src/changelog.json`
- `README.md`
- `CLAUDE.md`

**Example commit sequence**:
```
Update version in package.json and/or Cargo.toml
Update changelog.json with new version entry
Update README.md and CLAUDE.md with new version and features
Update src-tauri/tauri.conf.json with new version and/or signing keys
Commit and tag the release
```

### Add Or Enhance Feature

Implements a new feature or enhances an existing one, touching backend (Rust IPC), frontend components, and state stores.

**Frequency**: ~3 times per month

**Steps**:
1. Add or update Rust IPC command(s) in src-tauri/src/commands.rs and possibly other backend files
2. Add or update frontend React components in src/components/
3. Update or create hooks in src/hooks/ if needed
4. Update or create state stores in src/store/
5. Wire up new UI in src/App.tsx or relevant container
6. Update keyboard shortcuts if needed in src/hooks/useKeyboardShortcuts.ts
7. Update documentation if needed

**Files typically involved**:
- `src-tauri/src/commands.rs`
- `src-tauri/src/main.rs`
- `src-tauri/src/database.rs`
- `src-tauri/src/terminal.rs`
- `src/components/*.tsx`
- `src/hooks/*.ts`
- `src/store/*.ts`
- `src/App.tsx`

**Example commit sequence**:
```
Add or update Rust IPC command(s) in src-tauri/src/commands.rs and possibly other backend files
Add or update frontend React components in src/components/
Update or create hooks in src/hooks/ if needed
Update or create state stores in src/store/
Wire up new UI in src/App.tsx or relevant container
Update keyboard shortcuts if needed in src/hooks/useKeyboardShortcuts.ts
Update documentation if needed
```

### Fix Bug Or Improve Ui

Fixes a bug or makes UI/UX improvements, often in a single or few frontend files.

**Frequency**: ~4 times per month

**Steps**:
1. Identify and fix bug in relevant component or hook
2. Update related UI/UX as needed
3. Test the fix
4. Update documentation if needed

**Files typically involved**:
- `src/components/*.tsx`
- `src/hooks/*.ts`
- `src/store/*.ts`

**Example commit sequence**:
```
Identify and fix bug in relevant component or hook
Update related UI/UX as needed
Test the fix
Update documentation if needed
```

### Update Signing Keys Or Updater Config

Updates signing keys or updater configuration for release security and auto-update functionality.

**Frequency**: ~2 times per month

**Steps**:
1. Regenerate or update signing keypair
2. Update public key in src-tauri/tauri.conf.json
3. Commit and push changes

**Files typically involved**:
- `src-tauri/tauri.conf.json`

**Example commit sequence**:
```
Regenerate or update signing keypair
Update public key in src-tauri/tauri.conf.json
Commit and push changes
```

### Add Or Update Documentation And Screenshots

Updates documentation files and/or adds new screenshots or demo media.

**Frequency**: ~2 times per month

**Steps**:
1. Update README.md and/or CLAUDE.md
2. Add or update images or GIFs in docs/
3. Commit and push changes

**Files typically involved**:
- `README.md`
- `CLAUDE.md`
- `docs/*.png`
- `docs/*.gif`

**Example commit sequence**:
```
Update README.md and/or CLAUDE.md
Add or update images or GIFs in docs/
Commit and push changes
```


## Best Practices

Based on analysis of the codebase, follow these practices:

### Do

- Use camelCase for file names
- Prefer named exports

### Don't

- Don't deviate from established patterns without discussion

---

*This skill was auto-generated by [ECC Tools](https://ecc.tools). Review and customize as needed for your team.*
