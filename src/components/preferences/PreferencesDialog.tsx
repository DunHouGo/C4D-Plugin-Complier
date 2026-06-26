import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Info, Palette, Settings, XIcon } from 'lucide-react'
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from '@/components/ui/breadcrumb'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from '@/components/ui/dialog'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useUIStore } from '@/store/ui-store'
import { cn } from '@/lib/utils'
import { GeneralPane } from './panes/GeneralPane'
import { AppearancePane } from './panes/AppearancePane'
import { AboutPane } from './panes/AboutPane'
import { Button } from '../ui/button'

type PreferencePane = 'general' | 'appearance' | 'about'

const navigationItems = [
  {
    id: 'general' as const,
    labelKey: 'preferences.general',
    icon: Settings,
  },
  {
    id: 'appearance' as const,
    labelKey: 'preferences.appearance',
    icon: Palette,
  },
  {
    id: 'about' as const,
    labelKey: 'preferences.about',
    icon: Info,
  },
] as const

export function PreferencesDialog() {
  const { t } = useTranslation()
  const [activePane, setActivePane] = useState<PreferencePane>('general')
  const preferencesOpen = useUIStore(state => state.preferencesOpen)
  const setPreferencesOpen = useUIStore(state => state.setPreferencesOpen)

  const getPaneTitle = (pane: PreferencePane): string => {
    return t(`preferences.${pane}`)
  }

  return (
    <Dialog open={preferencesOpen} onOpenChange={setPreferencesOpen}>
      <DialogContent
        showCloseButton={false}
        className="bottom-4 top-4 flex h-auto max-h-none w-[calc(100vw-2rem)] max-w-none translate-y-0 flex-col gap-0 overflow-hidden rounded-xl p-0 font-sans sm:max-w-none md:w-[min(960px,calc(100vw-3rem))]"
      >
        <DialogTitle className="sr-only">{t('preferences.title')}</DialogTitle>
        <DialogDescription className="sr-only">
          {t('preferences.description')}
        </DialogDescription>

        <div className="flex h-full min-h-0 min-w-0 flex-1 overflow-hidden">
          <aside className="hidden w-48 shrink-0 border-r bg-sidebar text-sidebar-foreground md:block">
            <nav className="flex flex-col gap-1 p-2">
              {navigationItems.map(item => (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => setActivePane(item.id)}
                  className={cn(
                    'flex h-8 w-full items-center gap-2 rounded-md px-2 text-start text-sm outline-hidden transition-colors',
                    'hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 focus-visible:ring-sidebar-ring',
                    activePane === item.id &&
                      'bg-sidebar-accent font-medium text-sidebar-accent-foreground'
                  )}
                >
                  <item.icon className="size-4 shrink-0" />
                  <span className="truncate">{t(item.labelKey)}</span>
                </button>
              ))}
            </nav>
          </aside>

          <main className="flex h-full min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
            <header className="flex h-12 shrink-0 items-center gap-2 border-b">
              <div className="flex min-w-0 grow items-center gap-2 px-4">
                <Breadcrumb className="min-w-0 grow">
                  <BreadcrumbList>
                    <BreadcrumbItem className="hidden md:block">
                      <BreadcrumbLink asChild>
                        <span>{t('preferences.title')}</span>
                      </BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator className="hidden md:block" />
                    <BreadcrumbItem>
                      <BreadcrumbPage>
                        {getPaneTitle(activePane)}
                      </BreadcrumbPage>
                    </BreadcrumbItem>
                  </BreadcrumbList>
                </Breadcrumb>
                <Button
                  variant="ghost"
                  size="icon-sm"
                  onClick={() => setPreferencesOpen(false)}
                >
                  <XIcon className="h-4 w-4" />
                </Button>
              </div>
            </header>

            <ScrollArea className="h-full min-h-0 min-w-0 flex-1 overflow-hidden">
              <div className="min-w-0 pb-6 pl-3 pr-4 pt-3 sm:p-4 sm:pb-6">
                {activePane === 'general' && <GeneralPane />}
                {activePane === 'appearance' && <AppearancePane />}
                {activePane === 'about' && <AboutPane />}
              </div>
            </ScrollArea>
          </main>
        </div>
      </DialogContent>
    </Dialog>
  )
}
