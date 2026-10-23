use macroquad::audio::{play_sound_once, Sound};
use std::collections::HashMap;
use std::f32::consts::PI;

fn create_wav_header(sample_rate: u32, num_samples: u32) -> Vec<u8> {
    let data_size = num_samples;
    let file_size = 36 + data_size;
    let mut header = Vec::with_capacity(44);

    header.extend_from_slice(b"RIFF");
    header.extend_from_slice(&(file_size as u32).to_le_bytes());
    header.extend_from_slice(b"WAVEfmt ");
    header.extend_from_slice(&(16u32).to_le_bytes());
    header.extend_from_slice(&(1u16).to_le_bytes());
    header.extend_from_slice(&(1u16).to_le_bytes());
    header.extend_from_slice(&sample_rate.to_le_bytes());
    header.extend_from_slice(&sample_rate.to_le_bytes());
    header.extend_from_slice(&(1u16).to_le_bytes());
    header.extend_from_slice(&(8u16).to_le_bytes());
    header.extend_from_slice(b"data");
    header.extend_from_slice(&data_size.to_le_bytes());

    header
}

pub fn generate_select_sound() -> Vec<u8> {
    let sample_rate = 22050;
    let duration = 0.05;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut wav = create_wav_header(sample_rate, num_samples as u32);

    for i in 0..num_samples {
        let progress = i as f32 / num_samples as f32;
        let t = i as f32 / sample_rate as f32;
        let freq = 220.0 * (1.0 - progress * 0.36);
        let sq = if (t * freq * 2.0 * PI).sin() > 0.0 { 1.0 } else { -1.0 };
        let env = (1.0 - progress).powi(2);
        let val = 128.0 + (sq * 45.0 * env);
        wav.push(val.clamp(0.0, 255.0) as u8);
    }
    wav
}

pub fn generate_launch_sound() -> Vec<u8> {
    let sample_rate = 22050;
    let duration = 0.12;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut wav = create_wav_header(sample_rate, num_samples as u32);

    for i in 0..num_samples {
        let progress = i as f32 / num_samples as f32;
        let freq = 600.0 * (1.0 - progress * 0.7);
        let t = i as f32 / sample_rate as f32;
        let sample = if (t * freq * 2.0 * PI).sin() > 0.0 { 190 } else { 65 };
        let env = 1.0 - progress;
        let val = 128.0 + (sample as f32 - 128.0) * env;
        wav.push(val as u8);
    }
    wav
}

pub fn generate_merge_sound(value: u32) -> Vec<u8> {
    let sample_rate = 22050;
    let duration = 0.15;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut wav = create_wav_header(sample_rate, num_samples as u32);

    let level = (value as f32).log2().max(1.0);
    let base_freq = 220.0 * (2.0f32).powf((level - 1.0) * 0.25);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let progress = i as f32 / num_samples as f32;
        let env = (1.0 - progress).powi(2);

        let freq = if progress < 0.5 { base_freq } else { base_freq * 1.25 };
        let sq = if (t * freq * 2.0 * PI).sin() > 0.0 { 1.0 } else { -1.0 };

        let sample = 128.0 + (sq * 60.0 * env);
        wav.push(sample.clamp(0.0, 255.0) as u8);
    }
    wav
}

pub fn play_merge_sound(sounds: &HashMap<u32, Sound>, value: u32) {
    if let Some(snd) = sounds.get(&value) {
        play_sound_once(snd);
    }
}