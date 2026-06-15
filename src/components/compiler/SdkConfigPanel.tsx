import { useCallback, useEffect, useState } from 'react'
import { Download, FolderCog, RefreshCw, Search, Save } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Label } from '@/components/ui/label'
import { ScrollArea } from '@/components/ui/scroll-area'
import {
  commands,
  type InstalledC4dVersion,
  type SdkVersionOption,
} from '@/lib/tauri-bindings'
import { DEFAULT_SDK_START_VERSION, useCompilerStore } from '@/store/compiler-store'
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

export function SdkConfigPanel() {
  const [versions, setVersions] = useState<SdkVersionOption[]>([])
  const [installedVersions, setInstalledVersions] = useState<
    InstalledC4dVersion[]
  >([])
  const [selectedVersion, setSelectedVersion] = useState(
    DEFAULT_SDK_START_VERSION
  )
  const [sdkRoot, setSdkRoot] = useState('')
  const [loading, setLoading] = useState(true)
  const [message, setMessage] = useState<string | null>(null)
  const setSdkStartVersion = useCompilerStore(state => state.setSdkStartVersion)

  const selectedOption = versions.find(
    version => version.version === selectedVersion
  )

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
        const nextVersion = nextVersions[0]?.version ?? DEFAULT_SDK_START_VERSION
        setSelectedVersion(nextVersion)
        setSdkStartVersion(nextVersion, nextVersions.map(item => item.version))
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
    const result = await commands.saveSdkRootConfig({
      sdk_root: sdkRoot || null,
    })
    if (result.status === 'ok') {
      setSdkRoot(result.data.sdk_root ?? '')
      await loadSdkConfig()
      setMessage('Saved SDK root')
    } else {
      setMessage(result.error)
    }
  }

  async function autoDetect() {
    setLoading(true)
    const result = await commands.autoConfigureSdkSources()
    if (result.status === 'ok') {
      setSdkRoot(result.data.sdk_root ?? '')
      setInstalledVersions(result.data.installed_versions)
      setVersions(
        result.data.versions.length > 0
          ? result.data.versions
          : FALLBACK_SDK_VERSIONS
      )
      const nextVersions =
        result.data.versions.length > 0
          ? result.data.versions
          : FALLBACK_SDK_VERSIONS
      const nextVersion = nextVersions[0]?.version ?? DEFAULT_SDK_START_VERSION
      setSelectedVersion(nextVersion)
      setSdkStartVersion(nextVersion, nextVersions.map(item => item.version))
      setMessage(
        result.data.installed_versions.length > 0
          ? `Detected ${result.data.installed_versions.length} Cinema 4D install(s)`
          : 'No local Cinema 4D install detected; SDK URLs are still ready'
      )
    } else {
      setMessage(result.error)
    }
    setLoading(false)
  }

  return (
    <>
      <div className="border-b px-4 py-3">
        <div className="flex items-center gap-2 text-sm font-semibold">
          <FolderCog className="size-4" />
          SDK Sources
          <HelpHint text="Configure one SDK root. The app creates per-version folders and resolves Maxon C++ SDK download URLs automatically." />
        </div>
        <div className="mt-1 text-xs text-muted-foreground">
          Cinema 4D 2024.4 and newer
        </div>
      </div>

      <ScrollArea className="min-h-0 flex-1">
        <div className="space-y-4 p-4">
          <div className="space-y-1.5">
            <FieldLabel
              label="SDK Root"
              help="Choose one root folder without spaces, such as Documents\\Maxon_SDK. Version folders are created automatically under it."
            />
            <PathPicker
              value={sdkRoot}
              title="Choose SDK root"
              onChange={setSdkRoot}
            />
            <div className="flex gap-2">
              <Button className="flex-1" onClick={() => void autoDetect()}>
                <Search className="size-4" />
                Auto Detect
                <HelpHint text="Detect installed Cinema 4D versions, pick the matching SDK URL, and save the default Maxon_SDK root." />
              </Button>
              <Button
                variant="outline"
                size="icon"
                onClick={() => void saveRootConfig()}
                title="Save SDK root"
              >
                <Save className="size-4" />
              </Button>
              <Button
                variant="outline"
                size="icon"
                onClick={() => void loadSdkConfig()}
                title="Refresh SDK list"
              >
                <RefreshCw className="size-4" />
              </Button>
            </div>
          </div>

          <div className="space-y-2">
            <FieldLabel
              label="SDK Matrix"
              help="SDK folders and archives are resolved automatically from the root folder. Missing SDKs are downloaded from Maxon when resolving or building."
            />
            <div className="flex flex-wrap gap-2">
              {versions.map(version => (
                <button
                  key={version.version}
                  type="button"
                  className={cn(
                    'rounded-md border px-2 py-1 text-xs transition-colors',
                    selectedVersion === version.version
                      ? 'border-primary bg-primary text-primary-foreground'
                      : 'bg-background hover:bg-muted'
                  )}
                  onClick={() => setSelectedVersion(version.version)}
                >
                  {version.version}
                </button>
              ))}
            </div>
            {versions.length === 0 || loading ? (
              <div className="rounded-md border bg-muted/20 p-2 text-xs text-muted-foreground">
                {loading ? 'Loading SDK versions...' : 'No SDK versions found.'}
              </div>
            ) : null}
          </div>

          {selectedOption ? (
            <div className="space-y-3 rounded-md border bg-muted/20 p-3">
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
                  variant={selectedOption.sdk_root ? 'default' : 'outline'}
                >
                  {selectedOption.sdk_root ? 'Ready' : 'Auto'}
                </Badge>
              </div>
              <SourceLine label="SDK" value={selectedOption.sdk_root} />
              <SourceLine label="Zip" value={selectedOption.sdk_zip} />
              <SourceLine label="URL" value={selectedOption.download_url} />
            </div>
          ) : null}

          <div className="space-y-2">
            <FieldLabel
              label="Installed C4D"
              help="Detected local Cinema 4D installs. Each major version maps to the smallest supported SDK in that major version."
            />
            {installedVersions.length === 0 ? (
              <div className="rounded-md border bg-muted/20 p-2 text-xs text-muted-foreground">
                No local Cinema 4D install detected.
              </div>
            ) : (
              <div className="space-y-2">
                {installedVersions.map(item => (
                  <div
                    key={item.path}
                    className="rounded-md border bg-background p-2 text-xs"
                  >
                    <div className="flex items-center justify-between gap-2">
                      <span className="font-medium">
                        Cinema 4D {item.version}
                      </span>
                      <Badge variant="outline">{item.sdk_version}</Badge>
                    </div>
                    <div className="mt-1 truncate text-muted-foreground">
                      {item.path}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          {message ? (
            <div className="rounded-md border bg-background p-2 text-xs text-muted-foreground">
              {message}
            </div>
          ) : null}
        </div>
      </ScrollArea>
    </>
  )
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
    <div className="flex min-w-0 items-center gap-2 text-xs text-muted-foreground">
      <Download className="size-3.5 shrink-0" />
      <span className="shrink-0 font-medium text-foreground">{label}</span>
      <span className="truncate" title={value}>
        {value}
      </span>
    </div>
  )
}

function FieldLabel({ label, help }: { label: string; help: string }) {
  return (
    <Label className="flex items-center gap-1.5 text-xs text-muted-foreground">
      <span>{label}</span>
      <HelpHint text={help} />
    </Label>
  )
}
