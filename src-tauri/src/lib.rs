use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;

mod audio;
mod api;
mod input;
mod setup;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppConfig {
    hotkey: String,
    position: String,
    device: Option<String>,
    auto_type: bool,
    sensitivity: f32,
    dark_mode: bool,
    language: String,
    typing_speed: u32,
    auto_start: bool,
    minimize_to_tray: bool,
    sound_enabled: bool,
    start_minimized: bool,
    ai_mode: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hotkey: "F8".to_string(),
            position: "bottom-right".to_string(),
            device: None,
            auto_type: true,
            sensitivity: 1.0,
            dark_mode: false,
            language: "en".to_string(),
            typing_speed: 10,
            auto_start: false,
            minimize_to_tray: true,
            sound_enabled: true,
            start_minimized: false,
            ai_mode: false,
        }
    }
}

struct AppState {
    is_recording: Arc<Mutex<bool>>,
    is_testing: Arc<Mutex<bool>>,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: Arc<Mutex<u32>>,
    channels: Arc<Mutex<u16>>,
    selected_device: Arc<Mutex<Option<String>>>,
    selected_hotkey: Arc<Mutex<String>>,
    config: Arc<Mutex<AppConfig>>,
}

fn get_config_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap().join("config.json")
}

fn load_config(app: &AppHandle) -> AppConfig {
    let path = get_config_path(app);
    if path.exists() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
    }
    AppConfig::default()
}

fn save_config(app: &AppHandle, config: &AppConfig) {
    let path = get_config_path(app);
    let _ = fs::create_dir_all(path.parent().unwrap());
    if let Ok(content) = serde_json::to_string_pretty(config) {
        let _ = fs::write(path, content);
    }
}

fn play_feedback_sound(config: &AppConfig, is_start: bool) {
    if !config.sound_enabled { return; }
    
    // Using high-quality Windows system sounds
    let sound_path = if is_start {
        "C:\\Windows\\Media\\Windows Notify Messaging.wav"
    } else {
        "C:\\Windows\\Media\\Windows Notify System Generic.wav"
    };

    let script = format!("(New-Object Media.SoundPlayer '{}').Play()", sound_path);

    let mut cmd = std::process::Command::new("powershell");
    cmd.arg("-c").arg(script);

    #[cfg(windows)]
    cmd.creation_flags(0x08000000);
    
    let _ = cmd.spawn();
}

#[tauri::command]
async fn start_recording(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut is_testing = state.is_testing.lock().unwrap();
    if *is_testing {
        *is_testing = false;
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    let mut is_recording = state.is_recording.lock().unwrap();
    if *is_recording { return Ok(()); }
    
    let config = {
        let config_lock = state.config.lock().unwrap();
        config_lock.clone()
    };
    play_feedback_sound(&config, true);

    *is_recording = true;
    app.emit("recording-status", serde_json::json!({ "recording": true, "status": "Listening..." })).unwrap();
    
    if let Some(window) = app.get_webview_window("overlay") {
        let _ = window.set_focusable(false);
        let _ = window.show();
    }
    
    let audio_buffer = state.audio_buffer.clone();
    let is_recording_flag = state.is_recording.clone();
    let sample_rate_flag = state.sample_rate.clone();
    let channels_flag = state.channels.clone();
    let selected_device = state.selected_device.lock().unwrap().clone();
    let app_handle = app.clone();
    
    std::thread::spawn(move || {
        audio::record_audio(app_handle, audio_buffer, is_recording_flag, sample_rate_flag, channels_flag, selected_device);
    });
    
    Ok(())
}

#[tauri::command]
async fn stop_recording(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut is_recording = state.is_recording.lock().unwrap();
    if !*is_recording { return Ok(()); }
    
    *is_recording = false;
    drop(is_recording);
    
    let config = {
        let config_lock = state.config.lock().unwrap();
        config_lock.clone()
    };
    play_feedback_sound(&config, false);

    app.emit("recording-status", serde_json::json!({ "recording": false, "status": "Processing..." })).unwrap();
    
    let audio_data = {
        let mut buffer = state.audio_buffer.lock().unwrap();
        let data = buffer.clone();
        buffer.clear();
        data
    };

    let sample_rate = *state.sample_rate.lock().unwrap();
    let min_samples = (sample_rate as f32 * 0.4) as usize; 

    if audio_data.is_empty() || audio_data.len() < min_samples {
         app.emit("recording-status", serde_json::json!({ "recording": false, "status": "Too short!" })).unwrap();
         return Ok(());
    }

    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        process_audio(app_clone, audio_data, sample_rate).await;
    });
    
    Ok(())
}

