use screenshots::Screen;
#[tauri::command]
pub fn ocr_screenshot() -> Result<Vec<String>, String> {
    let screens = Screen::all().unwrap();
    for screen in screens {
        println!("capturer {screen:?}");
        let mut image = screen.capture().unwrap();
        image
            .save(format!("target/{}.png", screen.display_info.id))
            .unwrap();
    }

    // TODO: OCR the screenshot
    let mut response = Vec::new();
    response.push("dummy text1".to_string());

    return Ok(response);
}

#[tauri::command]
pub fn request_screen_recording_permissions() -> bool {
    let screens = Screen::all().unwrap();
    let first_screen = screens.first().unwrap();
    return first_screen.capture().is_ok()
}