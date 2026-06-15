import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import type { BuildRequest } from '@/lib/tauri-bindings'

export const DEFAULT_SDK_START_VERSION = '2024.4'

export const defaultBuildRequest: BuildRequest = {
  plugin_root: 'E:\\Boghma\\boghma hub\\Done Paid\\Boghma-WaterMark',
  module_name: 'postwatermark',
  package_name: 'Boghma WaterMark',
  versions: [DEFAULT_SDK_START_VERSION, '2025', '2026'],
  configuration: 'Release',
  sdk_source: 'ConfiguredThenInstalledThenOfficial',
  package_mode: 'Both',
  zip_enabled: true,
  clean_output: true,
  refresh_sdk_cache: false,
  output_dir: null,
}

interface CompilerState {
  request: BuildRequest
  sdkStartVersion: string
  setRequest: (request: BuildRequest) => void
  updateRequest: (patch: Partial<BuildRequest>) => void
  setSdkStartVersion: (version: string, availableVersions: string[]) => void
}

export const useCompilerStore = create<CompilerState>()(
  devtools(
    set => ({
      request: defaultBuildRequest,
      sdkStartVersion: DEFAULT_SDK_START_VERSION,

      setRequest: request => set({ request }, undefined, 'setRequest'),

      updateRequest: patch =>
        set(
          state => ({ request: { ...state.request, ...patch } }),
          undefined,
          'updateRequest'
        ),

      setSdkStartVersion: (version, availableVersions) =>
        set(
          state => {
            const startIndex = availableVersions.indexOf(version)
            const nextVersions =
              startIndex >= 0
                ? availableVersions.slice(startIndex)
                : state.request.versions

            return {
              sdkStartVersion: version,
              request: {
                ...state.request,
                versions: nextVersions,
              },
            }
          },
          undefined,
          'setSdkStartVersion'
        ),
    }),
    {
      name: 'compiler-store',
    }
  )
)
