import { useEffect, useMemo, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
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
import {
  commands,
  type BuildArtifact,
  type BuildConfiguration,
  type BuildFinishedEvent,
  type BuildLogEvent,
  type BuildProgressEvent,
  type BuildRequest,
  type EnvironmentReport,
  type PackageMode,
  type SdkResolution,
} from '@/lib/tauri-bindings'

type BuildState = 'idle' | 'running' | 'success' | 'failed'

const defaultRequest: BuildRequest = {
  plugin_root: 'E:\\Boghma\\boghma hub\\Done Paid\\Boghma-WaterMark',
  module_name: 'postwatermark',
  package_name: 'Boghma WaterMark',
  versions: ['2025', '2026'],
  configuration: 'Release',
  sdk_source: 'ConfiguredThenInstalledThenOfficial',
  package_mode: 'Both',
  zip_enabled: true,
  clean_output: true,
  refresh_sdk_cache: false,
  output_dir: null,
}

function parseVersionInput(value: string) {
  const rangeMatch = value.match(/^(\d{4})\s*-\s*(\d{4})$/)
  if (rangeMatch) {
    const start = Number(rangeMatch[1])
    const end = Number(rangeMatch[2])
    if (start <= end) {
      return Array.from({ length: end - start + 1 }, (_, index) =>
        String(start + index)
      )
    }
  }

  return value
    .split(',')
    .map(item => item.trim())
    .filter(Boolean)
}

export function CompilerWorkbench() {
  const [request, setRequest] = useState<BuildRequest>(defaultRequest)
  const [environment, setEnvironment] = useState<EnvironmentReport | null>(null)
  const [sdkResolutions, setSdkResolutions] = useState<SdkResolution[]>([])
  const [logs, setLogs] = useState<BuildLogEvent[]>([])
  const [artifacts, setArtifacts] = useState<BuildArtifact[]>([])
  const [progress, setProgress] = useState<BuildProgressEvent | null>(null)
  const [jobId, setJobId] = useState<string | null>(null)
  const [state, setState] = useState<BuildState>('idle')

  const versionText = useMemo(() => request.versions.join(', '), [request])

  useEffect(() => {
    void refreshEnvironment()

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
  }, [])

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

  function updateRequest(patch: Partial<BuildRequest>) {
    setRequest(current => ({ ...current, ...patch }))
  }

  return (
    <div className="grid h-full grid-cols-[minmax(340px,420px)_1fr] overflow-hidden">
      <aside className="flex min-h-0 flex-col border-r bg-muted/20">
        <div className="border-b px-4 py-3">
          <div className="flex items-center gap-2 text-sm font-semibold">
            <Hammer className="size-4" />
            C4D Plugin Compiler
          </div>
          <div className="mt-1 text-xs text-muted-foreground">
            Rust backend · Tauri 2 · CMake SDK workflow
          </div>
        </div>
        <ScrollArea className="flex-1">
          <div className="space-y-4 p-4">
            <Field label="Plugin Root">
              <Input
                value={request.plugin_root}
                onChange={event =>
                  updateRequest({ plugin_root: event.target.value })
                }
              />
            </Field>
            <div className="grid grid-cols-2 gap-3">
              <Field label="Module">
                <Input
                  value={request.module_name}
                  onChange={event =>
                    updateRequest({ module_name: event.target.value })
                  }
                />
              </Field>
              <Field label="Package">
                <Input
                  value={request.package_name}
                  onChange={event =>
                    updateRequest({ package_name: event.target.value })
                  }
                />
              </Field>
            </div>
            <Field label="Versions">
              <Input
                value={versionText}
                onChange={event =>
                  updateRequest({
                    versions: parseVersionInput(event.target.value),
                  })
                }
              />
            </Field>
            <div className="grid grid-cols-2 gap-3">
              <Field label="Configuration">
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
                    <SelectItem value="Debug">Debug</SelectItem>
                    <SelectItem value="Release">Release</SelectItem>
                    <SelectItem value="Both">Both</SelectItem>
                  </SelectContent>
                </Select>
              </Field>
              <Field label="Package Mode">
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
                    <SelectItem value="Merged">Merged</SelectItem>
                    <SelectItem value="PerVersion">Per Version</SelectItem>
                    <SelectItem value="Both">Both</SelectItem>
                  </SelectContent>
                </Select>
              </Field>
            </div>
            <Field label="Output Dir">
              <Input
                value={request.output_dir ?? ''}
                placeholder="Plugin root\\dist"
                onChange={event =>
                  updateRequest({ output_dir: event.target.value || null })
                }
              />
            </Field>
            <div className="grid grid-cols-3 gap-2">
              <Toggle
                checked={request.zip_enabled}
                label="Zip"
                onCheckedChange={value => updateRequest({ zip_enabled: value })}
              />
              <Toggle
                checked={request.clean_output}
                label="Clean"
                onCheckedChange={value =>
                  updateRequest({ clean_output: value })
                }
              />
              <Toggle
                checked={request.refresh_sdk_cache}
                label="Refresh SDK"
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
                Build
              </Button>
              <Button
                variant="outline"
                size="icon"
                title="Resolve SDKs"
                onClick={() => void resolveSdks()}
              >
                <Box className="size-4" />
              </Button>
              <Button
                variant="outline"
                size="icon"
                title="Refresh environment"
                onClick={() => void refreshEnvironment()}
              >
                <RefreshCw className="size-4" />
              </Button>
              <Button
                variant="outline"
                size="icon"
                title="Cancel"
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
          <StatusPanel title="Environment">
            <ToolLine label="CMake" tool={environment?.cmake} />
            <ToolLine label="VS 2022" tool={environment?.visual_studio} />
            <ToolLine label="Windows SDK" tool={environment?.windows_sdk} />
          </StatusPanel>
          <StatusPanel title="SDK Matrix">
            {sdkResolutions.length === 0 ? (
              <div className="text-xs text-muted-foreground">
                Resolve SDKs to preview sources.
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
          <StatusPanel title="Progress">
            <div className="flex items-center gap-2">
              {state === 'success' ? (
                <CheckCircle2 className="size-4 text-green-600" />
              ) : state === 'failed' ? (
                <XCircle className="size-4 text-destructive" />
              ) : (
                <Hammer className="size-4 text-muted-foreground" />
              )}
              <span className="text-sm capitalize">{state}</span>
            </div>
            <div className="text-xs text-muted-foreground">
              {progress
                ? `${progress.current}/${progress.total} ${progress.label}`
                : 'No active build'}
            </div>
          </StatusPanel>
        </div>

        <div className="grid min-h-0 flex-1 grid-cols-[1fr_320px] overflow-hidden">
          <section className="flex min-w-0 flex-col">
            <div className="flex h-10 items-center gap-2 border-b px-4 text-sm font-medium">
              Build Log
            </div>
            <ScrollArea className="flex-1">
              <div className="space-y-1 p-4 font-mono text-xs">
                {logs.length === 0 ? (
                  <div className="text-muted-foreground">No logs yet.</div>
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
              Artifacts
            </div>
            <ScrollArea className="flex-1">
              <div className="space-y-2 p-3">
                {artifacts.length === 0 ? (
                  <div className="text-xs text-muted-foreground">
                    Packages appear here after a build.
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
                        Open
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
  children,
}: {
  label: string
  children: React.ReactNode
}) {
  return (
    <div className="space-y-1.5">
      <Label className="text-xs text-muted-foreground">{label}</Label>
      {children}
    </div>
  )
}

function Toggle({
  checked,
  label,
  onCheckedChange,
}: {
  checked: boolean
  label: string
  onCheckedChange: (checked: boolean) => void
}) {
  return (
    <label className="flex h-9 items-center gap-2 rounded-md border px-2 text-xs">
      <Checkbox
        checked={checked}
        onCheckedChange={value => onCheckedChange(Boolean(value))}
      />
      <span className="truncate">{label}</span>
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
  tool?: { found: boolean; version?: string | null; path?: string | null }
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
        {tool?.found ? tool.version || 'found' : 'missing'}
      </span>
    </div>
  )
}
