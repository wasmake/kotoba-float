mod audio;
mod auth;
mod diagnostics;
mod provider;
mod scheduler;
mod settings;
mod text;
mod transcription;
mod vad;

use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
        Mutex,
    },
    time::Duration,
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

struct AppState {
    stream: Mutex<Option<audio::CaptureHandle>>,
    generation: AtomicU64,
    paused: AtomicBool,
    settings_path: PathBuf,
    api_verified: AtomicU8,
}
impl AppState {
    fn cancel(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.paused.store(false, Ordering::SeqCst);
        if let Ok(mut s) = self.stream.lock() {
            *s = None;
        }
    }
}

#[tauri::command]
fn get_capabilities(state: tauri::State<AppState>) -> auth::Capabilities {
    auth::capabilities(state.api_verified.load(Ordering::SeqCst))
}
#[tauri::command]
fn list_audio_sources() -> Vec<audio::AudioSource> {
    audio::sources()
}
#[tauri::command]
fn load_settings(state: tauri::State<AppState>) -> Option<serde_json::Value> {
    settings::load(&state.settings_path)
}
#[tauri::command]
fn save_settings(state: tauri::State<AppState>, settings: serde_json::Value) -> Result<(), String> {
    settings::save(&state.settings_path, &settings)
}

#[tauri::command]
fn start_session(
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
    settings: serde_json::Value,
) -> Result<(), String> {
    let local_only = settings
        .get("localCaptureOnly")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    state.cancel();
    let id = settings
        .get("sourceId")
        .and_then(|v| v.as_str())
        .ok_or("Select an audio source")?;
    let trailing = settings
        .get("trailingSilenceMs")
        .and_then(|v| v.as_u64())
        .unwrap_or(250);
    let max = settings
        .get("maxSegmentMs")
        .and_then(|v| v.as_u64())
        .unwrap_or(2200);
    let pre = settings
        .get("preRollMs")
        .and_then(|v| v.as_u64())
        .unwrap_or(120);
    let output = if local_only {
        None
    } else {
        if state.api_verified.load(Ordering::SeqCst) != 3 {
            return Err(
                "Test both transcription and translation capabilities before listening".into(),
            );
        }
        let key = auth::load_api_key()?;
        let transcription_model = settings
            .get("transcriptionModel")
            .and_then(|v| v.as_str())
            .unwrap_or("gpt-4o-mini-transcribe")
            .to_string();
        let text_model = settings
            .get("textModel")
            .and_then(|v| v.as_str())
            .unwrap_or("gpt-4.1-nano")
            .to_string();
        let target = settings
            .get("targetLanguage")
            .and_then(|v| v.as_str())
            .unwrap_or("Spanish")
            .to_string();
        Some(spawn_pipeline(
            app.clone(),
            key,
            transcription_model,
            text_model,
            target,
            state.generation.load(Ordering::SeqCst),
        ))
    };
    let stream = audio::start_capture(app.clone(), id, trailing, max, pre, output)?;
    *state
        .stream
        .lock()
        .map_err(|_| "Capture state unavailable")? = Some(stream);
    let _ = app.emit(
        "pipeline-status",
        if local_only {
            "Local capture active · inference disabled"
        } else {
            "Listening · audio segments are sent to OpenAI"
        },
    );
    Ok(())
}

