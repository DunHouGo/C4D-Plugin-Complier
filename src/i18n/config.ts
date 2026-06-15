import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import enUS from '../../locales/en-US.json'
import arSA from '../../locales/ar-SA.json'
import frFR from '../../locales/fr-FR.json'
import zhCN from '../../locales/zh-CN.json'

const resources = {
  'en-US': { translation: enUS },
  'ar-SA': { translation: arSA },
  'fr-FR': { translation: frFR },
  'zh-CN': { translation: zhCN },
}

// RTL language detection (includes languages not yet in resources for future expansion)
const rtlLanguages = ['ar', 'he', 'fa', 'ur']
const fallbackLanguage = 'en-US'
const languageAliases: Record<string, string> = {
  en: 'en-US',
  ar: 'ar-SA',
  fr: 'fr-FR',
  zh: 'zh-CN',
}

i18n.use(initReactI18next).init({
  resources,
  lng: fallbackLanguage,
  fallbackLng: fallbackLanguage,
  interpolation: {
    escapeValue: false, // React already escapes
  },
})

// Update document direction and lang on language change
i18n.on('languageChanged', lng => {
  const dir = isRTL(lng) ? 'rtl' : 'ltr'
  document.documentElement.dir = dir
  document.documentElement.lang = lng
})

export default i18n

// Export for use in non-React contexts (like menu building)
export { i18n }

// Helper to get available languages
export const availableLanguages = Object.keys(resources)
export const defaultLanguage = fallbackLanguage

// Resolve a locale like "zh-Hans-CN" or "en-US" to a supported resource key.
export function getBestAvailableLanguage(locale: string | null): string | null {
  if (!locale) return null

  const normalizedLocale = locale.toLowerCase().replaceAll('_', '-')

  const exactMatch = availableLanguages.find(
    language => language.toLowerCase() === normalizedLocale
  )
  if (exactMatch) return exactMatch

  const [languageCode] = normalizedLocale.split('-')
  if (!languageCode) return null

  return languageAliases[languageCode] ?? null
}

// Check if a language is RTL
export const isRTL = (lng: string): boolean => {
  const [languageCode] = lng.toLowerCase().split('-')
  return rtlLanguages.includes(languageCode ?? lng)
}
