use std::error::Error;
use std::fs::File;
use rodio::{Decoder, OutputStream, Sink};
use bytes::Bytes;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction};

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.min(max).max(min)
}

pub fn make_audio_louder(audio_samples: &[f32], gain: f32) -> Vec<f32> {
    audio_samples
        .iter()
        .map(|sample| {
            let louder_sample = sample * gain;
            clamp(louder_sample, -1.0, 1.0)
        })
        .collect()
}

fn high_pass_filter(data: &mut Vec<f32>, cutoff: f32, sample_rate: f32) {
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

pub fn vad_simple(
    mut pcmf32: &mut Vec<f32>,
    sample_rate: usize,
    last_ms: usize
) -> bool {
    let vad_thold = 0.6;
    let freq_thold = 100.0;

    let verbose = false;
    let n_samples = pcmf32.len();
    let n_samples_last = (sample_rate * last_ms) / 1000;

    if n_samples_last >= n_samples {
        // not enough samples - assume no speech
        return false;
    }

    if freq_thold > 0.0 {
        high_pass_filter(&mut pcmf32, freq_thold, sample_rate as f32);
    }

    let mut energy_all = 0.0f32;
    let mut energy_last = 0.0f32;

    for i in 0..n_samples {
        energy_all += pcmf32[i].clone().abs();

        if i >= n_samples - n_samples_last {
            energy_last += pcmf32[i].abs();
        }
    }

    energy_all /= n_samples as f32;
    energy_last /= n_samples_last as f32;

    if verbose {
        eprintln!(
            "vad_simple: energy_all: {}, energy_last: {}, vad_thold: {}, freq_thold: {}",
            energy_all, energy_last, vad_thold, freq_thold
        );
    }

    if energy_last > vad_thold * energy_all {
        return false;
    }

    true
}

pub fn convert_stereo_to_mono_audio(samples: Vec<f32>) -> Result<Vec<f32>, &'static str> {
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

pub fn play_audio_bytes(audio_bytes: Bytes) {
    let cursor = Cursor::new(audio_bytes);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let source = Decoder::new(cursor).unwrap();
    sink.append(source);

    sink.sleep_until_end();
}

pub fn play_audio_f32_vec(data: Vec<f32>, sample_rate: u32) {
    println!("Playing audio");
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let source = rodio::buffer::SamplesBuffer::new(1, sample_rate, data);
    sink.append(source);

    sink.sleep_until_end();
}

pub fn resample_audio(input: Vec<f32>, from_rate: usize, to_rate: usize) -> Result<Vec<f32>, Box<dyn Error>> {
    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        oversampling_factor: 128,
        interpolation: SincInterpolationType::Linear,
        window: WindowFunction::BlackmanHarris2,
    };

    let mut resampler = SincFixedIn::<f32>::new(
        to_rate as f64 / from_rate as f64,
        10.0,
        params,
        1024,
        1,
    ).unwrap();

    let output = resampler.process(&[input], None).unwrap();

    Ok(output[0].clone()) // Return the first (and only) inner vector
}

pub fn play_audio_from_wav(path: PathBuf) {
    if !path.exists() || !path.is_file() {
        println!("This was the path given: {:?}", path);
        println!("File does not exist or is not a file");
        return;
    }

    let mut file = File::open(path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    play_audio_bytes(Bytes::from(buffer));
}

pub fn write_to_wav(audio_vec: Vec<f32>, filename: &str) -> Result<(), hound::Error> {
    let original_sample_rate = 48000;
    let target_sample_rate = 16000;

    // Define the resampler
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
        target_sample_rate as f64 / original_sample_rate as f64,
        1.0,
        params,
        audio_vec.len(),
        1,
    ).map_err(|_| hound::Error::Unsupported)?;

    // Wrap the audio data in another vector to create a 2D structure
    let audio_data = vec![audio_vec];

    // Resample the audio
    let audio_vec_resampled = resampler.process(&audio_data, None).unwrap();

    // Flattening the resampled frames
    let audio_vec_flat: Vec<f32> = audio_vec_resampled.into_iter().flatten().collect();

    // Specify the WAV spec with the new sample rate
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: target_sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(filename, spec)?;

    // Write the resampled audio data to the WAV file
    for sample in audio_vec_flat {
        writer.write_sample((sample * std::i16::MAX as f32) as i16)?; // Convert f32 to i16
    }

    writer.finalize()?;
    Ok(())
}

pub fn read_from_wav(filename: &str) {
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
    let mut audio = whisper_rs::convert_integer_to_float_audio(
        &reader
            .samples::<i16>()
            .map(|s| s.expect("invalid sample"))
            .collect::<Vec<_>>(),
    );

    // Convert audio to 16KHz mono f32 samples, as required by the model.
    // These utilities are provided for convenience, but can be replaced with custom conversion logic.
    // SIMD variants of these functions are also available on nightly Rust (see the docs).
    if channels == 2 {
        audio = whisper_rs::convert_stereo_to_mono_audio(&audio).unwrap();
    } else if channels != 1 {
        panic!(">2 channels unsupported");
    }

    if sample_rate != 16000 {
        panic!("sample rate must be 16KHz");
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use bytes::Bytes;
    use crate::audio_utils::play_audio_bytes;

    #[test]
    fn test_play_audio() {
        let mut file = File::open("../assets/test.wav").unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        play_audio_bytes(Bytes::from(buffer));
    }
}
