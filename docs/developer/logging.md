# Logging

Simple logging setup for both Rust backend and TypeScript frontend.

## Quick Start

### Rust (Backend)

```rust
log::info!("Application starting up");
log::debug!("Debug info: {}", some_value);
log::warn!("Something unexpected happened");
log::error!("Error occurred: {}", error);
```

### TypeScript (Frontend)

```typescript
import { logger } from '@/lib/logger'

logger.info('User action completed')
logger.debug('Debug data', { userId: 123, action: 'click' })
logger.warn('Performance warning')
logger.error('Request failed', { error: response.error })
```

## Configuration

### Rust Backend

- Uses `tauri-plugin-log` with standard Rust `log` crate
- **Development**: Debug level, logs to stdout + webview console
- **Production**: Info level, logs to stdout + app log directory
- Configuration in `src-tauri/src/lib.rs`

### TypeScript Frontend

- **Development**: Logs go to browser console and the Tauri log plugin
- **Production**: Logs go to the Tauri log plugin and app log directory
- Logger utility at `src/lib/logger.ts`
- Frontend crashes are also appended to `crash.log` through `append_crash_log`

## Log Levels

| Level   | When to Use            | Dev | Prod |
| ------- | ---------------------- | --- | ---- |
| `trace` | Most verbose debugging | ✅  | ❌   |
| `debug` | Development debugging  | ✅  | ❌   |
| `info`  | General information    | ✅  | ✅   |
| `warn`  | Warning conditions     | ✅  | ✅   |
| `error` | Error conditions       | ✅  | ✅   |

## Where Logs Appear

### Development

- **Rust**: Terminal (stdout) + Browser DevTools console (webview)
- **TypeScript**: Browser DevTools console

### Production

- **Rust**: Terminal (stdout) + log file in app log directory
- **TypeScript**: Tauri log file in app log directory

Log directory locations vary by platform (e.g., `~/Library/Logs/` on macOS).
The app logs the resolved directory on startup. Crash diagnostics are appended to `crash.log` in the same directory; Rust panic hooks use a cache/data-directory fallback if Tauri is not initialized yet.

## Examples

### Rust Tauri Commands

```rust
#[tauri::command]
async fn save_data(data: MyData) -> Result<(), String> {
    log::info!("Saving data for user: {}", data.user_id);

    match save_to_disk(&data).await {
        Ok(_) => {
            log::info!("Data saved successfully");
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to save data: {}", e);
            Err(format!("Save failed: {}", e))
        }
    }
}
```

### TypeScript React Components

```typescript
import { logger } from '@/lib/logger'

function MyComponent() {
  const handleClick = () => {
    logger.debug('Button clicked', { component: 'MyComponent' })

    try {
      performAction()
      logger.info('Action completed successfully')
    } catch (error) {
      logger.error('Action failed', { error })
    }
  }

  return <button onClick={handleClick}>Click me</button>
}
```

## Best Practices

1. **Use appropriate log levels** - Don't log everything as `info`
2. **Include context** - Add relevant data to help debugging
3. **Log errors with details** - Include error messages and context
4. **Keep messages concise** - But descriptive enough to be useful
5. **Use structured logging** - Include objects/context for complex data

See [error-handling.md](./error-handling.md) for patterns on when to log vs show errors to users.

## Production Considerations

- Rust logs go to the app's log directory (platform-specific location)
- No sensitive data should be logged (passwords, tokens, etc.)
- The plugin supports log rotation when files reach size limits
- Frontend logs are forwarded to `@tauri-apps/plugin-log`
- Rust panics, React error-boundary crashes, window errors, and unhandled promise rejections are appended to `crash.log`
