extern crate anyhow;
extern crate cpal;
extern crate ringbuf;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperState};

use std::sync::{Arc, Mutex};
use anyhow::{Result, Error};
use cpal::{Stream, StreamConfig};
use once_cell::sync::OnceCell;
use tauri::{AppHandle, Icon};

pub const LATENCY_MS: f32 = 30000.0;
pub const WHISPER_PATH: &str = "resources/ggml-base.en.bin";
const APP_ICON_DEFAULT: &str = "resources/assets/sigma_master_512.png";
const APP_ICON_RECORDING: &str = "resources/assets/sigma_master_green_512.png";
const SESSION_START_SOUND_PATH: &str = "resources/assets/session_start.wav";
pub static WHISPER_CONTEXT: OnceCell<WhisperContext> = OnceCell::new();

pub fn init_whisper_context(app_handle: &AppHandle) {
    let resource_path = app_handle.path_resolver()
        .resolve_resource(WHISPER_PATH)
        .expect("Failed to resolve whisper model resource path");

    if !resource_path.exists() && !resource_path.is_file() {
        panic!("expected a whisper directory")
        // TODO: Should display error to user
    }

    if WHISPER_CONTEXT.get().is_none() {
        let ctx = WhisperContext::new(resource_path.to_str().unwrap()).expect("Failed to open model");
        WHISPER_CONTEXT.set(ctx).expect("Failed to set WhisperContext");
    }
}


pub fn get_audio_recording(hotkey_count: Arc<Mutex<u32>>, app_handle: AppHandle) -> Result<Vec<f32>> {
    let (buffer, input_stream, config) = setup_audio().expect("Failed to setup audio");
    loop {
        // check if the hotkey has been pressed twice
        if hotkey_count.lock().unwrap().clone() % 2 == 0 {
            println!("Hotkey pressed, stopping audio");
            input_stream.pause().expect("Failed to pause stream");
            // let samples = convert_stereo_to_mono_audio(buffer).unwrap();
            // let samples = make_audio_louder(&buffer, 1.0);
            // let sampling_freq = config.sample_rate.0 as f32 / 2.0; // TODO: Divide by 2 because of stereo to mono

            return Ok(buffer.lock().unwrap().clone());
        }
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

fn set_icon(path_str: &str, app_handle: &AppHandle, template: bool) {

    let resolved_path = app_handle.path_resolver()
        .resolve_resource(path_str)
        .expect("Failed to resolve session start sound resource path");

    if resolved_path.exists() && resolved_path.is_file() {
        let icon = Icon::File(resolved_path);
        if !template {
            app_handle.tray_handle().set_icon_as_template(template).expect("Failed to set icon as template");
        }
        app_handle.tray_handle().set_icon(icon).expect("Failed to set icon");
        if template {
            app_handle.tray_handle().set_icon_as_template(template).expect("Failed to set icon as template");
        }
    } else {
        println!("Icon path does not exist: {}", path_str);
    }
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