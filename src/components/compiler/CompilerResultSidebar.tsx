import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Archive, FolderOpen, FolderTree } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { useCompilerStore } from '@/store/compiler-store'
import { commands } from '@/lib/tauri-bindings'
import { FileTreePreview } from './FileTreePreview'
import { HelpHint } from './HelpHint'

type ResultPanel = 'artifacts' | 'preview'

export function CompilerResultSidebar() {
  const { t } = useTranslation()
  const artifacts = useCompilerStore(state => state.artifacts)
  const [resultPanel, setResultPanel] = useState<ResultPanel>(() =>
    artifacts.length > 0 ? 'artifacts' : 'preview'
  )
  const prevArtifactsLength = useRef(artifacts.length)

  useEffect(() => {
    // Auto-switch to artifacts panel when build completes and artifacts appear
    if (prevArtifactsLength.current === 0 && artifacts.length > 0) {
      setResultPanel('artifacts')
    }
    prevArtifactsLength.current = artifacts.length
  }, [artifacts.length])

  return (
    <>
      <div className="flex h-10 items-center justify-between gap-2 border-b px-3">
        <ToggleGroup
          type="single"
          value={resultPanel}
          variant="outline"
          size="sm"
          className="w-full"
          onValueChange={value => {
            if (value) {
              setResultPanel(value as ResultPanel)
            }
          }}
        >
          <ToggleGroupItem
            value="artifacts"
            className="gap-1.5 text-xs"
            title={t('compiler.panels.artifacts')}
          >
            <Archive className="size-3.5" />
            {t('compiler.panels.artifacts')}
          </ToggleGroupItem>
          <ToggleGroupItem
            value="preview"
            className="gap-1.5 text-xs"
            title={t('preview.title')}
          >
            <FolderTree className="size-3.5" />
            {t('preview.title')}
          </ToggleGroupItem>
        </ToggleGroup>
      </div>
      {resultPanel === 'artifacts' ? (
        <ScrollArea className="min-h-0 flex-1 overflow-hidden">
          <div className="space-y-2 p-3">
            {artifacts.length === 0 ? (
              <div className="text-xs text-muted-foreground">
                {t('compiler.empty.artifacts')}
              </div>
            ) : (
              artifacts.map(item => (
                <div
                  key={item.path}
                  className="rounded-md border bg-background p-3"
                >
                  <div className="text-sm font-medium">{item.kind}</div>
                  <div className="mt-1 truncate text-xs text-muted-foreground">
                    {item.path}
                  </div>
                  <Button
                    className="mt-2"
                    variant="outline"
                    size="sm"
                    onClick={() => void commands.openArtifactFolder(item.path)}
                  >
                    <FolderOpen className="size-3.5" />
                    {t('compiler.actions.open')}
                    <HelpHint text={t('compiler.help.openArtifact')} />
                  </Button>
                </div>
              ))
            )}
          </div>
        </ScrollArea>
      ) : (
        <FileTreePreview compact />
      )}
    </>
  )
}
