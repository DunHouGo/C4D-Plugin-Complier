import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Switch } from '@/components/ui/switch'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { SettingsField, SettingsSection } from '../shared/SettingsComponents'

export function GeneralPane() {
  const { t } = useTranslation()
  // Example local state - these are NOT persisted to disk
  // To add persistent preferences:
  // 1. Add the field to AppPreferences in both Rust and TypeScript
  // 2. Use usePreferencesManager() and updatePreferences()
  const [exampleText, setExampleText] = useState('Example value')
  const [exampleToggle, setExampleToggle] = useState(true)

  return (
    <div className="space-y-6">
      <SettingsSection title={t('preferences.general.exampleSettings')}>
        <SettingsField
          label={t('preferences.general.exampleText')}
          description={t('preferences.general.exampleTextDescription')}
        >
          <Input
            value={exampleText}
            onChange={e => setExampleText(e.target.value)}
            placeholder={t('preferences.general.exampleTextPlaceholder')}
          />
        </SettingsField>

        <SettingsField
          label={t('preferences.general.exampleToggle')}
          description={t('preferences.general.exampleToggleDescription')}
        >
          <div className="flex items-center space-x-2">
            <Switch
              id="example-toggle"
              checked={exampleToggle}
              onCheckedChange={setExampleToggle}
            />
            <Label htmlFor="example-toggle" className="text-sm">
              {exampleToggle ? t('common.enabled') : t('common.disabled')}
            </Label>
          </div>
        </SettingsField>
      </SettingsSection>
    </div>
  )
}