fn spawn_pipeline(
    app: tauri::AppHandle,
    key: String,
    transcription_model: String,
    text_model: String,
    target: String,
    generation: u64,
) -> std::sync::mpsc::SyncSender<audio::AudioSegment> {
    let (tx, rx) = std::sync::mpsc::sync_channel::<audio::AudioSegment>(3);
    std::thread::spawn(move || {
        let provider = match provider::OpenAiProvider::new(
            key.clone(),
            transcription_model.clone(),
            text_model.clone(),
        ) {
            Ok(p) => p,
            Err(e) => {
                let _ = app.emit("pipeline-status", e);
                return;
            }
        };
        let translation_provider =
            match provider::OpenAiProvider::new(key, transcription_model, text_model) {
                Ok(p) => p,
                Err(e) => {
                    let _ = app.emit("pipeline-status", e);
                    return;
                }
            };
        let (translation_tx, translation_rx) = std::sync::mpsc::sync_channel::<(
            scheduler::Subtitle,
            String,
            Vec<String>,
            std::time::Instant,
        )>(2);
        let translation_app = app.clone();
        std::thread::spawn(move || {
            while let Ok((base, japanese, recent, started)) = translation_rx.recv() {
                if translation_app
                    .state::<AppState>()
                    .generation
                    .load(Ordering::SeqCst)
                    != generation
                {
                    return;
                }
                match translation_provider.translate(&japanese, &target, &recent) {
                    Ok(text) => {
                        if translation_app
                            .state::<AppState>()
                            .generation
                            .load(Ordering::SeqCst)
                            != generation
                        {
                            return;
                        }
                        let _ = translation_app.emit(
                            "subtitle-update",
                            scheduler::Subtitle {
                                romaji: Some(text.romaji),
                                translation: Some(text.translation),
                                translation_latency_ms: Some(started.elapsed().as_millis() as u64),
                                ..base
                            },
                        );
                    }
                    Err(e) => {
                        let _ = translation_app
                            .emit("pipeline-status", format!("Translation failed: {e}"));
                        if e.contains("429") {
                            translation_app.state::<AppState>().cancel();
                            let _ = translation_app.emit(
                                "pipeline-status",
                                "OpenAI quota or rate limit reached · processing paused",
                            );
                            return;
                        }
                    }
                }
            }
        });
        let session = uuid::Uuid::new_v4().to_string();
        let mut segment_id = 0u64;
        let mut recent: Vec<String> = vec![];
        while let Ok(segment) = rx.recv() {
            if app.state::<AppState>().generation.load(Ordering::SeqCst) != generation {
                return;
            }
            let started = std::time::Instant::now();
            match provider.transcribe(&segment.samples, segment.sample_rate) {
                Ok(raw_japanese) => {
                    let japanese = recent
                        .last()
                        .map(|previous| scheduler::deduplicate(previous, &raw_japanese))
                        .unwrap_or(raw_japanese);
                    if japanese.trim().is_empty() {
                        continue;
                    }
                    if app.state::<AppState>().generation.load(Ordering::SeqCst) != generation {
                        return;
                    }
                    let base = scheduler::Subtitle {
                        session_id: session.clone(),
                        segment_id,
                        start_ms: segment.start_ms,
                        end_ms: segment.end_ms,
                        japanese: japanese.clone(),
                        romaji: None,
                        translation: None,
                        transcript_latency_ms: Some(started.elapsed().as_millis() as u64),
                        translation_latency_ms: None,
                    };
                    let _ = app.emit("subtitle-update", base.clone());
                    let context = recent.clone();
                    recent.push(japanese.clone());
                    if recent.len() > 4 {
                        recent.remove(0);
                    }
                    if translation_tx
                        .try_send((base, japanese, context, started))
                        .is_err()
                    {
                        let _ = app.emit(
                            "pipeline-status",
                            "Translation queue full · keeping live Japanese captions current",
                        );
                    }
                }
                Err(e) => {
                    let _ = app.emit("pipeline-status", format!("Transcription failed: {e}"));
                    if e.contains("429") {
                        app.state::<AppState>().cancel();
                        let _ = app.emit(
                            "pipeline-status",
                            "OpenAI quota or rate limit reached · processing paused",
                        );
                        return;
                    }
                }
            }
            segment_id += 1;
        }
    });
    tx
}

#[tauri::command]
fn configure_api_key(
    state: tauri::State<AppState>,
    api_key: String,
    transcription_model: String,
    text_model: String,
) -> Result<auth::Capabilities, String> {
    auth::save_api_key(&api_key)?;
    state.api_verified.store(0, Ordering::SeqCst);
    test_api_key(state, transcription_model, text_model)
}
#[tauri::command]
fn test_api_key(
    state: tauri::State<AppState>,
    transcription_model: String,
    text_model: String,
) -> Result<auth::Capabilities, String> {
    let provider =
        provider::OpenAiProvider::new(auth::load_api_key()?, transcription_model, text_model)?;
    let mut verified = 0u8;
    let transcription = provider.test_transcription();
    if transcription.is_ok() {
        verified |= 1
    }
    let translation = provider.test_translation();
    if translation.is_ok() {
        verified |= 2
    }
    state.api_verified.store(verified, Ordering::SeqCst);
    if verified == 0 {
        return Err(format!(
            "Transcription: {}; Translation: {}",
            transcription.unwrap_err(),
            translation.unwrap_err()
        ));
    }
    Ok(auth::capabilities(verified))
}
#[tauri::command]
fn disconnect_api(
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
) -> Result<auth::Capabilities, String> {
    state.cancel();
    auth::clear_api_key()?;
    state.api_verified.store(0, Ordering::SeqCst);
    let _ = app.emit(
        "pipeline-status",
        "OpenAI disconnected · processing stopped",
    );
    Ok(auth::capabilities(0))
}
#[tauri::command]
fn pause_session(app: tauri::AppHandle, state: tauri::State<AppState>) {
    state.cancel();
    state.paused.store(true, Ordering::SeqCst);
    let _ = app.emit("pipeline-status", "Paused · pending work cancelled");
}
#[tauri::command]
fn stop_session(app: tauri::AppHandle, state: tauri::State<AppState>) {
    state.cancel();
    let _ = app.emit("audio-level", 0.0f32);
    let _ = app.emit("pipeline-status", "Stopped · pending work cancelled");
}

