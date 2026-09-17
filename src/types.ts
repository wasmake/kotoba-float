export type CaptureKind = 'input' | 'virtual' | 'system' | 'application'
export type RunState = 'idle' | 'listening' | 'paused' | 'blocked' | 'error'

export interface AudioSource { id: string; name: string; kind: CaptureKind; available: boolean; detail: string }
export interface Capabilities { identity: boolean; transcription: boolean; translation: boolean; reason: string; checkedAt: string }
export interface Subtitle { sessionId: string; segmentId: number; startMs: number; endMs: number; japanese: string; romaji?: string; translation?: string; transcriptLatencyMs?: number; translationLatencyMs?: number }
export interface Appearance { japaneseSize: number; romajiSize: number; translationSize: number; japaneseColor: string; romajiColor: string; translationColor: string; opacity: number; backingOpacity: number; outline: number; shadow: number; width: number; align: 'left' | 'center' | 'right'; showJapanese: boolean; showRomaji: boolean; showTranslation: boolean }
export interface Settings { sourceId: string; targetLanguage: string; demoMode: boolean; transcriptionModel: string; textModel: string; trailingSilenceMs: number; maxSegmentMs: number; preRollMs: number; subtitleDurationMs: number; persistHistory: boolean; appearance: Appearance }

export const DEFAULT_SETTINGS: Settings = {
  sourceId: '', targetLanguage: 'Spanish', demoMode: true, transcriptionModel: 'gpt-4o-mini-transcribe', textModel: 'gpt-4o-mini', trailingSilenceMs: 550,
  maxSegmentMs: 6000, preRollMs: 240, subtitleDurationMs: 7000, persistHistory: false,
  appearance: { japaneseSize: 38, romajiSize: 19, translationSize: 25, japaneseColor: '#ffffff', romajiColor: '#9ee7f5', translationColor: '#fff1dc', opacity: 1, backingOpacity: .42, outline: 2, shadow: 8, width: 820, align: 'center', showJapanese: true, showRomaji: true, showTranslation: true },
}
