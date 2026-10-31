use macroquad::audio::{load_sound_from_bytes, play_sound_once, Sound};
use std::collections::HashMap;
use std::f32::consts::PI;

/// Génère un en-tête WAV PCM 16-bit Mono (chacun des échantillons prend 2 octets en Little Endian)
fn create_wav_header(sample_rate: u32, num_samples: u32) -> Vec<u8> {
    let bits_per_sample: u16 = 16;
    let num_channels: u16 = 1;
    let block_align: u16 = num_channels * (bits_per_sample / 8);
    let byte_rate: u32 = sample_rate * block_align as u32;
    let data_size: u32 = num_samples * block_align as u32;
    let file_size: u32 = 36 + data_size;

    let mut header = Vec::with_capacity(44);

    header.extend_from_slice(b"RIFF");
    header.extend_from_slice(&file_size.to_le_bytes());
    header.extend_from_slice(b"WAVEfmt ");
    header.extend_from_slice(&16u32.to_le_bytes());
    header.extend_from_slice(&1u16.to_le_bytes());
    header.extend_from_slice(&num_channels.to_le_bytes());
    header.extend_from_slice(&sample_rate.to_le_bytes());
    header.extend_from_slice(&byte_rate.to_le_bytes());
    header.extend_from_slice(&block_align.to_le_bytes());
    header.extend_from_slice(&bits_per_sample.to_le_bytes());
    header.extend_from_slice(b"data");
    header.extend_from_slice(&data_size.to_le_bytes());

    header
}

/// Génère le buffer WAV du son de sélection/clic (PCM 16-bit)
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

        let val = (sq * 12000.0 * env).clamp(-32768.0, 32767.0) as i16;
        wav.extend_from_slice(&val.to_le_bytes());
    }
    wav
}

/// Génère le buffer WAV du son de lancement (PCM 16-bit)
pub fn generate_launch_sound() -> Vec<u8> {
    let sample_rate = 22050;
    let duration = 0.12;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut wav = create_wav_header(sample_rate, num_samples as u32);

    for i in 0..num_samples {
        let progress = i as f32 / num_samples as f32;
        let freq = 600.0 * (1.0 - progress * 0.7);
        let t = i as f32 / sample_rate as f32;
        let sq = if (t * freq * 2.0 * PI).sin() > 0.0 { 1.0 } else { -1.0 };
        let env = 1.0 - progress;

        let val = (sq * 15000.0 * env).clamp(-32768.0, 32767.0) as i16;
        wav.extend_from_slice(&val.to_le_bytes());
    }
    wav
}

/// Génère le buffer WAV d'un son de fusion selon la valeur du blob (PCM 16-bit)
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

        let val = (sq * 16000.0 * env).clamp(-32768.0, 32767.0) as i16;
        wav.extend_from_slice(&val.to_le_bytes());
    }
    wav
}

/// Fonction utilitaire pour jouer un son de fusion depuis n'importe quelle HashMap
pub fn play_merge(merges: &HashMap<u32, Sound>, value: u32) {
    if let Some(snd) = merges.get(&value) {
        play_sound_once(snd);
    }
}

/// Structure regroupant tous les sons pré-chargés du jeu
pub struct AudioSystem {
    pub select: Sound,
    pub launch: Sound,
    pub merges: HashMap<u32, Sound>,
}

impl AudioSystem {
    pub async fn new() -> Self {
        let select_bytes = generate_select_sound();
        let select = load_sound_from_bytes(&select_bytes).await.unwrap();

        let launch_bytes = generate_launch_sound();
        let launch = load_sound_from_bytes(&launch_bytes).await.unwrap();

        let mut merges = HashMap::new();
        for i in 1..=13 { // Génère jusqu'à 8192 (2^13)
            let val = 2u32.pow(i);
            let bytes = generate_merge_sound(val);
            if let Ok(snd) = load_sound_from_bytes(&bytes).await {
                merges.insert(val, snd);
            }
        }

        Self {
            select,
            launch,
            merges,
        }
    }

    pub fn play_select(&self) {
        play_sound_once(&self.select);
    }

    pub fn play_launch(&self) {
        play_sound_once(&self.launch);
    }

    pub fn play_merge(&self, value: u32) {
        play_merge(&self.merges, value);
    }
}