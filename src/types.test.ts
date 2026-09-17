import { describe, expect, it } from 'vitest'
import { CALL_CAPTION_PRESET, DEFAULT_SETTINGS, normalizeSettings } from './types'

describe('call caption defaults', () => {
  it('keeps segmentation short and uses the fast low-cost models', () => {
    expect(DEFAULT_SETTINGS).toMatchObject(CALL_CAPTION_PRESET)
    expect(CALL_CAPTION_PRESET.trailingSilenceMs).toBeLessThanOrEqual(250)
    expect(CALL_CAPTION_PRESET.maxSegmentMs).toBeLessThanOrEqual(2200)
    expect(CALL_CAPTION_PRESET.transcriptionModel).toBe('gpt-4o-mini-transcribe')
    expect(CALL_CAPTION_PRESET.textModel).toBe('gpt-4.1-nano')
  })

  it('migrates the original high-latency defaults without overwriting custom tuning', () => {
    const legacy = { ...DEFAULT_SETTINGS, textModel: 'gpt-4o-mini', trailingSilenceMs: 550, maxSegmentMs: 6000, preRollMs: 240 }
    expect(normalizeSettings(legacy)).toMatchObject(CALL_CAPTION_PRESET)
    expect(normalizeSettings({ ...legacy, trailingSilenceMs: 400 }).trailingSilenceMs).toBe(400)
  })
})