#[tauri::command]
fn start_demo(app: tauri::AppHandle, state: tauri::State<AppState>, language: String) {
    state.cancel();
    let generation = state.generation.load(Ordering::SeqCst);
    let session = uuid::Uuid::new_v4().to_string();
    if let Some(w) = app.get_webview_window("overlay") {
        let _ = w.show();
    }
    std::thread::spawn(move || {
        let lines = [
            (
                "こんばんは、今日は来てくれてありがとう。",
                "Konbanwa, kyō wa kite kurete arigatō.",
                "Buenas noches, gracias por venir hoy.",
            ),
            (
                "このゲーム、思ったより難しいね。",
                "Kono gēmu, omotta yori muzukashii ne.",
                "Este juego es más difícil de lo que pensaba.",
            ),
            (
                "でも、みんなと一緒なら楽しいよ。",
                "Demo, minna to issho nara tanoshii yo.",
                "Pero es divertido si estamos todos juntos.",
            ),
        ];
        for (i, (jp, ro, es)) in lines.iter().enumerate() {
            std::thread::sleep(Duration::from_millis(if i == 0 { 300 } else { 2600 }));
            if app.state::<AppState>().generation.load(Ordering::SeqCst) != generation {
                return;
            }
            let translation = if language == "Spanish" {
                es.to_string()
            } else {
                format!("[{language} demo] {es}")
            };
            let base = scheduler::Subtitle {
                session_id: session.clone(),
                segment_id: i as u64,
                start_ms: i as u64 * 2600,
                end_ms: i as u64 * 2600 + 1800,
                japanese: jp.to_string(),
                romaji: None,
                translation: None,
                transcript_latency_ms: Some(0),
                translation_latency_ms: None,
            };
            let _ = app.emit("subtitle-update", base.clone());
            std::thread::sleep(Duration::from_millis(220));
            if app.state::<AppState>().generation.load(Ordering::SeqCst) != generation {
                return;
            }
            let _ = app.emit(
                "subtitle-update",
                scheduler::Subtitle {
                    romaji: Some(ro.to_string()),
                    translation: Some(translation),
                    translation_latency_ms: Some(220),
                    ..base
                },
            );
        }
    });
}

