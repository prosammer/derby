extern crate anyhow;
extern crate cpal;
extern crate ringbuf;

use std::fs;
use std::fs::File;
use std::io::{Read};
use std::path::{PathBuf};
use cpal::traits::{DeviceTrait, HostTrait};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperState};

use std::sync::{Arc, Mutex};
use anyhow::{Result, Error, anyhow};
use cpal::{Stream, StreamConfig};
use log::{error, info};
use once_cell::sync::OnceCell;
use reqwest::{StatusCode};
use tauri::{AppHandle, Manager};
use tauri::api::path::app_data_dir;
use crate::{TranscriptionMode, TranscriptionState};
use sha1::{Sha1, Digest};
use std::result::Result as StdResult;

pub const LATENCY_MS: f32 = 30000.0;
pub const WHISPER_FILE_NAME: &str = "ggml-base.en.bin";
pub const WHISPER_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin";
pub const WHISPER_FILE_SIZE: u64 = 147964211;
pub static WHISPER_CONTEXT: OnceCell<WhisperContext> = OnceCell::new();


#[derive(Clone, serde::Serialize)]
struct EventPayload {
    message: String,
}

#[tauri::command]
pub async fn handle_model_file(app_handle: AppHandle) -> StdResult<(), ()> {
    let app_data_dir = app_data_dir(&*app_handle.config()).expect("Failed to get app data dir");
    let path = app_data_dir.join(WHISPER_FILE_NAME);
    let path_str = path.to_str().unwrap();

    let size_matches = |path: &PathBuf, size: u64| -> std::io::Result<bool> {
        let metadata = fs::metadata(path)?;
        Ok(metadata.len() == size)
    };

    let needs_download = if !path.exists() {
        info!("Model file does not exist at {}", path_str);
        true
    } else {
        match size_matches(&path, WHISPER_FILE_SIZE) {
            Ok(true) => {
                info!("Model file exists and size matches at {}", path_str);
                false
            }
            _ => {
                info!("Model file exists at {}, but size does not match", path_str);
                true
            }
        }
    };
    if needs_download {
        match download_model(&path, WHISPER_URL).await {
            Ok(_) => {
                info!("Model file downloaded successfully");
            }
            Err(e) => {
                error!("Failed to download model file: {}", e);
                return Err(());
            }
        }
    }
    Ok(())
}


pub fn _hash_matches(path: &PathBuf, sha: &str) -> Result<bool> {
    let mut hasher = Sha1::new();
    let mut file = File::open(&path)?;
    let mut buffer = [0; 8192];

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let result = hasher.finalize();
    let result_str = format!("{:x}", result);
    Ok(result_str == sha)
}

async fn download_model(path: &PathBuf, url: &str) -> Result<()> {
    info!("Downloading model file from {} to {:?}",url,  path);
    let response = reqwest::get(url).await?;

    match response.status() {
        StatusCode::OK => {
            info!("Response successful");
            let bytes = response.bytes().await?;
            tokio::fs::write(&path, bytes).await?;
        }
        _ => {
            error!("Response status was: {}", response.status());
            error!("Response was: {:?}", response);
            return Err(anyhow!("Response status was: {}", response.status()));
        }
    }
    Ok(())
}

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


pub struct AudioRecordingBuilder {
    pub audio_data: Arc<Mutex<Vec<f32>>>,
    pub input_stream: Option<Stream>,
    pub config: StreamConfig,
    pub sample_format: cpal::SampleFormat,
}

pub struct AudioRecording {
    pub audio_data: Vec<f32>,
    pub config: StreamConfig,
    pub sample_format: cpal::SampleFormat,
}

