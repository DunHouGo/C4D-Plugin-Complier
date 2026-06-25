import ReactDOM from 'react-dom/client'
import { QueryClientProvider } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import './i18n'
import App from './App'
import { queryClient } from './lib/query-client'
import { logger } from './lib/logger'

const handleWindowError = (event: ErrorEvent) => {
  void logger.recordCrash(
    'frontend-window-error',
    event.error ?? event.message,
    {
      filename: event.filename,
      lineno: event.lineno,
      colno: event.colno,
    }
  )
}

const handleUnhandledRejection = (event: PromiseRejectionEvent) => {
  void logger.recordCrash('frontend-unhandled-rejection', event.reason)
}

const root = ReactDOM.createRoot(document.getElementById('root') as HTMLElement)

window.addEventListener('error', handleWindowError)
window.addEventListener('unhandledrejection', handleUnhandledRejection)

root.render(
  <QueryClientProvider client={queryClient}>
    <App />
    <ReactQueryDevtools initialIsOpen={false} />
  </QueryClientProvider>
)

if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    window.removeEventListener('error', handleWindowError)
    window.removeEventListener('unhandledrejection', handleUnhandledRejection)
    root.unmount()
  })
}
