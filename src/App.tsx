import { useEffect, useMemo, useState } from 'react'
import { Eye, LockOpen, Pause, Play, Radio, Square } from 'lucide-react'
import './App.css'
import { backend } from './lib/backend'
import { SubtitleScheduler } from './lib/scheduler'
import { DEFAULT_SETTINGS, normalizeSettings, type AudioSource, type Capabilities, type RunState, type Settings, type Subtitle } from './types'
import { SettingsPanel } from './components/SettingsPanel'
import { Overlay } from './components/Overlay'

export default function App() {
  const overlayWindow = new URLSearchParams(location.search).get('window') === 'overlay'
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS), [sources, setSources] = useState<AudioSource[]>([]), [capabilities, setCapabilities] = useState<Capabilities>()
  const [state, setState] = useState<RunState>('idle'), [status, setStatus] = useState('Ready'), [level, setLevel] = useState(0), [subtitle, setSubtitle] = useState<Subtitle>(), [locked, setLocked] = useState(false)
  const scheduler = useMemo(() => new SubtitleScheduler(), [])
  const refresh = () => backend.sources().then(setSources).catch(e => setStatus(String(e)))
  useEffect(() => {
    backend.loadSettings().then(s => s && setSettings(normalizeSettings(s))); backend.capabilities().then(setCapabilities); refresh(); const cleanups: Array<() => void> = []
    backend.onSubtitle(s => setSubtitle(old => { if (!old || old.sessionId !== s.sessionId) scheduler.reset(s.sessionId); return scheduler.accept(s, old) })).then(fn => cleanups.push(fn)); backend.onLevel(setLevel).then(fn => cleanups.push(fn)); backend.onStatus(setStatus).then(fn => cleanups.push(fn)); backend.onLock(setLocked).then(fn => cleanups.push(fn)); return () => cleanups.forEach(fn => fn())
  }, [scheduler])
  useEffect(() => { const t = setTimeout(() => backend.isDesktop() && backend.saveSettings(settings), 250); return () => clearTimeout(t) }, [settings])
  useEffect(() => { if(state==='listening'&&!settings.demoMode){backend.stop();setState('idle');setStatus('Audio source changed · previous session cancelled')} }, [settings.sourceId])
  if (overlayWindow) return <Overlay subtitle={subtitle} appearance={settings.appearance} locked={locked} onLock={setLocked}/>
  const start = async () => { try { if (settings.demoMode) { await backend.demo(settings.targetLanguage); setState('listening'); setStatus('DEMO · scripted subtitles · no audio sent') } else if (!capabilities?.transcription || !capabilities.translation) { setState('blocked'); setStatus('Live OpenAI processing blocked: subscription inference is unsupported') } else { await backend.start(settings); setState('listening') } } catch (e) { setState('error'); setStatus(String(e)) } }
  const stop = async () => { await backend.stop(); setState('idle'); setLevel(0); setStatus('Stopped · pending work cancelled') }
  return <div className="app-shell"><header><div className="brand"><div className="logo">訳</div><div><b>Kotoba Float</b><span>Japanese live subtitles</span></div></div><div className={`status ${state}`}><i/>{status}</div></header>
    <div className="hero"><div><span className="eyebrow">SETTINGS</span><h1>Listen. Understand.<br/><em>Stay in the moment.</em></h1><p>Local audio capture and a quiet three-line overlay for Japanese conversations.</p></div><div className="hero-actions"><button onClick={() => backend.showOverlay()}><Eye/>Preview overlay</button><button onClick={async () => { await backend.setLocked(false); setLocked(false); await backend.showOverlay() }}><LockOpen/>Unlock overlay</button></div></div>
    <SettingsPanel settings={settings} setSettings={setSettings} sources={sources} capabilities={capabilities} level={level} refresh={refresh} testCapture={async()=>{try{await backend.testCapture(settings);setState('listening');setStatus('Local capture test · audio is not sent')}catch(e){setState('error');setStatus(String(e))}}} connect={async key=>{const c=await backend.connectApiKey(key,settings.transcriptionModel,settings.textModel);setCapabilities(c);setStatus('OpenAI API capabilities checked')}} testConnection={async()=>{const c=await backend.testApiKey(settings.transcriptionModel,settings.textModel);setCapabilities(c);setStatus('OpenAI API capabilities checked')}} disconnect={async()=>{const c=await backend.disconnectApi();setCapabilities(c);setState('idle');setStatus('Disconnected · active processing stopped')}}/>
    <footer><div><Radio size={18}/><div><b>{state === 'listening' ? (settings.demoMode ? 'Demo running' : 'Listening') : 'Not listening'}</b><span>{settings.demoMode ? 'No audio leaves this device in demo mode' : 'Audio is sent to OpenAI only while listening'}</span></div></div><div className="controls">{state === 'listening' && <button onClick={async () => { await backend.pause(); setState('paused') }}><Pause/>Pause</button>}{(state === 'listening' || state === 'paused') && <button onClick={stop}><Square/>Stop</button>}{(state === 'idle' || state === 'blocked' || state === 'error') && <button className="start" onClick={start}><Play/>Start {settings.demoMode ? 'demo' : 'listening'}</button>}</div></footer>
  </div>
}
