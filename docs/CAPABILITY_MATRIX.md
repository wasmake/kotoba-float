# Capability matrix

Status as of 2026-09-17. “Implemented” means source-complete; packaged runtime verification remains pending on target hardware.

| Feature | Windows | macOS | Linux | Status |
|---|---|---|---|---|
| Physical/virtual input enumeration, level, VAD | Implemented, untested | Implemented, untested | Implemented, untested | CPAL backend |
| System output | Implemented, untested | Implemented, untested | Monitor input required | Built-in WASAPI / ScreenCaptureKit; Linux uses audio-server monitors |
| Application-only audio | Adapter blocked; OS requires build 20348+ | Adapter blocked; ScreenCaptureKit 13+ | Adapter blocked | Not window selection |
| Transparent always-on-top overlay | Source-complete, untested | Source-complete, untested | Source-complete, untested | Tauri window |
| Click-through lock | Source-complete, untested | Source-complete, untested | Source-complete, untested | Recover with Ctrl/Cmd+Shift+U |
| Global show/hide | Source-complete, untested | Source-complete, untested | Source-complete, untested | Ctrl/Cmd+Shift+S |
| ChatGPT OAuth identity | Unsupported | Unsupported | Unsupported | No eligible public registration |
| API-key transcription | Implemented, untested | Implemented, untested | Implemented, untested | Separately billed API; OS credential store |
| API-key translation | Implemented, untested | Implemented, untested | Implemented, untested | Separately billed API; strict JSON validation |
| Scripted demo | Implemented | Implemented | Implemented | Clearly labeled; no network/audio |

Exclusive-fullscreen overlays are not guaranteed. Borderless/windowed fullscreen is the expected mode. Multi-monitor placement uses native window movement, but position restoration and tray recovery remain untested in this build.

## Latency measurements

Automated demo staging measured by design: Japanese update `0 ms` after its scripted finalization event; romaji/translation `220 ms` later. These are **demo scheduler measurements, not OpenAI latency**. API mode records transcription and translation latency in each subtitle event, but no representative target-hardware measurements have been collected yet. Local VAD finalization is configured to 550 ms trailing silence by default.
