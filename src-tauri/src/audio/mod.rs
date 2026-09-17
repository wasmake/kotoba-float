use crate::vad::Segmenter;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::SyncSender,
    Arc, Mutex,
};
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSource {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub available: bool,
    pub detail: String,
}
pub struct AudioSegment {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub start_ms: u64,
    pub end_ms: u64,
}
pub struct CaptureHandle {
    stop: Arc<AtomicBool>,
}
impl CaptureHandle {
    fn new(stop: Arc<AtomicBool>) -> Self {
        Self { stop }
    }
}
impl Drop for CaptureHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst)
    }
}

fn looks_virtual(name: &str) -> bool {
    let n = name.to_lowercase();
    [
        "blackhole",
        "loopback",
        "vb-audio",
        "cable",
        "soundflower",
        "virtual",
    ]
    .iter()
    .any(|x| n.contains(x))
}
pub fn sources() -> Vec<AudioSource> {
    let host = cpal::default_host();
    let mut result = vec![];
    if let Ok(devices) = host.input_devices() {
        for (i, d) in devices.enumerate() {
            if let Ok(name) = d.name() {
                let virtual_device = looks_virtual(&name);
                result.push(AudioSource { id: format!("input:{i}"), name, kind: if virtual_device { "virtual" } else { "input" }.into(), available: true, detail: if virtual_device { "Virtual routing input; route Discord output to this device and monitor it through the device's companion output." } else { "Physical input. This does not capture Discord unless Discord is routed into it." }.into() });
            }
        }
    }
    result.extend(platform::native_sources());
    result
}

