import { cn } from '@/lib/utils'
import { CompilerWorkbench } from '@/components/compiler/CompilerWorkbench'

interface MainWindowContentProps {
  children?: React.ReactNode
  className?: string
}

export function MainWindowContent({
  children,
  className,
}: MainWindowContentProps) {
  return (
    <div
      className={cn(
        'flex h-full min-w-0 flex-1 flex-col bg-background',
        className
      )}
    >
      {children || <CompilerWorkbench />}
    </div>
  )
}
