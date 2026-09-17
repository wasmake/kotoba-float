# Kotoba Float

Live Japanese subtitles in a transparent desktop overlay.

Kotoba Float listens to a selected microphone, virtual input, or supported system-output source, detects utterances locally, transcribes Japanese through the OpenAI API, and displays three synchronized lines:

1. Original Japanese
2. Hepburn romaji
3. A natural translation—Spanish by default

It is built with [Tauri 2](https://v2.tauri.app/), Rust, React, and TypeScript.

> [!IMPORTANT]
> Kotoba Float uses an **OpenAI Platform API key**. API usage is billed separately from ChatGPT subscriptions. A ChatGPT Plus, Pro, Business, or Enterprise subscription does not provide API usage for this application. See [Authentication](docs/AUTHENTICATION.md).

## Project status

This repository contains a source-complete desktop application, but there are currently no signed release binaries. Build and runtime verification must be performed on Windows and macOS hardware before production distribution.

| Capability | Windows | macOS | Linux |
|---|---|---|---|
| Microphones and physical inputs | Implemented | Implemented | Implemented |
| Existing virtual inputs | Implemented | Implemented | Implemented |
| Built-in system output | WASAPI loopback | ScreenCaptureKit, macOS 13+ | Use a PulseAudio/PipeWire monitor input |
| Application-only audio | Not yet exposed | Not yet exposed | Not yet exposed |
| Transparent/click-through overlay | Implemented | Implemented | Implemented |
| Offline demo | Implemented | Implemented | Implemented |
| OpenAI API transcription/translation | Implemented | Implemented | Implemented |
| ChatGPT subscription OAuth inference | Unsupported by OpenAI | Unsupported by OpenAI | Unsupported by OpenAI |

See the detailed [capability matrix](docs/CAPABILITY_MATRIX.md) for verified, untested, and unsupported behavior.

## How it works

```text
Selected audio source
  → local level meter
  → local voice activity detection
  → bounded utterance segment
  → OpenAI Japanese transcription
  → one romaji + translation request
  → ordered three-line overlay
```

Silence does not trigger inference. The call-caption preset uses 120 ms pre-roll, 250 ms trailing silence, and 2.2-second maximum segments instead of repeatedly uploading an ever-growing recording. Transcription and translation run on separate bounded workers so translation cannot hold up the next Japanese caption.

## Requirements

### To run from source

- Node.js 20 or newer
- Rust 1.77.2 or newer
- The [Tauri 2 platform prerequisites](https://v2.tauri.app/start/prerequisites/)
- Windows 10/11, macOS 13+, or a modern Linux desktop with WebKitGTK
- An OpenAI Platform account with API billing enabled for live processing

Demo mode does not need an OpenAI account or network connection.

### OpenAI API key

1. Sign in to the [OpenAI Platform](https://platform.openai.com/).
2. Configure API billing and project limits as appropriate.
3. Create an API key from the [API keys page](https://platform.openai.com/api-keys).
4. Do not place the key in source files, environment files, or ordinary application settings.

Kotoba Float accepts the key through its connection UI and stores it in Windows Credential Manager, macOS Keychain, or the Linux Secret Service. The key is never written to the settings JSON or browser storage.

## Download a release

Tagged builds are published on the [GitHub Releases page](https://github.com/wasmake/kotoba-float/releases). Download the asset for your platform:

- **Windows:** `.exe` NSIS installer or `.msi`
- **macOS:** architecture-specific `.dmg` for Apple Silicon or Intel
- **Linux:** `.AppImage` or Debian `.deb`

The initial release artifacts are unsigned. Until signing certificates are configured:

- Windows SmartScreen may require **More info → Run anyway**.
- On macOS, Control-click the app and choose **Open**, then confirm. Do not bypass organization-managed security policy.
- For AppImage, make it executable with `chmod +x Kotoba*.AppImage`, then run it.

Checksums and code signing should be added before treating the project as a trusted production distribution.

If the desktop integration cannot initialize, Kotoba Float now keeps the main window open and records the failure in `%LOCALAPPDATA%\\app.kotobafloat.desktop\\logs` on Windows, `~/Library/Logs/app.kotobafloat.desktop` on macOS, or the application data directory on Linux.

## Installation from source

```bash
git clone https://github.com/wasmake/kotoba-float.git
cd kotoba-float
npm install
npm run desktop:dev
```

`npm run dev` starts a browser-only UI preview. Native audio capture, credential storage, global shortcuts, the tray, and the real overlay window require `npm run desktop:dev`.

## First-time setup

### 1. Connect OpenAI

1. Open **Connections → OpenAI API**.
2. Review the separate-billing warning.
3. Paste your API key.
4. Select or enter the transcription and text models.
5. Click **Save securely & test capabilities**.

Transcription and translation are tested independently. Both must pass before live listening can start. Disconnecting clears the stored credential and cancels active processing.

The defaults are:

- Transcription: `gpt-4o-mini-transcribe`
- Romaji and translation: `gpt-4o-mini`

Model availability depends on the user's OpenAI Platform project and may change. Use only models currently available to that project.

### 2. Select an audio source

Choose one of:

- **Input:** microphone or physical audio input
- **Virtual:** an already-installed virtual routing device
- **System:** all sound sent to the default output device

For Discord while wearing headphones, select the built-in system-output source:

- **Windows:** `Windows system output (built-in WASAPI loopback)`
- **macOS:** `macOS system audio (built-in ScreenCaptureKit)`

System capture includes every audible application. It does **not** isolate Discord. Selecting a Discord window is not application-audio isolation.

Press **Test source locally** and confirm that the level meter responds. The local test performs VAD but sends no audio to OpenAI.

### 3. Grant permissions

#### Windows

- Permit microphone access when using physical or virtual inputs.
- WASAPI system loopback normally does not require microphone permission.
- Ensure the intended headphones/speakers are the default Windows output device.

#### macOS

- Permit **Microphone** access for physical or virtual inputs.
- Permit **Screen & System Audio Recording** for built-in system capture.
- If permission was denied, enable Kotoba Float in System Settings → Privacy & Security and restart it.

See [Audio capture and permissions](docs/CAPTURE.md) for implementation details.

#### Linux

- Permit microphone access through the desktop portal or audio server when prompted.
- For desktop output, select the corresponding PulseAudio/PipeWire **monitor** source if it is exposed as an input.
- API credentials require a running Secret Service implementation such as GNOME Keyring or KWallet-compatible secret storage.

### 4. Configure subtitles

- Choose the target language; Spanish is the default.
- Adjust Japanese, romaji, and translation font sizes.
- Toggle individual subtitle lines.
- Adjust the translucent backing and subtitle width.
- Tune pre-roll, trailing silence, and maximum segment duration if needed.
- For calls and streamed talk, select **Use call-caption preset**. It uses `gpt-4o-mini-transcribe` plus the low-cost `gpt-4.1-nano` text model.

### 5. Position the overlay

1. Click **Preview overlay**.
2. Click **Unlock overlay**, then drag the native window using the bar above the captions.
3. Resize the frameless window at its edges while it is unlocked.
4. Click **Lock & click through** when positioned.

Locked mode allows clicks to reach the application underneath. Window size and position are restored between launches.

### 6. Start listening

Disable demo mode and click **Start listening**. The active status indicator means captured utterance segments may be sent to OpenAI.

Press **Pause** or **Stop** to terminate capture and invalidate pending work. Old results are discarded after stopping, disconnecting, or changing the source.

## Demo mode

Demo mode shows clearly labeled scripted Japanese, romaji, and translation without capturing or uploading audio. It is useful for testing overlay placement and appearance.

Demo latency values describe only the scripted scheduler and are not OpenAI performance measurements.

## Global shortcuts and tray

| Action | Shortcut |
|---|---|
| Show/hide overlay | `Ctrl/Cmd + Shift + S` |
| Pause and cancel pending work | `Ctrl/Cmd + Shift + P` |
| Show and unlock overlay | `Ctrl/Cmd + Shift + U` |

If another application reserves a shortcut, use the tray menu to open settings, recover the overlay, stop listening, or quit.

Always-on-top behavior is not guaranteed over every exclusive-fullscreen game. Borderless or windowed fullscreen is recommended.

## Privacy and security

- Listening starts only after explicit user action.
- Raw audio is held in memory and is not saved by default.
- Silence is processed locally and does not trigger API requests.
- API credentials stay in the OS credential store.
- Credentials and complete conversations are not logged.
- Subtitle history remains in memory unless persistence is explicitly enabled.
- A bounded queue applies backpressure; excess stale segments are discarded instead of building unlimited latency.
- Disconnect, stop, source changes, and new sessions invalidate older work.

Audio segments and recent transcript context are sent to OpenAI during live API processing. Review OpenAI's policies before processing sensitive conversations, and obtain consent where required by law.

## Development

### Commands

```bash
# Browser UI preview
npm run dev

# Full Tauri application
npm run desktop:dev

# TypeScript tests
npm run test

# Frontend production build
npm run build

# Native installer/package
npm run desktop:build
```

Packaged artifacts are written beneath `src-tauri/target/release/bundle/`. Pushing a version tag such as `v0.1.0` runs `.github/workflows/release.yml` and publishes Windows, Apple Silicon and Intel macOS, and Linux installers as a GitHub prerelease.

### Architecture

The React renderer is limited to presentation and user interaction. Native and sensitive work remains in Rust.

```text
src/
  components/       Settings and overlay presentation
  lib/backend.ts    Typed Tauri IPC boundary
  lib/scheduler.ts  Ordered staged subtitle updates

src-tauri/src/
  auth.rs           OS credential storage and capability state
  audio/            CPAL inputs, WASAPI, and ScreenCaptureKit
  vad.rs            Local segmentation and pre-roll
  provider.rs       OpenAI transcription and translation requests
  transcription.rs Provider boundary
  text.rs           Strict response validation
  scheduler.rs      Subtitle metadata and overlap deduplication
  settings.rs       Non-secret persisted configuration
  diagnostics.rs    Sanitized diagnostics and latency types
```

The inference worker is separate from the capture callback. It uses bounded channels, session generations, ordered segment IDs, and stale-result checks so networking cannot block audio capture.

### Testing

Focused tests cover:

- Ordered and staged subtitle updates
- Stale-session cancellation
- Segment-boundary deduplication
- Strict translation response validation
- Credential lifecycle behavior
- Silence suppression and VAD finalization
- WAV encoding

Run Rust tests on a target platform with all native Tauri prerequisites installed:

```bash
cd src-tauri
cargo test
```

## Troubleshooting

### The input meter is silent

- Refresh the source list.
- Verify the source is not muted and is still connected.
- Confirm OS privacy permissions.
- For system capture, confirm audio is playing through the default output device.
- Stop and restart capture after changing devices.

### OpenAI capability testing fails

- Confirm the key belongs to an OpenAI Platform project, not a ChatGPT session.
- Confirm API billing and project limits are configured.
- Verify both configured model names are available to the project.
- Check network access to `api.openai.com`.
- Disconnect and reconnect if the key was rotated.

### Quota or rate limit reached

Kotoba Float pauses processing rather than switching providers or billing methods. Review the OpenAI Platform usage and limits pages, then manually resume when access is available.

### Overlay cannot be clicked

The overlay is probably locked. Press `Ctrl/Cmd + Shift + U` or choose **Show and unlock overlay** from the tray menu.

## Known limitations

- Application-only Discord capture is not yet exposed; built-in capture records the full system mix.
- Native adapters and packaging still require runtime verification on each supported OS.
- Exclusive-fullscreen applications may cover the overlay.
- Remaining API allowance is unavailable unless OpenAI exposes it through a supported response or integration.
- Overlapping speakers are transcribed as mixed audio; speaker identities are not inferred.

## Documentation

- [Authentication and billing boundary](docs/AUTHENTICATION.md)
- [Audio capture and permissions](docs/CAPTURE.md)
- [Capability matrix and measured status](docs/CAPABILITY_MATRIX.md)

## License

Licensed under the MIT License. See [LICENSE](LICENSE).
