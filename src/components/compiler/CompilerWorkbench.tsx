import { useEffect, useMemo, useRef, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { save as saveDialog } from '@tauri-apps/plugin-dialog'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { useTranslation } from 'react-i18next'
import {
  ArrowDown,
  ArrowUp,
  Box,
  CheckCircle2,
  Copy,
  FilePenLine,
  FolderOpen,
  Hammer,
  ListFilter,
  ListPlus,
  Play,
  RefreshCw,
  RotateCcw,
  Save,
  Square,
  Trash2,
  XCircle,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Switch } from '@/components/ui/switch'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { logger } from '@/lib/logger'
import { notify } from '@/lib/notifications'
import { cn } from '@/lib/utils'
import { useCompilerStore } from '@/store/compiler-store'
import { HelpHint } from './HelpHint'
import { PathPicker } from './PathPicker'
import {
  commands,
  type BuildArtifact,
  type BuildConfiguration,
  type BuildFinishedEvent,
  type BuildLogEvent,
  type BuildProgressEvent,
  type EnvironmentReport,
  type PackageMode,
  type SdkResolution,
  type SdkVersionOption,
} from '@/lib/tauri-bindings'

type BuildState = 'idle' | 'running' | 'success' | 'failed'
type LogFilter = 'all' | 'warn' | 'error'
type OutputTab = 'logs' | 'artifacts'
type LogCategoryFilter =
  | 'all'
  | 'build'
  | 'sdk'
  | 'cmake'
  | 'xcode'
  | 'toolchain'
  | 'package'
  | 'system'
  | 'command'

const LOG_CATEGORY_FILTERS: LogCategoryFilter[] = [
  'all',
  'sdk',
  'cmake',
  'xcode',
  'toolchain',
  'package',
  'system',
]

export function CompilerWorkbench() {
  const { t } = useTranslation()
  const request = useCompilerStore(state => state.request)
  const updateRequest = useCompilerStore(state => state.updateRequest)
  const updatePackageName = useCompilerStore(state => state.updatePackageName)
  const updatePluginRoot = useCompilerStore(state => state.updatePluginRoot)
  const artifacts = useCompilerStore(state => state.artifacts)
  const setArtifacts = useCompilerStore(state => state.setArtifacts)
  const addArtifact = useCompilerStore(state => state.addArtifact)
  const buildQueue = useCompilerStore(state => state.buildQueue)
  const buildQueuePresets = useCompilerStore(state => state.buildQueuePresets)
  const addBuildQueueItem = useCompilerStore(state => state.addBuildQueueItem)
  const resetBuildQueue = useCompilerStore(state => state.resetBuildQueue)
  const removeBuildQueueItem = useCompilerStore(
    state => state.removeBuildQueueItem
  )
  const clearBuildQueue = useCompilerStore(state => state.clearBuildQueue)
  const moveBuildQueueItem = useCompilerStore(state => state.moveBuildQueueItem)
  const updateBuildQueueItem = useCompilerStore(
    state => state.updateBuildQueueItem
  )
  const updateBuildQueueItemRequest = useCompilerStore(
    state => state.updateBuildQueueItemRequest
  )
  const saveBuildQueuePreset = useCompilerStore(
    state => state.saveBuildQueuePreset
  )
  const createBuildQueuePreset = useCompilerStore(
    state => state.createBuildQueuePreset
  )
  const renameBuildQueuePreset = useCompilerStore(
    state => state.renameBuildQueuePreset
  )
  const applyBuildQueuePreset = useCompilerStore(
    state => state.applyBuildQueuePreset
  )
  const removeBuildQueuePreset = useCompilerStore(
    state => state.removeBuildQueuePreset
  )
  const sdkStartVersion = useCompilerStore(state => state.sdkStartVersion)
  const setSdkStartVersion = useCompilerStore(state => state.setSdkStartVersion)
  const setBuildVersions = useCompilerStore(state => state.setBuildVersions)
  const hydrateBuildQueuePresets = useCompilerStore(
    state => state.hydrateBuildQueuePresets
  )
  const [environment, setEnvironment] = useState<EnvironmentReport | null>(null)
  const [sdkResolutions, setSdkResolutions] = useState<SdkResolution[]>([])
  const [sdkVersions, setSdkVersions] = useState<SdkVersionOption[]>([])
  const [logs, setLogs] = useState<BuildLogEvent[]>([])
  const [logFilter, setLogFilter] = useState<LogFilter>('all')
  const [logCategoryFilter, setLogCategoryFilter] =
    useState<LogCategoryFilter>('all')
  const [outputTab, setOutputTab] = useState<OutputTab>('logs')
  const [autoScrollLogs, setAutoScrollLogs] = useState(true)
  const [logActionMessage, setLogActionMessage] = useState<string | null>(null)
  const [selectedQueuePresetId, setSelectedQueuePresetId] = useState('')
  const [queuePresetName, setQueuePresetName] = useState('')
  const [editingQueuePresetName, setEditingQueuePresetName] = useState(false)
  const [progress, setProgress] = useState<BuildProgressEvent | null>(null)
  const [jobId, setJobId] = useState<string | null>(null)
  const [runningQueueItemId, setRunningQueueItemId] = useState<string | null>(
    null
  )
  const [buildStartedAt, setBuildStartedAt] = useState<number | null>(null)
  const [buildFinishedAt, setBuildFinishedAt] = useState<number | null>(null)
  const [durationTick, setDurationTick] = useState(() => Date.now())
  const [editingQueueItemId, setEditingQueueItemId] = useState<string | null>(
    null
  )
  const [stopQueueAfterCurrent, setStopQueueAfterCurrent] = useState(false)
  const [state, setState] = useState<BuildState>('idle')
  const sdkStartVersionRef = useRef(sdkStartVersion)
  const buildFinishedResolverRef = useRef<{
    jobId: string
    resolve: (event: BuildFinishedEvent) => void
  } | null>(null)
  const stopQueueAfterCurrentRef = useRef(stopQueueAfterCurrent)
  const runningQueueItemIdRef = useRef(runningQueueItemId)
  const jobIdRef = useRef(jobId)
  const compilerPlatform = environment?.compiler_platform ?? 'Unsupported'
  const latestError = useMemo(
    () =>
      [...logs]
        .reverse()
        .find(item => item.level === 'error')
        ?.message.trim(),
    [logs]
  )
  const filteredLogs = useMemo(
    () =>
      logs.filter(
        item =>
          matchesLogLevelFilter(item, logFilter) &&
          matchesLogCategoryFilter(item, logCategoryFilter)
      ),
    [logCategoryFilter, logFilter, logs]
  )
  const filteredBuildLogText = useMemo(
    () => formatBuildLog(filteredLogs),
    [filteredLogs]
  )
  const logEndRef = useRef<HTMLDivElement>(null)

  const versionNames = useMemo(
    () => sdkVersions.map(version => version.version),
    [sdkVersions]
  )
  const buildableVersionNames = useMemo(
    () =>
      sdkVersions.filter(isBuildableSdkVersion).map(version => version.version),
    [sdkVersions]
  )

  useEffect(() => {
    sdkStartVersionRef.current = sdkStartVersion
  }, [sdkStartVersion])

  useEffect(() => {
    stopQueueAfterCurrentRef.current = stopQueueAfterCurrent
  }, [stopQueueAfterCurrent])

  useEffect(() => {
    runningQueueItemIdRef.current = runningQueueItemId
  }, [runningQueueItemId])

  useEffect(() => {
    jobIdRef.current = jobId
  }, [jobId])

  useEffect(() => {
    if (state !== 'running' || buildStartedAt === null) return

    setDurationTick(Date.now())
    const timer = window.setInterval(() => {
      setDurationTick(Date.now())
    }, 1000)

    return () => window.clearInterval(timer)
  }, [buildStartedAt, state])

  useEffect(() => {
    const preset = buildQueuePresets.find(
      item => item.id === selectedQueuePresetId
    )
    setQueuePresetName(preset?.name ?? '')
    setEditingQueuePresetName(false)
  }, [buildQueuePresets, selectedQueuePresetId])

  useEffect(() => {
    void hydrateBuildQueuePresets()
  }, [hydrateBuildQueuePresets])

  useEffect(() => {
    if (!autoScrollLogs) return
    if (typeof logEndRef.current?.scrollIntoView !== 'function') return
    logEndRef.current.scrollIntoView({ block: 'end' })
  }, [autoScrollLogs, filteredLogs.length])

  useEffect(() => {
    async function loadInitialEnvironment() {
      const result = await commands.detectEnvironment()
      if (result.status === 'ok') {
        setEnvironment(result.data)
      } else {
        setLogs(current => [...current, systemLog('error', result.error)])
      }
    }

    async function loadInitialSdkVersions() {
      const result = await commands.listSdkVersions()
      if (result.status === 'ok') {
        setSdkVersions(result.data)
        syncBuildVersionsFromSdkOptions(
          result.data,
          sdkStartVersionRef.current,
          setSdkStartVersion,
          setBuildVersions
        )
      } else {
        setLogs(current => [...current, systemLog('error', result.error)])
      }
    }

    void loadInitialEnvironment()
    void loadInitialSdkVersions()

    const unlisten = Promise.all([
      listen<BuildLogEvent>('build://log', event => {
        setLogs(current => [...current.slice(-500), event.payload])
      }),
      listen<BuildProgressEvent>('build://progress', event => {
        setProgress(event.payload)
      }),
      listen<BuildArtifact>('build://artifact', event => {
        addArtifact(event.payload)
      }),
      listen<BuildFinishedEvent>('build://finished', event => {
        setLogs(current => [
          ...current,
          {
            job_id: event.payload.job_id,
            level: event.payload.success ? 'info' : 'error',
            category: 'system',
            timestamp: String(Date.now()),
            message: event.payload.message,
          },
        ])
        if (event.payload.job_id !== jobIdRef.current) return

        setState(event.payload.success ? 'success' : 'failed')
        setBuildFinishedAt(Date.now())
        const queueItemId = runningQueueItemIdRef.current
        if (queueItemId) {
          updateBuildQueueItem(queueItemId, {
            status: event.payload.success ? 'success' : 'failed',
            message: event.payload.message,
            finishedAt: Date.now(),
          })
        }

        if (buildFinishedResolverRef.current?.jobId === event.payload.job_id) {
          buildFinishedResolverRef.current.resolve(event.payload)
          buildFinishedResolverRef.current = null
        }
      }),
    ])

    return () => {
      void unlisten
        .then(items => {
          for (const item of items) {
            try {
              Promise.resolve(item()).catch(error => {
                logger.warn('Failed to unregister build event listener', {
                  error,
                })
              })
            } catch (error) {
              logger.warn('Failed to unregister build event listener', {
                error,
              })
            }
          }
        })
        .catch(error => {
          logger.warn('Failed to resolve build event listeners', { error })
        })
    }
  }, [addArtifact, setBuildVersions, setSdkStartVersion, updateBuildQueueItem])

  async function refreshSdkVersions() {
    const result = await commands.listSdkVersions()
    if (result.status === 'ok') {
      setSdkVersions(result.data)
      syncBuildVersionsFromSdkOptions(
        result.data,
        sdkStartVersionRef.current,
        setSdkStartVersion,
        setBuildVersions
      )
    } else {
      setLogs(current => [...current, systemLog('error', result.error)])
    }
  }

  async function refreshEnvironment() {
    const result = await commands.detectEnvironment()
    if (result.status === 'ok') {
      setEnvironment(result.data)
    } else {
      setLogs(current => [...current, systemLog('error', result.error)])
    }
  }

  async function resolveSdks(nextRequest = request) {
    const result = await commands.resolveSdkVersions(nextRequest)
    if (result.status === 'ok') {
      setSdkResolutions(result.data)
      return true
    }

    setLogs(current => [...current, systemLog('error', result.error)])
    return false
  }

  async function startBuild() {
    const nextRequest = useCompilerStore.getState().request
    const startedAt = Date.now()
    setRunningQueueItemId(null)
    runningQueueItemIdRef.current = null
    setStopQueueAfterCurrent(false)
    stopQueueAfterCurrentRef.current = false
    setBuildStartedAt(startedAt)
    setBuildFinishedAt(null)
    setDurationTick(startedAt)
    setLogs([])
    setArtifacts([])
    setProgress(null)
    await runBuildRequest(nextRequest, {
      startedAt,
      queueItemId: null,
      queueItemName: queueItemLabel({ request: nextRequest }),
      notifyOnComplete: true,
    })
  }

  async function cancelBuild() {
    if (jobIdRef.current) {
      await commands.cancelBuild(jobIdRef.current)
    }
    if (runningQueueItemIdRef.current) {
      updateBuildQueueItem(runningQueueItemIdRef.current, {
        status: 'failed',
        message: t('compiler.queue.cancelled'),
        finishedAt: Date.now(),
      })
    }
    setStopQueueAfterCurrent(true)
    stopQueueAfterCurrentRef.current = true
    setState('idle')
    setBuildStartedAt(null)
    setBuildFinishedAt(null)
  }

  function addCurrentRequestToQueue() {
    const nextRequest = useCompilerStore.getState().request
    if (editingQueueItemId) {
      updateBuildQueueItemRequest(editingQueueItemId, nextRequest)
      setLogs(current => [
        ...current,
        systemLog(
          'info',
          t('compiler.queue.updatedLog', {
            name: queueItemLabel({ request: nextRequest }),
            versions: nextRequest.versions.join(', '),
          })
        ),
      ])
      setEditingQueueItemId(null)
      return
    }

    addBuildQueueItem(nextRequest)
    setLogs(current => [
      ...current,
      systemLog(
        'info',
        t('compiler.queue.addedLog', {
          name: queueItemLabel({ request: nextRequest }),
          versions: nextRequest.versions.join(', '),
        })
      ),
    ])
  }

  function editQueueItem(id: string) {
    const item = useCompilerStore
      .getState()
      .buildQueue.find(queueItem => queueItem.id === id)
    if (!item) return
    useCompilerStore.getState().setRequest(item.request)
    setEditingQueueItemId(id)
  }

  function cancelQueueItemEdit() {
    setEditingQueueItemId(null)
  }

  function removeQueueItem(id: string) {
    removeBuildQueueItem(id)
    if (editingQueueItemId === id) {
      setEditingQueueItemId(null)
    }
  }

  function clearQueue() {
    clearBuildQueue()
    setEditingQueueItemId(null)
  }

  function restartQueue() {
    resetBuildQueue()
    setRunningQueueItemId(null)
    runningQueueItemIdRef.current = null
    setStopQueueAfterCurrent(false)
    stopQueueAfterCurrentRef.current = false
  }

  function saveQueuePreset() {
    const id = saveBuildQueuePreset(
      queuePresetName,
      selectedQueuePresetId || undefined
    )
    if (!id) return
    setSelectedQueuePresetId(id)
    const preset = useCompilerStore
      .getState()
      .buildQueuePresets.find(item => item.id === id)
    setLogs(current => [
      ...current,
      systemLog(
        'info',
        t('compiler.queue.presetSavedLog', {
          name: preset?.name ?? t('compiler.queue.presetFallbackName'),
        })
      ),
    ])
  }

  function createQueuePreset() {
    const id = createBuildQueuePreset(queuePresetName)
    setSelectedQueuePresetId(id)
    setEditingQueuePresetName(false)
  }

  function renameSelectedQueuePreset() {
    if (!selectedQueuePresetId) return
    renameBuildQueuePreset(selectedQueuePresetId, queuePresetName)
    setEditingQueuePresetName(false)
  }

  function toggleQueuePresetNameEdit() {
    if (!selectedQueuePresetId) return
    if (editingQueuePresetName) {
      renameSelectedQueuePreset()
      return
    }
    setEditingQueuePresetName(true)
  }

  function selectQueuePreset(id: string) {
    setSelectedQueuePresetId(id)
    applyBuildQueuePreset(id)
    setEditingQueueItemId(null)
    const preset = useCompilerStore
      .getState()
      .buildQueuePresets.find(item => item.id === id)
    if (!preset) return
    setLogs(current => [
      ...current,
      systemLog(
        'info',
        t('compiler.queue.presetLoadedLog', { name: preset.name })
      ),
    ])
  }

  function deleteSelectedQueuePreset() {
    if (!selectedQueuePresetId) return
    removeBuildQueuePreset(selectedQueuePresetId)
    setSelectedQueuePresetId('')
    setQueuePresetName('')
    setEditingQueuePresetName(false)
  }

  function clearLogs() {
    setLogs([])
    setLogActionMessage(null)
  }

  async function startQueuedBuilds() {
    const pendingQueue = useCompilerStore
      .getState()
      .buildQueue.filter(item => item.status === 'queued')
    if (pendingQueue.length === 0) return

    const queueStartedAt = Date.now()
    let successCount = 0
    let failedName = ''
    let stopped = false

    setStopQueueAfterCurrent(false)
    stopQueueAfterCurrentRef.current = false
    setLogs([])
    setArtifacts([])
    setProgress(null)

    for (const item of pendingQueue) {
      if (stopQueueAfterCurrentRef.current) {
        stopped = true
        break
      }

      const startedAt = Date.now()
      setRunningQueueItemId(item.id)
      runningQueueItemIdRef.current = item.id
      setBuildStartedAt(startedAt)
      setBuildFinishedAt(null)
      setDurationTick(startedAt)
      updateBuildQueueItem(item.id, {
        status: 'running',
        message: t('compiler.queue.running'),
        jobId: null,
        startedAt,
        finishedAt: null,
      })
      setLogs(current => [
        ...current,
        systemLog(
          'info',
          t('compiler.queue.startLog', {
            name: queueItemLabel(item),
            versions: item.request.versions.join(', '),
          })
        ),
      ])

      const result = await runBuildRequest(item.request, {
        startedAt,
        queueItemId: item.id,
        queueItemName: queueItemLabel(item),
        notifyOnComplete: false,
      })
      updateBuildQueueItem(item.id, {
        status: result.success ? 'success' : 'failed',
        message: result.message,
        finishedAt: Date.now(),
      })

      if (!result.success) {
        failedName = queueItemLabel(item)
        break
      }

      successCount += 1
    }

    setRunningQueueItemId(null)
    runningQueueItemIdRef.current = null
    stopped = stopped || stopQueueAfterCurrentRef.current
    showQueueSummary({
      total: pendingQueue.length,
      successCount,
      failedName,
      stopped,
      duration: formatDuration(Date.now() - queueStartedAt),
    })
  }

  async function runBuildRequest(
    nextRequest: typeof request,
    options: {
      startedAt: number
      queueItemId: string | null
      queueItemName: string
      notifyOnComplete: boolean
    }
  ) {
    setState('running')
    setProgress(null)
    setBuildStartedAt(options.startedAt)
    setBuildFinishedAt(null)
    setDurationTick(options.startedAt)
    const resolved = await resolveSdks(nextRequest)
    if (!resolved) {
      const message = t('compiler.queue.sdkResolveFailed')
      setState('failed')
      setBuildFinishedAt(Date.now())
      if (options.notifyOnComplete) {
        showBuildSummary({
          name: options.queueItemName,
          success: false,
          message,
          duration: formatDuration(Date.now() - options.startedAt),
        })
      }
      return { success: false, message }
    }

    const result = await commands.startBuild(nextRequest)
    if (result.status === 'error') {
      setState('failed')
      setLogs(current => [...current, systemLog('error', result.error)])
      setBuildFinishedAt(Date.now())
      if (options.notifyOnComplete) {
        showBuildSummary({
          name: options.queueItemName,
          success: false,
          message: result.error,
          duration: formatDuration(Date.now() - options.startedAt),
        })
      }
      return { success: false, message: result.error }
    }

    setJobId(result.data.id)
    jobIdRef.current = result.data.id
    if (options.queueItemId) {
      updateBuildQueueItem(options.queueItemId, {
        jobId: result.data.id,
      })
    }

    const finished = await waitForBuildFinished(result.data.id)
    const finishedAt = Date.now()
    setBuildFinishedAt(finishedAt)
    if (options.notifyOnComplete) {
      showBuildSummary({
        name: options.queueItemName,
        success: finished.success,
        message: finished.message,
        duration: formatDuration(finishedAt - options.startedAt),
      })
    }

    return {
      success: finished.success,
      message: finished.message,
    }
  }

  function waitForBuildFinished(activeJobId: string) {
    return new Promise<BuildFinishedEvent>(resolve => {
      buildFinishedResolverRef.current = { jobId: activeJobId, resolve }
    })
  }

  async function copyBuildLog() {
    if (!filteredBuildLogText) return
    try {
      await writeText(filteredBuildLogText)
      setLogActionMessage(t('compiler.logs.copied'))
    } catch (error) {
      setLogActionMessage(t('compiler.logs.copyFailed', { message: error }))
    }
  }

  async function saveBuildLog() {
    if (!filteredBuildLogText) return
    try {
      const path = await saveDialog({
        defaultPath: `c4d-build-${new Date().toISOString().replace(/[:.]/g, '-')}.log`,
        filters: [{ name: 'Log', extensions: ['log', 'txt'] }],
      })
      if (!path) return
      const result = await commands.saveBuildLog(path, filteredBuildLogText)
      if (result.status === 'error') {
        throw result.error
      }
      setLogActionMessage(t('compiler.logs.saved'))
    } catch (error) {
      setLogActionMessage(t('compiler.logs.saveFailed', { message: error }))
    }
  }

  function showQueueSummary({
    total,
    successCount,
    failedName,
    stopped,
    duration,
  }: {
    total: number
    successCount: number
    failedName: string
    stopped: boolean
    duration: string
  }) {
    const failedCount = failedName ? 1 : 0
    const skippedCount = total - successCount - failedCount
    const message = failedName
      ? t('compiler.queue.summaryFailed', {
          success: successCount,
          total,
          failed: failedName,
          duration,
        })
      : stopped
        ? t('compiler.queue.summaryStopped', {
            success: successCount,
            skipped: skippedCount,
            total,
            duration,
          })
        : t('compiler.queue.summarySuccess', {
            success: successCount,
            total,
            duration,
          })

    setLogs(current => [
      ...current,
      systemLog(failedName ? 'error' : 'info', message),
    ])
    void notify(t('compiler.queue.summaryTitle'), message, {
      type: failedName ? 'error' : 'success',
      duration: 8000,
    })
  }

  function showBuildSummary({
    name,
    success,
    message,
    duration,
  }: {
    name: string
    success: boolean
    message: string
    duration: string
  }) {
    const title = t('compiler.build.summaryTitle')
    const content = success
      ? t('compiler.build.summarySuccess', { name, duration })
      : t('compiler.build.summaryFailed', { name, duration, message })

    setLogs(current => [
      ...current,
      systemLog(success ? 'info' : 'error', content),
    ])
    void notify(title, content, {
      type: success ? 'success' : 'error',
      duration: 8000,
    })
  }

  return (
    <div className="grid h-full min-w-0 grid-cols-[minmax(380px,420px)_minmax(0,1fr)] overflow-hidden">
      <aside className="flex min-h-0 flex-col overflow-hidden border-r bg-muted/20">
        <div className="border-b px-4 py-3">
          <div className="flex items-center gap-2 text-sm font-semibold">
            <Hammer className="size-4" />
            {t('compiler.title')}
          </div>
          <div className="mt-1 text-xs text-muted-foreground">
            {t('compiler.subtitle')}
          </div>
        </div>
        <ScrollArea className="min-h-0 flex-1 overflow-hidden">
          <div className="space-y-4 p-4">
            <Field
              label={t('compiler.fields.pluginRoot')}
              help={t('compiler.help.pluginRoot')}
            >
              <PathPicker
                value={request.plugin_root}
                title={t('compiler.picker.pluginRoot')}
                onChange={value => updatePluginRoot(value)}
              />
            </Field>
            <Field
              label={t('compiler.fields.package')}
              help={t('compiler.help.package')}
            >
              <Input
                value={request.package_name}
                onChange={event => updatePackageName(event.target.value)}
              />
            </Field>
            <Field
              label={t('compiler.fields.c4dVersions')}
              help={t('compiler.help.c4dVersions')}
            >
              <Select
                value={sdkStartVersion}
                onValueChange={value =>
                  setSdkStartVersion(
                    value,
                    buildableVersionNames.length > 0
                      ? buildableVersionNames
                      : versionNames
                  )
                }
              >
                <SelectTrigger className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {sdkVersions.map(version => (
                    <SelectItem
                      key={version.version}
                      value={version.version}
                      disabled={
                        buildableVersionNames.length > 0 &&
                        !buildableVersionNames.includes(version.version)
                      }
                    >
                      {version.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <div className="mt-2 flex flex-wrap gap-1.5">
                {request.versions.map(version => (
                  <Badge key={version} variant="outline">
                    {version}
                  </Badge>
                ))}
              </div>
            </Field>
            <div className="grid grid-cols-2 gap-3">
              <Field
                label={t('compiler.fields.configuration')}
                help={t('compiler.help.configuration')}
              >
                <Select
                  value={request.configuration}
                  onValueChange={value =>
                    updateRequest({
                      configuration: value as BuildConfiguration,
                    })
                  }
                >
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="Debug">
                      {t('compiler.configuration.debug')}
                    </SelectItem>
                    <SelectItem value="Release">
                      {t('compiler.configuration.release')}
                    </SelectItem>
                    <SelectItem value="Both">
                      {t('compiler.configuration.both')}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </Field>
              <Field
                label={t('compiler.fields.packageMode')}
                help={t('compiler.help.packageMode')}
              >
                <Select
                  value={request.package_mode}
                  onValueChange={value =>
                    updateRequest({ package_mode: value as PackageMode })
                  }
                >
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="Merged">
                      {t('compiler.packageMode.merged')}
                    </SelectItem>
                    <SelectItem value="PerVersion">
                      {t('compiler.packageMode.perVersion')}
                    </SelectItem>
                    <SelectItem value="Both">
                      {t('compiler.packageMode.both')}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </Field>
            </div>
            <Field
              label={t('compiler.fields.outputDir')}
              help={t('compiler.help.outputDir')}
            >
              <PathPicker
                value={request.output_dir ?? ''}
                placeholder={t('compiler.placeholder.outputDir')}
                title={t('compiler.picker.outputDir')}
                onChange={value => updateRequest({ output_dir: value || null })}
              />
            </Field>
            <div className="grid grid-cols-[repeat(auto-fit,minmax(112px,1fr))] gap-2">
              <Toggle
                checked={request.zip_enabled}
                label={t('compiler.toggles.zip')}
                help={t('compiler.help.zip')}
                onCheckedChange={value => updateRequest({ zip_enabled: value })}
              />
              <Toggle
                checked={request.clean_output}
                label={t('compiler.toggles.clean')}
                help={t('compiler.help.clean')}
                onCheckedChange={value =>
                  updateRequest({ clean_output: value })
                }
              />
              <Toggle
                checked={request.refresh_sdk_cache}
                label={t('compiler.toggles.refreshSdk')}
                help={t('compiler.help.refreshSdk')}
                onCheckedChange={value =>
                  updateRequest({ refresh_sdk_cache: value })
                }
              />
            </div>
            <div className="grid grid-cols-[minmax(0,1fr)_repeat(4,36px)] gap-2 pt-1">
              <Button
                className="flex-1"
                disabled={state === 'running'}
                onClick={() => void startBuild()}
              >
                <Play className="size-4" />
                {t('compiler.actions.build')}
                <HelpHint text={t('compiler.help.build')} />
              </Button>
              <Button
                variant="outline"
                size="icon"
                title={
                  editingQueueItemId
                    ? t('compiler.actions.updateQueueItem')
                    : t('compiler.actions.addToQueue')
                }
                disabled={state === 'running'}
                onClick={addCurrentRequestToQueue}
              >
                {editingQueueItemId ? (
                  <FilePenLine className="size-4" />
                ) : (
                  <ListPlus className="size-4" />
                )}
              </Button>
              {editingQueueItemId ? (
                <Button
                  variant="outline"
                  size="icon"
                  title={t('compiler.actions.cancelQueueEdit')}
                  disabled={state === 'running'}
                  onClick={cancelQueueItemEdit}
                >
                  <XCircle className="size-4" />
                </Button>
              ) : null}
              <Button
                variant="outline"
                size="icon"
                title={t('compiler.actions.resolveSdks')}
                onClick={() => void resolveSdks()}
              >
                <Box className="size-4" />
              </Button>
              <Button
                variant="outline"
                size="icon"
                title={t('compiler.actions.refreshEnvironment')}
                onClick={() => {
                  void refreshEnvironment()
                  void refreshSdkVersions()
                }}
              >
                <RefreshCw className="size-4" />
              </Button>
              <Button
                variant="outline"
                size="icon"
                title={t('compiler.actions.cancel')}
                disabled={state !== 'running'}
                onClick={() => void cancelBuild()}
              >
                <Square className="size-4" />
              </Button>
            </div>
            <section className="rounded-md border bg-background">
              <div className="grid grid-cols-[auto_minmax(0,1fr)_repeat(4,32px)] items-center gap-1 p-3">
                <div className="flex min-w-0 items-center gap-2 pr-1">
                  <Save className="size-4 text-muted-foreground" />
                  <span className="truncate text-sm font-medium">
                    {t('compiler.panels.queuePresets')}
                  </span>
                </div>
                {editingQueuePresetName ? (
                  <Input
                    className="h-8 min-w-0 text-xs"
                    value={queuePresetName}
                    placeholder={t('compiler.queue.presetNamePlaceholder')}
                    disabled={state === 'running'}
                    autoFocus
                    onChange={event => setQueuePresetName(event.target.value)}
                    onBlur={renameSelectedQueuePreset}
                    onKeyDown={event => {
                      if (event.key === 'Enter') {
                        renameSelectedQueuePreset()
                        event.currentTarget.blur()
                      }
                      if (event.key === 'Escape') {
                        const preset = buildQueuePresets.find(
                          item => item.id === selectedQueuePresetId
                        )
                        setQueuePresetName(preset?.name ?? '')
                        setEditingQueuePresetName(false)
                        event.currentTarget.blur()
                      }
                    }}
                  />
                ) : (
                  <Select
                    value={selectedQueuePresetId}
                    onValueChange={selectQueuePreset}
                    disabled={
                      state === 'running' || buildQueuePresets.length === 0
                    }
                  >
                    <SelectTrigger
                      className="h-8 min-w-0 text-xs"
                      title={t('compiler.actions.selectQueuePreset')}
                    >
                      <SelectValue
                        placeholder={t('compiler.queue.presetSelect')}
                      />
                    </SelectTrigger>
                    <SelectContent>
                      {buildQueuePresets.map(preset => (
                        <SelectItem key={preset.id} value={preset.id}>
                          {preset.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                )}
                <Button
                  variant="outline"
                  size="icon-sm"
                  title={t('compiler.actions.renameQueuePreset')}
                  disabled={state === 'running' || !selectedQueuePresetId}
                  onClick={toggleQueuePresetNameEdit}
                >
                  <FilePenLine className="size-3.5" />
                </Button>
                <Button
                  variant="outline"
                  size="icon-sm"
                  title={t('compiler.actions.newQueuePreset')}
                  disabled={state === 'running'}
                  onClick={createQueuePreset}
                >
                  <ListPlus className="size-3.5" />
                </Button>
                <Button
                  variant="outline"
                  size="icon-sm"
                  title={t('compiler.actions.saveQueuePreset')}
                  disabled={state === 'running' || buildQueue.length === 0}
                  onClick={saveQueuePreset}
                >
                  <Save className="size-3.5" />
                </Button>
                <Button
                  variant="outline"
                  size="icon-sm"
                  title={t('compiler.actions.removeQueuePreset')}
                  disabled={state === 'running' || !selectedQueuePresetId}
                  onClick={deleteSelectedQueuePreset}
                >
                  <Trash2 className="size-3.5" />
                </Button>
              </div>
            </section>
            <section className="rounded-md border bg-background">
              <div className="flex min-h-10 items-center justify-between gap-2 border-b px-3">
                <div className="flex min-w-0 items-center gap-2">
                  <ListPlus className="size-4 text-muted-foreground" />
                  <span className="truncate text-sm font-medium">
                    {t('compiler.panels.queue')}
                  </span>
                  <Badge variant="outline" className="font-normal">
                    {buildQueue.length}
                  </Badge>
                </div>
                <div className="flex min-w-0 flex-wrap items-center justify-end gap-1">
                  <div className="grid grid-cols-3 gap-1">
                    <Button
                      variant="outline"
                      size="icon-sm"
                      title={t('compiler.actions.restartQueue')}
                      disabled={state === 'running' || buildQueue.length === 0}
                      onClick={restartQueue}
                    >
                      <RotateCcw className="size-3.5" />
                    </Button>
                    <Button
                      variant="outline"
                      size="icon-sm"
                      title={t('compiler.actions.runQueue')}
                      disabled={
                        state === 'running' ||
                        !buildQueue.some(item => item.status === 'queued')
                      }
                      onClick={() => void startQueuedBuilds()}
                    >
                      <Play className="size-3.5" />
                    </Button>
                    <Button
                      variant="outline"
                      size="icon-sm"
                      title={t('compiler.actions.clearQueue')}
                      disabled={state === 'running' || buildQueue.length === 0}
                      onClick={clearQueue}
                    >
                      <Trash2 className="size-3.5" />
                    </Button>
                  </div>
                </div>
              </div>
              <div className="space-y-2 p-3">
                {editingQueueItemId ? (
                  <div className="rounded-sm border border-primary/30 bg-primary/5 px-2 py-1.5 text-xs text-primary">
                    {t('compiler.queue.editing')}
                  </div>
                ) : null}
                {buildQueue.length === 0 ? (
                  <div className="text-xs text-muted-foreground">
                    {t('compiler.empty.queue')}
                  </div>
                ) : (
                  buildQueue.map((item, index) => (
                    <div
                      key={item.id}
                      className={cn(
                        'grid min-h-20 grid-cols-[minmax(0,1fr)_auto] gap-2 rounded-md border bg-muted/20 p-2',
                        editingQueueItemId === item.id &&
                          'border-primary bg-primary/5',
                        item.status === 'running' && 'border-primary/50',
                        item.status === 'success' &&
                          'border-emerald-500/40 bg-emerald-500/10 shadow-[0_0_0_1px_rgba(16,185,129,0.08)]',
                        item.status === 'failed' &&
                          'border-destructive/30 bg-destructive/5'
                      )}
                    >
                      <div className="min-w-0">
                        <div className="truncate text-sm font-medium">
                          {queueItemLabel(item)}
                        </div>
                        <div className="mt-1 flex flex-wrap gap-1">
                          {item.request.versions.map(version => (
                            <Badge
                              key={`${item.id}-${version}`}
                              variant="outline"
                              className="text-[11px]"
                            >
                              {version}
                            </Badge>
                          ))}
                        </div>
                        <div className="mt-1 truncate text-xs text-muted-foreground">
                          {item.request.configuration} ·{' '}
                          {item.request.package_mode}
                          <QueueDurationText
                            item={item}
                            currentTime={durationTick}
                          />
                          {item.message ? ` · ${item.message}` : ''}
                        </div>
                      </div>
                      <div className="flex flex-col items-end justify-between gap-2">
                        <Badge
                          variant={
                            item.status === 'failed'
                              ? 'destructive'
                              : 'secondary'
                          }
                          className={cn(
                            item.status === 'success' &&
                              'border-transparent bg-emerald-600 text-white shadow-sm shadow-emerald-950/10',
                            item.status === 'running' &&
                              'border-transparent bg-primary text-primary-foreground'
                          )}
                        >
                          {t(`compiler.queue.status.${item.status}`)}
                        </Badge>
                        <div className="grid grid-cols-2 gap-1">
                          <Button
                            variant="ghost"
                            size="icon"
                            className="size-7"
                            title={t('compiler.actions.moveQueueItemUp')}
                            disabled={state === 'running' || index === 0}
                            onClick={() => moveBuildQueueItem(item.id, 'up')}
                          >
                            <ArrowUp className="size-3.5" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="size-7"
                            title={t('compiler.actions.moveQueueItemDown')}
                            disabled={
                              state === 'running' ||
                              index === buildQueue.length - 1
                            }
                            onClick={() => moveBuildQueueItem(item.id, 'down')}
                          >
                            <ArrowDown className="size-3.5" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="size-7"
                            title={t('compiler.actions.editQueueItem')}
                            disabled={state === 'running'}
                            onClick={() => editQueueItem(item.id)}
                          >
                            <FilePenLine className="size-3.5" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="size-7"
                            title={t('compiler.actions.removeQueueItem')}
                            disabled={state === 'running'}
                            onClick={() => removeQueueItem(item.id)}
                          >
                            <Trash2 className="size-3.5" />
                          </Button>
                        </div>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </section>
          </div>
        </ScrollArea>
      </aside>

      <main className="flex min-h-0 min-w-0 flex-col overflow-hidden">
        <div className="grid grid-cols-[repeat(auto-fit,minmax(240px,1fr))] gap-3 border-b p-4">
          <StatusPanel title={t('compiler.panels.environment')}>
            <ToolLine label="CMake" tool={environment?.cmake} />
            {compilerPlatform === 'Macos' ? (
              <>
                <ToolLine label="Xcode" tool={environment?.xcode} />
                <ToolLine label="Clang" tool={environment?.clang} />
                <ToolLine label="Python" tool={environment?.python} />
              </>
            ) : (
              <>
                <ToolLine label="VS 2022" tool={environment?.visual_studio} />
                <ToolLine label="Windows SDK" tool={environment?.windows_sdk} />
              </>
            )}
            <ToolLine
              label="Preset"
              tool={
                environment?.cmake_preset
                  ? {
                      found: true,
                      path: null,
                      version: null,
                      message: environment.cmake_preset,
                    }
                  : undefined
              }
            />
          </StatusPanel>
          <StatusPanel title={t('compiler.panels.sdkMatrix')}>
            {sdkResolutions.length === 0 ? (
              <div className="text-xs text-muted-foreground">
                {t('compiler.empty.sdkResolutions')}
              </div>
            ) : (
              sdkResolutions.map(item => (
                <div key={item.version} className="flex items-center gap-2">
                  <Badge variant="outline">{item.version}</Badge>
                  <span
                    className={cn(
                      'truncate text-xs',
                      isResolvedSdk(item) && 'text-green-600'
                    )}
                  >
                    {item.status}
                  </span>
                </div>
              ))
            )}
          </StatusPanel>
          <StatusPanel title={t('compiler.panels.progress')}>
            <div className="flex items-center gap-2">
              {state === 'success' ? (
                <CheckCircle2 className="size-4 text-green-600" />
              ) : state === 'failed' ? (
                <XCircle className="size-4 text-destructive" />
              ) : (
                <Hammer className="size-4 text-muted-foreground" />
              )}
              <span className="text-sm">{t(`compiler.state.${state}`)}</span>
            </div>
            <div className="text-xs text-muted-foreground">
              {progress
                ? `${progress.current}/${progress.total} ${progress.label}`
                : t('compiler.empty.noActiveBuild')}
            </div>
            <BuildDurationText
              startedAt={buildStartedAt}
              finishedAt={buildFinishedAt}
              currentTime={durationTick}
            />
            {state === 'failed' && latestError ? (
              <ScrollArea className="h-20 overflow-hidden rounded-sm">
                <div className="whitespace-pre-wrap break-words pr-2 text-xs text-destructive">
                  {latestError}
                </div>
              </ScrollArea>
            ) : null}
          </StatusPanel>
        </div>

        <div className="min-h-0 flex-1 overflow-hidden">
          <section className="flex h-full min-h-0 min-w-0 flex-col overflow-hidden">
            <div className="flex min-h-12 flex-wrap items-center justify-between gap-2 border-b px-4 py-2 text-sm font-medium">
              <div className="flex min-w-0 flex-wrap items-center gap-2">
                <ToggleGroup
                  type="single"
                  value={outputTab}
                  variant="outline"
                  size="sm"
                  onValueChange={value => {
                    if (value) setOutputTab(value as OutputTab)
                  }}
                >
                  <ToggleGroupItem value="logs" className="h-7 px-3">
                    {t('compiler.panels.buildLog')}
                  </ToggleGroupItem>
                  <ToggleGroupItem value="artifacts" className="h-7 px-3">
                    {t('compiler.panels.artifacts')}
                  </ToggleGroupItem>
                </ToggleGroup>
                {outputTab === 'logs' ? (
                  <Badge variant="outline" className="font-normal">
                    {filteredLogs.length}/{logs.length}
                  </Badge>
                ) : (
                  <Badge variant="outline" className="font-normal">
                    {artifacts.length}
                  </Badge>
                )}
                {logActionMessage ? (
                  <span className="truncate text-xs font-normal text-muted-foreground">
                    {logActionMessage}
                  </span>
                ) : null}
              </div>
              {outputTab === 'logs' ? (
                <div className="flex min-w-0 flex-wrap items-center justify-end gap-2">
                  <ToggleGroup
                    type="single"
                    value={logFilter}
                    variant="outline"
                    size="sm"
                    onValueChange={value => {
                      if (value) setLogFilter(value as LogFilter)
                    }}
                  >
                    <ToggleGroupItem value="all" className="h-7 px-2">
                      {t('compiler.logs.filter.all')}
                    </ToggleGroupItem>
                    <ToggleGroupItem value="warn" className="h-7 px-2">
                      {t('compiler.logs.filter.warn')}
                    </ToggleGroupItem>
                    <ToggleGroupItem value="error" className="h-7 px-2">
                      {t('compiler.logs.filter.error')}
                    </ToggleGroupItem>
                  </ToggleGroup>
                  <div className="flex items-center gap-1">
                    <ListFilter className="size-3.5 text-muted-foreground" />
                    <Select
                      value={logCategoryFilter}
                      onValueChange={value =>
                        setLogCategoryFilter(value as LogCategoryFilter)
                      }
                    >
                      <SelectTrigger className="h-7 w-[118px] text-xs">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {LOG_CATEGORY_FILTERS.map(category => (
                          <SelectItem key={category} value={category}>
                            {t(`compiler.logs.category.${category}`)}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <label className="flex items-center gap-1.5 text-xs font-normal text-muted-foreground">
                    <Switch
                      checked={autoScrollLogs}
                      onCheckedChange={setAutoScrollLogs}
                    />
                    <span>{t('compiler.logs.autoScroll')}</span>
                  </label>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="size-7"
                    disabled={logs.length === 0}
                    title={t('compiler.actions.clearLog')}
                    onClick={clearLogs}
                  >
                    <Trash2 className="size-3.5" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="size-7"
                    disabled={!filteredBuildLogText}
                    title={t('compiler.actions.copyLog')}
                    onClick={() => void copyBuildLog()}
                  >
                    <Copy className="size-3.5" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="size-7"
                    disabled={!filteredBuildLogText}
                    title={t('compiler.actions.saveLog')}
                    onClick={() => void saveBuildLog()}
                  >
                    <Save className="size-3.5" />
                  </Button>
                </div>
              ) : null}
            </div>
            {outputTab === 'logs' ? (
              <ScrollArea className="min-h-0 flex-1 overflow-hidden">
                <div className="space-y-1 p-4 font-mono text-xs select-text">
                  {logs.length === 0 ? (
                    <div className="text-muted-foreground">
                      {t('compiler.empty.noLogs')}
                    </div>
                  ) : filteredLogs.length === 0 ? (
                    <div className="text-muted-foreground">
                      {t('compiler.empty.noFilteredLogs')}
                    </div>
                  ) : (
                    filteredLogs.map((item, index) => (
                      <LogRow key={`${item.job_id}-${index}`} item={item} />
                    ))
                  )}
                  <div ref={logEndRef} />
                </div>
              </ScrollArea>
            ) : (
              <ScrollArea className="min-h-0 flex-1 overflow-hidden">
                <div className="grid gap-3 p-4 md:grid-cols-2 xl:grid-cols-3">
                  {artifacts.length === 0 ? (
                    <div className="text-xs text-muted-foreground">
                      {t('compiler.empty.artifacts')}
                    </div>
                  ) : (
                    artifacts.map(item => (
                      <div
                        key={item.path}
                        className="rounded-md border bg-background p-3"
                      >
                        <div className="text-sm font-medium">{item.kind}</div>
                        <div className="mt-1 truncate text-xs text-muted-foreground">
                          {item.path}
                        </div>
                        <Button
                          className="mt-2"
                          variant="outline"
                          size="sm"
                          onClick={() =>
                            void commands.openArtifactFolder(item.path)
                          }
                        >
                          <FolderOpen className="size-3.5" />
                          {t('compiler.actions.open')}
                          <HelpHint text={t('compiler.help.openArtifact')} />
                        </Button>
                      </div>
                    ))
                  )}
                </div>
              </ScrollArea>
            )}
          </section>
        </div>
      </main>
    </div>
  )
}

function isResolvedSdk(item: SdkResolution) {
  return Boolean(item.sdk_root || item.archive_path)
}

function queueItemLabel(item: {
  request: { package_name: string; module_name: string; plugin_root: string }
}) {
  return (
    item.request.package_name ||
    item.request.module_name ||
    detectPathName(item.request.plugin_root) ||
    'Plugin'
  )
}

function queueItemDuration(
  item: {
    startedAt: number | null
    finishedAt: number | null
  },
  currentTime: number
) {
  if (!item.startedAt) return ''
  const endTime = item.finishedAt ?? currentTime
  return formatDuration(endTime - item.startedAt)
}

function QueueDurationText({
  item,
  currentTime,
}: {
  item: { startedAt: number | null; finishedAt: number | null }
  currentTime: number
}) {
  const { t } = useTranslation()
  const duration = queueItemDuration(item, currentTime)
  if (!duration) return null

  return <> - {t('compiler.queue.duration', { duration })}</>
}

function BuildDurationText({
  startedAt,
  finishedAt,
  currentTime,
}: {
  startedAt: number | null
  finishedAt: number | null
  currentTime: number
}) {
  const { t } = useTranslation()
  if (!startedAt) return null

  const duration = formatDuration((finishedAt ?? currentTime) - startedAt)
  return (
    <div className="text-xs text-muted-foreground">
      {t('compiler.build.duration', { duration })}
    </div>
  )
}

function formatDuration(milliseconds: number) {
  const totalSeconds = Math.max(0, Math.round(milliseconds / 1000))
  const hours = Math.floor(totalSeconds / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = totalSeconds % 60

  if (hours > 0) {
    return `${hours}h ${minutes}m ${seconds}s`
  }
  if (minutes > 0) {
    return `${minutes}m ${seconds}s`
  }
  return `${seconds}s`
}

function detectPathName(path: string) {
  const normalized = path.trim().replace(/[/\\]+$/, '')
  if (!normalized) return ''
  return normalized.split(/[/\\]/).pop() ?? ''
}

function formatBuildLog(logs: BuildLogEvent[]) {
  return logs
    .map(
      item =>
        `${formatLogTimestamp(item.timestamp)} [${item.level}] [${item.category}] ${item.message}`
    )
    .join('\n')
}

function isBuildableSdkVersion(item: SdkVersionOption) {
  if (item.status.startsWith('invalid ')) return false
  return Boolean(item.sdk_root || item.sdk_zip)
}

function syncBuildVersionsFromSdkOptions(
  options: SdkVersionOption[],
  currentStart: string,
  setSdkStartVersion: (version: string, availableVersions: string[]) => void,
  setBuildVersions: (versions: string[]) => void
) {
  const availableVersions = options.map(version => version.version)
  const buildableVersions = options
    .filter(isBuildableSdkVersion)
    .map(version => version.version)
  const fallbackVersion = buildableVersions[0] ?? availableVersions[0]
  if (!fallbackVersion) return

  const startVersion = buildableVersions.includes(currentStart)
    ? currentStart
    : fallbackVersion
  const startIndex = buildableVersions.indexOf(startVersion)
  const nextVersions =
    startIndex >= 0
      ? buildableVersions.slice(startIndex)
      : [startVersion].filter(version => buildableVersions.includes(version))

  if (
    startVersion !== currentStart ||
    !availableVersions.includes(currentStart)
  ) {
    setSdkStartVersion(startVersion, buildableVersions)
  }
  setBuildVersions(nextVersions.length > 0 ? nextVersions : [fallbackVersion])
}

function matchesLogLevelFilter(item: BuildLogEvent, filter: LogFilter) {
  if (filter === 'all') return true
  if (filter === 'warn') return item.level === 'warn' || item.level === 'error'
  return item.level === 'error'
}

function matchesLogCategoryFilter(
  item: BuildLogEvent,
  filter: LogCategoryFilter
) {
  return filter === 'all' || item.category === filter
}

function systemLog(level: 'info' | 'warn' | 'error', message: string) {
  return {
    job_id: 'system',
    level,
    category: 'system',
    timestamp: String(Date.now()),
    message,
  }
}

function formatLogTimestamp(timestamp: string) {
  const date = new Date(Number(timestamp))
  if (Number.isNaN(date.getTime())) return timestamp

  return date.toLocaleTimeString(undefined, {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    fractionalSecondDigits: 3,
    hour12: false,
  })
}

function LogRow({ item }: { item: BuildLogEvent }) {
  return (
    <div
      className={cn(
        'grid grid-cols-[88px_54px_82px_minmax(0,1fr)] gap-2 rounded border px-2 py-1.5 leading-relaxed',
        item.level === 'error' &&
          'border-destructive/20 bg-destructive/10 text-destructive',
        item.level === 'warn' && 'border-amber-500/20 bg-amber-500/10',
        item.level !== 'error' &&
          item.level !== 'warn' &&
          'border-transparent bg-muted/20'
      )}
    >
      <span className="text-muted-foreground">
        {formatLogTimestamp(item.timestamp)}
      </span>
      <span
        className={cn(
          'font-semibold uppercase',
          item.level === 'error' && 'text-destructive',
          item.level === 'warn' && 'text-amber-600',
          item.level === 'info' && 'text-emerald-600'
        )}
      >
        {item.level}
      </span>
      <span className="truncate text-muted-foreground">{item.category}</span>
      <span className="whitespace-pre-wrap break-words text-foreground">
        {item.message}
      </span>
    </div>
  )
}

function Field({
  label,
  help,
  children,
}: {
  label: string
  help: string
  children: React.ReactNode
}) {
  return (
    <div className="space-y-1.5">
      <Label className="flex items-center gap-1.5 text-xs text-muted-foreground">
        <span>{label}</span>
        <HelpHint text={help} />
      </Label>
      {children}
    </div>
  )
}

function Toggle({
  checked,
  label,
  help,
  onCheckedChange,
}: {
  checked: boolean
  label: string
  help: string
  onCheckedChange: (checked: boolean) => void
}) {
  return (
    <label className="flex h-9 items-center gap-2 rounded-md border px-2 text-xs">
      <Checkbox
        checked={checked}
        onCheckedChange={value => onCheckedChange(Boolean(value))}
      />
      <span className="truncate">{label}</span>
      <HelpHint text={help} />
    </label>
  )
}

function StatusPanel({
  title,
  children,
}: {
  title: string
  children: React.ReactNode
}) {
  return (
    <div className="min-w-0 rounded-md border bg-background p-3">
      <div className="mb-2 text-xs font-semibold uppercase text-muted-foreground">
        {title}
      </div>
      <div className="space-y-1">{children}</div>
    </div>
  )
}

function ToolLine({
  label,
  tool,
}: {
  label: string
  tool?: {
    found: boolean
    message?: string | null
    version?: string | null
    path?: string | null
  }
}) {
  return (
    <div className="flex min-w-0 items-center justify-between gap-2 text-xs">
      <span className="text-muted-foreground">{label}</span>
      <span
        className={cn(
          'truncate',
          tool?.found ? 'text-green-600' : 'text-destructive'
        )}
        title={tool?.path ?? undefined}
      >
        {tool?.found ? tool.version || tool.message || 'found' : 'missing'}
      </span>
    </div>
  )
}
