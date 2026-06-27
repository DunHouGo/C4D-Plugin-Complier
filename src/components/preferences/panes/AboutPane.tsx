import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ExternalLink, Github, RefreshCw } from 'lucide-react'
import { getVersion } from '@tauri-apps/api/app'
import { openUrl } from '@tauri-apps/plugin-opener'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { SettingsField, SettingsSection } from '../shared/SettingsComponents'
import { checkAndInstallUpdate } from '@/lib/updater'
import { logger } from '@/lib/logger'

const GITHUB_REPOSITORY_URL = 'https://github.com/DunHouGo/C4D-Plugin-Complier'
const GITHUB_RELEASES_URL = `${GITHUB_REPOSITORY_URL}/releases/latest`

export function AboutPane() {
  const { t } = useTranslation()
  const [appVersion, setAppVersion] = useState<string>('...')
  const [checkingUpdate, setCheckingUpdate] = useState(false)
  const [openingUrl, setOpeningUrl] = useState<string | null>(null)

  useEffect(() => {
    let mounted = true

    const loadVersion = async () => {
      try {
        const version = await getVersion()
        if (mounted) {
          setAppVersion(version)
        }
      } catch (error) {
        logger.error('Failed to read app version', { error })
        if (mounted) {
          setAppVersion(__APP_VERSION__)
        }
      }
    }

    void loadVersion()

    return () => {
      mounted = false
    }
  }, [])

  const handleCheckForUpdates = async () => {
    setCheckingUpdate(true)
    try {
      await checkAndInstallUpdate({
        source: 'preferences-about',
        silentNoUpdate: true,
        notifyOnError: true,
        onNoUpdate: version => {
          toast.success(t('preferences.about.latestVersion', { version }))
        },
      })
    } finally {
      setCheckingUpdate(false)
    }
  }

  const handleOpenUrl = async (url: string) => {
    setOpeningUrl(url)
    try {
      await openUrl(url)
    } catch (error) {
      logger.error('Failed to open external URL', { url, error })
      toast.error(t('preferences.about.openFailed'))
    } finally {
      setOpeningUrl(null)
    }
  }

  return (
    <div className="space-y-6">
      <SettingsSection title={t('preferences.about')}>
        <div className="space-y-1">
          <h3 className="text-xl font-semibold text-foreground">
            {t('app.name')}
          </h3>
          <p className="text-sm text-muted-foreground">
            {t('preferences.about.subtitle')}
          </p>
        </div>

        <SettingsField
          label={t('preferences.about.version')}
          description={t('preferences.about.versionDescription')}
        >
          <div className="inline-flex h-9 items-center rounded-md border bg-muted/40 px-3 font-mono text-sm text-foreground">
            {appVersion}
          </div>
        </SettingsField>

        <SettingsField
          label={t('preferences.about.updates')}
          description={t('preferences.about.updatesDescription')}
        >
          <div className="flex flex-wrap gap-2">
            <Button
              type="button"
              onClick={handleCheckForUpdates}
              disabled={checkingUpdate}
            >
              <RefreshCw
                className={checkingUpdate ? 'animate-spin' : undefined}
              />
              {checkingUpdate
                ? t('preferences.about.checkingUpdates')
                : t('preferences.about.checkUpdates')}
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => handleOpenUrl(GITHUB_RELEASES_URL)}
              disabled={openingUrl === GITHUB_RELEASES_URL}
            >
              <ExternalLink />
              {t('preferences.about.openReleases')}
            </Button>
          </div>
        </SettingsField>

        <SettingsField
          label={t('preferences.about.github')}
          description={t('preferences.about.githubDescription')}
        >
          <Button
            type="button"
            variant="outline"
            onClick={() => handleOpenUrl(GITHUB_REPOSITORY_URL)}
            disabled={openingUrl === GITHUB_REPOSITORY_URL}
          >
            <Github />
            {t('preferences.about.openGithub')}
          </Button>
        </SettingsField>
      </SettingsSection>
    </div>
  )
}
