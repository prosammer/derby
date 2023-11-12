use std::thread::spawn;
use anyhow::Error;
use async_openai::types::Role;
use cpal::SampleRate;
use log::info;
use tauri::{AppHandle};
use crate::{gpt, whisper};
use crate::audio_utils::{resample_audio, TARGET_SAMPLE_RATE};
use crate::gpt::{GptClient, messages_setup};
use crate::screenshot::{screenshot};
use crate::whisper::{AudioRecording, WHISPER_CONTEXT};

pub fn user_speech_to_gpt_response(app_handle: AppHandle) {
    // record audio in this thread until the hotkey is pressed again
    // ocr the screenshot
    let app_handle_clone = app_handle.clone();
    let screenshot_handle = spawn(|| {
       let image_path = screenshot(app_handle_clone);
        return image_path;
    });

    let app_handle_clone = app_handle.clone();

    whisper::init_whisper_context(&app_handle);
    let ctx = WHISPER_CONTEXT.get().expect("WhisperContext not initialized");
    let mut state = ctx.create_state().expect("failed to create key");

    info!("Initialization complete, starting audio thread");
    let audio_res: anyhow::Result<AudioRecording, Error>= whisper::get_audio_recording(app_handle);

    let mut audio_recording = audio_res.unwrap();
    if audio_recording.config.sample_rate != SampleRate(TARGET_SAMPLE_RATE as u32) {
        audio_recording = resample_audio(audio_recording);
    } else {
        info!("Target sample rate is: {}. Audio sample rate is already {}. Not resampling", TARGET_SAMPLE_RATE, audio_recording.config.sample_rate.0);
    }


    let speech_text = whisper::speech_to_text(audio_recording, &mut state);
    info!("Speech to text: {}", speech_text);

    let screenshot_path = screenshot_handle.join().unwrap();


    let mut messages = messages_setup();
    let user_message = gpt::create_chat_completion_request_msg(speech_text, Role::User);
    messages.push(user_message);
    let rt = tokio::runtime::Runtime::new().unwrap();

    let app_handle_2 = app_handle_clone.clone();
    rt.block_on(async {
        // Create an instance of GptClient
        let gpt_client = GptClient::new(app_handle_clone);

        // Call the method to get the GPT response
        match gpt_client.get_gpt_response(messages, screenshot_path.clone(), app_handle_2).await {
            Ok(_) => println!("GPT response received successfully"),
            Err(e) => eprintln!("Error while getting GPT response: {}", e),
        }
    });
    // match write_to_wav(&resampled_audio, "/Users/samfinton/Downloads/output_resampled.wav") {
    //     Ok(()) => println!("Successfully written to WAV file"),
    //     Err(e) => eprintln!("Failed to write to WAV file: {}", e),
    // }
}

