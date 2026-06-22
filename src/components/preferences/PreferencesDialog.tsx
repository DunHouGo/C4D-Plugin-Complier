import { useState, type CSSProperties } from 'react'
import { useTranslation } from 'react-i18next'
import { Settings, Palette, XIcon } from 'lucide-react'
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
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
} from '@/components/ui/sidebar'
import { useUIStore } from '@/store/ui-store'
import { GeneralPane } from './panes/GeneralPane'
import { AppearancePane } from './panes/AppearancePane'
import { Button } from '../ui/button'

type PreferencePane = 'general' | 'appearance'

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
        className="overflow-hidden p-0 md:max-h-[76vh] md:max-w-[820px] lg:max-w-[920px] font-sans rounded-xl"
      >
        <DialogTitle className="sr-only">{t('preferences.title')}</DialogTitle>
        <DialogDescription className="sr-only">
          {t('preferences.description')}
        </DialogDescription>

        <SidebarProvider
          className="items-start"
          style={
            {
              '--sidebar-width': '12rem',
            } as CSSProperties
          }
        >
          <Sidebar collapsible="none" className="hidden md:flex">
            <SidebarContent className="p-2">
              <SidebarGroup className="p-0">
                <SidebarGroupContent>
                  <SidebarMenu>
                    {navigationItems.map(item => (
                      <SidebarMenuItem key={item.id}>
                        <SidebarMenuButton
                          asChild
                          isActive={activePane === item.id}
                        >
                          <button
                            onClick={() => setActivePane(item.id)}
                            className="w-full text-sm"
                          >
                            <item.icon />
                            <span>{t(item.labelKey)}</span>
                          </button>
                        </SidebarMenuButton>
                      </SidebarMenuItem>
                    ))}
                  </SidebarMenu>
                </SidebarGroupContent>
              </SidebarGroup>
            </SidebarContent>
          </Sidebar>

          <main className="flex flex-1 flex-col overflow-hidden ">
            <header className="flex h-12 shrink-0 items-center gap-2 border-b">
              <div className="flex items-center gap-2 px-4 grow">
                <Breadcrumb className="grow">
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

            <div className="flex flex-1 flex-col gap-3 overflow-y-auto p-4 max-h-[calc(76vh-3rem)]">
              {activePane === 'general' && <GeneralPane />}
              {activePane === 'appearance' && <AppearancePane />}
            </div>
          </main>
        </SidebarProvider>
      </DialogContent>
    </Dialog>
  )
}