async fn process_audio(app: AppHandle, data: Vec<f32>, sample_rate: u32) {
    app.emit("recording-status", serde_json::json!({ "recording": false, "status": "Transcribing..." })).unwrap();
    let app_data_dir = app.path().app_data_dir().unwrap();
    let resource_dir = app.path().resource_dir().unwrap().join("resources");

    let wav_path = match audio::save_to_wav(data, sample_rate, app_data_dir) {
        Ok(path) => path,
        Err(_) => return,
    };

    let language = {
        let state = app.state::<AppState>();
        let config = state.config.lock().unwrap();
        config.language.clone()
    };

    match api::transcribe_local(&wav_path, &resource_dir, &language).await {
        Ok(text) => {
            if !text.is_empty() {
                let (auto_type, typing_speed, ai_mode) = {
                    let state = app.state::<AppState>();
                    let config = state.config.lock().unwrap();
                    (config.auto_type, config.typing_speed, config.ai_mode)
                };
                
                if auto_type {
                    app.emit("recording-status", serde_json::json!({ "recording": false, "status": "Typing..." })).unwrap();
                    input::type_text(&text, typing_speed).await;
                    if ai_mode {
                        input::press_enter().await;
                    }
                } else {
                    let _ = app.emit("copy-to-clipboard", text.clone());
                    app.emit("recording-status", serde_json::json!({ "recording": false, "status": "Copied!" })).unwrap();
                    tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
                }
                app.emit("recording-status", serde_json::json!({ "recording": false, "status": "Done" })).unwrap();
            } else {
                app.emit("recording-status", serde_json::json!({ "recording": false, "status": "No text detected" })).unwrap();
            }
            if let Some(window) = app.get_webview_window("overlay") {
                let _ = window.hide();
            }
        }
        Err(_) => {
            if let Some(window) = app.get_webview_window("overlay") {
                let _ = window.hide();
            }
        }
    }
}

#[tauri::command]
fn get_config(state: tauri::State<'_, AppState>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn update_config(app: AppHandle, state: tauri::State<'_, AppState>, 
    auto_type: bool, sensitivity: f32, dark_mode: bool, language: String, 
    typing_speed: u32, auto_start: bool, minimize_to_tray: bool, 
    sound_enabled: bool, start_minimized: bool, ai_mode: bool
) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    
    if config.auto_start != auto_start {
        if cfg!(windows) {
            let app_path = std::env::current_exe().unwrap();
            let app_path_str = app_path.to_str().unwrap();
            let app_name = "SwiftSpeak";
            if auto_start {
                let _ = std::process::Command::new("reg")
                    .arg("add").arg("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run")
                    .arg("/v").arg(app_name).arg("/t").arg("REG_SZ")
                    .arg("/d").arg(format!("\"{}\" --minimized", app_path_str))
                    .arg("/f").output();
            } else {
                let _ = std::process::Command::new("reg")
                    .arg("delete").arg("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run")
                    .arg("/v").arg(app_name).arg("/f").output();
            }
        }
    }

    config.auto_type = auto_type;
    config.sensitivity = sensitivity;
    config.dark_mode = dark_mode;
    config.language = language;
    config.typing_speed = typing_speed;
    config.auto_start = auto_start;
    config.minimize_to_tray = minimize_to_tray;
    config.sound_enabled = sound_enabled;
    config.start_minimized = start_minimized;
    config.ai_mode = ai_mode;
    
    save_config(&app, &config);
    let _ = app.emit("config-changed", config.clone());
    Ok(())
}

#[tauri::command]
fn reset_config(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<AppConfig, String> {
    let default_config = AppConfig::default();
    let mut config = state.config.lock().unwrap();
    *config = default_config.clone();
    save_config(&app, &config);
    let _ = app.emit("config-changed", config.clone());
    Ok(default_config)
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command] fn get_audio_devices() -> Vec<String> {
    use cpal::traits::{HostTrait, DeviceTrait};
    let host = cpal::default_host();
    match host.input_devices() {
        Ok(devices) => devices.map(|d| d.name().unwrap_or_else(|_| "Unknown Device".to_string())).collect(),
        Err(_) => Vec::new(),
    }
}
#[tauri::command] fn set_audio_device(app: AppHandle, state: tauri::State<'_, AppState>, name: String) {
    let name_val = if name == "Default" { None } else { Some(name) };
    *state.selected_device.lock().unwrap() = name_val.clone();
    let mut config = state.config.lock().unwrap();
    config.device = name_val;
    save_config(&app, &config);
}
#[tauri::command] fn set_overlay_position(app: AppHandle, state: tauri::State<'_, AppState>, position: String) {
    if let Some(window) = app.get_webview_window("overlay") {
        let _ = setup::position_overlay(&window, &position);
        let mut config = state.config.lock().unwrap();
        config.position = position;
        save_config(&app, &config);
    }
}
#[tauri::command] fn start_mic_test(app: AppHandle, state: tauri::State<'_, AppState>) {
    let mut is_testing = state.is_testing.lock().unwrap();
    if *is_testing { return; }
    *is_testing = true;
    let is_testing_flag = state.is_testing.clone();
    let selected_device = state.selected_device.lock().unwrap().clone();
    std::thread::spawn(move || {
        audio::record_audio(app, Arc::new(Mutex::new(Vec::new())), is_testing_flag, Arc::new(Mutex::new(44100)), Arc::new(Mutex::new(1)), selected_device);
    });
}
#[tauri::command] fn stop_mic_test(state: tauri::State<'_, AppState>) { *state.is_testing.lock().unwrap() = false; }

