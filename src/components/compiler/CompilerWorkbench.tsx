import { useEffect, useMemo, useRef, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { save as saveDialog } from '@tauri-apps/plugin-dialog'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { useTranslation } from 'react-i18next'
import {
  Archive,
  Box,
  CheckCircle2,
  Copy,
  FolderOpen,
  Hammer,
  ListFilter,
  Play,
  RefreshCw,
  Save,
  Square,
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
  const updatePluginRoot = useCompilerStore(state => state.updatePluginRoot)
  const artifacts = useCompilerStore(state => state.artifacts)
  const setArtifacts = useCompilerStore(state => state.setArtifacts)
  const addArtifact = useCompilerStore(state => state.addArtifact)
  const sdkStartVersion = useCompilerStore(state => state.sdkStartVersion)
  const setSdkStartVersion = useCompilerStore(state => state.setSdkStartVersion)
  const setBuildVersions = useCompilerStore(state => state.setBuildVersions)
  const [environment, setEnvironment] = useState<EnvironmentReport | null>(null)
  const [sdkResolutions, setSdkResolutions] = useState<SdkResolution[]>([])
  const [sdkVersions, setSdkVersions] = useState<SdkVersionOption[]>([])
  const [logs, setLogs] = useState<BuildLogEvent[]>([])
  const [logFilter, setLogFilter] = useState<LogFilter>('all')
  const [logCategoryFilter, setLogCategoryFilter] =
    useState<LogCategoryFilter>('all')
  const [autoScrollLogs, setAutoScrollLogs] = useState(true)
  const [logActionMessage, setLogActionMessage] = useState<string | null>(null)
  const [progress, setProgress] = useState<BuildProgressEvent | null>(null)
  const [jobId, setJobId] = useState<string | null>(null)
  const [state, setState] = useState<BuildState>('idle')
  const sdkStartVersionRef = useRef(sdkStartVersion)
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
        setState(event.payload.success ? 'success' : 'failed')
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
      }),
    ])

    return () => {
      void unlisten.then(items => items.forEach(item => item()))
    }
  }, [addArtifact, setBuildVersions, setSdkStartVersion])

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

  async function resolveSdks() {
    const result = await commands.resolveSdkVersions(request)
    if (result.status === 'ok') {
      setSdkResolutions(result.data)
      return true
    }

    setLogs(current => [...current, systemLog('error', result.error)])
    return false
  }

  async function startBuild() {
    setLogs([])
    setArtifacts([])
    setProgress(null)
    setState('running')
    const resolved = await resolveSdks()
    if (!resolved) {
      setState('failed')
      return
    }

    const result = await commands.startBuild(request)
    if (result.status === 'ok') {
      setJobId(result.data.id)
    } else {
      setState('failed')
      setLogs([systemLog('error', result.error)])
    }
  }

  async function cancelBuild() {
    if (!jobId) return
    await commands.cancelBuild(jobId)
    setState('idle')
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

  return (
    <div className="grid h-full grid-cols-[minmax(320px,390px)_1fr] overflow-hidden">
      <aside className="flex min-h-0 flex-col border-r bg-muted/20">
        <div className="border-b px-4 py-3">
          <div className="flex items-center gap-2 text-sm font-semibold">
            <Hammer className="size-4" />
            {t('compiler.title')}
          </div>
          <div className="mt-1 text-xs text-muted-foreground">
            {t('compiler.subtitle')}
          </div>
        </div>
        <ScrollArea className="flex-1">
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
            <div className="grid grid-cols-2 gap-3">
              <Field
                label={t('compiler.fields.module')}
                help={t('compiler.help.module')}
              >
                <Input
                  value={request.module_name}
                  onChange={event =>
                    updateRequest({ module_name: event.target.value })
                  }
                />
              </Field>
              <Field
                label={t('compiler.fields.package')}
                help={t('compiler.help.package')}
              >
                <Input
                  value={request.package_name}
                  onChange={event =>
                    updateRequest({ package_name: event.target.value })
                  }
                />
              </Field>
            </div>
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
            <div className="grid grid-cols-3 gap-2">
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
            <div className="flex gap-2 pt-1">
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
          </div>
        </ScrollArea>
      </aside>

      <main className="flex min-h-0 min-w-0 flex-col overflow-hidden">
        <div className="grid grid-cols-3 gap-3 border-b p-4">
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
            {state === 'failed' && latestError ? (
              <ScrollArea className="h-20 overflow-hidden rounded-sm">
                <div className="whitespace-pre-wrap break-words pr-2 text-xs text-destructive">
                  {latestError}
                </div>
              </ScrollArea>
            ) : null}
          </StatusPanel>
        </div>

        <div className="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_320px] overflow-hidden">
          <section className="flex min-h-0 min-w-0 flex-col overflow-hidden">
            <div className="flex min-h-12 flex-wrap items-center justify-between gap-2 border-b px-4 py-2 text-sm font-medium">
              <div className="flex min-w-0 flex-wrap items-center gap-2">
                <span className="shrink-0">
                  {t('compiler.panels.buildLog')}
                </span>
                <Badge variant="outline" className="font-normal">
                  {filteredLogs.length}/{logs.length}
                </Badge>
                {logActionMessage ? (
                  <span className="truncate text-xs font-normal text-muted-foreground">
                    {logActionMessage}
                  </span>
                ) : null}
              </div>
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
            </div>
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
          </section>

          <aside className="flex min-h-0 min-w-0 flex-col overflow-hidden border-l">
            <div className="flex h-10 items-center gap-2 border-b px-4 text-sm font-medium">
              <Archive className="size-4" />
              {t('compiler.panels.artifacts')}
            </div>
            <ScrollArea className="min-h-0 flex-1 overflow-hidden">
              <div className="space-y-2 p-3">
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
          </aside>
        </div>
      </main>
    </div>
  )
}

function isResolvedSdk(item: SdkResolution) {
  return Boolean(item.sdk_root || item.archive_path)
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
