import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import type { BuildArtifact, BuildRequest } from '@/lib/tauri-bindings'

export const DEFAULT_SDK_START_VERSION = '2024.4'
export type BuildQueueStatus = 'queued' | 'running' | 'success' | 'failed'

export interface BuildQueueItem {
  id: string
  request: BuildRequest
  status: BuildQueueStatus
  message: string | null
  jobId: string | null
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
  sdkStartVersion: string
  setRequest: (request: BuildRequest) => void
  setArtifacts: (artifacts: BuildArtifact[]) => void
  addArtifact: (artifact: BuildArtifact) => void
  addBuildQueueItem: (request: BuildRequest) => string
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
}

export const useCompilerStore = create<CompilerState>()(
  devtools(
    set => ({
      request: defaultBuildRequest,
      artifacts: [],
      buildQueue: [],
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
              },
            ],
          }),
          undefined,
          'addBuildQueueItem'
        )
        return id
      },

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
