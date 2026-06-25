import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  CheckCircle2,
  Download,
  FolderCog,
  RefreshCw,
  Save,
  Sparkles,
  Stethoscope,
  TriangleAlert,
  XCircle,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Label } from '@/components/ui/label'
import { ScrollArea } from '@/components/ui/scroll-area'
import {
  commands,
  type InstalledC4dVersion,
  type SdkSetupReport,
  type SdkVersionOption,
  type SetupRequirement,
} from '@/lib/tauri-bindings'
import {
  DEFAULT_SDK_START_VERSION,
  useCompilerStore,
} from '@/store/compiler-store'
import { logger } from '@/lib/logger'
import { cn } from '@/lib/utils'
import { HelpHint } from './HelpHint'
import { PathPicker } from './PathPicker'

const FALLBACK_SDK_VERSIONS: SdkVersionOption[] = [
  {
    version: '2024.4',
    label: 'Cinema 4D 2024.4',
    configured: false,
    sdk_root: null,
    sdk_zip: null,
    download_url:
      'https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2024_4_0.zip',
    status: 'auto download',
  },
  {
    version: '2025',
    label: 'Cinema 4D 2025',
    configured: false,
    sdk_root: null,
    sdk_zip: null,
    download_url:
      'https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2025_0_1.zip',
    status: 'auto download',
  },
  {
    version: '2026',
    label: 'Cinema 4D 2026',
    configured: false,
    sdk_root: null,
    sdk_zip: null,
    download_url:
      'https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2026_0_0.zip',
    status: 'auto download',
  },
]

interface SdkConfigPanelProps {
  variant?: 'sidebar' | 'settings'
}

