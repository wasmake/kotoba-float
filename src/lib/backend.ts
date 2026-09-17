import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { AudioSource, Capabilities, Settings, Subtitle } from '../types'

const inTauri = () => '__TAURI_INTERNALS__' in window
export const backend = {
  isDesktop: inTauri,
  capabilities: async (): Promise<Capabilities> => inTauri() ? invoke('get_capabilities') : ({ identity: false, transcription: false, translation: false, reason: 'Browser preview: native services are available only in the packaged desktop app.', checkedAt: new Date().toISOString() }),
  connectApiKey: (apiKey: string, transcriptionModel: string, textModel: string) => invoke<Capabilities>('configure_api_key', { apiKey, transcriptionModel, textModel }),
  testApiKey: (transcriptionModel: string, textModel: string) => invoke<Capabilities>('test_api_key', { transcriptionModel, textModel }),
  disconnectApi: () => invoke<Capabilities>('disconnect_api'),
  sources: async (): Promise<AudioSource[]> => inTauri() ? invoke('list_audio_sources') : [{ id: 'preview', name: 'Browser preview (no capture)', kind: 'input', available: false, detail: 'Run npm run desktop:dev' }],
  start: (settings: Settings) => invoke<void>('start_session', { settings }),
  testCapture: (settings: Settings) => invoke<void>('start_session', { settings: { ...settings, localCaptureOnly: true } }),
  pause: () => invoke<void>('pause_session'), stop: () => invoke<void>('stop_session'),
  demo: (language: string) => invoke<void>('start_demo', { language }),
  setLocked: (locked: boolean) => invoke<void>('set_overlay_locked', { locked }),
  showOverlay: () => invoke<void>('show_overlay'),
  saveSettings: (settings: Settings) => invoke<void>('save_settings', { settings }),
  loadSettings: async (): Promise<Settings | null> => inTauri() ? invoke('load_settings') : null,
  onSubtitle: (cb: (s: Subtitle) => void): Promise<UnlistenFn> => listen('subtitle-update', e => cb(e.payload as Subtitle)),
  onLevel: (cb: (n: number) => void): Promise<UnlistenFn> => listen('audio-level', e => cb(e.payload as number)),
  onStatus: (cb: (s: string) => void): Promise<UnlistenFn> => listen('pipeline-status', e => cb(e.payload as string)),
  onLock: (cb: (locked: boolean) => void): Promise<UnlistenFn> => listen('overlay-lock', e => cb(e.payload as boolean)),
}
