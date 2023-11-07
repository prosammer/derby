#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod whisper;
mod stores;
mod audio_utils;
mod voice_chat;
mod gpt;
mod screenshot;
mod text_to_speech;

use std::sync::{ Mutex};
use std::thread::spawn;
use dotenv::dotenv;
use tauri::{ActivationPolicy, AppHandle, CustomMenuItem, GlobalShortcutManager, Icon, Manager, SystemTray, SystemTrayMenu, SystemTrayMenuItem, WindowBuilder, WindowUrl};
use tauri::TitleBarStyle::{Transparent};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_positioner::{Position, WindowExt};
use crate::stores::{get_from_store, set_in_store};

use crate::voice_chat::user_speech_to_gpt_response;
use crate::screenshot::request_screen_recording_permissions;

const APP_ICON_DEFAULT: &str = "resources/assets/sigma_master_512.png";
const APP_ICON_LISTENING: &str = "resources/assets/sigma_master_green_512.png";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TranscriptionMode {
    Inactive,
    Listening,
    Processing,
}

pub struct TranscriptionState {
    mode: Mutex<TranscriptionMode>,
}

impl TranscriptionState {
    pub fn new() -> Self {
        TranscriptionState {
            mode: Mutex::new(TranscriptionMode::Inactive),
        }
    }

    pub fn set_mode(&self, new_mode: TranscriptionMode, app_handle: &AppHandle) {
        let mut mode = self.mode.lock().unwrap();
        *mode = new_mode;

        match new_mode {
            TranscriptionMode::Inactive => self.on_inactive(app_handle.clone()),
            TranscriptionMode::Listening => self.on_listening(app_handle.clone()),
            TranscriptionMode::Processing => self.on_processing(app_handle.clone()),
        }
    }

    fn on_inactive(&self, app_handle: AppHandle) {
        println!("Transcription mode: Inactive");
        let resource_path = app_handle.path_resolver()
            .resolve_resource(APP_ICON_DEFAULT)
            .expect("Failed to resolve icon resource path");

        app_handle.tray_handle().set_icon_as_template(true).unwrap();
        app_handle.tray_handle().set_icon(Icon::File(resource_path)).unwrap();
    }

    fn on_listening(&self, app_handle: AppHandle) {
        println!("Transcription mode: Listening");
        let resource_path = app_handle.path_resolver()
            .resolve_resource(APP_ICON_LISTENING)
            .expect("Failed to resolve icon resource path");

        app_handle.tray_handle().set_icon_as_template(false).unwrap();
        app_handle.tray_handle().set_icon(Icon::File(resource_path)).unwrap();

        let app_handle_clone = app_handle.clone();
        spawn(move || {
            user_speech_to_gpt_response(app_handle_clone);
        });
    }

    fn on_processing(&self, app_handle: AppHandle) {
        println!("Transcription mode: Processing");

        let resource_path = app_handle.path_resolver()
            .resolve_resource(APP_ICON_DEFAULT)
            .expect("Failed to resolve icon resource path");

        app_handle.tray_handle().set_icon_as_template(true).unwrap();
        app_handle.tray_handle().set_icon(Icon::File(resource_path)).unwrap();

        // TODO: I think there's a race condition here
        let _window = create_transcription_window(&app_handle);
    }
}

fn main() {
    dotenv().ok();

    let transcription_state = TranscriptionState::new();

    let tray = tray_setup();

    let mut app = tauri::Builder::default()
        .setup( |app| {
            let app_handle = app.handle();
            app_handle.manage(TranscriptionState::new());

            // let _window = create_transcription_window(&app_handle);
            if get_from_store(&app_handle, "first_run").is_none() {
                create_first_run_window(&app_handle);
            }

            let app_handle_clone = app_handle.clone();
            app_handle.global_shortcut_manager().register("F5", move || {
                change_transcription_state(&app_handle);
            }).unwrap();

            Ok(())
        })
        .manage(transcription_state)
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--flag1", "--flag2"])))
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![request_screen_recording_permissions])
        .system_tray(tray)
        .on_system_tray_event(|app_handle, event| {
            match event {
                tauri::SystemTrayEvent::LeftClick { .. } => {
                    change_transcription_state(&app_handle);
                }
                tauri::SystemTrayEvent::MenuItemClick { id, .. } => {
                    match id.as_str() {
                        "settings" => {
                            let window_exists = app_handle.get_window("settings_window").is_some();
                            if !window_exists {
                                let _window = create_settings_window(&app_handle);
                            }
                        }
                        "quit" => {
                            app_handle.exit(0);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.set_activation_policy(ActivationPolicy::Accessory);

    app.run(move |_app_handle, event|
        match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        _ => {}
    });
}
fn change_transcription_state(app_handle: &AppHandle) {
    let app_state = app_handle.state::<TranscriptionState>();

    let current_mode = {
        let mode_lock = app_state.mode.lock().unwrap();
        (*mode_lock).clone() // Clone the current mode to avoid moving it
    };

    let next_mode = match current_mode {
        TranscriptionMode::Inactive => TranscriptionMode::Listening,
        TranscriptionMode::Listening => TranscriptionMode::Processing,
        TranscriptionMode::Processing => TranscriptionMode::Inactive,
    };

    // Set the new mode, which will also trigger the corresponding function
    app_state.set_mode(next_mode, &app_handle);

    println!("Shortcut pressed and mode changed to {:?}", next_mode);
}


fn create_transcription_window(app_handle: &AppHandle) -> tauri::Window {
    let new_window = WindowBuilder::new(
        app_handle,
        "transcription_window",
        WindowUrl::App("transcription".into())
    )
        .decorations(false)
        .title_bar_style(Transparent)
        .hidden_title(true)
        .transparent(true)
        .always_on_top(true)
        .inner_size(400.0,400.0)
        .build()
        .expect("Failed to create transcription_window");

    let _ = new_window.move_window(Position::RightCenter);
    new_window
}
fn create_settings_window(app_handle: &AppHandle) -> tauri::Window {
    let new_window = WindowBuilder::new(
        app_handle,
        "settings_window",
        WindowUrl::App("settings".into())
    )
        .build()
        .expect("Failed to create settings_window");

    new_window
}

fn create_first_run_window(app_handle: &AppHandle) -> tauri::Window {
    let new_window = WindowBuilder::new(
        app_handle,
        "first_run_window",
        WindowUrl::App("first_run".into())
    )
        .build()
        .expect("Failed to create settings_window");

    set_in_store(app_handle, "first_run".to_string(), serde_json::Value::Bool(true));

    new_window
}

fn tray_setup() -> SystemTray {
    let settings = CustomMenuItem::new("settings".to_string(), "Settings");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit").accelerator("Cmd+Q");
    let tray_menu = SystemTrayMenu::new()
        .add_item(settings)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let tray = SystemTray::new().with_menu(tray_menu).with_menu_on_left_click(false);
    tray
}
