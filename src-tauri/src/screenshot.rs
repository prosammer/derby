use std::fs;
use screenshots::Screen;

pub fn screenshot() -> String {
    let screenshot_path = "/Users/samfinton/Downloads/screenshot.png";
    let screens = Screen::all().unwrap();
    let screen = screens[0];
    println!("capturer {screen:?}");
    let image = screen.capture().unwrap();
    image
        .save(screenshot_path)
        .unwrap();

    let metadata = fs::metadata(screenshot_path).unwrap();
    // Get the file size from the metadata
    let file_size = metadata.len();
    println!("File size is: {} bytes", file_size);

    return screenshot_path.to_string();
}

#[tauri::command]
pub fn request_screen_recording_permissions() -> bool {
    let screens = Screen::all().unwrap();
    let first_screen = screens.first().unwrap();
    return first_screen.capture().is_ok()
}