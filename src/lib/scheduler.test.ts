import { describe, expect, it } from 'vitest'
import { deduplicateBoundary, SubtitleScheduler, validateTextResponse } from './scheduler'
import type { Subtitle } from '../types'
const sub = (sessionId: string, segmentId: number, japanese='日本語'): Subtitle => ({ sessionId, segmentId, japanese, startMs: 0, endMs: 100 })
describe('subtitle scheduling', () => {
  it('merges staged updates without replacing the wrong segment', () => { const q=new SubtitleScheduler();q.reset('a');const first=q.accept(sub('a',2))!;expect(q.accept({...first,romaji:'nihongo'},first)?.romaji).toBe('nihongo');expect(q.accept(sub('a',1),first)).toEqual(first) })
  it('discards work from cancelled sessions', () => { const q=new SubtitleScheduler();q.reset('new');expect(q.accept(sub('old',4))).toBeUndefined() })
  it('deduplicates overlap only at boundaries', () => { expect(deduplicateBoundary('今日はいい天気','いい天気ですね')).toBe('ですね');expect(deduplicateBoundary('はい','そうです')).toBe('そうです') })
  it('strictly validates text responses', () => { expect(validateTextResponse('{"romaji":"hai","translation":"Sí"}')).toEqual({romaji:'hai',translation:'Sí'});expect(()=>validateTextResponse('{"translation":"Sí"}')).toThrow();expect(()=>validateTextResponse('oops')).toThrow() })
})
