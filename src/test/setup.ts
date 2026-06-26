import '@testing-library/jest-dom'
import { vi } from 'vitest'

// Mock matchMedia for tests
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(), // deprecated
    removeListener: vi.fn(), // deprecated
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
})

// Mock Tauri APIs for tests
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {
    // Mock unlisten function
  }),
}))

vi.mock('@tauri-apps/plugin-updater', () => ({
  check: vi.fn().mockResolvedValue(null),
}))

vi.mock('@tauri-apps/plugin-opener', () => ({
  openUrl: vi.fn().mockResolvedValue(undefined),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  save: vi.fn().mockResolvedValue('/tmp/c4d-build.log'),
}))

vi.mock('@tauri-apps/plugin-fs', () => ({
  writeTextFile: vi.fn().mockResolvedValue(undefined),
}))

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  writeText: vi.fn().mockResolvedValue(undefined),
}))

// Mock typed Tauri bindings (tauri-specta generated)
vi.mock('@/lib/tauri-bindings', () => ({
  commands: {
    greet: vi.fn().mockResolvedValue('Hello, test!'),
    loadPreferences: vi
      .fn()
      .mockResolvedValue({ status: 'ok', data: { theme: 'system' } }),
    savePreferences: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
    sendNativeNotification: vi
      .fn()
      .mockResolvedValue({ status: 'ok', data: null }),
    saveEmergencyData: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
    loadEmergencyData: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
    cleanupOldRecoveryFiles: vi
      .fn()
      .mockResolvedValue({ status: 'ok', data: 0 }),
    loadBuildQueuePresets: vi.fn().mockResolvedValue({
      status: 'ok',
      data: { presets: [] },
    }),
    saveBuildQueuePresets: vi.fn().mockResolvedValue({
      status: 'ok',
      data: null,
    }),
    detectEnvironment: vi.fn().mockResolvedValue({
      status: 'ok',
      data: {
        os: 'windows',
        supported: true,
        compiler_platform: 'Windows',
        cmake_preset: 'windows_x86_64',
        binary_extension: 'xdl64',
        cmake: {
          found: true,
          path: 'C:\\Program Files\\CMake\\bin\\cmake.exe',
          version: 'cmake version 4.0.0',
          message: null,
        },
        visual_studio: {
          found: true,
          path: 'C:\\Program Files\\Microsoft Visual Studio\\2022',
          version: null,
          message: null,
        },
        windows_sdk: {
          found: true,
          path: 'C:\\Program Files (x86)\\Windows Kits\\10\\Include',
          version: '10.0.28000.0',
          message: null,
        },
        installed_sdk_zips: [],
        installed_c4d_versions: [
          {
            version: '2026',
            path: 'C:\\Program Files\\Maxon Cinema 4D 2026',
            sdk_version: '2026',
            download_url:
              'https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2026_0_0.zip',
          },
        ],
        cache_root:
          'C:\\Users\\test\\AppData\\Local\\Boghma\\C4DPluginCompiler',
      },
    }),
    listSdkVersions: vi.fn().mockResolvedValue({
      status: 'ok',
      data: [
        {
          version: '2024.4',
          label: 'Cinema 4D 2024.4',
          configured: false,
          sdk_root: null,
          sdk_zip: null,
          download_url:
            'https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2024_4_0.zip',
          status: 'auto download',
        },
        {
          version: '2025',
          label: 'Cinema 4D 2025',
          configured: false,
          sdk_root: null,
          sdk_zip: 'C:\\Program Files\\Maxon Cinema 4D 2025\\sdk.zip',
          download_url:
            'https://developers.maxon.net/downloads/Cinema_4D_CPP_SDK_2025_0_1.zip',
          status: 'configured archive',
        },
      ],
    }),
    loadSdkSources: vi.fn().mockResolvedValue({
      status: 'ok',
      data: { sdk_root: 'C:\\Users\\test\\Documents\\Maxon_SDK' },
    }),
    saveSdkRootConfig: vi.fn().mockResolvedValue({
      status: 'ok',
      data: { sdk_root: 'C:\\Users\\test\\Documents\\Maxon_SDK' },
    }),
    autoConfigureSdkSources: vi.fn().mockResolvedValue({
      status: 'ok',
      data: {
        sdk_root: 'C:\\Users\\test\\Documents\\Maxon_SDK',
        installed_versions: [],
        versions: [],
      },
    }),
    inspectSdkSetup: vi.fn().mockResolvedValue({
      status: 'ok',
      data: {
        sdk_root: 'C:\\Users\\test\\Documents\\Maxon_SDK',
        installed_versions: [],
        versions: [],
        prepared_versions: [],
        requirements: [],
        summary: 'Ready for Cinema 4D C++ SDK builds',
      },
    }),
    configureRequiredSdks: vi.fn().mockResolvedValue({
      status: 'ok',
      data: {
        sdk_root: 'C:\\Users\\test\\Documents\\Maxon_SDK',
        installed_versions: [],
        versions: [],
        prepared_versions: [],
        requirements: [],
        summary: 'Ready for Cinema 4D C++ SDK builds',
      },
    }),
    saveSdkSource: vi.fn().mockResolvedValue({
      status: 'ok',
      data: {
        version: '2024.4',
        label: 'Cinema 4D 2024.4',
        configured: true,
        sdk_root: 'C:\\SDK\\2024.4',
        sdk_zip: null,
        download_url: null,
        status: 'configured root',
      },
    }),
    removeSdkSource: vi.fn().mockResolvedValue({ status: 'ok', data: [] }),
    resolveSdkVersions: vi.fn().mockResolvedValue({ status: 'ok', data: [] }),
    startBuild: vi
      .fn()
      .mockResolvedValue({ status: 'ok', data: { id: 'test-job' } }),
    cancelBuild: vi.fn().mockResolvedValue({ status: 'ok', data: true }),
    listArtifacts: vi.fn().mockResolvedValue({ status: 'ok', data: [] }),
    openArtifactFolder: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
    saveBuildLog: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
  },
  unwrapResult: vi.fn((result: { status: string; data?: unknown }) => {
    if (result.status === 'ok') return result.data
    throw result
  }),
}))
