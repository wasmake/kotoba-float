import type { Subtitle } from '../types'
export class SubtitleScheduler {
  private session = ''; private latest = -1
  reset(sessionId: string) { this.session = sessionId; this.latest = -1 }
  accept(next: Subtitle, current?: Subtitle): Subtitle | undefined {
    if (next.sessionId !== this.session || next.segmentId < this.latest) return current
    if (current?.segmentId === next.segmentId) { this.latest = next.segmentId; return { ...current, ...next } }
    this.latest = next.segmentId; return next
  }
}
export function deduplicateBoundary(previous: string, next: string): string { const max = Math.min(24, previous.length, next.length); for (let n = max; n >= 2; n--) if (previous.slice(-n) === next.slice(0, n)) return next.slice(n); return next }
export function validateTextResponse(raw: string): { romaji: string; translation: string } { const parsed: unknown = JSON.parse(raw); if (!parsed || typeof parsed !== 'object') throw new Error('Response must be an object'); const value = parsed as Record<string, unknown>; if (typeof value.romaji !== 'string' || typeof value.translation !== 'string') throw new Error('Missing romaji or translation'); if (!value.romaji.trim() || !value.translation.trim()) throw new Error('Empty response field'); return { romaji: value.romaji, translation: value.translation } }
