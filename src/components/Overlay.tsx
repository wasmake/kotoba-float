import { Lock, Unlock } from 'lucide-react'
import type { Appearance, Subtitle } from '../types'
import { backend } from '../lib/backend'
export function Overlay({ subtitle, appearance, locked, onLock }: { subtitle?: Subtitle; appearance: Appearance; locked: boolean; onLock(v: boolean): void }) {
  const outline = `${appearance.outline}px ${appearance.outline}px 0 #000, -${appearance.outline}px -${appearance.outline}px 0 #000`
  return <main className={`overlay ${locked ? 'locked' : 'editing'}`} style={{ width: Math.min(appearance.width, window.innerWidth - 24), textAlign: appearance.align, opacity: appearance.opacity }}>
    {!locked && <div className="overlay-tools" data-tauri-drag-region><span>Drag to position · resize at edges</span><button onClick={() => { backend.setLocked(true); onLock(true) }}><Lock size={14}/> Lock & click through</button></div>}
    <section className="subtitle-card" style={{ background: `rgba(9,14,22,${appearance.backingOpacity})`, textShadow: `${outline}, 0 3px ${appearance.shadow}px #000` }}>
      {!subtitle && <div className="waiting">字幕を待っています…</div>}
      {appearance.showJapanese && subtitle?.japanese && <div className="japanese" style={{ fontSize: appearance.japaneseSize, color: appearance.japaneseColor }}>{subtitle.japanese}</div>}
      {appearance.showRomaji && subtitle?.romaji && <div className="romaji" style={{ fontSize: appearance.romajiSize, color: appearance.romajiColor }}>{subtitle.romaji}</div>}
      {appearance.showTranslation && subtitle?.translation && <div className="translation" style={{ fontSize: appearance.translationSize, color: appearance.translationColor }}>{subtitle.translation}</div>}
    </section><div className="overlay-status"><i /> {locked ? 'Overlay locked' : <><Unlock size={11}/> Edit mode</>}</div>
  </main>
}