export function SdkConfigPanel({ variant = 'sidebar' }: SdkConfigPanelProps) {
  const { t } = useTranslation()
  const [versions, setVersions] = useState<SdkVersionOption[]>([])
  const [installedVersions, setInstalledVersions] = useState<
    InstalledC4dVersion[]
  >([])
  const [selectedVersion, setSelectedVersion] = useState(
    DEFAULT_SDK_START_VERSION
  )
  const [sdkRoot, setSdkRoot] = useState('')
  const [loading, setLoading] = useState(true)
  const [setupReport, setSetupReport] = useState<SdkSetupReport | null>(null)
  const [activeAction, setActiveAction] = useState<
    'inspect' | 'configure' | null
  >(null)
  const [message, setMessage] = useState<string | null>(null)
  const setSdkStartVersion = useCompilerStore(state => state.setSdkStartVersion)

  const selectedOption = versions.find(
    version => version.version === selectedVersion
  )
  const installedSdkVersionNames = new Set(
    installedVersions.map(item => item.sdk_version)
  )
  const selectedC4dInstalled = selectedOption
    ? installedSdkVersionNames.has(selectedOption.version)
    : true

  const loadSdkConfig = useCallback(async () => {
    setLoading(true)
    const [sourceResult, versionResult, environmentResult] = await Promise.all([
      commands.loadSdkSources(),
      commands.listSdkVersions(),
      commands.detectEnvironment(),
    ])

    if (sourceResult.status === 'ok') {
      setSdkRoot(sourceResult.data.sdk_root ?? '')
    } else {
      setMessage(sourceResult.error)
    }

    if (environmentResult.status === 'ok') {
      setInstalledVersions(environmentResult.data.installed_c4d_versions)
    }

    if (versionResult.status === 'ok') {
      const nextVersions =
        versionResult.data.length > 0
          ? versionResult.data
          : FALLBACK_SDK_VERSIONS
      setVersions(nextVersions)
      if (!nextVersions.some(item => item.version === selectedVersion)) {
        const nextVersion =
          nextVersions[0]?.version ?? DEFAULT_SDK_START_VERSION
        setSelectedVersion(nextVersion)
        setSdkStartVersion(
          nextVersion,
          nextVersions.map(item => item.version)
        )
      }
      if (sourceResult.status === 'ok') {
        setMessage(null)
      }
    } else {
      setVersions(FALLBACK_SDK_VERSIONS)
      setMessage(versionResult.error)
    }
    setLoading(false)
  }, [selectedVersion, setSdkStartVersion])

  useEffect(() => {
    void loadSdkConfig()
  }, [loadSdkConfig])

  async function saveRootConfig() {
    try {
      const result = await commands.saveSdkRootConfig({
        sdk_root: sdkRoot || null,
      })
      if (result.status === 'ok') {
        setSdkRoot(result.data.sdk_root ?? '')
        await loadSdkConfig()
        setMessage(t('sdk.savedRoot'))
      } else {
        setMessage(result.error)
      }
    } catch (error) {
      setMessage(errorMessage(error))
    }
  }

  async function autoDetect() {
    if (activeAction !== null) {
      return
    }

    setActiveAction('inspect')
    logger.info('Inspecting SDK setup')
    try {
      const result = await commands.inspectSdkSetup()
      if (result.status === 'ok') {
        setSdkRoot(result.data.sdk_root ?? '')
        setInstalledVersions(result.data.installed_versions)
        applySdkVersions(result.data.versions, result.data.prepared_versions)
        setSetupReport(result.data)
        setMessage(result.data.summary)
        logger.info('SDK setup inspection completed', {
          summary: result.data.summary,
          versions: result.data.versions.map(item => item.version),
        })
      } else {
        setMessage(result.error)
        logger.error('SDK setup inspection failed', { error: result.error })
      }
    } catch (error) {
      setMessage(errorMessage(error))
      void logger.recordCrash('sdk-setup-inspect', error)
    } finally {
      setActiveAction(null)
    }
  }

  async function configureRequiredSdks() {
    if (activeAction !== null) {
      return
    }

    setActiveAction('configure')
    logger.info('Configuring required SDKs', { sdkRoot })
    try {
      const result = await commands.configureRequiredSdks(
        { sdk_root: sdkRoot || null },
        false
      )
      if (result.status === 'ok') {
        setSdkRoot(result.data.sdk_root ?? '')
        setInstalledVersions(result.data.installed_versions)
        applySdkVersions(result.data.versions, result.data.prepared_versions)
        setSetupReport(result.data)
        setMessage(result.data.summary)
        logger.info('SDK setup configuration completed', {
          summary: result.data.summary,
          preparedVersions: result.data.prepared_versions.map(item => ({
            version: item.version,
            status: item.status,
            source: item.source,
            sdkRoot: item.sdk_root,
            archivePath: item.archive_path,
          })),
        })
      } else {
        setMessage(result.error)
        logger.error('SDK setup configuration failed', { error: result.error })
      }
    } catch (error) {
      setMessage(errorMessage(error))
      void logger.recordCrash('sdk-setup-configure', error, { sdkRoot })
    } finally {
      setActiveAction(null)
    }
  }

  function applySdkVersions(
    next: SdkVersionOption[],
    preparedVersions: SdkSetupReport['prepared_versions'] = []
  ) {
    const nextVersions = mergePreparedVersionStatuses(
      next.length > 0 ? next : FALLBACK_SDK_VERSIONS,
      preparedVersions
    )
    setVersions(nextVersions)
    if (!nextVersions.some(item => item.version === selectedVersion)) {
      const nextVersion = nextVersions[0]?.version ?? DEFAULT_SDK_START_VERSION
      setSelectedVersion(nextVersion)
      setSdkStartVersion(
        nextVersion,
        nextVersions.map(item => item.version)
      )
    }
  }

  const settingsLayout = variant === 'settings'
  const content = (
    <div
      className={cn(
        'min-w-0 overflow-x-hidden',
        settingsLayout ? 'space-y-3 p-0' : 'space-y-4 p-4'
      )}
    >
      <div className="space-y-1.5">
        <FieldLabel
          label={t('sdk.fields.sdkRoot')}
          help={t('sdk.help.sdkRoot')}
        />
        <PathPicker
          value={sdkRoot}
          title={t('sdk.picker.sdkRoot')}
          size={settingsLayout ? 'sm' : 'default'}
          onChange={setSdkRoot}
        />
        <div
          className={cn(
            'grid min-w-0 gap-2',
            settingsLayout ? 'grid-cols-1 lg:grid-cols-2' : 'grid-cols-2'
          )}
        >
          <Button
            variant="secondary"
            size={settingsLayout ? 'sm' : 'default'}
            onClick={() => void autoDetect()}
            disabled={activeAction !== null}
          >
            <Stethoscope className="size-4" />
            {t('sdk.actions.inspect')}
            <HelpHint text={t('sdk.help.autoDetect')} />
          </Button>
          <Button
            size={settingsLayout ? 'sm' : 'default'}
            onClick={() => void configureRequiredSdks()}
            disabled={activeAction !== null}
          >
            <Sparkles className="size-4" />
            {activeAction === 'configure'
              ? t('sdk.actions.configuring')
              : t('sdk.actions.configure')}
            <HelpHint text={t('sdk.help.configure')} />
          </Button>
        </div>
        <div
          className={cn(
            'grid min-w-0 gap-2',
            settingsLayout ? 'grid-cols-1 lg:grid-cols-2' : 'sm:grid-cols-2'
          )}
        >
          <Button
            variant="outline"
            size={settingsLayout ? 'sm' : 'default'}
            className="flex-1"
            onClick={() => void saveRootConfig()}
            title={t('sdk.actions.saveRoot')}
          >
            <Save className="size-4" />
            {t('sdk.actions.saveRoot')}
          </Button>
          <Button
            variant="outline"
            size={settingsLayout ? 'sm' : 'default'}
            className="flex-1"
            onClick={() => void loadSdkConfig()}
            title={t('sdk.actions.refresh')}
          >
            <RefreshCw className="size-4" />
            {t('sdk.actions.refresh')}
          </Button>
        </div>
      </div>

      <div
        className={cn(
          'space-y-2 rounded-md border bg-muted/20',
          settingsLayout ? 'p-2.5' : 'p-3'
        )}
      >
        <div className="text-xs font-medium">{t('sdk.rules.title')}</div>
        <div
          className={cn(
            'text-xs text-muted-foreground leading-relaxed',
            settingsLayout
              ? 'grid min-w-0 gap-x-4 gap-y-1.5 xl:grid-cols-2'
              : 'space-y-1.5'
          )}
        >
          <p>{t('sdk.rules.detectC4d')}</p>
          <p>{t('sdk.rules.sdkRoot')}</p>
          <p>{t('sdk.rules.manual')}</p>
          <p>{t('sdk.rules.tools')}</p>
        </div>
      </div>

      <SetupChecklist
        requirements={setupReport?.requirements ?? []}
        loading={activeAction !== null}
        wide={settingsLayout}
      />

      <div className="space-y-2">
        <FieldLabel
          label={t('sdk.fields.sdkMatrix')}
          help={t('sdk.help.sdkMatrix')}
        />
        <div className="flex flex-wrap gap-2">
          {versions.map(version => {
            const c4dInstalled = installedSdkVersionNames.has(version.version)

            return (
              <button
                key={version.version}
                type="button"
                className={cn(
                  'rounded-md border px-2 py-1 text-xs transition-colors',
                  selectedVersion === version.version
                    ? 'border-primary bg-primary text-primary-foreground'
                    : c4dInstalled
                      ? 'bg-background hover:bg-muted'
                      : 'border-yellow-500/50 bg-yellow-500/10 text-yellow-700 hover:bg-yellow-500/15 dark:text-yellow-300'
                )}
                onClick={() => setSelectedVersion(version.version)}
                title={
                  c4dInstalled
                    ? version.label
                    : t('sdk.missingC4d.installHint', {
                        version: c4dInstallLabel(version.version),
                      })
                }
              >
                {version.version}
                {!c4dInstalled ? (
                  <span className="ml-1 text-[10px]">!</span>
                ) : null}
              </button>
            )
          })}
        </div>
        {versions.length === 0 || loading ? (
          <div className="rounded-md border bg-muted/20 px-2.5 py-2 text-xs text-muted-foreground">
            {loading ? t('sdk.loading') : t('sdk.noVersions')}
          </div>
        ) : null}
      </div>

      {selectedOption ? (
        <div
          className={cn(
            'rounded-md border bg-muted/20',
            settingsLayout ? 'space-y-2 p-2.5' : 'space-y-3 p-3'
          )}
        >
          <div className="flex items-center justify-between gap-2">
            <div className="min-w-0">
              <div className="truncate text-sm font-medium">
                {selectedOption.label}
              </div>
              <div className="truncate text-xs text-muted-foreground">
                {selectedOption.status}
              </div>
            </div>
            <Badge
              variant={
                selectedOption.sdk_root || selectedOption.sdk_zip
                  ? 'default'
                  : 'outline'
              }
            >
              {selectedOption.sdk_root || selectedOption.sdk_zip
                ? t('sdk.status.ready')
                : t('sdk.status.auto')}
            </Badge>
          </div>
          <SourceLine label="SDK" value={selectedOption.sdk_root} />
          <SourceLine label="Zip" value={selectedOption.sdk_zip} />
          <SourceLine label="URL" value={selectedOption.download_url} />
          {!selectedC4dInstalled ? (
            <div className="rounded-md border border-yellow-500/40 bg-yellow-500/10 px-2.5 py-2 text-xs text-yellow-700 dark:text-yellow-300">
              {t('sdk.missingC4d.installHint', {
                version: c4dInstallLabel(selectedOption.version),
              })}
            </div>
          ) : null}
        </div>
      ) : null}

      <div className="space-y-2">
        <FieldLabel
          label={t('sdk.fields.installedC4d')}
          help={t('sdk.help.installedC4d')}
        />
        {installedVersions.length === 0 ? (
          <div className="rounded-md border bg-muted/20 p-2 text-xs text-muted-foreground">
            {t('sdk.noInstall')}
          </div>
        ) : (
          <div
            className={cn(
              'grid min-w-0 gap-2',
              settingsLayout && 'xl:grid-cols-2'
            )}
          >
            {installedVersions.map(item => (
              <div
                key={item.path}
                className="rounded-md border bg-background px-2.5 py-2 text-xs"
              >
                <div className="flex items-center justify-between gap-2">
                  <span className="font-medium">Cinema 4D {item.version}</span>
                  <Badge variant="outline">{item.sdk_version}</Badge>
                </div>
                <div className="mt-1 break-all text-muted-foreground">
                  {item.path}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {message ? (
        <div className="break-words rounded-md border bg-background px-2.5 py-2 text-xs text-muted-foreground">
          {message}
        </div>
      ) : null}
    </div>
  )

  if (settingsLayout) {
    return (
      <div className="mx-auto w-full min-w-0 max-w-none space-y-3">
        <div className="space-y-0.5">
          <div className="flex items-center gap-2 text-sm font-semibold">
            <FolderCog className="size-4" />
            {t('sdk.title')}
            <HelpHint text={t('sdk.help.title')} />
          </div>
          <div className="text-xs text-muted-foreground">
            {t('sdk.subtitle')}
          </div>
        </div>
        {content}
      </div>
    )
  }

  return (
    <>
      <div className="border-b px-4 py-3">
        <div className="flex items-center gap-2 text-sm font-semibold">
          <FolderCog className="size-4" />
          {t('sdk.title')}
          <HelpHint text={t('sdk.help.title')} />
        </div>
        <div className="mt-1 text-xs text-muted-foreground">
          {t('sdk.subtitle')}
        </div>
      </div>

      <ScrollArea className="min-h-0 flex-1">{content}</ScrollArea>
    </>
  )
}

function mergePreparedVersionStatuses(
  versions: SdkVersionOption[],
  preparedVersions: SdkSetupReport['prepared_versions']
) {
  if (preparedVersions.length === 0) {
    return versions
  }

  return versions.map(version => {
    const prepared = preparedVersions.find(
      item => item.version === version.version
    )
    if (!prepared || prepared.status === 'ready') {
      return version
    }

    return {
      ...version,
      sdk_root: prepared.sdk_root,
      sdk_zip: prepared.archive_path,
      download_url: prepared.download_url ?? version.download_url,
      status: prepared.status,
    }
  })
}

function c4dInstallLabel(sdkVersion: string) {
  if (sdkVersion === '2024.4') {
    return 'Cinema 4D 2024.4+'
  }

  return `Cinema 4D ${sdkVersion}`
}

function SetupChecklist({
  requirements,
  loading,
  wide = false,
}: {
  requirements: SetupRequirement[]
  loading: boolean
  wide?: boolean
}) {
  const { t } = useTranslation()

  if (loading) {
    return (
      <div className="rounded-md border bg-muted/20 p-2 text-xs text-muted-foreground">
        {t('sdk.inspecting')}
      </div>
    )
  }

  if (requirements.length === 0) {
    return (
      <div className="rounded-md border bg-muted/20 p-2 text-xs text-muted-foreground">
        {t('sdk.inspectEmpty')}
      </div>
    )
  }

  return (
    <div className="space-y-2">
      <FieldLabel
        label={t('sdk.fields.requiredEnvironment')}
        help={t('sdk.help.requiredEnvironment')}
      />
      <div className={cn('grid min-w-0 gap-2', wide && 'xl:grid-cols-2')}>
        {requirements.map(item => (
          <div
            key={item.key}
            className={cn(
              'rounded-md border bg-background',
              wide ? 'px-2.5 py-2' : 'p-2'
            )}
          >
            <div className="flex items-start justify-between gap-2">
              <div className="flex min-w-0 items-center gap-2">
                <RequirementIcon status={item.status} />
                <div className="min-w-0">
                  <div className="break-words text-xs font-medium">
                    {item.label}
                  </div>
                  <div className="break-words text-xs text-muted-foreground">
                    {item.version ?? item.detail}
                  </div>
                </div>
              </div>
              <Badge variant={item.status === 'Ready' ? 'default' : 'outline'}>
                {t(`sdk.requirement.${item.status}`)}
              </Badge>
            </div>
            {item.path ? (
              <div className="mt-1 break-all pl-5 text-xs text-muted-foreground">
                {item.path}
              </div>
            ) : null}
            {item.install_hint ? (
              <div className="mt-1 break-words pl-5 text-xs text-muted-foreground">
                {item.install_hint}
              </div>
            ) : null}
          </div>
        ))}
      </div>
    </div>
  )
}

function RequirementIcon({ status }: { status: SetupRequirement['status'] }) {
  if (status === 'Ready') {
    return <CheckCircle2 className="size-3.5 shrink-0 text-green-500" />
  }

  if (status === 'Warning') {
    return <TriangleAlert className="size-3.5 shrink-0 text-yellow-500" />
  }

  return <XCircle className="size-3.5 shrink-0 text-destructive" />
}

function SourceLine({
  label,
  value,
}: {
  label: string
  value?: string | null
}) {
  if (!value) return null

  return (
    <div className="flex min-w-0 items-start gap-2 text-xs text-muted-foreground">
      <Download className="size-3.5 shrink-0" />
      <span className="shrink-0 font-medium text-foreground">{label}</span>
      <span className="min-w-0 break-all" title={value}>
        {value}
      </span>
    </div>
  )
}

function errorMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message
  }

  if (typeof error === 'string') {
    return error
  }

  return 'Unknown error'
}

function FieldLabel({ label, help }: { label: string; help: string }) {
  return (
    <Label className="flex items-center gap-1.5 text-xs text-muted-foreground">
      <span>{label}</span>
      <HelpHint text={help} />
    </Label>
  )
}