pub fn get_audio_recording(app_handle: AppHandle) -> Result<AudioRecording, Error> {
    let mut audio_recording_builder = setup_audio().expect("Failed to setup audio");

    loop {
        let app_state = app_handle.state::<TranscriptionState>();
        let mode_lock = app_state.mode.lock().unwrap();

        // Check if the mode is not Listening (which means recording should stop)
        if *mode_lock != TranscriptionMode::Listening {
            // Drop the stream so I can access the audio data
            audio_recording_builder.input_stream = None;

            let audio_data_result = Arc::try_unwrap(audio_recording_builder.audio_data);

            let audio_data = match audio_data_result {
                Ok(data) => data.into_inner().unwrap(),
                Err(_) => panic!("Failed to get audio data"),
            };

            let audio_recording = AudioRecording {
                audio_data,
                config: audio_recording_builder.config,
                sample_format: audio_recording_builder.sample_format,
            };

                return Ok(audio_recording);
            }
        std::thread::sleep(std::time::Duration::from_millis(200)); // Sleep to prevent a busy-wait loop
    }
}

fn setup_audio() -> Result<AudioRecordingBuilder, Error> {
    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .expect("failed to get default input device");
    info!("Using default input device: \"{}\"", input_device.name().unwrap());
    let supported_stream_config = input_device
        .default_input_config()
        .expect("Failed to get default input config");

    let config = supported_stream_config.config();
    info!("Default input config: {:?}", config);


    // The buffer to share samples
    let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
    let buffer_clone = Arc::clone(&buffer);


    // Setup microphone callback
    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let mut buffer = buffer_clone.lock().unwrap();
        for &sample in data {
            buffer.push(sample);
        }
    };

    // Build streams.
    info!(
        "Attempting to build both streams with f32 samples and `{:?}`.",
        config
    );
    let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn, None).unwrap();
    info!("Successfully built stream.");

    let audio_recording = AudioRecordingBuilder {
        audio_data: buffer,
        input_stream: Some(input_stream),
        config,
        sample_format: supported_stream_config.sample_format(),
    };
    Ok(audio_recording)
}

pub fn speech_to_text(audio_recording: AudioRecording, state: &mut WhisperState) -> String {
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

    let samples = audio_recording.audio_data;

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
    error!("an error occurred on stream: {}", err);
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

    let (tx, rx) = mpsc::channel();

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use tokio::runtime::Runtime;

    #[test]
    fn test_hash_matches() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let expected_hash = "09fac8dbfd27bd9b4d23a00eb648aa751789536d";
        assert!(hash_matches(&file_path, expected_hash).is_ok());
        assert_eq!(hash_matches(&file_path, expected_hash).unwrap(), true);
    }

    #[test]
    fn test_hash_matches_wrong_hash() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let wrong_hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        assert!(hash_matches(&file_path, wrong_hash).is_ok());
        assert_eq!(hash_matches(&file_path, wrong_hash).unwrap(), false);
    }

    #[test]
    fn test_hash_matches_invalid_path() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        assert!(hash_matches(&file_path, "any_hash").is_err());
    }
    #[test]
    fn test_download_model() {
        let rt = Runtime::new().unwrap();
        let path = PathBuf::from("test-5mb.bin");
        let url = "https://github.com/yourkin/fileupload-fastapi/raw/a85a697cab2f887780b3278059a0dd52847d80f3/tests/data/test-5mb.bin".to_string();

        // Run the async function in a synchronous test
        rt.block_on(download_model(&path, &url)).unwrap();

        // Check if the file was downloaded correctly
        let mut file = File::open(&path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        assert!(contents.len() > 0);

        // Clean up the test file
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_download_actual_model() {
        let rt = Runtime::new().unwrap();
        let path = PathBuf::from("test-5mb.bin");
        let url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin?download=true".to_string();

        // Run the async function in a synchronous test
        rt.block_on(download_model(&path, &url)).unwrap();

        // Check if the file was downloaded correctly
        let mut file = File::open(&path).unwrap();
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap();

        assert!(contents.len() > 0);

        // Clean up the test file
        std::fs::remove_file(path).unwrap();
    }
}
