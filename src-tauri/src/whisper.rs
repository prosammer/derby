extern crate anyhow;
extern crate cpal;
extern crate ringbuf;

use std::mem::MaybeUninit;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{Consumer, SharedRb};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperState};

use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use anyhow::Error;
use cpal::{Stream, StreamConfig};
use once_cell::sync::OnceCell;
use std::path::Path;
use std::sync::mpsc::Sender;
use crate::audio_utils;
use crate::audio_utils::{convert_stereo_to_mono_audio, make_audio_louder};

pub const LATENCY_MS: f32 = 30000.0;
pub static WHISPER_CONTEXT: OnceCell<WhisperContext> = OnceCell::new();

pub fn init_whisper_context() {
    let whisper_path_str = "src/ggml-base.en.bin";
    let whisper_path = Path::new(whisper_path_str);
    if !whisper_path.exists() && !whisper_path.is_file() {
        panic!("expected a whisper directory")
    }
    let ctx = WhisperContext::new(whisper_path_str).expect("Failed to open model");
    if WHISPER_CONTEXT.get().is_none() {
        WHISPER_CONTEXT.set(ctx).expect("Failed to set WhisperContext");
    }
}


pub fn send_system_audio_to_channel(audio_tx: Sender<Vec<f32>>, hotkey_count: Arc<Mutex<i32>>) {
    let (config, mut consumer, input_stream) = setup_audio().expect("Failed to setup audio");

    // Ensure the initial speech is finished before starting the input stream
    input_stream.play().expect("Failed to play input stream");
    // Remove the initial samples
    consumer.clear();
    sleep(Duration::from_millis(2000));

    loop {
        let samples: Vec<f32> = consumer.iter().map(|x| *x).collect();
        // TODO: Instead of removing every second sample, just set the input data fn to only push every second sample
        let samples = convert_stereo_to_mono_audio(samples).unwrap();
        let mut samples = make_audio_louder(&samples, 2.0);

        let sampling_freq = config.sample_rate.0 as f32 / 2.0; // TODO: Divide by 2 because of stereo to mono

        if audio_utils::vad_simple(&mut samples, sampling_freq as usize, 1000) {
            // the last 1000ms of audio was silent and there was talking before it
            println!("Speech detected!");
            audio_tx.send(samples).expect("Failed to send audio to channel");
            consumer.clear();
        } else {
            // Else, there is just silence. The samples should be deleted
            println!("Silence Detected!");
            sleep(Duration::from_secs(1));
            // drop the oldest second of audio
            consumer.pop_iter().take(1000).for_each(drop);
        }

        if hotkey_count.lock().unwrap().clone() % 2 == 0 {
            println!("Hotkey pressed, stopping audio");
            input_stream.pause().expect("Failed to pause input stream");
            break;
        }
    }
}

fn setup_audio() -> Result<(StreamConfig, Consumer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>, Stream), Error> {
    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .expect("failed to get default input device");
    println!("Using default input device: \"{}\"", input_device.name()?);
    let config = input_device
        .default_input_config()
        .expect("Failed to get default input config").config();
    println!("Default input config: {:?}", config);

    // Top level variables
    let latency_frames = (LATENCY_MS / 1_000.0) * config.sample_rate.0 as f32;
    let latency_samples = latency_frames as usize * config.channels as usize;
    println!("{}", latency_samples);

    // The buffer to share samples
    let ring = SharedRb::new(latency_samples * 2);
    let (mut producer, consumer) = ring.split();

    // Setup microphone callback
    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let mut output_fell_behind = false;
        for &sample in data {
            if producer.push(sample).is_err() {
                output_fell_behind = true;
            }
        }
        if output_fell_behind {
            eprintln!("output stream fell behind: try increasing latency");
        }
    };

    // Build streams.
    println!(
        "Attempting to build both streams with f32 samples and `{:?}`.",
        config
    );
    println!("Setup input stream");
    let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn, None)?;
    Ok((config, consumer, input_stream))
}


pub fn speech_to_text(samples: &Vec<f32>, state: &mut WhisperState) -> String {
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

    state
        .full(params, &*samples)
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