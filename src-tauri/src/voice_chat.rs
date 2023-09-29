use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread::spawn;
use tauri::{AppHandle};
use crate::whisper;
use crate::gpt::get_gpt_response;
use crate::screenshot::{ocr_screenshot, take_screenshot};
use crate::whisper::WHISPER_CONTEXT;

pub fn user_speech_to_gpt_response(handle: AppHandle, hotkey_count: Arc<Mutex<i32>>) {
    // let (audio_tx, audio_rx) = channel();

    let user_speech_to_text = Arc::new(Mutex::new(String::new()));
    let user_speech_to_text_clone = user_speech_to_text.clone();

    let screenshot_path = "/Users/samfinton/Documents/Programming/swift_ocr/ocr.png".to_string();
    take_screenshot(&screenshot_path);
    println!("Screenshot taken");
    println!("Sleeping for 10 seconds");
    std::thread::sleep(std::time::Duration::from_secs(10));
    println!("Waking up");
    // ocr the screenshot
    ocr_screenshot(&screenshot_path);
    
    // include the ocr text in the prompt

    // whisper::init_whisper_context();
    // let ctx = WHISPER_CONTEXT.get().expect("WhisperContext not initialized");
    // let mut state = ctx.create_state().expect("failed to create key");
    //
    // println!("Initialization complete, starting audio thread");
    // // start cpal audio recording to channel
    // // when the hotkey_rx receives a message, the audio thread is stopped.
    // spawn(|| {
    //     whisper::send_system_audio_to_channel(audio_tx, hotkey_count);
    // });
    //
    // // This will keep looping until the hotkey is pressed again (there is nothing in the channel)
    // loop {
    //     if let Ok(audio) = audio_rx.recv() {
    //         println!("Received audio");
    //         let text = whisper::speech_to_text(&audio, &mut state);
    //         user_speech_to_text_clone.lock().unwrap().push_str(&text);
    //     } else {
    //         break;
    //     }
    // }
    //
    // println!("User Speech: {}", user_speech_to_text_clone.lock().unwrap());
    //
    // // finally, the entire_text is sent to GPT and the response is copy/pasted
    // let gpt_response = get_gpt_response(user_speech_to_text.lock().unwrap().clone()).expect("Failed to get GPT response");
    // println!("GPT Response: {}", gpt_response.content.as_ref().unwrap());
    // pbcopy the gpt_response
}

