#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod stores;
mod gpt;
mod screenshot;

use std::env;
use dotenv::dotenv;
use log::{info, LevelFilter};
use tauri::{ActivationPolicy, AppHandle, CustomMenuItem, GlobalShortcutManager, Manager, SystemTray, SystemTrayMenu, SystemTrayMenuItem, WindowBuilder, WindowUrl};
use tauri::TitleBarStyle::{Transparent};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use tauri_plugin_log::LogTarget;
use tauri_plugin_positioner::{Position, WindowExt};

use crate::stores::{get_from_store, set_in_store};
use crate::gpt::check_api_key_validity;
use crate::screenshot::request_screen_recording_permissions;

const APP_ICON_DEFAULT: &str = "resources/assets/sigma_master_512.png";
const APP_ICON_LISTENING: &str = "resources/assets/sigma_master_green_512.png";

#[macro_export]
macro_rules! notify {
    ($title:expr, $body:expr, $bundle_id:expr) => {{
        match tauri::api::notification::Notification::new($bundle_id)
            .title($title)
            .body($body)
            .show() {
                Ok(_) => info!("Notification shown"),
                Err(e) => error!("Failed to show notification: {:?}", e),
        }
    }};
}

fn main() {
    dotenv().ok();
    let tray = tray_setup();

    let mut app = tauri::Builder::default()
        .setup( |app| {
            let app_handle = app.handle();

            let is_testing_env = env::var("TESTING_ENV").map(|val| val == "true").unwrap_or(false);
            if is_testing_env {
                if get_from_store(&app_handle, "first_run").is_none() {
                    info!("First run detected");
                    create_first_run_window(&app_handle);
                } else {
                    toggle_transcription_window(&app_handle);
                }
            }

            app_handle.global_shortcut_manager().register("F5", move || {
                toggle_transcription_window(&app_handle);
            }).unwrap();

            Ok(())
        })
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--flag1", "--flag2"])))
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_upload::init())
        .plugin(tauri_plugin_log::Builder::default().targets([
            LogTarget::LogDir,
            LogTarget::Stdout,
            // LogTarget::Webview,
        ])
            .level(LevelFilter::Info)
            .level_for("tauri", LevelFilter::Warn)
            .level_for("tao", LevelFilter::Warn)
            .with_colors(ColoredLevelConfig::default())
            .build())
        .invoke_handler(tauri::generate_handler![
            request_screen_recording_permissions,
            check_api_key_validity,
            get_env_var
        ])
        .system_tray(tray)
        .on_system_tray_event(|app_handle, event| {
            match event {
                tauri::SystemTrayEvent::LeftClick { .. } => {
                    toggle_transcription_window(&app_handle);
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

fn toggle_transcription_window(app_handle: &AppHandle) {
    let window_exists = app_handle.get_window("transcription_window").is_some();
    if window_exists {
        // close the window
        let window = app_handle.get_window("transcription_window").unwrap();
        window.close().unwrap();
    } else {
        let new_window = WindowBuilder::new(
            app_handle,
            "transcription_window",
            WindowUrl::App("transcription".into())
        )
            .title_bar_style(Transparent)
            .hidden_title(true)
            .transparent(true)
            .always_on_top(true)
            .inner_size(400.0,100.0)
            .build()
            .expect("Failed to create transcription_window");

        let _ = new_window.move_window(Position::RightCenter);
    }
}
fn create_settings_window(app_handle: &AppHandle) -> tauri::Window {
    let new_window = WindowBuilder::new(
        app_handle,
        "settings_window",
        WindowUrl::App("settings".into())
    )
        .title("Settings")
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
        .title("Welcome to Derby")
        .always_on_top(true)
        .build()
        .expect("Failed to create settings_window");

    set_in_store(app_handle, "first_run".to_string(), serde_json::Value::Bool(true));

    new_window
}

fn tray_setup() -> SystemTray {
    // let settings = CustomMenuItem::new("settings".to_string(), "Settings");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit").accelerator("Cmd+Q");
    let tray_menu = SystemTrayMenu::new()
        // .add_item(settings)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let tray = SystemTray::new().with_menu(tray_menu).with_menu_on_left_click(false);
    tray
}

#[tauri::command]
fn get_env_var(key: &str) -> Result<String, String> {
    match env::var(key) {
        Ok(val) => Ok(val),
        Err(e) => Err(e.to_string()),
    }
}