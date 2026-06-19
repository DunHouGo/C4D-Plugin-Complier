import { cn } from '@/lib/utils'
import { CompilerResultSidebar } from '@/components/compiler/CompilerResultSidebar'

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
      {children ?? <CompilerResultSidebar />}
    </div>
  )
}
