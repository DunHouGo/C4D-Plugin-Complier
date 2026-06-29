import { render, waitFor } from '@/test/test-utils'
import { commands } from '@/lib/tauri-bindings'
import { describe, expect, it, vi } from 'vitest'
import { CompilerWorkbench } from './CompilerWorkbench'
import { SdkConfigPanel } from './SdkConfigPanel'

describe('Sdk source refresh', () => {
  it('refreshes the workbench after sdk sources change', async () => {
    vi.mocked(commands.listSdkVersions)
      .mockResolvedValueOnce({
        status: 'ok',
        data: [
          {
            version: '2024.4',
            label: 'Cinema 4D 2024.4',
            configured: false,
            sdk_root: null,
            sdk_zip: null,
            download_url: null,
            status: 'auto download',
          },
        ],
      })
      .mockResolvedValueOnce({
        status: 'ok',
        data: [
          {
            version: '2024.4',
            label: 'Cinema 4D 2024.4',
            configured: true,
            sdk_root: 'C:\\SDK\\2024.4',
            sdk_zip: null,
            download_url: null,
            status: 'configured root',
          },
        ],
      })

    render(<CompilerWorkbench />)

    await waitFor(() => {
      expect(commands.listSdkVersions).toHaveBeenCalledTimes(1)
    })

    const eventModule = await import('@tauri-apps/api/event')
    const listenMock = vi.mocked(eventModule.listen)
    const handler = listenMock.mock.calls.find(
      call => call[0] === 'sdk://sources-changed'
    )?.[1]

    expect(handler).toBeDefined()

    await handler?.({
      event: 'sdk://sources-changed',
      id: 1,
      payload: null,
    })

    await waitFor(() => {
      expect(commands.listSdkVersions).toHaveBeenCalledTimes(2)
    })
  })

  it('broadcasts sdk source changes after one-click setup', async () => {
    const eventModule = await import('@tauri-apps/api/event')
    const emitMock = vi.mocked(eventModule.emit)

    render(<SdkConfigPanel />)

    await waitFor(() => {
      expect(commands.configureRequiredSdks).not.toHaveBeenCalled()
    })

    const configureButton = Array.from(document.querySelectorAll('button')).find(
      button => button.textContent?.includes('One-click Setup')
    ) as HTMLButtonElement | undefined

    expect(configureButton).toBeDefined()
    configureButton?.click()

    await waitFor(() => {
      expect(commands.configureRequiredSdks).toHaveBeenCalled()
    })

    await waitFor(() => {
      expect(emitMock).toHaveBeenCalledWith('sdk://sources-changed', null)
    })
  })
})
