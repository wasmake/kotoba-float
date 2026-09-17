export type CaptureKind = 'input' | 'virtual' | 'system' | 'application'
export type RunState = 'idle' | 'listening' | 'paused' | 'blocked' | 'error'

export interface AudioSource { id: string; name: string; kind: CaptureKind; available: boolean; detail: string }
export interface Capabilities { identity: boolean; transcription: boolean; translation: boolean; reason: string; checkedAt: string }
export interface Subtitle { sessionId: string; segmentId: number; startMs: number; endMs: number; japanese: string; romaji?: string; translation?: string; transcriptLatencyMs?: number; translationLatencyMs?: number }
export interface Appearance { japaneseSize: number; romajiSize: number; translationSize: number; japaneseColor: string; romajiColor: string; translationColor: string; opacity: number; backingOpacity: number; outline: number; shadow: number; width: number; align: 'left' | 'center' | 'right'; showJapanese: boolean; showRomaji: boolean; showTranslation: boolean }
export interface Settings { sourceId: string; targetLanguage: string; demoMode: boolean; transcriptionModel: string; textModel: string; trailingSilenceMs: number; maxSegmentMs: number; preRollMs: number; subtitleDurationMs: number; persistHistory: boolean; appearance: Appearance }

export const DEFAULT_SETTINGS: Settings = {
  sourceId: '', targetLanguage: 'Spanish', demoMode: true, transcriptionModel: 'gpt-4o-mini-transcribe', textModel: 'gpt-4.1-nano', trailingSilenceMs: 250,
  maxSegmentMs: 2200, preRollMs: 120, subtitleDurationMs: 7000, persistHistory: false,
  appearance: { japaneseSize: 38, romajiSize: 19, translationSize: 25, japaneseColor: '#ffffff', romajiColor: '#9ee7f5', translationColor: '#fff1dc', opacity: 1, backingOpacity: .18, outline: 2, shadow: 8, width: 820, align: 'center', showJapanese: true, showRomaji: true, showTranslation: true },
}

export const CALL_CAPTION_PRESET: Pick<Settings, 'transcriptionModel' | 'textModel' | 'trailingSilenceMs' | 'maxSegmentMs' | 'preRollMs'> = {
  transcriptionModel: 'gpt-4o-mini-transcribe', textModel: 'gpt-4.1-nano', trailingSilenceMs: 250, maxSegmentMs: 2200, preRollMs: 120,
}

export function normalizeSettings(saved: Settings): Settings {
  const merged = { ...DEFAULT_SETTINGS, ...saved, appearance: { ...DEFAULT_SETTINGS.appearance, ...saved.appearance } }
  const legacyDefaults = merged.trailingSilenceMs === 550 && merged.maxSegmentMs === 6000 && merged.preRollMs === 240
  return legacyDefaults ? { ...merged, ...CALL_CAPTION_PRESET } : merged
}
