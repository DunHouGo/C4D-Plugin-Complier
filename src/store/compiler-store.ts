import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import {
  commands,
  type BuildArtifact,
  type BuildRequest,
} from '@/lib/tauri-bindings'

export const DEFAULT_SDK_START_VERSION = '2024.4'
export type BuildQueueStatus = 'queued' | 'running' | 'success' | 'failed'

export interface BuildQueueItem {
  id: string
  request: BuildRequest
  status: BuildQueueStatus
  message: string | null
  jobId: string | null
  startedAt: number | null
  finishedAt: number | null
}

export interface BuildQueuePreset {
  id: string
  name: string
  requests: BuildRequest[]
  createdAt: string
}

export const defaultBuildRequest: BuildRequest = {
  plugin_root: '',
  module_name: '',
  package_name: '',
  versions: [DEFAULT_SDK_START_VERSION],
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
  artifacts: BuildArtifact[]
  buildQueue: BuildQueueItem[]
  buildQueuePresets: BuildQueuePreset[]
  sdkStartVersion: string
  setRequest: (request: BuildRequest) => void
  setArtifacts: (artifacts: BuildArtifact[]) => void
  addArtifact: (artifact: BuildArtifact) => void
  addBuildQueueItem: (request: BuildRequest) => string
  resetBuildQueue: () => void
  removeBuildQueueItem: (id: string) => void
  clearBuildQueue: () => void
  moveBuildQueueItem: (id: string, direction: 'up' | 'down') => void
  updateBuildQueueItem: (
    id: string,
    patch: Partial<Omit<BuildQueueItem, 'id' | 'request'>>
  ) => void
  updateBuildQueueItemRequest: (id: string, request: BuildRequest) => void
  updateRequest: (patch: Partial<BuildRequest>) => void
  updatePackageName: (packageName: string) => void
  updatePluginRoot: (pluginRoot: string) => void
  setSdkStartVersion: (version: string, availableVersions: string[]) => void
  setBuildVersions: (versions: string[]) => void
  hydrateBuildQueuePresets: () => Promise<void>
  saveBuildQueuePreset: (name?: string, id?: string) => string | null
  createBuildQueuePreset: (name?: string) => string
  renameBuildQueuePreset: (id: string, name: string) => void
  applyBuildQueuePreset: (id: string) => void
  removeBuildQueuePreset: (id: string) => void
}

