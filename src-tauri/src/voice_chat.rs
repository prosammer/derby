use std::sync::{Arc, Mutex};
use std::thread::spawn;
use async_openai::types::Role;
use tauri::{AppHandle};
use crate::{gpt, whisper};
use crate::audio_utils::{ resample_audio, write_to_wav};
use crate::gpt::{get_gpt_response, messages_setup};
use crate::screenshot::{screenshot_and_upload};
use crate::whisper::WHISPER_CONTEXT;

pub fn user_speech_to_gpt_response(app_handle: AppHandle, hotkey_count: Arc<Mutex<i32>>) {
    // record audio in this thread until the hotkey is pressed again

    let user_speech_to_text = Arc::new(Mutex::new(String::new()));
    let user_speech_to_text_clone = user_speech_to_text.clone();

    // ocr the screenshot
    let screenshot_handle = spawn(|| {
       screenshot_and_upload();
    });

    whisper::init_whisper_context(&app_handle);
    let ctx = WHISPER_CONTEXT.get().expect("WhisperContext not initialized");
    let mut state = ctx.create_state().expect("failed to create key");

    println!("Initialization complete, starting audio thread");
    // audio_res is f32, 48khz, float data from CPAL
    let audio_res: anyhow::Result<Vec<f32>>= whisper::get_audio_recording(hotkey_count, app_handle);

    let audio_vec = audio_res.unwrap();
    let resampled_audio = resample_audio(&audio_vec, 48000, 16000);

    let speech_text = whisper::speech_to_text(&resampled_audio, &mut state);
    println!("Speech to text: {}", speech_text);

    let screenshot_url = screenshot_handle.join().unwrap();


    // TODO: Send the image and the user speech to chatgpt
    let mut messages = messages_setup();
    let user_message = gpt::create_chat_completion_request_msg(speech_text, Role::User);
    messages.push(user_message);
    // let response = get_gpt_response(messages, screenshot_url);



    match write_to_wav(&resampled_audio, "/Users/samfinton/Downloads/output_resampled.wav") {
        Ok(()) => println!("Successfully written to WAV file"),
        Err(e) => eprintln!("Failed to write to WAV file: {}", e),
    }
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

