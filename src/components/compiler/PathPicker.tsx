import { useEffect, useRef, useState } from 'react'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open } from '@tauri-apps/plugin-dialog'
import { FolderOpen, X } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { logger } from '@/lib/logger'
import { cn } from '@/lib/utils'

interface PathPickerProps {
  value: string
  placeholder?: string
  title: string
  directory?: boolean
  filters?: { name: string; extensions: string[] }[]
  size?: 'default' | 'sm'
  onChange: (value: string) => void
}

export function PathPicker({
  value,
  placeholder,
  title,
  directory = true,
  filters,
  size = 'default',
  onChange,
}: PathPickerProps) {
  const rootRef = useRef<HTMLDivElement>(null)
  const valueRef = useRef(value)
  const onChangeRef = useRef(onChange)
  const [dragging, setDragging] = useState(false)

  useEffect(() => {
    valueRef.current = value
    onChangeRef.current = onChange
  }, [onChange, value])

  useEffect(() => {
    let cancelled = false
    let didUnlisten = false
    let unlisten: (() => void) | null = null

    try {
      void getCurrentWebview()
        .onDragDropEvent(event => {
          if (event.payload.type === 'drop') {
            const point = {
              x: event.payload.position.x,
              y: event.payload.position.y,
            }
            if (isPointInside(rootRef.current, point)) {
              onChangeRef.current(event.payload.paths[0] ?? valueRef.current)
            }
            setDragging(false)
          } else if (event.payload.type === 'enter') {
            setDragging(true)
          } else if (event.payload.type === 'leave') {
            setDragging(false)
          }
        })
        .then(nextUnlisten => {
          unlisten = nextUnlisten
          if (cancelled) {
            safeUnlisten()
          }
        })
        .catch(error => {
          logger.warn('Failed to register path drag-and-drop listener', {
            error,
          })
        })
    } catch {
      return undefined
    }

    return () => {
      cancelled = true
      safeUnlisten()
    }

    function safeUnlisten() {
      if (!unlisten || didUnlisten) {
        return
      }

      didUnlisten = true
      try {
        Promise.resolve(unlisten()).catch(error => {
          logger.warn('Failed to unregister path drag-and-drop listener', {
            error,
          })
        })
      } catch (error) {
        logger.warn('Failed to unregister path drag-and-drop listener', {
          error,
        })
      }
    }
  }, [])

  async function choosePath() {
    const selected = await open({
      title,
      directory,
      multiple: false,
      filters,
      defaultPath: value || undefined,
    })
    if (typeof selected === 'string') {
      onChange(selected)
    }
  }

  return (
    <div
      ref={rootRef}
      className={cn(
        'flex min-w-0 rounded-md ring-offset-background transition-shadow',
        dragging && 'ring-2 ring-ring'
      )}
    >
      <Input
        className={cn('min-w-0 rounded-r-none', size === 'sm' && 'h-8 text-sm')}
        value={value}
        placeholder={placeholder}
        onChange={event => onChange(event.target.value)}
      />
      {value ? (
        <Button
          type="button"
          variant="outline"
          size={size === 'sm' ? 'icon-sm' : 'icon'}
          className="rounded-none border-l-0"
          onClick={() => onChange('')}
          title="Clear path"
        >
          <X className="size-4" />
        </Button>
      ) : null}
      <Button
        type="button"
        variant="outline"
        size={size === 'sm' ? 'icon-sm' : 'icon'}
        className="rounded-l-none border-l-0"
        onClick={() => void choosePath()}
        title={title}
      >
        <FolderOpen className="size-4" />
      </Button>
    </div>
  )
}

function isPointInside(
  element: HTMLElement | null,
  point: { x: number; y: number }
) {
  if (!element) {
    return false
  }

  const rect = element.getBoundingClientRect()
  return (
    point.x >= rect.left &&
    point.x <= rect.right &&
    point.y >= rect.top &&
    point.y <= rect.bottom
  )
}
