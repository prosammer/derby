use std::sync::{Arc, Mutex};
use std::thread::spawn;
use async_openai::types::Role;
use tauri::{AppHandle};
use crate::gpt;
use crate::gpt::{get_gpt_response, messages_setup};

pub fn user_speech_to_gpt_response(app_handle: AppHandle, hotkey_count: Arc<Mutex<i32>>) {
    let mut messages = messages_setup();

    let user_speech_to_text = Arc::new(Mutex::new(String::new()));
    let user_speech_to_text_clone = user_speech_to_text.clone();

    // ocr the screenshot
    let ocr_thread = spawn(|| {
        // ocr_screenshot()
    });


    // let window_ocr_text_list = ocr_thread.join().unwrap().unwrap();
    // let window_ocr_text = window_ocr_text_list.join(" ");
    // println!("{}", window_ocr_text);
    // messages.push(gpt::create_chat_completion_request_msg(window_ocr_text, Role::User));
    //


    println!("User Speech: {}", user_speech_to_text_clone.lock().unwrap());
    messages.push(gpt::create_chat_completion_request_msg(user_speech_to_text_clone.lock().unwrap().clone(), Role::User));


    // finally, the entire_text is sent to GPT and the response is copy/pasted
    let gpt_response = get_gpt_response(messages).expect("Failed to get GPT response");
    println!("GPT Response: {}", gpt_response.content.as_ref().unwrap());
    // speak_string(gpt_response.content.as_ref().unwrap().clone());
    // pbcopy the gpt_response
}

