use std::fs;
use std::path::PathBuf;
use screenshots::Screen;
use anyhow::{Result};
pub fn screenshot(screenshot_path: PathBuf) -> Result<PathBuf> {
    let screens = Screen::all()?;
    let screen = screens[0];
    println!("capturer {screen:?}");
    let image = screen.capture()?;
    image.save(screenshot_path.clone())?;
    let metadata = fs::metadata(screenshot_path.clone())?;
    // Get the file size from the metadata
    let file_size = metadata.len();
    println!("File size is: {} bytes", file_size);

    return Ok(screenshot_path)
}

#[tauri::command]
pub fn request_screen_recording_permissions() -> bool {
    let screens = Screen::all().unwrap();
    let first_screen = screens.first().unwrap();
    return first_screen.capture().is_ok()
}

// tests for screenshot fn
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_screenshot() {
        let screenshot_path = PathBuf::from("derby_latest_screenshot.png");

        let screenshot_res = screenshot(screenshot_path);
        assert!(screenshot_res.is_ok());
    }
}