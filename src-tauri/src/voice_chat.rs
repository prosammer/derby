use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread::spawn;
use async_openai::types::Role;
use tauri::{AppHandle};
use crate::{gpt, whisper};
use crate::audio_utils::{resample_audio, write_to_wav};
use crate::gpt::{get_gpt_response, messages_setup};
use crate::screenshot::{ocr_screenshot};
use crate::text_to_speech::{speak_string};
use crate::whisper::WHISPER_CONTEXT;

pub fn user_speech_to_gpt_response(app_handle: AppHandle, hotkey_count: Arc<Mutex<i32>>) {
    // record audio in this thread until the hotkey is pressed again

    let user_speech_to_text = Arc::new(Mutex::new(String::new()));
    let user_speech_to_text_clone = user_speech_to_text.clone();

    // ocr the screenshot
    // let ocr_thread = spawn(|| {
    //     ocr_screenshot()
    // });

    whisper::init_whisper_context(&app_handle);
    let ctx = WHISPER_CONTEXT.get().expect("WhisperContext not initialized");
    let mut state = ctx.create_state().expect("failed to create key");

    println!("Initialization complete, starting audio thread");
    let audio_res: anyhow::Result<Vec<f32>>= whisper::get_audio_recording(hotkey_count, app_handle);

    let audio_vec = audio_res.unwrap();
    let audio_vec_clone = audio_vec.clone();

    match write_to_wav(audio_vec, "/Users/samfinton/Downloads/output.wav") {
        Ok(()) => println!("Successfully written to WAV file"),
        Err(e) => eprintln!("Failed to write to WAV file: {}", e),
    }



    let speech_text = whisper::speech_to_text(&audio_vec_clone, &mut state);
    println!("Speech to text: {}", speech_text);


    // let window_ocr_text_list = ocr_thread.join().unwrap().unwrap();
    // let window_ocr_text = window_ocr_text_list.join(" ");
    // println!("{}", window_ocr_text);
    // messages.push(gpt::create_chat_completion_request_msg(window_ocr_text, Role::User));
    //
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
    // messages.push(gpt::create_chat_completion_request_msg(user_speech_to_text_clone.lock().unwrap().clone(), Role::User));
    //
    //
    // // finally, the entire_text is sent to GPT and the response is copy/pasted
    // let gpt_response = get_gpt_response(messages).expect("Failed to get GPT response");
    // println!("GPT Response: {}", gpt_response.content.as_ref().unwrap());
    // speak_string(gpt_response.content.as_ref().unwrap().clone());
    // pbcopy the gpt_response
}

