use std::path::Path;
use swift_rs::SwiftLinker;

fn main() {
  const PACKAGE_NAME: &str = "swift_ocr";
  const PACKAGE_PATH_STR: &str = "../sidecar/swift_ocr";
  SwiftLinker::new("11")
      .with_package(PACKAGE_NAME, Path::new(PACKAGE_PATH_STR))
      .link();

  tauri_build::build()
}
