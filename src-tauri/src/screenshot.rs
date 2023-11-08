use std::fs;
use screenshots::Screen;
use tauri::api::path::app_data_dir;
use tauri::AppHandle;

const SCREENSHOT_FILE_NAME: &str = "derby_latest_screenshot.png";
pub fn screenshot(app_handle: AppHandle) -> String {
    let config = app_handle.config();
    let app_data_dir_path = app_data_dir(&*config).expect("Failed to get app data dir");

    let screenshot_path = app_data_dir_path.join(SCREENSHOT_FILE_NAME);
    let screens = Screen::all().unwrap();
    let screen = screens[0];
    println!("capturer {screen:?}");
    let image = screen.capture().unwrap();
    image
        .save(screenshot_path.clone())
        .unwrap();

    let metadata = fs::metadata(screenshot_path.clone()).unwrap();
    // Get the file size from the metadata
    let file_size = metadata.len();
    println!("File size is: {} bytes", file_size);

    return screenshot_path.to_str().unwrap().into();
}

#[tauri::command]
pub fn request_screen_recording_permissions() -> bool {
    let screens = Screen::all().unwrap();
    let first_screen = screens.first().unwrap();
    return first_screen.capture().is_ok()
}