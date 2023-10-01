use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread::spawn;
use async_openai::types::Role;
use tauri::{AppHandle};
use crate::{gpt, whisper};
use crate::audio_utils::{play_audio_f32_vec};
use crate::gpt::{get_gpt_response, messages_setup};
use crate::screenshot::{ocr_screenshot};
use crate::text_to_speech::{speak_string};
use crate::whisper::WHISPER_CONTEXT;

pub fn user_speech_to_gpt_response(handle: AppHandle, hotkey_count: Arc<Mutex<i32>>) {
    let mut messages = messages_setup();

    let (audio_tx, audio_rx) = channel();

    let user_speech_to_text = Arc::new(Mutex::new(String::new()));
    let user_speech_to_text_clone = user_speech_to_text.clone();

    // ocr the screenshot
    let window_text_list = ocr_screenshot().unwrap();
    let window_text = window_text_list.join(" ");
    println!("{}", window_text);

    messages.push(gpt::create_chat_completion_request_msg(window_text, Role::User));


    whisper::init_whisper_context();
    let ctx = WHISPER_CONTEXT.get().expect("WhisperContext not initialized");
    let mut state = ctx.create_state().expect("failed to create key");

    println!("Initialization complete, starting audio thread");
    // start cpal audio recording to channel
    // when the hotkey_rx receives a message, the audio thread is stopped.
    let tray_handle = handle.tray_handle().clone();
    spawn(|| {
        whisper::send_system_audio_to_channel(audio_tx, hotkey_count, tray_handle);
    });

    // This will keep looping until the hotkey is pressed again (there is nothing in the channel)
    loop {
        if let Ok(audio) = audio_rx.recv() {
            println!("Received audio");
            play_audio_f32_vec(audio.clone(), 24000);
            let text = whisper::speech_to_text(&audio, &mut state);
            user_speech_to_text_clone.lock().unwrap().push_str(&text);
        } else {
            break;
        }
    }

    println!("User Speech: {}", user_speech_to_text_clone.lock().unwrap());
    messages.push(gpt::create_chat_completion_request_msg(user_speech_to_text_clone.lock().unwrap().clone(), Role::User));


    // finally, the entire_text is sent to GPT and the response is copy/pasted
    let gpt_response = get_gpt_response(messages).expect("Failed to get GPT response");
    println!("GPT Response: {}", gpt_response.content.as_ref().unwrap());
    speak_string(gpt_response.content.as_ref().unwrap().clone());
    // pbcopy the gpt_response
}

