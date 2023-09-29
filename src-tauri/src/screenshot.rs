use screenshots::Screen;
use swift_rs::{SRArray, SRString, swift, SRObject};

swift!(fn recognize_text(path_srstring: &SRString) -> SRObject<RustResponse>);

#[repr(C)]
struct RustResponse {
    success: bool,
    content: SRArray<SRString>
}



pub fn take_screenshot(path: &String) {
    // TODO: Capture only the relevant parts of the screen via the frontmost window coords?
    let screens = Screen::all().unwrap();

    let selected_screen = screens.get(0).unwrap();
    println!("capturer {selected_screen:?}");
    let mut image = selected_screen.capture().unwrap();
    image
        .save(path)
        .unwrap();
}

pub fn ocr_screenshot(path: &str) {
    let path: SRString = path.into();
    let rust_response = unsafe { recognize_text(&path) };

    if rust_response.success {
        println!("Response: {}", rust_response.success);
        for sr_string in rust_response.content.as_slice() {
            println!("Response: {}", sr_string.as_str());
        }
    } else {
        println!("Response returned false");
    }
}