/**
 * Re-export generated Tauri bindings with project conventions
 *
 * This file provides type-safe access to all Tauri commands.
 * Types are auto-generated from Rust by tauri-specta.
 *
 * @example
 * ```typescript
 * import { commands, unwrapResult } from '@/lib/tauri-bindings'
 *
 * // In TanStack Query - let errors propagate
 * const prefs = unwrapResult(await commands.loadPreferences())
 *
 * // In event handlers - explicit error handling
 * const result = await commands.savePreferences(prefs)
 * if (result.status === 'error') {
 *   toast.error(result.error)
 * }
 * ```
 *
 * @see docs/developer/tauri-commands.md for full documentation
 */

export { commands, type Result } from './bindings'
export type {
  AppPreferences,
  BuildArtifact,
  BuildConfiguration,
  BuildJobId,
  BuildRequest,
  EnvironmentReport,
  InstalledSdkZip,
  JsonValue,
  PackageMode,
  RecoveryError,
  SdkResolution,
  SdkResolutionSource,
  SdkSourceMode,
  ToolStatus,
} from './bindings'

export interface BuildLogEvent {
  job_id: string
  level: string
  message: string
}

export interface BuildProgressEvent {
  job_id: string
  current: number
  total: number
  label: string
}

export interface BuildFinishedEvent {
  job_id: string
  success: boolean
  message: string
}

/**
 * Helper to unwrap a Result type, throwing on error
 */
export function unwrapResult<T, E>(
  result: { status: 'ok'; data: T } | { status: 'error'; error: E }
): T {
  if (result.status === 'ok') {
    return result.data
  }
  throw result.error
}
