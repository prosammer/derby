extern crate anyhow;
extern crate cpal;
extern crate ringbuf;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperState};

use std::sync::{Arc, Mutex};
use anyhow::{Result, Error};
use cpal::{Stream, StreamConfig};
use once_cell::sync::OnceCell;
use tauri::{AppHandle, Icon, Manager};
use tauri::api::path::app_data_dir;
use crate::{TranscriptionMode, TranscriptionState};

pub const LATENCY_MS: f32 = 30000.0;
pub const WHISPER_FILE_NAME: &str = "ggml-base.en.bin";
pub static WHISPER_CONTEXT: OnceCell<WhisperContext> = OnceCell::new();

pub fn init_whisper_context(app_handle: &AppHandle) {
    let config = app_handle.config();
    let app_data_dir_path = app_data_dir(&*config).expect("Failed to get app data dir");

    let whisper_path = app_data_dir_path.join(WHISPER_FILE_NAME);

    if !whisper_path.exists() && !whisper_path.is_file() {
        panic!("expected a whisper directory")
        // TODO: Should display error to user
    }

    if WHISPER_CONTEXT.get().is_none() {
        let ctx = WhisperContext::new(whisper_path.to_str().unwrap()).expect("Failed to open model");
        WHISPER_CONTEXT.set(ctx).expect("Failed to set WhisperContext");
    }
}


pub fn get_audio_recording(app_handle: AppHandle) -> Result<Vec<f32>, anyhow::Error> {
    let (buffer, input_stream, config) = setup_audio().expect("Failed to setup audio");

    loop {
        let app_state = app_handle.state::<TranscriptionState>();
        // Lock the mode to check its value
        let current_mode = {
            let mode_lock = app_state.mode.lock().unwrap();
            (*mode_lock).clone() // Clone the current mode to avoid moving it
        };

        // Check if the mode is not Listening (which means recording should stop)
        if current_mode != TranscriptionMode::Listening {
            drop(current_mode);
            input_stream.pause().expect("Failed to pause stream");
            return Ok(buffer.lock().unwrap().clone());
        }
        // It's important to not hold the lock while sleeping to avoid deadlocks
        drop(current_mode); // Explicitly drop the lock before sleeping
        std::thread::sleep(std::time::Duration::from_millis(200)); // Sleep to prevent a busy-wait loop
    }
}

fn setup_audio() -> Result<(Arc<Mutex<Vec<f32>>>, Stream, StreamConfig), Error> {
    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .expect("failed to get default input device");
    println!("Using default input device: \"{}\"", input_device.name().unwrap());
    let config = input_device
        .default_input_config()
        .expect("Failed to get default input config").config();
    println!("Default input config: {:?}", config);


    // The buffer to share samples
    let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
    let buffer_clone = Arc::clone(&buffer);


    let buffer_clone_for_closure = Arc::clone(&buffer_clone);
    // Setup microphone callback
    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let mut buffer = buffer_clone_for_closure.lock().unwrap();
        for &sample in data {
            buffer.push(sample);
        }
    };

    // Build streams.
    println!(
        "Attempting to build both streams with f32 samples and `{:?}`.",
        config
    );
    let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn, None).unwrap();
    println!("Successfully built stream.");
    Ok((buffer_clone, input_stream, config))
}

pub fn speech_to_text(samples: &[f32], state: &mut WhisperState) -> String {
    let mut params = FullParams::new(SamplingStrategy::default());
    params.set_print_progress(false);
    params.set_print_special(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_suppress_blank(true);
    params.set_language(Some("en"));
    params.set_token_timestamps(true);
    params.set_duration_ms(LATENCY_MS as i32);
    params.set_no_context(true);
    params.set_n_threads(8);

    //params.set_no_speech_thold(0.3);
    //params.set_split_on_word(true);

    let audio = if samples.len() % 2 == 0 {
        samples.to_vec()
    } else {
        samples[..samples.len() - 1].to_vec()
    };

    state
        .full(params, &audio)
        .expect("failed to convert samples");

    let num_tokens = state.full_n_tokens(0).expect("Error");
    let words = (1..num_tokens - 1)
        .map(|i| state.full_get_token_text(0, i).expect("Error"))
        .collect::<String>();

    words
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

// https://discord.com/channels/616186924390023171/1087197552094490754/1087378596626173962
#[tauri::command]
pub fn request_mic_permissions() -> bool {
    use std::sync::mpsc;

    use block::ConcreteBlock;
    use cocoa::base::YES;
    use objc::runtime::BOOL;
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};

    let (tx, mut rx) = mpsc::channel();

    unsafe {
        let av_audio_session_class = Class::get("AVAudioSession").unwrap();
        let shared_instance: *mut Object = msg_send![av_audio_session_class, sharedInstance];

        let block = ConcreteBlock::new(move |granted: BOOL| {
            println!("Permission granted: {}", granted == YES);
            tx.send(()).unwrap();
        });
        let block = block.copy();

        let _: () = msg_send![shared_instance, requestRecordPermission: block];
    }

    // Wait for the callback to be called
    let response = rx.recv().unwrap();
    return response == ();
}