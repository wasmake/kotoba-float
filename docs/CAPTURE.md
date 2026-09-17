# Audio capture and permissions

## Implemented vertical slice

Physical inputs and virtual input devices are enumerated and captured natively through CPAL. Audio stays in the Rust process, feeds an RMS meter and local energy VAD, keeps pre-roll, and finalizes bounded segments after trailing silence or the maximum duration. Raw audio is not written to disk or played back.

Use **Test source locally** to verify capture without inference. A finalized utterance is reported locally; API requests occur only after connecting, disabling demo mode, and pressing Start.

## Windows

- Input and virtual devices: implemented; normal Windows microphone privacy permission applies.
- Entire output: implemented with built-in WASAPI loopback against the default render endpoint. It captures the whole system mix and needs no VB-CABLE.
- Process-only output: Microsoft documents `AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS` from Windows 10 build 20348. It is distinct from selecting a window. The adapter is disabled in this build.
- For Discord-only isolation, use a virtual device or the future process selector; system loopback includes every audible application.
- Official references: [Loopback Recording](https://learn.microsoft.com/en-us/windows/win32/coreaudio/loopback-recording), [Application Loopback sample](https://learn.microsoft.com/en-us/samples/microsoft/windows-classic-samples/applicationloopbackaudio-sample/).

## macOS

- Input and virtual devices: implemented; grant Microphone permission when prompted.
- System audio: implemented with ScreenCaptureKit on macOS 13+ and requires Screen & System Audio Recording consent. It captures the complete system mix without BlackHole.
- Application-only filtering is not yet exposed in the selector. Use system capture or an existing virtual input when isolation is required.
- Official references: [ScreenCaptureKit](https://developer.apple.com/documentation/screencapturekit), [Meet ScreenCaptureKit](https://developer.apple.com/videos/play/wwdc2022/10156/).

Device refresh is explicit. A disconnected device ends the stream with a visible error. Capture never starts without a button press.
