import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from '@/components/ui/empty'
import { FolderCog, Settings } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useUIStore } from '@/store/ui-store'

interface LeftSideBarProps {
  children?: React.ReactNode
  className?: string
}

export function LeftSideBar({ children, className }: LeftSideBarProps) {
  return (
    <div
      className={cn(
        'flex h-full min-h-0 min-w-0 flex-col overflow-hidden border-r bg-background',
        className
      )}
    >
      {children ?? <SdkSettingsShortcut />}
    </div>
  )
}

function SdkSettingsShortcut() {
  const { t } = useTranslation()
  const setPreferencesOpen = useUIStore(state => state.setPreferencesOpen)

  return (
    <Empty className="h-full rounded-none border-0">
      <EmptyHeader>
        <EmptyMedia variant="icon">
          <FolderCog className="size-5" />
        </EmptyMedia>
        <EmptyTitle>{t('sdk.sidebar.title')}</EmptyTitle>
        <EmptyDescription>{t('sdk.sidebar.description')}</EmptyDescription>
      </EmptyHeader>
      <EmptyContent>
        <Button onClick={() => setPreferencesOpen(true)}>
          <Settings className="size-4" />
          {t('sdk.sidebar.openSettings')}
        </Button>
      </EmptyContent>
    </Empty>
  )
}
