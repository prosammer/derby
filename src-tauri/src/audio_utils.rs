use std::fs::File;
use rodio::{Decoder, OutputStream, Sink};
use bytes::Bytes;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use log::info;
use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction};

pub const TARGET_SAMPLE_RATE: usize = 16000;
fn _clamp(value: f32, min: f32, max: f32) -> f32 {
    value.min(max).max(min)
}

pub fn _make_audio_louder(audio_samples: &[f32], gain: f32) -> Vec<f32> {
    audio_samples
        .iter()
        .map(|sample| {
            let louder_sample = sample * gain;
            _clamp(louder_sample, -1.0, 1.0)
        })
        .collect()
}

fn _high_pass_filter(data: &mut Vec<f32>, cutoff: f32, sample_rate: f32) {
    const M_PI: f32 = std::f32::consts::PI;

    let rc = 1.0 / (2.0 * M_PI * cutoff);
    let dt = 1.0 / sample_rate;
    let alpha = dt / (rc + dt);

    let mut y = data[0];

    for i in 1..data.len() {
        y = alpha * (y + data[i] - data[i - 1]);
        data[i] = y;
    }
}

pub fn _convert_stereo_to_mono_audio(samples: Vec<f32>) -> Result<Vec<f32>, &'static str> {
    let mono_samples: Vec<f32> = samples
        .chunks_exact(2)
        .map(|x| (x[0] + x[1]) / 2.0)
        .collect();

    // If there's an odd number of samples, append the last sample as is.
    if samples.len() % 2 != 0 {
        let last_sample = samples[samples.len() - 1];
        return Ok([mono_samples, vec![last_sample]].concat());
    }

    Ok(mono_samples)
}

pub fn _play_audio_bytes(audio_bytes: Bytes) {
    let cursor = Cursor::new(audio_bytes);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let source = Decoder::new(cursor).unwrap();
    sink.append(source);

    sink.sleep_until_end();
}

pub fn _play_audio_f32_vec(data: Vec<f32>, sample_rate: u32) {
    println!("Playing audio");
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let source = rodio::buffer::SamplesBuffer::new(1, sample_rate, data);
    sink.append(source);

    sink.sleep_until_end();
}

pub fn resample_audio(mut audio_recording: AudioRecording) -> AudioRecording {
    info!("Resampling audio from {} to {}", audio_recording.config.sample_rate.0, TARGET_SAMPLE_RATE);

    let sinc_len = 256;
    let f_cutoff = 0.95;
    let params = SincInterpolationParameters {
        sinc_len,
        f_cutoff,
        oversampling_factor: 160,
        interpolation: SincInterpolationType::Cubic,
        window: WindowFunction::BlackmanHarris2,
    };

    let mut resampler = SincFixedIn::<f32>::new(
        TARGET_SAMPLE_RATE as f64 / audio_recording.config.sample_rate.0 as f64,
        1.0,
        params,
        audio_recording.audio_data.len(),
        1,
    ).expect("Failed to create resampler");

    let audio_data = vec![audio_recording.audio_data.to_vec()];

    let audio_vec_resampled = resampler.process(&audio_data, None).unwrap();
    audio_recording.audio_data = audio_vec_resampled.into_iter().flatten().collect();
    audio_recording.config.sample_rate.0 = TARGET_SAMPLE_RATE as u32;
    audio_recording
}

pub fn _play_audio_from_wav(path: PathBuf) {
    if !path.exists() || !path.is_file() {
        println!("This was the path given: {:?}", path);
        println!("File does not exist or is not a file");
        return;
    }

    let mut file = File::open(path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    _play_audio_bytes(Bytes::from(buffer));
}

pub fn _write_to_wav(audio_samples: &AudioRecording, filename: &str) -> Result<(), hound::Error> {

    info!("Writing audio to {}", filename);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: TARGET_SAMPLE_RATE as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(filename, spec)?;

    for &sample in audio_samples.audio_data.iter() {
        writer.write_sample((sample * i16::MAX as f32) as i16)?;
    }
    writer.finalize()?;
    info!("Finished writing audio to {}", filename);
    Ok(())
}

pub fn _read_from_wav(filename: &str) {
    let mut reader = hound::WavReader::open(filename).expect("failed to open file");
    #[allow(unused_variables)]
        let hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample,
        ..
    } = reader.spec();

    println!("Reader Spec: {:?}", reader.spec());
    // Convert the audio to floating point samples.
    let audio = whisper_rs::convert_integer_to_float_audio(
        &reader
            .samples::<i16>()
            .map(|s| s.expect("invalid sample"))
            .collect::<Vec<_>>(),
    );

    // Convert audio to 16KHz mono f32 samples, as required by the model.
    // These utilities are provided for convenience, but can be replaced with custom conversion logic.
    // SIMD variants of these functions are also available on nightly Rust (see the docs).
    if channels == 2 {
        whisper_rs::convert_stereo_to_mono_audio(&audio).unwrap();
    } else if channels != 1 {
        panic!(">2 channels unsupported");
    }

    if sample_rate != 16000 {
        panic!("sample rate must be 16KHz");
    }
}