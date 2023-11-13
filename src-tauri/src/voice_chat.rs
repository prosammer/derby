use std::thread::spawn;
use anyhow::{anyhow, Context};
use async_openai::types::Role;
use cpal::SampleRate;
use log::{error, info};
use tauri::{AppHandle};
use tauri::api::path::app_data_dir;
use crate::{gpt, notify, whisper};
use crate::audio_utils::{resample_audio, TARGET_SAMPLE_RATE};
use crate::gpt::{GptClient, messages_setup};
use crate::screenshot::{screenshot};
use crate::whisper::{WHISPER_CONTEXT};

const SCREENSHOT_FILE_NAME: &str = "derby_latest_screenshot.png";

pub fn user_speech_to_gpt_response(app_handle: &AppHandle) -> anyhow::Result<()> {
    // record audio in this thread until the hotkey is pressed again
    // ocr the screenshot
    let app_handle_clone = app_handle.clone();
    let screenshot_handle = spawn(move || {
        let app_data_dir_path = app_data_dir(&*app_handle_clone.config()).context("failed to get app data dir")?;
        let screenshot_path = app_data_dir_path.join(SCREENSHOT_FILE_NAME);

        return screenshot(screenshot_path);
    });

    let app_handle_clone = app_handle.clone();

    whisper::init_whisper_context(&app_handle);
    let ctx = WHISPER_CONTEXT.get().context("WhisperContext not initialized")?;
    let mut state = ctx.create_state().context("failed to create key")?;

    info!("Initialization complete, starting audio thread");
    let mut audio_recording = whisper::get_audio_recording(app_handle)?;

    if audio_recording.config.sample_rate != SampleRate(TARGET_SAMPLE_RATE as u32) {
        audio_recording = resample_audio(audio_recording);
    } else {
        info!("Target sample rate is: {}. Audio sample rate is already {}. Not resampling", TARGET_SAMPLE_RATE, audio_recording.config.sample_rate.0);
    }


    let speech_text = whisper::speech_to_text(audio_recording, &mut state);
    info!("Speech to text: {}", speech_text);


    let screenshot_path = match screenshot_handle.join() {
        Ok(Ok(screenshot_actual_path)) => screenshot_actual_path,
        Ok(Err(e)) => {
            notify!("Failed to get screenshot", "Try enabling screen recording permissions", &app_handle_clone.config().tauri.bundle.identifier.clone());
            error!("Failed to get screenshot: {:?}", e);
            return Err(e);
        },
        Err(e) => {
            notify!("Failed to get screenshot", "Please try again",  &app_handle_clone.config().tauri.bundle.identifier.clone());
            error!("Failed to join screenshot handle: {:?}", e);
            return Err(anyhow!("Failed to join screenshot handle"));
        }
    };


    let mut messages = messages_setup();
    let user_message = gpt::create_chat_completion_request_msg(speech_text, Role::User);
    messages.push(user_message);
    let rt = tokio::runtime::Runtime::new().unwrap();

    let app_handle_2 = app_handle_clone.clone();
    rt.block_on(async {
        // Create an instance of GptClient
        let gpt_client = GptClient::new(app_handle_clone);

        // Call the method to get the GPT response
        match gpt_client.get_gpt_response(messages, screenshot_path, app_handle_2).await {
            Ok(_) => println!("GPT response received successfully"),
            Err(e) => eprintln!("Error while getting GPT response: {}", e),
        }
    });
    Ok(())
}

