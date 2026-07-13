import { check, type DownloadEvent } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { getVersion } from '@tauri-apps/api/app'
import { logger } from '@/lib/logger'
import { notifications } from '@/lib/notifications'

type UpdateCheckStatus = 'installed' | 'cancelled' | 'none' | 'error'

interface UpdateCheckOptions {
  source: string
  silentNoUpdate?: boolean
  notifyOnError?: boolean
  onNoUpdate?: (version: string) => void
}

const STARTUP_RETRY_DELAYS_MS = [5_000, 30_000, 120_000]
const UPDATE_PROGRESS_LOG_INTERVAL_BYTES = 8 * 1024 * 1024

/**
 * Check, prompt, download, install, and optionally relaunch the app.
 */
export async function checkAndInstallUpdate({
  source,
  silentNoUpdate = false,
  notifyOnError = true,
  onNoUpdate,
}: UpdateCheckOptions): Promise<UpdateCheckStatus> {
  logger.info('Checking for updates', { source })

  try {
    const update = await check()
    if (!update) {
      logger.info('No update available', { source })
      if (!silentNoUpdate) {
        notifications.success(
          'Up to Date',
          'You are running the latest version'
        )
      }
      const version = await getVersion()
      onNoUpdate?.(version)
      return 'none'
    }

    logger.info('Update available', {
      source,
      version: update.version,
      date: update.date,
    })

    const shouldUpdate = confirm(
      `Update available: ${update.version}\n\nWould you like to install this update now?`
    )
    if (!shouldUpdate) {
      logger.info('Update skipped by user', {
        source,
        version: update.version,
      })
      return 'cancelled'
    }

    const downloadLogger = createDownloadEventLogger()
    await update.downloadAndInstall(event => {
      downloadLogger(event)
    })

    logger.info('Update installed successfully', {
      source,
      version: update.version,
    })

    const shouldRestart = confirm(
      'Update completed successfully!\n\nWould you like to restart the app now to use the new version?'
    )
    if (shouldRestart) {
      await relaunch()
    }

    return 'installed'
  } catch (error) {
    logger.error('Update check failed', {
      source,
      error: String(error),
    })

    if (notifyOnError) {
      notifications.error(
        'Update Check Failed',
        `Could not check for updates: ${String(error)}`
      )
    }

    return 'error'
  }
}

/**
 * Schedule startup update checks with retry for transient network failures.
 */
export function scheduleStartupUpdateChecks(): () => void {
  let stopped = false
  const timers: number[] = []

  const scheduleAttempt = (attemptIndex: number) => {
    const delay = STARTUP_RETRY_DELAYS_MS[attemptIndex]
    if (delay === undefined) return

    const timer = window.setTimeout(async () => {
      if (stopped) return

      const status = await checkAndInstallUpdate({
        source: `startup-${attemptIndex + 1}`,
        silentNoUpdate: true,
        notifyOnError: attemptIndex === STARTUP_RETRY_DELAYS_MS.length - 1,
      })

      if (status === 'error') {
        scheduleAttempt(attemptIndex + 1)
      }
    }, delay)

    timers.push(timer)
  }

  scheduleAttempt(0)

  return () => {
    stopped = true
    for (const timer of timers) {
      window.clearTimeout(timer)
    }
  }
}

function createDownloadEventLogger(): (event: DownloadEvent) => void {
  let downloadedBytes = 0
  let lastLoggedBytes = 0

  return event => {
    switch (event.event) {
      case 'Started':
        downloadedBytes = 0
        lastLoggedBytes = 0
        logger.info(`Downloading ${event.data.contentLength} bytes`)
        break
      case 'Progress':
        downloadedBytes += event.data.chunkLength
        if (
          downloadedBytes - lastLoggedBytes >=
          UPDATE_PROGRESS_LOG_INTERVAL_BYTES
        ) {
          lastLoggedBytes = downloadedBytes
          logger.info(`Downloaded: ${downloadedBytes} bytes`)
        }
        break
      case 'Finished':
        logger.info('Download complete, installing...')
        break
    }
  }
}