pub fn start_capture(
    app: AppHandle,
    source_id: &str,
    trailing: u64,
    max: u64,
    pre: u64,
    output: Option<SyncSender<AudioSegment>>,
) -> Result<CaptureHandle, String> {
    #[cfg(target_os = "windows")]
    if source_id == "wasapi:system" {
        return platform::start_system(app, trailing, max, pre, output);
    }
    #[cfg(target_os = "macos")]
    if source_id == "sck:system" {
        return platform::start_system(app, trailing, max, pre, output);
    }
    if !source_id.starts_with("input:") {
        return Err("Selected native capture source is unavailable on this platform".into());
    }
    start_input(app, source_id, trailing, max, pre, output)
}
fn start_input(
    app: AppHandle,
    source_id: &str,
    trailing: u64,
    max: u64,
    pre: u64,
    output: Option<SyncSender<AudioSegment>>,
) -> Result<CaptureHandle, String> {
    let index: usize = source_id
        .strip_prefix("input:")
        .ok_or("Selected native source adapter is unavailable in this build")?
        .parse()
        .map_err(|_| "Invalid source")?;
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = stop.clone();
    let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("audio-input".into())
        .spawn(move || {
            let result = run_input(app, index, trailing, max, pre, output);
            match result {
                Ok(stream) => {
                    if ready_tx.send(Ok(())).is_err() {
                        return;
                    }
                    while !thread_stop.load(Ordering::SeqCst) {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                    drop(stream);
                }
                Err(error) => {
                    let _ = ready_tx.send(Err(error));
                }
            }
        })
        .map_err(|e| e.to_string())?;
    ready_rx
        .recv()
        .map_err(|_| "Audio input thread stopped during startup".to_string())??;
    Ok(CaptureHandle::new(stop))
}

fn run_input(
    app: AppHandle,
    index: usize,
    trailing: u64,
    max: u64,
    pre: u64,
    output: Option<SyncSender<AudioSegment>>,
) -> Result<cpal::Stream, String> {
    let host = cpal::default_host();
    let device = host
        .input_devices()
        .map_err(|e| e.to_string())?
        .nth(index)
        .ok_or("Audio source disconnected")?;
    let supported = device
        .default_input_config()
        .map_err(|e| format!("Source has no input configuration: {e}"))?;
    let rate = supported.sample_rate().0;
    let segmenter = Arc::new(Mutex::new(Segmenter::new(rate, trailing, max, pre)));
    let config = supported.config();
    let channels = config.channels;
    let stream = match supported.sample_format() {
        cpal::SampleFormat::F32 => {
            let a = app.clone();
            let ae = app.clone();
            let v = segmenter.clone();
            let o = output.clone();
            device.build_input_stream(
                &config,
                move |d: &[f32], _| process(d.iter().copied(), &a, &v, &o, rate, channels),
                move |e| {
                    let _ = ae.emit("pipeline-status", format!("Audio device error: {e}"));
                },
                None,
            )
        }
        cpal::SampleFormat::I16 => {
            let a = app.clone();
            let ae = app.clone();
            let v = segmenter.clone();
            let o = output.clone();
            device.build_input_stream(
                &config,
                move |d: &[i16], _| {
                    process(
                        d.iter().map(|x| *x as f32 / 32768.0),
                        &a,
                        &v,
                        &o,
                        rate,
                        channels,
                    )
                },
                move |e| {
                    let _ = ae.emit("pipeline-status", format!("Audio device error: {e}"));
                },
                None,
            )
        }
        cpal::SampleFormat::U16 => {
            let a = app.clone();
            let ae = app.clone();
            let v = segmenter.clone();
            let o = output.clone();
            device.build_input_stream(
                &config,
                move |d: &[u16], _| {
                    process(
                        d.iter().map(|x| *x as f32 / 32768.0 - 1.0),
                        &a,
                        &v,
                        &o,
                        rate,
                        channels,
                    )
                },
                move |e| {
                    let _ = ae.emit("pipeline-status", format!("Audio device error: {e}"));
                },
                None,
            )
        }
        _ => return Err("Unsupported device sample format".into()),
    }
    .map_err(|e| e.to_string())?;
    stream.play().map_err(|e| e.to_string())?;
    Ok(stream)
}
fn process(
    samples: impl Iterator<Item = f32>,
    app: &AppHandle,
    vad: &Arc<Mutex<Segmenter>>,
    output: &Option<SyncSender<AudioSegment>>,
    sample_rate: u32,
    channels: u16,
) {
    let interleaved: Vec<f32> = samples.collect();
    let frame: Vec<f32> = if channels > 1 {
        interleaved
            .chunks(channels as usize)
            .map(|c| c.iter().sum::<f32>() / c.len() as f32)
            .collect()
    } else {
        interleaved
    };
    if frame.is_empty() {
        return;
    }
    let rms = (frame.iter().map(|x| x * x).sum::<f32>() / frame.len() as f32).sqrt();
    let _ = app.emit("audio-level", (rms * 8.0).clamp(0.0, 1.0));
    if let Ok(mut v) = vad.lock() {
        if let Some(samples) = v.push(&frame) {
            let _ = app.emit("pipeline-status", "Speech segment finalized locally");
            if let Some(tx) = output {
                let end_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let start_ms =
                    end_ms.saturating_sub((samples.len() as u64 * 1000) / sample_rate as u64);
                if tx
                    .try_send(AudioSegment {
                        samples,
                        sample_rate,
                        start_ms,
                        end_ms,
                    })
                    .is_err()
                {
                    let _ = app.emit(
                        "pipeline-status",
                        "Inference queue full · segment discarded to control latency",
                    );
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::{process, AudioSegment, AudioSource, CaptureHandle};
    use crate::vad::Segmenter;
    use std::{
        collections::VecDeque,
        sync::{
            atomic::{AtomicBool, Ordering},
            mpsc::SyncSender,
            Arc, Mutex,
        },
    };
    use tauri::{AppHandle, Emitter};
    use wasapi::*;
    pub fn native_sources() -> Vec<AudioSource> {
        vec![AudioSource{id:"wasapi:system".into(),name:"Windows system output (built-in WASAPI loopback)".into(),kind:"system".into(),available:true,detail:"Captures the complete system mix sent to the default output. It does not isolate Discord and requires no virtual cable.".into()},AudioSource{id:"wasapi:application".into(),name:"Application audio (process-isolated)".into(),kind:"application".into(),available:false,detail:"Process-isolated capture requires selecting a process and Windows 10 build 20348+. Selecting a Discord window is not equivalent.".into()}]
    }
    pub fn start_system(
        app: AppHandle,
        trailing: u64,
        max: u64,
        pre: u64,
        output: Option<SyncSender<AudioSegment>>,
    ) -> Result<CaptureHandle, String> {
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = stop.clone();
        std::thread::Builder::new()
            .name("wasapi-loopback".into())
            .spawn(move || {
                if let Err(e) = run(app.clone(), thread_stop, trailing, max, pre, output) {
                    let _ = app.emit("pipeline-status", format!("WASAPI loopback failed: {e}"));
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(CaptureHandle::new(stop))
    }
    fn run(
        app: AppHandle,
        stop: Arc<AtomicBool>,
        trailing: u64,
        max: u64,
        pre: u64,
        output: Option<SyncSender<AudioSegment>>,
    ) -> Result<(), String> {
        initialize_mta().ok().map_err(|e| e.to_string())?;
        let enumerator = DeviceEnumerator::new().map_err(|e| e.to_string())?;
        let device = enumerator
            .get_default_device(&Direction::Render)
            .map_err(|e| e.to_string())?;
        let mut client = device.get_iaudioclient().map_err(|e| e.to_string())?;
        let format = WaveFormat::new(32, 32, &SampleType::Float, 48000, 2, None);
        let (_, min) = client.get_device_period().map_err(|e| e.to_string())?;
        client
            .initialize_client(
                &format,
                &Direction::Capture,
                &StreamMode::EventsShared {
                    autoconvert: true,
                    buffer_duration_hns: min,
                },
            )
            .map_err(|e| e.to_string())?;
        let event = client.set_get_eventhandle().map_err(|e| e.to_string())?;
        let capture = client.get_audiocaptureclient().map_err(|e| e.to_string())?;
        let mut bytes = VecDeque::new();
        let vad = Arc::new(Mutex::new(Segmenter::new(48000, trailing, max, pre)));
        client.start_stream().map_err(|e| e.to_string())?;
        while !stop.load(Ordering::SeqCst) {
            if capture
                .get_next_packet_size()
                .map_err(|e| e.to_string())?
                .unwrap_or(0)
                > 0
            {
                capture
                    .read_from_device_to_deque(&mut bytes)
                    .map_err(|e| e.to_string())?;
                let contiguous: Vec<u8> = bytes.drain(..).collect();
                let samples = contiguous
                    .chunks_exact(4)
                    .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]));
                process(samples, &app, &vad, &output, 48000, 2)
            }
            let _ = event.wait_for_event(100);
        }
        let _ = client.stop_stream();
        Ok(())
    }
}
#[cfg(target_os = "macos")]
mod platform {
    use super::{process, AudioSegment, AudioSource, CaptureHandle};
    use crate::vad::Segmenter;
    use screencapturekit::prelude::*;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::SyncSender,
        Arc, Mutex,
    };
    use tauri::AppHandle;
    pub fn native_sources() -> Vec<AudioSource> {
        vec![AudioSource{id:"sck:system".into(),name:"macOS system audio (built-in ScreenCaptureKit)".into(),kind:"system".into(),available:true,detail:"Captures the system mix on macOS 13+ after Screen & System Audio Recording consent. It requires no BlackHole device.".into()},AudioSource{id:"sck:application".into(),name:"Application audio (ScreenCaptureKit filter)".into(),kind:"application".into(),available:false,detail:"Application-only selection is not yet exposed; system capture includes every audible application.".into()}]
    }
    struct Handler {
        app: AppHandle,
        vad: Arc<Mutex<Segmenter>>,
        output: Option<SyncSender<AudioSegment>>,
    }
    impl SCStreamOutputTrait for Handler {
        fn did_output_sample_buffer(
            &self,
            sample: CMSampleBuffer,
            output_type: SCStreamOutputType,
        ) {
            if output_type != SCStreamOutputType::Audio {
                return;
            }
            if let Some(list) = sample.audio_buffer_list() {
                for buffer in list.iter() {
                    let data = buffer.data();
                    let samples = data
                        .chunks_exact(4)
                        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]));
                    process(
                        samples,
                        &self.app,
                        &self.vad,
                        &self.output,
                        48000,
                        buffer.number_channels() as u16,
                    );
                }
            }
        }
    }
    pub fn start_system(
        app: AppHandle,
        trailing: u64,
        max: u64,
        pre: u64,
        output: Option<SyncSender<AudioSegment>>,
    ) -> Result<CaptureHandle, String> {
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = stop.clone();
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("screencapturekit-audio".into())
            .spawn(
                move || match create_stream(app, trailing, max, pre, output) {
                    Ok(mut stream) => {
                        if ready_tx.send(Ok(())).is_err() {
                            let _ = stream.stop_capture();
                            return;
                        }
                        while !thread_stop.load(Ordering::SeqCst) {
                            std::thread::sleep(std::time::Duration::from_millis(50));
                        }
                        let _ = stream.stop_capture();
                    }
                    Err(error) => {
                        let _ = ready_tx.send(Err(error));
                    }
                },
            )
            .map_err(|e| e.to_string())?;
        ready_rx
            .recv()
            .map_err(|_| "ScreenCaptureKit thread stopped during startup".to_string())??;
        Ok(CaptureHandle::new(stop))
    }

    fn create_stream(
        app: AppHandle,
        trailing: u64,
        max: u64,
        pre: u64,
        output: Option<SyncSender<AudioSegment>>,
    ) -> Result<SCStream, String> {
        let content = SCShareableContent::get().map_err(|e| e.to_string())?;
        let display = content
            .displays()
            .into_iter()
            .next()
            .ok_or("No display available for system audio capture")?;
        let filter = SCContentFilter::create()
            .with_display(&display)
            .with_excluding_windows(&[])
            .build();
        let config = SCStreamConfiguration::new()
            .with_width(2)
            .with_height(2)
            .with_captures_audio(true)
            .with_sample_rate(48000)
            .with_channel_count(2);
        let mut stream = SCStream::new(&filter, &config);
        stream.add_output_handler(
            Handler {
                app,
                vad: Arc::new(Mutex::new(Segmenter::new(48000, trailing, max, pre))),
                output,
            },
            SCStreamOutputType::Audio,
        );
        stream
            .start_capture()
            .map_err(|e| format!("ScreenCaptureKit permission or capture error: {e}"))?;
        Ok(stream)
    }
}
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod platform {
    use super::AudioSource;
    pub fn native_sources() -> Vec<AudioSource> {
        vec![AudioSource{id:"unsupported:system".into(),name:"System output capture".into(),kind:"system".into(),available:false,detail:"This release targets Windows and macOS. On this platform, expose system audio through a virtual input.".into()}]
    }
}
