/**
 * Language initialization utilities for detecting and applying the user's
 * preferred language at app startup.
 */
import { locale } from '@tauri-apps/plugin-os'
import i18n, {
  availableLanguages,
  defaultLanguage,
  getBestAvailableLanguage,
} from './config'
import { logger } from '@/lib/logger'

/**
 * Initialize the application language.
 *
 * Priority:
 * 1. User's saved language preference (if set)
 * 2. System locale (if we have translations for it)
 * 3. English (fallback)
 *
 * @param savedLanguage - The user's saved language preference from preferences
 */
export async function initializeLanguage(
  savedLanguage: string | null
): Promise<void> {
  try {
    if (savedLanguage) {
      // User has an explicit preference
      const targetLanguage = getBestAvailableLanguage(savedLanguage)

      if (targetLanguage) {
        await i18n.changeLanguage(targetLanguage)
        logger.info('Language set from user preference', {
          language: targetLanguage,
        })
      } else {
        logger.warn('Saved language not available, using English', {
          savedLanguage,
          availableLanguages,
        })
        await i18n.changeLanguage(defaultLanguage)
      }
      return
    }

    // No saved preference, try to detect system locale
    const systemLocale = await locale()
    logger.debug('Detected system locale', { systemLocale })

    if (systemLocale) {
      const targetLanguage = getBestAvailableLanguage(systemLocale)

      if (targetLanguage) {
        await i18n.changeLanguage(targetLanguage)
        logger.info('Language set from system locale', {
          systemLocale,
          language: targetLanguage,
        })
        return
      }

      logger.debug('System locale not available in translations', {
        systemLocale,
        availableLanguages,
      })
    }

    // Fallback to English
    await i18n.changeLanguage(defaultLanguage)
    logger.info('Language set to English (fallback)')
  } catch (error) {
    logger.error('Failed to initialize language', { error })
    // Ensure we have some language set
    await i18n.changeLanguage(defaultLanguage)
  }
}
