import {
  debug as tauriDebug,
  error as tauriError,
  info as tauriInfo,
  trace as tauriTrace,
  warn as tauriWarn,
} from '@tauri-apps/plugin-log'
import { commands, type JsonValue } from '@/lib/tauri-bindings'

type LogLevel = 'trace' | 'debug' | 'info' | 'warn' | 'error'

interface LogEntry {
  level: LogLevel
  message: string
  timestamp: Date
  context?: Record<string, unknown>
}

class Logger {
  private isDevelopment = import.meta.env.DEV
  private logDirectory: string | null = null

  /**
   * Log a trace message (most verbose)
   */
  trace(message: string, context?: Record<string, unknown>): void {
    this.log('trace', message, context)
  }

  /**
   * Log a debug message (development only)
   */
  debug(message: string, context?: Record<string, unknown>): void {
    this.log('debug', message, context)
  }

  /**
   * Log an info message
   */
  info(message: string, context?: Record<string, unknown>): void {
    this.log('info', message, context)
  }

  /**
   * Log a warning message
   */
  warn(message: string, context?: Record<string, unknown>): void {
    this.log('warn', message, context)
  }

  /**
   * Log an error message
   */
  error(message: string, context?: Record<string, unknown>): void {
    this.log('error', message, context)
  }

  private log(
    level: LogLevel,
    message: string,
    context?: Record<string, unknown>
  ): void {
    const entry: LogEntry = {
      level,
      message,
      timestamp: new Date(),
      context,
    }

    // Always log to console in development
    if (this.isDevelopment) {
      this.logToConsole(entry)
    }

    void this.logToBackend(entry)
  }

  async getLogDirectory(): Promise<string | null> {
    if (this.logDirectory) {
      return this.logDirectory
    }

    const result = await commands.getLogDir()
    if (result.status === 'ok') {
      this.logDirectory = result.data
      return result.data
    }

    this.warn('Failed to resolve log directory', { error: result.error })
    return null
  }

  async recordCrash(
    source: string,
    error: unknown,
    context?: Record<string, unknown>
  ): Promise<void> {
    const normalized = normalizeError(error)
    this.error(`Crash captured from ${source}: ${normalized.message}`, {
      ...context,
      stack: normalized.stack,
    })

    try {
      const result = await commands.appendCrashLog(
        source,
        normalized.message,
        normalized.stack ?? null,
        toJsonContext(context)
      )
      if (result.status === 'ok') {
        this.logDirectory = parentPath(result.data)
        return
      }
      this.logToConsole({
        level: 'error',
        message: `Failed to save crash log: ${result.error}`,
        timestamp: new Date(),
      })
    } catch (saveError) {
      this.logToConsole({
        level: 'error',
        message: `Failed to save crash log: ${String(saveError)}`,
        timestamp: new Date(),
      })
    }
  }

  private logToConsole(entry: LogEntry): void {
    const timestamp = entry.timestamp.toISOString()
    const prefix = `[${timestamp}] [${entry.level.toUpperCase()}]`

    const args = entry.context
      ? [prefix, entry.message, entry.context]
      : [prefix, entry.message]

    switch (entry.level) {
      case 'trace':
      case 'debug':
        console.debug(...args)
        break
      case 'info':
        console.info(...args)
        break
      case 'warn':
        console.warn(...args)
        break
      case 'error':
        console.error(...args)
        break
    }
  }

  private async logToBackend(entry: LogEntry): Promise<void> {
    const text = entry.context
      ? `${entry.message} ${safeStringify(entry.context)}`
      : entry.message
    try {
      switch (entry.level) {
        case 'trace':
          await tauriTrace(text)
          break
        case 'debug':
          await tauriDebug(text)
          break
        case 'info':
          await tauriInfo(text)
          break
        case 'warn':
          await tauriWarn(text)
          break
        case 'error':
          await tauriError(text)
          break
      }
    } catch (error) {
      this.logToConsole({
        level: 'warn',
        message: `Failed to send log to backend: ${String(error)}`,
        timestamp: new Date(),
      })
    }
  }
}

function normalizeError(error: unknown) {
  if (error instanceof Error) {
    return {
      message: `${error.name}: ${error.message}`,
      stack: error.stack,
    }
  }

  return {
    message: typeof error === 'string' ? error : safeStringify(error),
    stack: undefined,
  }
}

function toJsonContext(context?: Record<string, unknown>): JsonValue | null {
  if (!context) {
    return null
  }

  try {
    return JSON.parse(safeStringify(context)) as JsonValue
  } catch {
    return safeStringify(context)
  }
}

function safeStringify(value: unknown) {
  try {
    return JSON.stringify(value)
  } catch {
    return String(value)
  }
}

function parentPath(path: string) {
  const normalized = path.replaceAll('\\', '/')
  const index = normalized.lastIndexOf('/')
  return index === -1 ? path : path.slice(0, index)
}

// Export a singleton logger instance
export const logger = new Logger()

// Export individual logging functions for convenience
export const { trace, debug, info, warn, error } = logger
