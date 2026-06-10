import { Component, ErrorInfo, ReactNode } from 'react'
import { MainLayout } from '@/components/layout/MainLayout'
import { OfflineBanner } from '@/components/common/OfflineBanner'
import { isOffline } from './adapters'

class ErrorBoundary extends Component<
  { children: ReactNode },
  { hasError: boolean; error: Error | null }
> {
  state = { hasError: false, error: null }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('App crashed:', error, info)
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="flex flex-col items-center justify-center h-screen w-screen bg-background text-foreground gap-4 p-8">
          <h1 className="text-xl font-bold text-red-400">应用出错了</h1>
          <pre className="text-sm text-muted-foreground bg-muted p-4 rounded-lg max-w-2xl w-full overflow-auto max-h-[50vh]">
            {this.state.error?.message}
          </pre>
          <button
            className="px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm"
            onClick={() => window.location.reload()}
          >
            重新加载
          </button>
        </div>
      )
    }
    return this.props.children
  }
}

function App() {
  const offline = isOffline()

  return (
    <ErrorBoundary>
      <MainLayout />
      {offline && <OfflineBanner />}
    </ErrorBoundary>
  )
}

export default App
