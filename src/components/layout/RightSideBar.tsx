import { cn } from '@/lib/utils'
import { FileTreePreview } from '@/components/compiler/FileTreePreview'

interface RightSideBarProps {
  children?: React.ReactNode
  className?: string
}

export function RightSideBar({ children, className }: RightSideBarProps) {
  return (
    <div
      className={cn(
        'flex h-full min-w-0 flex-col border-l bg-background',
        className
      )}
    >
      {children ?? <FileTreePreview />}
    </div>
  )
}