#[tauri::command]
fn update_hotkey(app: AppHandle, state: tauri::State<'_, AppState>, new_hotkey: String) -> Result<(), String> {
    let mut current_hotkey = state.selected_hotkey.lock().unwrap();
    let _ = app.global_shortcut().unregister_all();
    let shortcut: Shortcut = new_hotkey.parse().map_err(|_| "Invalid hotkey format".to_string())?;
    app.global_shortcut().register(shortcut).map_err(|e| format!("Failed to register {}: {}", new_hotkey, e))?;
    *current_hotkey = new_hotkey.clone();
    let mut config = state.config.lock().unwrap();
    config.hotkey = new_hotkey;
    save_config(&app, &config);
    Ok(())
}

pub fn run() {
    let state = AppState {
        is_recording: Arc::new(Mutex::new(false)),
        is_testing: Arc::new(Mutex::new(false)),
        audio_buffer: Arc::new(Mutex::new(Vec::new())),
        sample_rate: Arc::new(Mutex::new(44100)),
        channels: Arc::new(Mutex::new(1)),
        selected_device: Arc::new(Mutex::new(None)),
        selected_hotkey: Arc::new(Mutex::new("F8".to_string())),
        config: Arc::new(Mutex::new(AppConfig::default())),
    };

    tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(move |app, _shortcut, event| {
            match event.state() {
                tauri_plugin_global_shortcut::ShortcutState::Pressed => {
                    let app_handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = start_recording(app_handle.clone(), app_handle.state::<AppState>()).await;
                    });
                }
                tauri_plugin_global_shortcut::ShortcutState::Released => {
                    let app_handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = stop_recording(app_handle.clone(), app_handle.state::<AppState>()).await;
                    });
                }
            }
        }).build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let config = load_config(app.app_handle());
            {
                let state = app.state::<AppState>();
                let mut state_config = state.config.lock().unwrap();
                *state_config = config.clone();
                *state.selected_hotkey.lock().unwrap() = config.hotkey.clone();
                *state.selected_device.lock().unwrap() = config.device.clone();
            }

            let shortcut: Shortcut = config.hotkey.parse().unwrap_or_else(|_| "F8".parse().unwrap());
            let _ = app.global_shortcut().register(shortcut);

            if let Some(window) = app.get_webview_window("overlay") {
                let _ = setup::position_overlay(&window, &config.position);
                let _ = window.set_focusable(false);
            }

            let args: Vec<String> = std::env::args().collect();
            let should_minimize = args.contains(&"--minimized".to_string()) || config.start_minimized;
            if let Some(window) = app.get_webview_window("main") {
                if should_minimize { 
                    let _ = window.hide(); 
                    // Small delay to ensure any plugin-induced visibility is overridden
                    let window_clone = window.clone();
                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        let _ = window_clone.hide();
                    });
                } else { 
                    let _ = window.show(); 
                }

                let app_handle = app.app_handle().clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        let state = app_handle.state::<AppState>();
                        let config = state.config.lock().unwrap();
                        if config.minimize_to_tray {
                            api.prevent_close();
                            let _ = app_handle.get_webview_window("main").unwrap().hide();
                        }
                    }
                });
            }

            let show_i = MenuItem::with_id(app, "show", "Show Dashboard", true, None::<&str>).unwrap();
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
            let sep = tauri::menu::PredefinedMenuItem::separator(app).unwrap();
            let menu = Menu::with_items(app, &[&show_i, &sep, &quit_i]).unwrap();

            let tray_icon = tauri::image::Image::from_bytes(include_bytes!("tray_icon.png")).unwrap();
            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "quit" => app.exit(0),
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    match event {
                        TrayIconEvent::DoubleClick { .. } => {
                             let app = tray.app_handle();
                             if let Some(window) = app.get_webview_window("main") {
                                 let _ = window.show();
                                 let _ = window.unminimize();
                                 let _ = window.set_focus();
                             }
                        }
                        // Left click is handled by menu_on_left_click(true)
                        _ => {}
                    }
                })
                .show_menu_on_left_click(true)
                .build(app).unwrap();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_recording, stop_recording, download_engine, get_config, update_config, reset_config,
            get_audio_devices, set_audio_device, set_overlay_position, start_mic_test, stop_mic_test, quit_app, update_hotkey
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command] async fn download_engine(app: AppHandle) -> Result<(), String> { setup::download_engine(app).await }