export const useCompilerStore = create<CompilerState>()(
  devtools(
    set => ({
      request: defaultBuildRequest,
      artifacts: [],
      buildQueue: [],
      buildQueuePresets: loadLegacyBuildQueuePresets(),
      sdkStartVersion: DEFAULT_SDK_START_VERSION,

      setRequest: request => set({ request }, undefined, 'setRequest'),

      setArtifacts: artifacts => set({ artifacts }, undefined, 'setArtifacts'),

      addArtifact: artifact =>
        set(
          state => ({
            artifacts: state.artifacts.some(item => item.path === artifact.path)
              ? state.artifacts
              : [...state.artifacts, artifact],
          }),
          undefined,
          'addArtifact'
        ),

      addBuildQueueItem: request => {
        const id = createBuildQueueItemId()
        set(
          state => ({
            buildQueue: [
              ...state.buildQueue,
              {
                id,
                request: cloneBuildRequest(request),
                status: 'queued',
                message: null,
                jobId: null,
                startedAt: null,
                finishedAt: null,
              },
            ],
          }),
          undefined,
          'addBuildQueueItem'
        )
        return id
      },

      resetBuildQueue: () =>
        set(
          state => ({
            buildQueue: state.buildQueue.map(item => ({
              ...item,
              status: 'queued',
              message: null,
              jobId: null,
              startedAt: null,
              finishedAt: null,
            })),
          }),
          undefined,
          'resetBuildQueue'
        ),

      removeBuildQueueItem: id =>
        set(
          state => ({
            buildQueue: state.buildQueue.filter(item => item.id !== id),
          }),
          undefined,
          'removeBuildQueueItem'
        ),

      clearBuildQueue: () =>
        set({ buildQueue: [] }, undefined, 'clearBuildQueue'),

      moveBuildQueueItem: (id, direction) =>
        set(
          state => ({
            buildQueue: moveQueueItem(state.buildQueue, id, direction),
          }),
          undefined,
          'moveBuildQueueItem'
        ),

      updateBuildQueueItem: (id, patch) =>
        set(
          state => ({
            buildQueue: state.buildQueue.map(item =>
              item.id === id ? { ...item, ...patch } : item
            ),
          }),
          undefined,
          'updateBuildQueueItem'
        ),

      updateBuildQueueItemRequest: (id, request) =>
        set(
          state => ({
            buildQueue: state.buildQueue.map(item =>
              item.id === id
                ? {
                    ...item,
                    request: cloneBuildRequest(request),
                    status: 'queued',
                    message: null,
                    jobId: null,
                    startedAt: null,
                    finishedAt: null,
                  }
                : item
            ),
          }),
          undefined,
          'updateBuildQueueItemRequest'
        ),

      updateRequest: patch =>
        set(
          state => ({ request: { ...state.request, ...patch } }),
          undefined,
          'updateRequest'
        ),

      updatePackageName: packageName =>
        set(
          state => ({
            request: {
              ...state.request,
              module_name: packageName,
              package_name: packageName,
            },
          }),
          undefined,
          'updatePackageName'
        ),

      setBuildVersions: versions =>
        set(
          state => ({
            request: {
              ...state.request,
              versions,
            },
          }),
          undefined,
          'setBuildVersions'
        ),

      updatePluginRoot: pluginRoot =>
        set(
          state => {
            const detectedName = detectPluginName(pluginRoot)
            return {
              request: {
                ...state.request,
                plugin_root: pluginRoot,
                module_name: detectedName,
                package_name: detectedName,
              },
            }
          },
          undefined,
          'updatePluginRoot'
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

      hydrateBuildQueuePresets: async () => {
        const legacyPresets = loadLegacyBuildQueuePresets()
        const result = await commands.loadBuildQueuePresets()
        if (result.status === 'error') {
          if (legacyPresets.length > 0) {
            set(
              { buildQueuePresets: legacyPresets },
              undefined,
              'hydrateBuildQueuePresets'
            )
          }
          return
        }

        const diskPresets = result.data.presets
          .map(buildQueuePresetFromDisk)
          .filter(isBuildQueuePreset)
        const buildQueuePresets =
          diskPresets.length > 0 ? diskPresets : legacyPresets
        if (buildQueuePresets.length > 0 && diskPresets.length === 0) {
          void persistBuildQueuePresets(buildQueuePresets)
        }

        set({ buildQueuePresets }, undefined, 'hydrateBuildQueuePresets')
      },

      saveBuildQueuePreset: (name, id) => {
        let presetId: string | null = null
        set(
          state => {
            if (state.buildQueue.length === 0) {
              return state
            }

            const existingPreset = id
              ? state.buildQueuePresets.find(preset => preset.id === id)
              : null
            const nextPreset = createBuildQueuePreset(
              name?.trim() ||
                existingPreset?.name ||
                `Queue preset ${state.buildQueuePresets.length + 1}`,
              state.buildQueue.map(item => item.request),
              existingPreset?.id,
              existingPreset?.createdAt
            )
            presetId = nextPreset.id
            const buildQueuePresets = [
              nextPreset,
              ...state.buildQueuePresets.filter(
                preset =>
                  preset.id !== nextPreset.id && preset.name !== nextPreset.name
              ),
            ]
            void persistBuildQueuePresets(buildQueuePresets)
            return { buildQueuePresets }
          },
          undefined,
          'saveBuildQueuePreset'
        )
        return presetId
      },

      createBuildQueuePreset: name => {
        let presetId = ''
        set(
          state => {
            const nextPreset = createBuildQueuePreset(
              name?.trim() ||
                `Queue preset ${state.buildQueuePresets.length + 1}`,
              state.buildQueue.map(item => item.request)
            )
            presetId = nextPreset.id
            const buildQueuePresets = [nextPreset, ...state.buildQueuePresets]
            void persistBuildQueuePresets(buildQueuePresets)
            return { buildQueuePresets }
          },
          undefined,
          'createBuildQueuePreset'
        )
        return presetId
      },

      renameBuildQueuePreset: (id, name) =>
        set(
          state => {
            const trimmedName = name.trim()
            if (!trimmedName) {
              return state
            }

            const buildQueuePresets = state.buildQueuePresets.map(preset =>
              preset.id === id ? { ...preset, name: trimmedName } : preset
            )
            void persistBuildQueuePresets(buildQueuePresets)
            return { buildQueuePresets }
          },
          undefined,
          'renameBuildQueuePreset'
        ),

      applyBuildQueuePreset: id =>
        set(
          state => {
            const preset = state.buildQueuePresets.find(item => item.id === id)
            if (!preset) {
              return state
            }

            return {
              buildQueue: preset.requests.map(request => ({
                id: createBuildQueueItemId(),
                request: cloneBuildRequest(request),
                status: 'queued',
                message: null,
                jobId: null,
                startedAt: null,
                finishedAt: null,
              })),
            }
          },
          undefined,
          'applyBuildQueuePreset'
        ),

      removeBuildQueuePreset: id =>
        set(
          state => {
            const buildQueuePresets = state.buildQueuePresets.filter(
              preset => preset.id !== id
            )
            void persistBuildQueuePresets(buildQueuePresets)
            return { buildQueuePresets }
          },
          undefined,
          'removeBuildQueuePreset'
        ),
    }),
    {
      name: 'compiler-store',
    }
  )
)

function detectPluginName(pluginRoot: string): string {
  const normalized = pluginRoot.trim().replace(/[/\\]+$/, '')
  if (!normalized) {
    return ''
  }

  return normalized.split(/[/\\]/).pop() ?? ''
}

function cloneBuildRequest(request: BuildRequest): BuildRequest {
  return {
    ...request,
    versions: [...request.versions],
  }
}

function moveQueueItem(
  queue: BuildQueueItem[],
  id: string,
  direction: 'up' | 'down'
): BuildQueueItem[] {
  const index = queue.findIndex(item => item.id === id)
  if (index < 0) {
    return queue
  }

  const nextIndex = direction === 'up' ? index - 1 : index + 1
  if (nextIndex < 0 || nextIndex >= queue.length) {
    return queue
  }

  const nextQueue = [...queue]
  const currentItem = nextQueue[index]
  const targetItem = nextQueue[nextIndex]
  if (!currentItem || !targetItem) {
    return queue
  }

  nextQueue[index] = targetItem
  nextQueue[nextIndex] = currentItem
  return nextQueue
}

function createBuildQueueItemId(): string {
  return `queue-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
}

function createBuildQueuePresetId(): string {
  return `preset-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
}

function createBuildQueuePreset(
  name: string,
  requests: BuildRequest[],
  id = createBuildQueuePresetId(),
  createdAt = new Date().toISOString()
): BuildQueuePreset {
  return {
    id,
    name,
    requests: requests.map(cloneBuildRequest),
    createdAt,
  }
}

const BUILD_QUEUE_PRESETS_KEY = 'c4d-plugin-compiler.buildQueuePresets'

interface DiskBuildQueuePreset {
  id: string
  name: string
  requests: BuildRequest[]
  created_at: string
}

function loadLegacyBuildQueuePresets(): BuildQueuePreset[] {
  if (typeof window === 'undefined') {
    return []
  }

  try {
    const text = window.localStorage.getItem(BUILD_QUEUE_PRESETS_KEY)
    if (!text) {
      return []
    }
    const value = JSON.parse(text) as BuildQueuePreset[]
    if (!Array.isArray(value)) {
      return []
    }
    return value.filter(isBuildQueuePreset)
  } catch {
    return []
  }
}

async function persistBuildQueuePresets(presets: BuildQueuePreset[]) {
  window.localStorage.setItem(BUILD_QUEUE_PRESETS_KEY, JSON.stringify(presets))
  await commands.saveBuildQueuePresets({
    presets: presets.map(buildQueuePresetToDisk),
  })
}

function buildQueuePresetToDisk(
  preset: BuildQueuePreset
): DiskBuildQueuePreset {
  return {
    id: preset.id,
    name: preset.name,
    requests: preset.requests.map(cloneBuildRequest),
    created_at: preset.createdAt,
  }
}

function buildQueuePresetFromDisk(
  preset: DiskBuildQueuePreset
): BuildQueuePreset {
  return {
    id: preset.id,
    name: preset.name,
    requests: preset.requests.map(cloneBuildRequest),
    createdAt: preset.created_at,
  }
}

function isBuildQueuePreset(value: unknown): value is BuildQueuePreset {
  if (!value || typeof value !== 'object') {
    return false
  }

  const preset = value as BuildQueuePreset
  return (
    typeof preset.id === 'string' &&
    typeof preset.name === 'string' &&
    typeof preset.createdAt === 'string' &&
    Array.isArray(preset.requests)
  )
}
