/**
 * Application menu builder using Tauri's JavaScript API.
 *
 * This module creates native menus from JavaScript, enabling i18n support
 * through react-i18next. Menus are rebuilt when the language changes.
 */
import {
  Menu,
  MenuItem,
  Submenu,
  PredefinedMenuItem,
} from '@tauri-apps/api/menu'
import { getVersion } from '@tauri-apps/api/app'
import i18n from '@/i18n/config'
import { useUIStore } from '@/store/ui-store'
import { logger } from '@/lib/logger'
import { checkAndInstallUpdate } from '@/lib/updater'

/**
 * Build and set the application menu with translated labels.
 */
export async function buildAppMenu(): Promise<Menu> {
  const t = i18n.t.bind(i18n)
  const appName = t('app.name')

  try {
    // Build the main application submenu (appears as app name on macOS)
    const appSubmenu = await Submenu.new({
      text: appName,
      items: [
        await MenuItem.new({
          id: 'about',
          text: t('menu.about', { appName }),
          action: handleAbout,
        }),
        await PredefinedMenuItem.new({ item: 'Separator' }),
        await MenuItem.new({
          id: 'check-updates',
          text: t('menu.checkForUpdates'),
          action: handleCheckForUpdates,
        }),
        await PredefinedMenuItem.new({ item: 'Separator' }),
        await MenuItem.new({
          id: 'preferences',
          text: t('menu.preferences'),
          accelerator: 'CmdOrCtrl+,',
          action: handleOpenPreferences,
        }),
        await PredefinedMenuItem.new({ item: 'Separator' }),
        await PredefinedMenuItem.new({
          item: 'Hide',
          text: t('menu.hide', { appName }),
        }),
        await PredefinedMenuItem.new({
          item: 'HideOthers',
          text: t('menu.hideOthers'),
        }),
        await PredefinedMenuItem.new({
          item: 'ShowAll',
          text: t('menu.showAll'),
        }),
        await PredefinedMenuItem.new({ item: 'Separator' }),
        await PredefinedMenuItem.new({
          item: 'Quit',
          text: t('menu.quit', { appName }),
        }),
      ],
    })

    // Build the complete menu
    const menu = await Menu.new({
      items: [appSubmenu],
    })

    // Set as the application menu
    await menu.setAsAppMenu()

    logger.info('Application menu built successfully')
    return menu
  } catch (error) {
    logger.error('Failed to build application menu', { error })
    throw error
  }
}

/**
 * Set up a listener to rebuild the menu when the language changes.
 * Returns an unsubscribe function for cleanup.
 */
export function setupMenuLanguageListener(): () => void {
  const handler = async () => {
    logger.info('Language changed, rebuilding menu')
    try {
      await buildAppMenu()
    } catch (error) {
      logger.error('Failed to rebuild menu on language change', { error })
    }
  }
  i18n.on('languageChanged', handler)
  return () => i18n.off('languageChanged', handler)
}

// Menu action handlers

function handleAbout(): void {
  const appName = i18n.t('app.name')
  logger.info('About menu item clicked')
  void showAboutDialog(appName)
}

async function handleCheckForUpdates(): Promise<void> {
  logger.info('Check for Updates menu item clicked')
  await checkAndInstallUpdate({
    source: 'menu',
    silentNoUpdate: false,
    notifyOnError: true,
  })
}

function handleOpenPreferences(): void {
  logger.info('Preferences menu item clicked')
  useUIStore.getState().setPreferencesOpen(true)
}

async function showAboutDialog(appName: string): Promise<void> {
  try {
    const version = await getVersion()
    alert(
      `${appName}\n\nVersion: ${version}\n\nBuilt with Tauri v2 + React + TypeScript`
    )
  } catch (error) {
    logger.error('Failed to read app version for About dialog', { error })
    alert(
      `${appName}\n\nVersion: ${__APP_VERSION__}\n\nBuilt with Tauri v2 + React + TypeScript`
    )
  }
}
