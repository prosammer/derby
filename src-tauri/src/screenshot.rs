use swift_rs::{SRArray, SRString, swift, SRObject};
use anyhow::{Result, anyhow};

swift!(fn screenshot_ocr() -> SRObject<RustResponse>);

#[repr(C)]
struct RustResponse {
    success: bool,
    content: SRArray<SRString>
}

pub fn ocr_screenshot() -> Result<Vec<String>> {
    let rust_response = unsafe { screenshot_ocr() };

    let mut response = Vec::new();
    if rust_response.success {
        println!("Response: {}", rust_response.success);
        for sr_string in rust_response.content.as_slice() {
            println!("Response: {}", sr_string.as_str());
            response.push(sr_string.as_str().to_string());
        }
    } else {
        return Err(anyhow!("Swift returned false"));
    }

    return Ok(response);
}