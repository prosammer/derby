use std::sync::{Arc, Mutex};
use std::thread::spawn;
use async_openai::types::Role;
use tauri::{AppHandle};
use crate::{gpt, whisper};
use crate::audio_utils::{ resample_audio, write_to_wav};
use crate::gpt::{GptClient, messages_setup};
use crate::screenshot::{screenshot};
use crate::whisper::WHISPER_CONTEXT;

pub fn user_speech_to_gpt_response(app_handle: AppHandle, hotkey_count: Arc<Mutex<u32>>) {
    // record audio in this thread until the hotkey is pressed again
    // ocr the screenshot
    let screenshot_handle = spawn(|| {
       let image_path = screenshot();
        return image_path;
    });

    let app_handle_clone = app_handle.clone();


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

    let screenshot_path = screenshot_handle.join().unwrap();


    let mut messages = messages_setup();
    let user_message = gpt::create_chat_completion_request_msg(speech_text, Role::User);
    messages.push(user_message);
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        // Create an instance of GptClient
        let gpt_client = GptClient::new(app_handle_clone);

        // Call the method to get the GPT response
        match gpt_client.get_gpt_response(messages, screenshot_path.clone()).await {
            Ok(_) => println!("GPT response received successfully"),
            Err(e) => eprintln!("Error while getting GPT response: {}", e),
        }
    });
    match write_to_wav(&resampled_audio, "/Users/samfinton/Downloads/output_resampled.wav") {
        Ok(()) => println!("Successfully written to WAV file"),
        Err(e) => eprintln!("Failed to write to WAV file: {}", e),
    }
}

