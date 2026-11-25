//! Global hit/miss sound effects for all aim-training tasks.
//! Scales 16-bit PCM WAV samples in memory according to `Settings.audio.volume`
//! and streams them to the system audio player on a background thread so gameplay
//! never blocks.

use std::io::Write;
use std::process::{Command, Stdio};

use bevy::prelude::*;

use crate::core::config::Settings;
use crate::core::hit::{Miss, TargetHit};
use crate::core::state::AppState;

const EMBEDDED_HIT_WAV: &[u8] = include_bytes!("../../assets/audio/hitmarker.wav");
const EMBEDDED_MISS_WAV: &[u8] = include_bytes!("../../assets/audio/miss.wav");

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_sound_bank).add_systems(
            Update,
            play_hit_miss_sounds.run_if(in_state(AppState::Playing)),
        );
    }
}

#[derive(Resource)]
struct SoundBank {
    hit_wav: Vec<u8>,
    miss_wav: Vec<u8>,
}

fn load_wav_or_fallback(path: &str, fallback: &[u8]) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|_| fallback.to_vec())
}

fn load_sound_bank(mut commands: Commands) {
    commands.insert_resource(SoundBank {
        hit_wav: load_wav_or_fallback("assets/audio/hitmarker.wav", EMBEDDED_HIT_WAV),
        miss_wav: load_wav_or_fallback("assets/audio/miss.wav", EMBEDDED_MISS_WAV),
    });
}

fn play_hit_miss_sounds(
    mut hits: MessageReader<TargetHit>,
    mut misses: MessageReader<Miss>,
    settings: Res<Settings>,
    bank: Option<Res<SoundBank>>,
) {
    let mut hit_fired = false;
    for _ in hits.read() {
        hit_fired = true;
    }
    let mut miss_fired = false;
    for _ in misses.read() {
        miss_fired = true;
    }

    if !settings.audio.enabled || settings.audio.volume == 0 {
        return;
    }
    let Some(bank) = bank else {
        return;
    };

    let gain = (settings.audio.volume.min(100) as f32) / 100.0;

    if hit_fired {
        play_wav_async(scale_pcm16_wav(&bank.hit_wav, gain));
    } else if miss_fired {
        play_wav_async(scale_pcm16_wav(&bank.miss_wav, gain));
    }
}

/// Finds the byte range `(start, end)` of the PCM payload inside a RIFF WAVE file's `data` chunk.
fn find_wav_data_chunk(wav: &[u8]) -> Option<(usize, usize)> {
    if wav.len() < 12 || &wav[0..4] != b"RIFF" || &wav[8..12] != b"WAVE" {
        return None;
    }
    let mut offset = 12;
    while offset + 8 <= wav.len() {
        let id = &wav[offset..offset + 4];
        let size = u32::from_le_bytes([
            wav[offset + 4],
            wav[offset + 5],
            wav[offset + 6],
            wav[offset + 7],
        ]) as usize;
        let data_start = offset + 8;
        let data_end = data_start.saturating_add(size).min(wav.len());
        if id == b"data" {
            return Some((data_start, data_end));
        }
        // Chunks are word-aligned (padded to even byte boundary)
        let step = size + (size & 1);
        offset = data_start.saturating_add(step);
    }
    None
}

/// Scales 16-bit signed little-endian PCM samples in a WAV buffer by `gain` (`0.0..=1.0`).
pub fn scale_pcm16_wav(wav: &[u8], gain: f32) -> Vec<u8> {
    let mut out = wav.to_vec();
    if (gain - 1.0).abs() < 1e-4 {
        return out;
    }
    let Some((start, end)) = find_wav_data_chunk(&out) else {
        return out;
    };
    let clamped_gain = gain.clamp(0.0, 1.0);
    let sample_end = start + ((end - start) & !1);
    let (samples, _) = out[start..sample_end].as_chunks_mut::<2>();
    for chunk in samples {
        let sample = i16::from_le_bytes(*chunk);
        let scaled = (sample as f32 * clamped_gain)
            .round()
            .clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        *chunk = scaled.to_le_bytes();
    }
    out
}

#[cfg(target_os = "linux")]
fn play_wav_async(wav_bytes: Vec<u8>) {
    std::thread::spawn(move || {
        if let Ok(mut child) = Command::new("aplay")
            .args(["-q", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(&wav_bytes);
            }
            let _ = child.wait();
        }
    });
}

#[cfg(not(target_os = "linux"))]
fn play_wav_async(_wav_bytes: Vec<u8>) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales_embedded_wav_samples() {
        let half = scale_pcm16_wav(EMBEDDED_HIT_WAV, 0.5);
        assert_eq!(half.len(), EMBEDDED_HIT_WAV.len());
        let (start, end) = find_wav_data_chunk(&half).expect("valid wav data chunk");
        assert!(end > start);

        let muted = scale_pcm16_wav(EMBEDDED_HIT_WAV, 0.0);
        assert!(muted[start..end].iter().all(|&b| b == 0));
    }
}
