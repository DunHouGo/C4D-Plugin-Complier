# Tauri React Template

![alt text](images/tauri-destop-starter.png)

[简体中文](README.zh-CN.md) | English

A batteries-included desktop application template that helps you start building quickly.

This project is for building production-ready **Tauri v2**, **React**, and **TypeScript** applications. It provides opinionated engineering conventions that help both human developers and AI coding agents collaborate on a stable architecture from the start.

> This project is derived from Danny's [tauri-template](https://github.com/dannysmith/tauri-template). Danny's project has become increasingly comprehensive, while this template aims to keep the initial setup simpler and more practical. Some dependencies can be added later as needed. If you are looking for a more complex industrial-grade template, take a look at Danny's work.

## Why This Template?

Most Tauri starters give you a blank canvas. This template gives you a working application with patterns already established:

- **Type-safe Rust-TypeScript bridge** via tauri-specta.
- **Performance patterns enforced by tooling** with Vite+, Oxlint, Oxfmt, React Compiler linting, and focused tests.
- **Cross-platform ready** with platform-specific title bars, window controls, and native menu integration.
- **i18n built in** with RTL support and locale files for `en-US`, `fr-FR`, `ar-SA`, and `zh-CN`.

## Stack

| Layer    | Technologies                                      |
| -------- | ------------------------------------------------- |
| Frontend | React 19, TypeScript, Vite 8, Vite+               |
| UI       | shadcn/ui v4, Tailwind CSS v4, Lucide React       |
| State    | Zustand v5, TanStack Query v5                     |
| Backend  | Tauri v2, Rust                                    |
| Testing  | Vitest v4, Testing Library                        |
| Quality  | Vite+, Oxlint, Oxfmt, React Compiler lint, clippy |

## What's Already Built

### Core Features

- **Command Palette** (`Cmd+K`) - Searchable command launcher with keyboard navigation.
- **Keyboard Shortcuts** - Platform-aware shortcuts with menu integration.
- **Native Menus** - File, Edit, and View menus built from JavaScript with i18n support.
- **Preferences System** - Settings dialog with Rust-side persistence, React hooks, and type-safe access.
- **Collapsible Sidebars** - Left and right sidebars with state persistence via resizable panels.
- **Theme System** - Light, dark, and system theme modes.
- **Notifications** - Toast notifications and native system notifications.
- **Auto-updates** - Tauri updater plugin configured for GitHub Releases.
- **Logging** - Structured logging utilities for Rust and TypeScript.
- **Crash Recovery** - Emergency persistence for recovering data after unexpected exits.

### Architecture Patterns

- **Three-layer state management** - `useState` (component) -> `Zustand` (global UI) -> `TanStack Query` (persistent data).
- **Event-driven Rust-React bridge** - Menus, shortcuts, and command palette all route through the same command system.
- **React Compiler** - Automatic memoization means no manual `useMemo` or `useCallback` is needed by default.

### Cross-Platform

| Platform | Title Bar            | Window Controls | Bundle Format |
| -------- | -------------------- | --------------- | ------------- |
| macOS    | Custom with vibrancy | Traffic lights  | `.dmg`        |
| Windows  | Custom               | Right side      | `.msi`        |
| Linux    | Native + toolbar     | Native          | `.AppImage`   |

Platform detection utilities, platform-specific UI strings, and Tauri configuration are already set up.

### Developer Experience

- **Type-safe Tauri commands** - tauri-specta generates TypeScript bindings from Rust.
- **Static analysis** - Vite+ runs Oxlint, Oxfmt, React Compiler linting, TypeScript, and Vitest from `vite.config.ts`.
- **Single quality gate** - `vpr check:all` runs frontend checks, Rust formatting, clippy, and tests.
- **Testing patterns** - Vitest setup with Tauri command mocking.

## Tauri Plugins Included

| Plugin            | Purpose                              |
| ----------------- | ------------------------------------ |
| single-instance   | Prevent multiple app instances       |
| window-state      | Remember window position and size    |
| fs                | File system access                   |
| dialog            | Native open/save dialogs             |
| notification      | System notifications                 |
| clipboard-manager | Clipboard access                     |
| updater           | In-app auto-updates                  |
| opener            | Open URLs/files with the default app |

## AI-Ready Development

This template is designed to work well with AI coding agents:

- **Developer documentation** in `docs/developer/` explains the project patterns and decisions.
- **Agent instructions** in `AGENTS.md` define the current Vite+ workflow.
- **Predictable organization** keeps React code in `src/` and Rust code in `src-tauri/src/`.

## Getting Started

See **[Using This Template](docs/USING_THIS_TEMPLATE.md)** for setup instructions and workflow guidance.

```bash
# Prerequisites: Node.js 20+ and stable Rust
# See https://tauri.app/start/prerequisites/ for platform-specific dependencies.

git clone <your-repo>
cd your-app
vp install
vpr dev
```

## Documentation

- **[Developer Docs](docs/developer/)** - Architecture, patterns, and detailed guides.
- **[User Guide](docs/userguide/)** - End-user documentation template.
- **[Using This Template](docs/USING_THIS_TEMPLATE.md)** - Setup and workflow guide.

## License

[MIT](LICENSE.md)

---

Built with [Tauri](https://tauri.app) | [shadcn/ui](https://ui.shadcn.com) | [React](https://react.dev) | [TypeScript](https://www.typescriptlang.org) | [Vite+](https://plu.dev)
