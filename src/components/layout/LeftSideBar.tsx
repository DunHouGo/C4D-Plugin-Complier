import { cn } from '@/lib/utils'
import { SdkConfigPanel } from '@/components/compiler/SdkConfigPanel'

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
      {children ?? <SdkConfigPanel />}
    </div>
  )
}
