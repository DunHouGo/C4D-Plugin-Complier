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

import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import {
  commands as generatedCommands,
  type Result,
  type EnvironmentReport as GeneratedEnvironmentReport,
  type SdkVersionOption,
} from './bindings'

export type {
  AppPreferences,
  BuildArtifact,
  BuildConfiguration,
  BuildJobId,
  BuildRequest,
  InstalledSdkZip,
  JsonValue,
  PackageMode,
  RecoveryError,
  SdkResolution,
  SdkResolutionSource,
  SdkSourceMode,
  SdkSourceOverride,
  SdkVersionOption,
  ToolStatus,
} from './bindings'

export type { Result }

export interface InstalledC4dVersion {
  version: string
  path: string
  sdk_version: string
  download_url: string
}

export interface EnvironmentReport extends GeneratedEnvironmentReport {
  installed_c4d_versions: InstalledC4dVersion[]
}

export interface SdkSourceConfig {
  sdk_root: string | null
}

export interface SdkRootConfig {
  sdk_root: string | null
}

export interface SdkAutoConfigReport {
  sdk_root: string | null
  installed_versions: InstalledC4dVersion[]
  versions: SdkVersionOption[]
}

async function invokeResult<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<Result<T, string>> {
  try {
    return { status: 'ok', data: await tauriInvoke<T>(command, args) }
  } catch (error) {
    if (error instanceof Error) throw error
    return { status: 'error', error: String(error) }
  }
}

export const commands = {
  ...generatedCommands,
  detectEnvironment: async () =>
    (await generatedCommands.detectEnvironment()) as Result<
      EnvironmentReport,
      string
    >,
  loadSdkSources: async () =>
    (await generatedCommands.loadSdkSources()) as Result<SdkSourceConfig, string>,
  saveSdkRootConfig: (config: SdkRootConfig) =>
    invokeResult<SdkSourceConfig>('save_sdk_root_config', { config }),
  autoConfigureSdkSources: () =>
    invokeResult<SdkAutoConfigReport>('auto_configure_sdk_sources'),
}

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
