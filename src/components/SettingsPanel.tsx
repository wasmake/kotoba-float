import { useState } from 'react'
import { AlertTriangle, Check, CircleOff, Headphones, RefreshCw, ShieldCheck } from 'lucide-react'
import type { AudioSource, Capabilities, Settings } from '../types'

export function SettingsPanel({ settings, setSettings, sources, capabilities, level, refresh, testCapture, connect, testConnection, disconnect }: { settings: Settings; setSettings(s: Settings): void; sources: AudioSource[]; capabilities?: Capabilities; level: number; refresh(): void; testCapture(): void; connect(key:string):Promise<void>; testConnection():Promise<void>; disconnect():Promise<void> }) {
  const [apiKey,setApiKey]=useState(''),[connectionBusy,setConnectionBusy]=useState(false),[connectionError,setConnectionError]=useState('')
  const patch = (p: Partial<Settings>) => setSettings({ ...settings, ...p })
  const appearance = (p: Partial<Settings['appearance']>) => patch({ appearance: { ...settings.appearance, ...p } })
  return <div className="settings-grid">
    <section className="card connection"><div className="card-title"><div><span className="eyebrow">CONNECTIONS</span><h2>OpenAI API</h2></div>{capabilities?.identity?<Check className="success"/>:<CircleOff className="danger"/>}</div>
      <div className="billing-warning"><AlertTriangle size={18}/><div><b>Separately billed API access</b><p>This does not use your ChatGPT subscription. Requests use your OpenAI Platform account and its API billing and limits.</p></div></div>
      {!capabilities?.identity&&<><label>API key<input className="text-input" type="password" autoComplete="off" value={apiKey} placeholder="sk-…" onChange={e=>setApiKey(e.target.value)}/></label><button disabled={connectionBusy||!apiKey} className="primary active" onClick={async()=>{setConnectionBusy(true);setConnectionError('');try{await connect(apiKey);setApiKey('')}catch(e){setConnectionError(String(e))}finally{setConnectionBusy(false)}}}>Save securely & test capabilities</button></>}
      {capabilities?.identity&&<div className="connection-actions"><button disabled={connectionBusy} onClick={async()=>{setConnectionBusy(true);setConnectionError('');try{await testConnection()}catch(e){setConnectionError(String(e))}finally{setConnectionBusy(false)}}}>Test connection</button><button disabled={connectionBusy} onClick={async()=>{try{await disconnect()}catch(e){setConnectionError(String(e))}}}>Disconnect</button></div>}
      {connectionError&&<div className="inline-error">{connectionError}</div>}
      <div className="capabilities"><Capability label="API credential" ok={capabilities?.identity}/><Capability label="Audio transcription" ok={capabilities?.transcription}/><Capability label="Text translation" ok={capabilities?.translation}/></div>
      <small>The key is stored in Windows Credential Manager or macOS Keychain, never settings or logs. Remaining API quota is shown only if OpenAI exposes it; otherwise it is unavailable.</small>
    </section>
    <section className="card"><div className="card-title"><div><span className="eyebrow">AUDIO</span><h2>Capture source</h2></div><button className="icon" onClick={refresh} aria-label="Refresh"><RefreshCw size={17}/></button></div>
      <label>Source<select value={settings.sourceId} onChange={e => patch({ sourceId: e.target.value })}><option value="">Choose a source…</option>{sources.map(s => <option key={s.id} value={s.id} disabled={!s.available}>{s.name} — {s.kind}</option>)}</select></label>
      <div className="meter"><span style={{ width: `${Math.max(2, level * 100)}%` }}/></div>
      <button className="secondary" disabled={!settings.sourceId} onClick={testCapture}>Test source locally</button>
      <div className="source-note"><Headphones size={17}/><p>For Discord, choose <b>system output</b> (all desktop audio) or a virtual device. An application source is process-isolated only when explicitly labeled.</p></div>
      {sources.find(s => s.id === settings.sourceId)?.detail && <small>{sources.find(s => s.id === settings.sourceId)?.detail}</small>}
    </section>
    <section className="card"><span className="eyebrow">LANGUAGE & MODE</span><h2>Output</h2>
      <label>Translate Japanese into<select value={settings.targetLanguage} onChange={e => patch({ targetLanguage: e.target.value })}>{['Spanish','English','French','German','Portuguese','Italian'].map(x => <option key={x}>{x}</option>)}</select></label>
      <label className="toggle"><input type="checkbox" checked={settings.demoMode} onChange={e => patch({ demoMode: e.target.checked })}/><span/><div><b>Demo mode</b><small>Clearly labeled scripted subtitles; no audio is sent.</small></div></label>
      <label>Transcription model<input className="text-input" value={settings.transcriptionModel} onChange={e=>patch({transcriptionModel:e.target.value})}/></label>
      <label>Romaji & translation model<input className="text-input" value={settings.textModel} onChange={e=>patch({textModel:e.target.value})}/></label>
    </section>
    <section className="card"><span className="eyebrow">SEGMENTATION</span><h2>Latency tuning</h2>
      <Range label="Trailing silence" value={settings.trailingSilenceMs} min={400} max={700} unit="ms" onChange={v => patch({ trailingSilenceMs: v })}/>
      <Range label="Maximum segment" value={settings.maxSegmentMs} min={3000} max={10000} step={500} unit="ms" onChange={v => patch({ maxSegmentMs: v })}/>
      <Range label="Pre-roll" value={settings.preRollMs} min={100} max={500} unit="ms" onChange={v => patch({ preRollMs: v })}/>
    </section>
    <section className="card wide"><span className="eyebrow">OVERLAY</span><h2>Appearance</h2><div className="appearance-grid">
      <Range label="Japanese" value={settings.appearance.japaneseSize} min={22} max={64} unit="px" onChange={v => appearance({ japaneseSize: v })}/><Range label="Romaji" value={settings.appearance.romajiSize} min={12} max={36} unit="px" onChange={v => appearance({ romajiSize: v })}/><Range label="Translation" value={settings.appearance.translationSize} min={16} max={48} unit="px" onChange={v => appearance({ translationSize: v })}/><Range label="Backing" value={Math.round(settings.appearance.backingOpacity * 100)} min={0} max={90} unit="%" onChange={v => appearance({ backingOpacity: v / 100 })}/></div>
      <div className="line-toggles">{(['Japanese','Romaji','Translation'] as const).map(x => { const k = `show${x}` as keyof Settings['appearance']; return <label key={x}><input type="checkbox" checked={settings.appearance[k] as boolean} onChange={e => appearance({ [k]: e.target.checked })}/>{x}</label> })}</div>
    </section>
    <section className="card wide privacy"><ShieldCheck/><div><b>Privacy by default</b><p>Capture starts only when you press Start. Raw audio is not saved. History remains in memory unless persistence is enabled. Credentials and conversations are never logged.</p></div></section>
  </div>
}
function Capability({ label, ok }: { label: string; ok?: boolean }) { return <div>{ok ? <Check/> : <CircleOff/>}<span>{label}</span><b>{ok ? 'Available' : 'Unsupported'}</b></div> }
function Range({ label, value, min, max, step = 10, unit, onChange }: { label: string; value: number; min: number; max: number; step?: number; unit: string; onChange(v: number): void }) { return <label className="range"><span>{label}<b>{value}{unit}</b></span><input type="range" min={min} max={max} step={step} value={value} onChange={e => onChange(+e.target.value)}/></label> }
