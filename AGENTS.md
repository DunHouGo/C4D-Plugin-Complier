# AI Agent Instructions

## Overview

This repository is a Tauri v2 + React template using Vite+ as the frontend toolchain. Frontend formatting, linting, and Vitest configuration live in `vite.config.ts`.

## Core Rules

### New Sessions

- Check git status and project structure before editing.
- Review `docs/developer/architecture-guide.md` for high-level patterns.
- Check `docs/developer/README.md` for the full documentation index.
- Read `docs/tasks.md` only when working with task documents in `docs/tasks-todo/` or `docs/tasks-done/`.

### Development Practices

**CRITICAL:** Follow these strictly:

0. **Use Vite+ entrypoints**: use `vp` for toolchain commands and `vpr` for package tasks; do not use legacy package-manager script invocation.
1. **Use Vite+ package commands**: the project keeps `package-lock.json`, but install/update flows should use `vp install`, `vp add`, `vp remove`, and related Vite+ commands.
2. **Read Before Editing**: always read files first to understand context.
3. **Follow Established Patterns**: use patterns from this file and `docs/developer`.
4. **Senior Architect Mindset**: consider performance, maintainability, and testability.
5. **Batch Operations**: use multiple tool calls in single responses when practical.
6. **Match Code Style**: follow existing formatting and patterns.
7. **Test Coverage**: write focused tests for business logic and shared behavior.
8. **Quality Gates**: run the relevant Vite+ checks before finishing.
9. **No Dev Server By Default**: ask the user to run dev servers unless they request otherwise.
10. **No Unsolicited Commits**: commit only when explicitly requested.
11. **Documentation**: update relevant `docs/developer/` files for new patterns.
12. **Changelog**: update `CHANGELOG.md` for repository or code changes.

**CRITICAL:** Use Tauri v2 docs only. Always use modern Rust formatting: `format!("{variable}")`.

## Vite+ Workflow

### Commands

- `vpr dev` -> `vp dev`
- `vpr build` -> `vp exec tsc --noEmit && vp build`
- `vpr preview` -> `vp preview`
- `vpr typecheck` -> `vp exec tsc --noEmit`
- `vpr lint` -> `vp lint . --max-warnings 0`
- `vpr format` -> `vp fmt . --write`
- `vpr format:check` -> `vp fmt . --check`
- `vpr test` -> `vp exec vitest`
- `vpr test:run` -> `vp exec vitest run`
- `vpr check:all` -> frontend Vite+ check, Rust fmt/clippy, frontend tests, Rust tests

### Frontend Checks

For frontend-only changes, run:

```bash
vp check
vpr typecheck
vpr test:run
vpr build
```

For formatting fixes, use:

```bash
vp check --fix
```

For Rust or Tauri changes, also run the relevant Rust checks:

```bash
vpr rust:fmt:check
vpr rust:clippy
vpr rust:test
```

Use `vpr check:all` for full quality gates when the change crosses frontend and Rust boundaries.

### Configuration Ownership

- Vite, Vite+, Oxlint, Oxfmt, and Vitest settings are centralized in `vite.config.ts`.
- Do not add standalone ESLint, Prettier, `.oxlintrc.json`, `.oxfmtrc.json`, or `vitest.config.ts` files unless explicitly requested.
- Do not reintroduce removed cleanup tooling such as knip, jscpd, or ast-grep unless explicitly requested.
- Generated Tauri bindings live at `src/lib/bindings.ts`; do not edit them manually.

## Architecture Patterns

### State Management Onion

```text
useState (component) -> Zustand (global UI) -> TanStack Query (persistent data)
```

**Decision**: Is data needed across components? Does it persist between sessions?

### Performance Pattern

```typescript
// GOOD: Selector syntax only re-renders when this value changes.
const leftSidebarVisible = useUIStore(state => state.leftSidebarVisible)

// BAD: Destructuring causes render cascades.
const { leftSidebarVisible } = useUIStore()

// GOOD: Use getState() in callbacks for current state.
const handleAction = () => {
  const { data, setData } = useStore.getState()
  setData(newData)
}
```

### Static Analysis

- **React Compiler**: handles memoization automatically; do not add manual `useMemo` or `useCallback` unless there is a measured reason.
- **Vite+**: runs frontend formatting, linting, and test tooling through `vp`.
- **Oxlint/Oxfmt**: configured through `vite.config.ts`.
- **Removed tools**: knip, jscpd, and ast-grep are no longer part of this repository workflow.

### Event-Driven Bridge

- **Rust to React**: `app.emit("event-name", data)` -> `listen("event-name", handler)`
- **React to Rust**: use typed commands from `@/lib/tauri-bindings` through tauri-specta.
- **Commands**: all actions flow through the centralized command system.

### Tauri Command Pattern

```typescript
// GOOD: Type-safe commands with Result handling.
import { commands } from '@/lib/tauri-bindings'

const result = await commands.loadPreferences()
if (result.status === 'ok') {
  console.log(result.data.theme)
}

// BAD: String-based invoke has no generated type safety.
const prefs = await invoke('load_preferences')
```

**Adding commands**: see `docs/developer/tauri-commands.md`.

### Internationalization

```typescript
// GOOD: Use useTranslation in React components.
import { useTranslation } from 'react-i18next'

function MyComponent() {
  const { t } = useTranslation()
  return <h1>{t('myFeature.title')}</h1>
}

// GOOD: Non-React contexts can bind once for repeated calls.
import i18n from '@/i18n/config'

const t = i18n.t.bind(i18n)
i18n.t('key')
```

- Translation files live in `locales/*.json`.
- Locale file names and i18n resource keys use standard locale tags such as `en-US` and `zh-CN`.
- Add new languages by creating `locales/<locale>.json`, registering it in `src/i18n/config.ts`, and adding a native display name in `AppearancePane`.
- Use `getBestAvailableLanguage()` for system locale or saved preference matching.
- RTL support is language-code based; use CSS logical properties such as `text-start` instead of `text-left`.
- See `docs/developer/i18n-patterns.md` before changing i18n architecture.

## Documentation & Versions

- **Context7 First**: use Context7 for framework docs before web search.
- **Version Requirements**: Tauri v2.x, shadcn/ui v4.x, Tailwind v4.x, React 19.x, Zustand v5.x, Vite v8.x, Vitest v4.x, Vite+ v0.1.x.

## Developer Documentation

For complete patterns and detailed guidance, see `docs/developer/README.md`.

Key documents:

- `architecture-guide.md` - mental models, security, anti-patterns
- `state-management.md` - state onion and `getState()` pattern details
- `tauri-commands.md` - adding Rust commands and generated bindings
- `static-analysis.md` - linting, formatting, and quality gates

## Claude Code Commands & Agents

These are specific to Claude Code but documented here for context.

### Commands

- `/check` - check work against architecture, run quality gates, suggest a commit message
- `/init` - one-time template initialization

### Agents

Task-focused agents that use separate context for focused work:

- `plan-checker` - validate implementation plans against documented architecture
- `docs-reviewer` - review developer docs for accuracy and codebase consistency
- `userguide-reviewer` - review user guide against actual system features
