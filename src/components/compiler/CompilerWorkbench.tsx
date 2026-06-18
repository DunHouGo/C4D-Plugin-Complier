import { useEffect, useMemo, useRef, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { useTranslation } from 'react-i18next'
import {
  Archive,
  Box,
  CheckCircle2,
  FolderOpen,
  Hammer,
  Play,
  RefreshCw,
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

export function CompilerWorkbench() {
  const { t } = useTranslation()
  const request = useCompilerStore(state => state.request)
  const updateRequest = useCompilerStore(state => state.updateRequest)
  const updatePluginRoot = useCompilerStore(state => state.updatePluginRoot)
  const sdkStartVersion = useCompilerStore(state => state.sdkStartVersion)
  const setSdkStartVersion = useCompilerStore(state => state.setSdkStartVersion)
  const [environment, setEnvironment] = useState<EnvironmentReport | null>(null)
  const [sdkResolutions, setSdkResolutions] = useState<SdkResolution[]>([])
  const [sdkVersions, setSdkVersions] = useState<SdkVersionOption[]>([])
  const [logs, setLogs] = useState<BuildLogEvent[]>([])
  const [artifacts, setArtifacts] = useState<BuildArtifact[]>([])
  const [progress, setProgress] = useState<BuildProgressEvent | null>(null)
  const [jobId, setJobId] = useState<string | null>(null)
  const [state, setState] = useState<BuildState>('idle')
  const sdkStartVersionRef = useRef(sdkStartVersion)
  const compilerPlatform = environment?.compiler_platform ?? 'Unsupported'

  const versionNames = useMemo(
    () => sdkVersions.map(version => version.version),
    [sdkVersions]
  )

  useEffect(() => {
    sdkStartVersionRef.current = sdkStartVersion
  }, [sdkStartVersion])

  useEffect(() => {
    async function loadInitialEnvironment() {
      const result = await commands.detectEnvironment()
      if (result.status === 'ok') {
        setEnvironment(result.data)
      } else {
        setLogs(current => [
          ...current,
          { job_id: 'system', level: 'error', message: result.error },
        ])
      }
    }

    async function loadInitialSdkVersions() {
      const result = await commands.listSdkVersions()
      if (result.status === 'ok') {
        setSdkVersions(result.data)
        const nextVersion = result.data[0]?.version
        if (
          nextVersion &&
          !result.data.some(
            version => version.version === sdkStartVersionRef.current
          )
        ) {
          setSdkStartVersion(
            nextVersion,
            result.data.map(version => version.version)
          )
        }
      } else {
        setLogs(current => [
          ...current,
          { job_id: 'system', level: 'error', message: result.error },
        ])
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
        setArtifacts(current => [...current, event.payload])
      }),
      listen<BuildFinishedEvent>('build://finished', event => {
        setState(event.payload.success ? 'success' : 'failed')
        setLogs(current => [
          ...current,
          {
            job_id: event.payload.job_id,
            level: event.payload.success ? 'info' : 'error',
            message: event.payload.message,
          },
        ])
      }),
    ])

    return () => {
      void unlisten.then(items => items.forEach(item => item()))
    }
  }, [setSdkStartVersion])

  async function refreshSdkVersions() {
    const result = await commands.listSdkVersions()
    if (result.status === 'ok') {
      setSdkVersions(result.data)
      const nextVersion = result.data[0]?.version
      if (
        nextVersion &&
        !result.data.some(
          version => version.version === sdkStartVersionRef.current
        )
      ) {
        setSdkStartVersion(
          nextVersion,
          result.data.map(version => version.version)
        )
      }
    } else {
      setLogs(current => [
        ...current,
        { job_id: 'system', level: 'error', message: result.error },
      ])
    }
  }

  async function refreshEnvironment() {
    const result = await commands.detectEnvironment()
    if (result.status === 'ok') {
      setEnvironment(result.data)
    } else {
      setLogs(current => [
        ...current,
        { job_id: 'system', level: 'error', message: result.error },
      ])
    }
  }

  async function resolveSdks() {
    const result = await commands.resolveSdkVersions(request)
    if (result.status === 'ok') {
      setSdkResolutions(result.data)
    } else {
      setLogs(current => [
        ...current,
        { job_id: 'system', level: 'error', message: result.error },
      ])
    }
  }

  async function startBuild() {
    setLogs([])
    setArtifacts([])
    setProgress(null)
    setState('running')
    await resolveSdks()
    const result = await commands.startBuild(request)
    if (result.status === 'ok') {
      setJobId(result.data.id)
    } else {
      setState('failed')
      setLogs([{ job_id: 'system', level: 'error', message: result.error }])
    }
  }

  async function cancelBuild() {
    if (!jobId) return
    await commands.cancelBuild(jobId)
    setState('idle')
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
                onValueChange={value => setSdkStartVersion(value, versionNames)}
              >
                <SelectTrigger className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {sdkVersions.map(version => (
                    <SelectItem key={version.version} value={version.version}>
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

      <main className="flex min-w-0 flex-col overflow-hidden">
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
                  <span className="truncate text-xs">{item.status}</span>
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
          </StatusPanel>
        </div>

        <div className="grid min-h-0 flex-1 grid-cols-[1fr_320px] overflow-hidden">
          <section className="flex min-w-0 flex-col">
            <div className="flex h-10 items-center gap-2 border-b px-4 text-sm font-medium">
              {t('compiler.panels.buildLog')}
            </div>
            <ScrollArea className="flex-1">
              <div className="space-y-1 p-4 font-mono text-xs">
                {logs.length === 0 ? (
                  <div className="text-muted-foreground">
                    {t('compiler.empty.noLogs')}
                  </div>
                ) : (
                  logs.map((item, index) => (
                    <div
                      key={`${item.job_id}-${index}`}
                      className={cn(
                        'rounded px-2 py-1',
                        item.level === 'error' && 'bg-destructive/10',
                        item.level === 'warn' && 'bg-amber-500/10'
                      )}
                    >
                      <span className="mr-2 text-muted-foreground">
                        {item.level}
                      </span>
                      {item.message}
                    </div>
                  ))
                )}
              </div>
            </ScrollArea>
          </section>

          <aside className="flex min-w-0 flex-col border-l">
            <div className="flex h-10 items-center gap-2 border-b px-4 text-sm font-medium">
              <Archive className="size-4" />
              {t('compiler.panels.artifacts')}
            </div>
            <ScrollArea className="flex-1">
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
