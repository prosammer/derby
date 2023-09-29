use screenshots::Screen;

pub fn take_screenshot() {
    // TODO: Capture only the relevant parts of the screen via the frontmost window coords?
    let screens = Screen::all().unwrap();

    for screen in screens {
        println!("capturer {screen:?}");
        let mut image = screen.capture().unwrap();
        image
            .save(format!("target/{}.png", screen.display_info.id))
            .unwrap();
    }
}