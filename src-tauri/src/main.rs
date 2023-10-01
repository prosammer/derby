#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod whisper;
mod stores;
mod audio_utils;
mod voice_chat;
mod gpt;
mod screenshot;
mod text_to_speech;

use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use dotenv::dotenv;
use tauri::{ActivationPolicy, AppHandle, CustomMenuItem, GlobalShortcutManager, Manager, SystemTray, SystemTrayMenu, SystemTrayMenuItem, WindowBuilder, WindowUrl};
use tauri_plugin_autostart::MacosLauncher;

use crate::voice_chat::user_speech_to_gpt_response;


fn main() {
    dotenv().ok();

    let record = CustomMenuItem::new("talk".to_string(), "Talk");
    let settings = CustomMenuItem::new("settings".to_string(), "Settings");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(record)
        .add_item(settings)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let tray = SystemTray::new().with_menu(tray_menu);

    let hotkey_count = Arc::new(Mutex::new(0));

    let mut app = tauri::Builder::default()
        .setup( |app| {
            let app_handle = app.handle();
            let (shortcut_pressed_tx, shortcut_pressed_rx) = channel();
            app_handle.global_shortcut_manager().register("F5", move || {
                shortcut_pressed_tx.send(true).unwrap();
                println!("Shortcut pressed")
            }).unwrap();

            println!("Waiting for shortcut...");

            spawn(move || {
                loop {
                    if shortcut_pressed_rx.recv().is_ok() {
                        let mut locked_count = hotkey_count.lock().unwrap();
                        *locked_count += 1;
                        println!("Hotkey count: {}", locked_count);

                        let hotkey_count_clone = hotkey_count.clone();
                        let handle_clone = app_handle.clone();
                        if *locked_count % 2 != 0 {
                            spawn(move || {
                                user_speech_to_gpt_response(handle_clone, hotkey_count_clone);
                            });
                        }
                    }
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--flag1", "--flag2"])))
        .plugin(tauri_plugin_store::Builder::default().build())
        // .invoke_handler(tauri::generate_handler![user_speech_to_gpt_response])
        .system_tray(tray)
        .on_system_tray_event(|app_handle, event| {
            match event {
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

    app.run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}
fn create_settings_window(handle: &AppHandle) -> tauri::Window {
    let new_window = WindowBuilder::new(
        handle,
        "settings_window",
        WindowUrl::App("settings".into())
    )
        .build()
        .expect("Failed to create settings_window");

    new_window
}