#[tauri::command]
fn set_overlay_locked(app: tauri::AppHandle, locked: bool) -> Result<(), String> {
    let w = app
        .get_webview_window("overlay")
        .ok_or("Overlay unavailable")?;
    w.set_ignore_cursor_events(locked)
        .map_err(|e| e.to_string())?;
    let _ = app.emit("overlay-lock", locked);
    Ok(())
}
#[tauri::command]
fn show_overlay(app: tauri::AppHandle) -> Result<(), String> {
    let w = app
        .get_webview_window("overlay")
        .ok_or("Overlay unavailable")?;
    w.show().map_err(|e| e.to_string())?;
    w.set_focus().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            log::info!("Kotoba Float desktop startup began");
            let mut startup_warnings = Vec::new();
            let path = app.path().app_config_dir()?.join("settings.json");
            app.manage(AppState {
                stream: Mutex::new(None),
                generation: AtomicU64::new(1),
                paused: AtomicBool::new(false),
                settings_path: path,
                api_verified: AtomicU8::new(0),
            });
            if let Err(error) = WebviewWindowBuilder::new(
                app,
                "overlay",
                WebviewUrl::App("index.html?window=overlay".into()),
            )
            .title("Kotoba Float Overlay")
            .inner_size(900.0, 230.0)
            .min_inner_size(420.0, 120.0)
            .transparent(true)
            .decorations(false)
            .always_on_top(true)
            .resizable(true)
            .skip_taskbar(true)
            .visible(false)
            .build()
            {
                let message = format!("Overlay could not be created: {error}");
                log::error!("{message}");
                startup_warnings.push(message);
            }
            if let Err(error) =
                app.global_shortcut()
                    .on_shortcut("CommandOrControl+Shift+S", |app, _, event| {
                        if event.state() == ShortcutState::Pressed {
                            if let Some(w) = app.get_webview_window("overlay") {
                                if w.is_visible().unwrap_or(false) {
                                    let _ = w.hide();
                                } else {
                                    let _ = w.show();
                                }
                            }
                        }
                    })
            {
                let message = format!("Show/hide shortcut unavailable: {error}");
                log::error!("{message}");
                startup_warnings.push(message);
            }
            if let Err(error) =
                app.global_shortcut()
                    .on_shortcut("CommandOrControl+Shift+U", |app, _, event| {
                        if event.state() == ShortcutState::Pressed {
                            if let Some(w) = app.get_webview_window("overlay") {
                                let _ = w.set_ignore_cursor_events(false);
                                let _ = w.show();
                            }
                        }
                    })
            {
                let message = format!("Unlock shortcut unavailable: {error}");
                log::error!("{message}");
                startup_warnings.push(message);
            }
            if let Err(error) =
                app.global_shortcut()
                    .on_shortcut("CommandOrControl+Shift+P", |app, _, event| {
                        if event.state() == ShortcutState::Pressed {
                            app.state::<AppState>().cancel();
                            let _ = app.emit(
                                "pipeline-status",
                                "Paused from global shortcut · pending work cancelled",
                            );
                        }
                    })
            {
                let message = format!("Pause shortcut unavailable: {error}");
                log::error!("{message}");
                startup_warnings.push(message);
            }

            if let Some(icon) = app.default_window_icon().cloned() {
                let tray_result = (|| -> tauri::Result<()> {
                    let settings_i =
                        MenuItem::with_id(app, "settings", "Open settings", true, None::<&str>)?;
                    let overlay_i = MenuItem::with_id(
                        app,
                        "overlay",
                        "Show and unlock overlay",
                        true,
                        None::<&str>,
                    )?;
                    let stop_i =
                        MenuItem::with_id(app, "stop", "Stop listening", true, None::<&str>)?;
                    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
                    let menu = Menu::with_items(app, &[&settings_i, &overlay_i, &stop_i, &quit_i])?;
                    TrayIconBuilder::new()
                        .icon(icon)
                        .menu(&menu)
                        .on_menu_event(|app, event| match event.id.as_ref() {
                            "settings" => {
                                if let Some(w) = app.get_webview_window("main") {
                                    let _ = w.show();
                                    let _ = w.set_focus();
                                }
                            }
                            "overlay" => {
                                if let Some(w) = app.get_webview_window("overlay") {
                                    let _ = w.set_ignore_cursor_events(false);
                                    let _ = w.show();
                                }
                            }
                            "stop" => {
                                app.state::<AppState>().cancel();
                                let _ = app.emit(
                                    "pipeline-status",
                                    "Stopped from tray · pending work cancelled",
                                );
                            }
                            "quit" => app.exit(0),
                            _ => {}
                        })
                        .build(app)?;
                    Ok(())
                })();
                if let Err(error) = tray_result {
                    let message = format!("System tray unavailable: {error}");
                    log::error!("{message}");
                    startup_warnings.push(message);
                }
            } else {
                let message = "System tray unavailable: application icon is missing".to_string();
                log::error!("{message}");
                startup_warnings.push(message);
            }

            if !startup_warnings.is_empty() {
                let app = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_secs(1));
                    let _ = app.emit(
                        "pipeline-status",
                        format!(
                            "Started with limited desktop integration: {}",
                            startup_warnings.join("; ")
                        ),
                    );
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_capabilities,
            configure_api_key,
            test_api_key,
            disconnect_api,
            list_audio_sources,
            load_settings,
            save_settings,
            start_session,
            pause_session,
            stop_session,
            start_demo,
            set_overlay_locked,
            show_overlay
        ])
        .run(tauri::generate_context!())
        .expect("error while running Kotoba Float")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_invalidates_old_work() {
        let s = AppState {
            stream: Mutex::new(None),
            generation: AtomicU64::new(7),
            paused: AtomicBool::new(false),
            settings_path: PathBuf::new(),
            api_verified: AtomicU8::new(0),
        };
        let old = s.generation.load(Ordering::SeqCst);
        s.cancel();
        assert_ne!(old, s.generation.load(Ordering::SeqCst));
    }
}
