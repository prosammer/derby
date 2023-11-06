use cloudinary::{Cloudinary, Source};
use cloudinary::upload::result::UploadResult;
use cloudinary::upload::UploadOptions;
use screenshots::image::RgbaImage;
use screenshots::Screen;
use tokio::task;

#[tauri::command]
pub fn screenshot_and_upload() -> String {
    let screenshot_path = "/Users/samfinton/Downloads/screenshot.png";
    let screens = Screen::all().unwrap();
    let screen = screens[0];
    println!("capturer {screen:?}");
    let image = screen.capture().unwrap();
    write_image_to_file(&image, screenshot_path);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let url = rt.block_on(async {
        task::spawn_blocking(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(upload_screenshot_to_cloudinary(screenshot_path))
        }).await.unwrap()
    });
    println!("Screenshot URL: {}", url);
    return url;
}

fn write_image_to_file(image: &RgbaImage, path: &str) {
    image.save(path).unwrap();
}

async fn upload_screenshot_to_cloudinary(file_path: &str) -> String {
    let cloudinary_cloud_name = std::env::var("CLOUDINARY_CLOUD_NAME").unwrap().to_string();
    let cloudinary_api_key = std::env::var("CLOUDINARY_API_KEY").unwrap().to_string();
    let cloudinary_api_secret = std::env::var("CLOUDINARY_API_SECRET").unwrap().to_string();
    let options = UploadOptions::new().set_public_id("screenshot".to_string());

    let cloudinary = Cloudinary::new(cloudinary_api_key, cloudinary_cloud_name, cloudinary_api_secret );
    let result = cloudinary.upload_image(Source::Path(file_path.into()), &options).await.unwrap();
    match result {
        UploadResult::Success(boxed_response) => {
            boxed_response.secure_url
        },
        UploadResult::Error(_) => panic!("Failed to upload image"),
    }
}

#[tauri::command]
pub fn request_screen_recording_permissions() -> bool {
    let screens = Screen::all().unwrap();
    let first_screen = screens.first().unwrap();
    return first_screen.capture().is_ok()
}