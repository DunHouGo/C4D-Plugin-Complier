# Static Analysis

All static analysis tools configured in this app and how to use them.

## Quick Reference

| Tool                   | Purpose                  | Command              | In check:all |
| ---------------------- | ------------------------ | -------------------- | ------------ |
| Vite+ check            | Format, lint, type check | `vp check`           | Yes          |
| Oxlint                 | Syntax, style, TS rules  | `vpr lint`           | Yes          |
| Oxfmt                  | Code formatting          | `vpr format:check`   | Yes          |
| React Compiler linting | Compiler compatibility   | `vpr lint`           | Yes          |
| cargo fmt              | Rust formatting          | `vpr rust:fmt:check` | Yes          |
| clippy                 | Rust linting             | `vpr rust:clippy`    | Yes          |
| Vitest                 | Frontend tests           | `vpr test:run`       | Yes          |
| cargo test             | Rust tests               | `vpr rust:test`      | Yes          |

## Running All Checks

```bash
vpr check:all    # Must pass before commits
vpr fix:all      # Auto-fix what can be fixed
```

## Tool Details

### Oxlint

Handles syntax, style, and TypeScript-specific rules.

```bash
vpr lint        # Check for issues
vpr lint:fix    # Auto-fix issues
```

Configuration in `vite.config.ts`.

### Oxfmt

Consistent code formatting.

```bash
vpr format:check   # Check formatting
vpr format         # Fix formatting
```

Configuration in `vite.config.ts`.

### React Compiler Linting

Oxlint loads `eslint-plugin-react-compiler` to catch patterns that are incompatible with the React compiler. You generally do **not** need to manually add:

- `useMemo` for computed values
- `useCallback` for function references
- `React.memo` for components

The compiler analyzes code and adds memoization where beneficial.

**Note:** The `getState()` pattern is still critical - it avoids store subscriptions, not memoization. See [state-management.md](./state-management.md).

### Rust Tooling

```bash
vpr rust:fmt:check   # Check formatting
vpr rust:fmt         # Fix formatting
vpr rust:clippy      # Lint with clippy
vpr rust:clippy:fix  # Auto-fix clippy warnings
vpr rust:test        # Run Rust tests
```

## CI Integration

`check:all` runs in CI. Ensure it passes locally before pushing:

```bash
vpr check:all
```

## Adding New Rules

**Oxlint:** Add rules to the `lint` section in `vite.config.ts`.

**Oxfmt:** Modify the `fmt` section in `vite.config.ts`.
