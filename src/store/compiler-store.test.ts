import { beforeEach, describe, expect, it, vi } from 'vitest'
import { commands } from '@/lib/tauri-bindings'
import {
  DEFAULT_SDK_START_VERSION,
  defaultBuildRequest,
  useCompilerStore,
} from './compiler-store'

describe('CompilerStore', () => {
  beforeEach(() => {
    useCompilerStore.setState({
      request: defaultBuildRequest,
      artifacts: [],
      buildQueue: [],
      buildQueuePresets: [],
      sdkStartVersion: DEFAULT_SDK_START_VERSION,
    })
    window.localStorage.clear()
    vi.mocked(commands.loadBuildQueuePresets).mockResolvedValue({
      status: 'ok',
      data: { presets: [] },
    })
    vi.mocked(commands.saveBuildQueuePresets).mockResolvedValue({
      status: 'ok',
      data: null,
    })
  })

  it('detects module and package names from plugin root', () => {
    const { updatePluginRoot } = useCompilerStore.getState()

    updatePluginRoot('/Users/test/Plugins/SplineTools')

    expect(useCompilerStore.getState().request).toMatchObject({
      plugin_root: '/Users/test/Plugins/SplineTools',
      module_name: 'SplineTools',
      package_name: 'SplineTools',
    })
  })

  it('refreshes detected names when plugin root changes', () => {
    const { updatePluginRoot, updatePackageName } = useCompilerStore.getState()

    updatePluginRoot('/Users/test/Plugins/FirstPlugin')
    updatePackageName('Custom Package')
    updatePluginRoot('C:\\Plugins\\DetectedName\\')

    expect(useCompilerStore.getState().request).toMatchObject({
      plugin_root: 'C:\\Plugins\\DetectedName\\',
      module_name: 'DetectedName',
      package_name: 'DetectedName',
    })
  })

  it('uses package name as the internal module name', () => {
    const { updatePackageName } = useCompilerStore.getState()

    updatePackageName('Back Highlight')

    expect(useCompilerStore.getState().request).toMatchObject({
      module_name: 'Back Highlight',
      package_name: 'Back Highlight',
    })
  })

  it('stores build artifacts for the result panel', () => {
    const { addArtifact, setArtifacts } = useCompilerStore.getState()

    addArtifact({
      version: '2026',
      configuration: 'Release',
      kind: 'version-package',
      path: '/tmp/plugin',
    })

    expect(useCompilerStore.getState().artifacts).toHaveLength(1)

    setArtifacts([])

    expect(useCompilerStore.getState().artifacts).toEqual([])
  })

  it('uses the configured buildable versions from the selected start version', () => {
    const { setSdkStartVersion } = useCompilerStore.getState()

    setSdkStartVersion(DEFAULT_SDK_START_VERSION, ['2024.4', '2026'])

    expect(useCompilerStore.getState().request.versions).toEqual([
      '2024.4',
      '2026',
    ])
  })

  it('queues an isolated copy of the current build request', () => {
    const { addBuildQueueItem, updateRequest } = useCompilerStore.getState()

    const id = addBuildQueueItem({
      ...defaultBuildRequest,
      plugin_root: '/Plugins/Watermark',
      module_name: 'Watermark',
      package_name: 'Watermark',
      versions: ['2024.4', '2026'],
    })
    updateRequest({ package_name: 'Changed' })

    const queuedItem = useCompilerStore
      .getState()
      .buildQueue.find(item => item.id === id)

    expect(queuedItem?.request.package_name).toBe('Watermark')
    expect(queuedItem?.request.versions).toEqual(['2024.4', '2026'])
    expect(queuedItem?.status).toBe('queued')
  })

  it('updates, reorders, and removes queued builds', () => {
    const {
      addBuildQueueItem,
      moveBuildQueueItem,
      removeBuildQueueItem,
      updateBuildQueueItem,
      updateBuildQueueItemRequest,
    } = useCompilerStore.getState()

    const firstId = addBuildQueueItem({
      ...defaultBuildRequest,
      package_name: 'First',
    })
    const secondId = addBuildQueueItem({
      ...defaultBuildRequest,
      package_name: 'Second',
    })
    updateBuildQueueItem(firstId, {
      status: 'running',
      message: 'Building',
      jobId: 'build-1',
      startedAt: 1000,
      finishedAt: 2000,
    })
    updateBuildQueueItemRequest(firstId, {
      ...defaultBuildRequest,
      package_name: 'Updated',
      versions: ['2026'],
    })

    expect(useCompilerStore.getState().buildQueue[0]).toMatchObject({
      status: 'queued',
      message: null,
      jobId: null,
      startedAt: null,
      finishedAt: null,
      request: {
        package_name: 'Updated',
      },
    })
    expect(useCompilerStore.getState().buildQueue[0]?.request.versions).toEqual(
      ['2026']
    )

    moveBuildQueueItem(secondId, 'up')

    expect(useCompilerStore.getState().buildQueue[0]?.id).toBe(secondId)

    removeBuildQueueItem(firstId)

    expect(useCompilerStore.getState().buildQueue).toHaveLength(1)
  })

  it('resets queued builds to run them again', () => {
    const { addBuildQueueItem, resetBuildQueue, updateBuildQueueItem } =
      useCompilerStore.getState()

    const id = addBuildQueueItem({
      ...defaultBuildRequest,
      package_name: 'Watermark',
    })
    updateBuildQueueItem(id, {
      status: 'failed',
      message: 'SDK failed',
      jobId: 'build-1',
      startedAt: 1000,
      finishedAt: 2000,
    })

    resetBuildQueue()

    expect(useCompilerStore.getState().buildQueue[0]).toMatchObject({
      status: 'queued',
      message: null,
      jobId: null,
      startedAt: null,
      finishedAt: null,
    })
  })

  it('saves and applies build queue presets', () => {
    const { addBuildQueueItem, applyBuildQueuePreset, saveBuildQueuePreset } =
      useCompilerStore.getState()

    addBuildQueueItem({
      ...defaultBuildRequest,
      package_name: 'BackHighlight',
      versions: ['2025', '2026'],
    })
    addBuildQueueItem({
      ...defaultBuildRequest,
      package_name: 'Boghma WaterMark',
      versions: ['2026'],
    })

    const presetId = saveBuildQueuePreset('Paid release queue')
    useCompilerStore.setState({ buildQueue: [] })
    applyBuildQueuePreset(presetId ?? '')

    expect(useCompilerStore.getState().buildQueuePresets[0]).toMatchObject({
      name: 'Paid release queue',
    })
    expect(useCompilerStore.getState().buildQueue).toHaveLength(2)
    expect(useCompilerStore.getState().buildQueue[0]).toMatchObject({
      status: 'queued',
      message: null,
      jobId: null,
      startedAt: null,
      finishedAt: null,
      request: {
        package_name: 'BackHighlight',
      },
    })
    expect(useCompilerStore.getState().buildQueue[0]?.request.versions).toEqual(
      ['2025', '2026']
    )
    expect(commands.saveBuildQueuePresets).toHaveBeenCalled()
  })

  it('hydrates build queue presets from backend storage', async () => {
    vi.mocked(commands.loadBuildQueuePresets).mockResolvedValue({
      status: 'ok',
      data: {
        presets: [
          {
            id: 'preset-1',
            name: 'Loaded preset',
            requests: [
              {
                ...defaultBuildRequest,
                package_name: 'Loaded',
                versions: ['2026'],
              },
            ],
            created_at: '2026-06-25T00:00:00.000Z',
          },
        ],
      },
    })

    const { hydrateBuildQueuePresets } = useCompilerStore.getState()

    await hydrateBuildQueuePresets()

    expect(useCompilerStore.getState().buildQueuePresets[0]).toMatchObject({
      id: 'preset-1',
      name: 'Loaded preset',
      createdAt: '2026-06-25T00:00:00.000Z',
    })
    expect(
      useCompilerStore.getState().buildQueuePresets[0]?.requests[0]
    ).toMatchObject({
      package_name: 'Loaded',
    })
  })

  it('falls back to local storage presets and migrates them', async () => {
    window.localStorage.setItem(
      'c4d-plugin-compiler.buildQueuePresets',
      JSON.stringify([
        {
          id: 'legacy-preset',
          name: 'Legacy preset',
          createdAt: '2026-06-25T00:00:00.000Z',
          requests: [
            {
              ...defaultBuildRequest,
              package_name: 'Legacy',
            },
          ],
        },
      ])
    )

    const { hydrateBuildQueuePresets } = useCompilerStore.getState()

    await hydrateBuildQueuePresets()

    expect(useCompilerStore.getState().buildQueuePresets[0]).toMatchObject({
      id: 'legacy-preset',
      name: 'Legacy preset',
    })
    expect(commands.saveBuildQueuePresets).toHaveBeenCalled()
  })
})
