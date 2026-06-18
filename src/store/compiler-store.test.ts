import { beforeEach, describe, expect, it } from 'vitest'
import {
  DEFAULT_SDK_START_VERSION,
  defaultBuildRequest,
  useCompilerStore,
} from './compiler-store'

describe('CompilerStore', () => {
  beforeEach(() => {
    useCompilerStore.setState({
      request: defaultBuildRequest,
      sdkStartVersion: DEFAULT_SDK_START_VERSION,
    })
  })

  it('detects module and package names from plugin root', () => {
    const { updatePluginRoot } = useCompilerStore.getState()

    updatePluginRoot('/Users/test/Plugins/SplineTools')

    expect(useCompilerStore.getState().request).toMatchObject({
      plugin_root: '/Users/test/Plugins/SplineTools',
      module_name: 'SplineTools',
      package_name: 'SplineTools',
    })
  })

  it('keeps manually entered module and package names', () => {
    const { updatePluginRoot, updateRequest } = useCompilerStore.getState()

    updateRequest({
      module_name: 'custom_module',
      package_name: 'Custom Package',
    })
    updatePluginRoot('C:\\Plugins\\DetectedName\\')

    expect(useCompilerStore.getState().request).toMatchObject({
      plugin_root: 'C:\\Plugins\\DetectedName\\',
      module_name: 'custom_module',
      package_name: 'Custom Package',
    })
  })
})
