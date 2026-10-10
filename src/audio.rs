use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustfft::{num_complex::Complex, FftPlanner};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::config::AudioConfig;

const AUDIO_BUFFER_SIZE: usize = 512;

static BASS_VAL: AtomicU32 = AtomicU32::new(0);
static MID_VAL: AtomicU32 = AtomicU32::new(0);
static TREBLE_VAL: AtomicU32 = AtomicU32::new(0);
static VOLUME_VAL: AtomicU32 = AtomicU32::new(0);
static PEAK_FREQ_VAL: AtomicU32 = AtomicU32::new(0);
static BPM_VAL: AtomicU32 = AtomicU32::new(0);

fn store_bass(val: f32) { BASS_VAL.store(val.to_bits(), Ordering::Relaxed); }
pub fn get_bass() -> f32 { f32::from_bits(BASS_VAL.load(Ordering::Relaxed)) }

fn store_mid(val: f32) { MID_VAL.store(val.to_bits(), Ordering::Relaxed); }
pub fn get_mid() -> f32 { f32::from_bits(MID_VAL.load(Ordering::Relaxed)) }

fn store_treble(val: f32) { TREBLE_VAL.store(val.to_bits(), Ordering::Relaxed); }
pub fn get_treble() -> f32 { f32::from_bits(TREBLE_VAL.load(Ordering::Relaxed)) }

fn store_volume(val: f32) { VOLUME_VAL.store(val.to_bits(), Ordering::Relaxed); }
pub fn get_volume() -> f32 { f32::from_bits(VOLUME_VAL.load(Ordering::Relaxed)) }

fn store_peak_freq(val: f32) { PEAK_FREQ_VAL.store(val.to_bits(), Ordering::Relaxed); }
pub fn get_peak_freq() -> f32 { f32::from_bits(PEAK_FREQ_VAL.load(Ordering::Relaxed)) }

fn store_bpm(val: f32) { BPM_VAL.store(val.to_bits(), Ordering::Relaxed); }
pub fn get_bpm() -> f32 { f32::from_bits(BPM_VAL.load(Ordering::Relaxed)) }

pub fn build_audio_stream(shared_config: Arc<RwLock<AudioConfig>>) -> Result<cpal::Stream, anyhow::Error> {
    let host = cpal::default_host();
    let device = host.default_input_device().ok_or_else(|| anyhow::anyhow!("No input device found"))?;
    let config = device.default_input_config()?;
    
    println!("Audio Client initialized.");

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => run_stream::<f32>(&device, &config.into(), shared_config),
        cpal::SampleFormat::I16 => run_stream::<i16>(&device, &config.into(), shared_config),
        _ => anyhow::bail!("Unsupported sample format"),
    }?;
    
    stream.play()?;
    Ok(stream)
}

fn run_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    shared_config: Arc<RwLock<AudioConfig>>,
) -> Result<cpal::Stream, anyhow::Error>
where
    T: cpal::Sample + cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    let sample_rate = config.sample_rate.0 as f32;
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(AUDIO_BUFFER_SIZE);
    let mut buffer = vec![0.0f32; AUDIO_BUFFER_SIZE];
    let mut idx = 0;

    let mut bass_history: VecDeque<f32> = VecDeque::with_capacity(60);
    let mut beat_times: VecDeque<Instant> = VecDeque::with_capacity(8);
    let mut last_beat = Instant::now();

    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &_| {
            let cfg = shared_config.read().unwrap().clone();

            for sample in data {
                buffer[idx] = sample.to_sample::<f32>();
                idx += 1;
                
                if idx >= AUDIO_BUFFER_SIZE {
                    let mut complex_buffer: Vec<Complex<f32>> = buffer
                        .iter()
                        .map(|&v| Complex { re: v, im: 0.0 })
                        .collect();
                    
                    fft.process(&mut complex_buffer);

                    let mut volume = 0.0;
                    let mut max_amp = 0.0;
                    let mut peak_bin = 0;
                    
                    for i in 0..(AUDIO_BUFFER_SIZE / 2) {
                        let amp = complex_buffer[i].norm();
                        volume += amp;
                        if amp > max_amp {
                            max_amp = amp;
                            peak_bin = i;
                        }
                    }

                    let mut bass = 0.0;
                    for i in cfg.bass_bin_start..=cfg.bass_bin_end { bass += complex_buffer[i].norm(); }
                    
                    let mut mid = 0.0;
                    for i in cfg.mid_bin_start..=cfg.mid_bin_end { mid += complex_buffer[i].norm(); }

                    let mut treble = 0.0;
                    for i in cfg.treble_bin_start..=cfg.treble_bin_end { 
                        if i < complex_buffer.len() { treble += complex_buffer[i].norm(); }
                    }

                    let old_bass = get_bass();
                    let raw_bass = bass / cfg.bass_norm_factor;
                    
                    if bass_history.len() >= 60 { bass_history.pop_front(); }
                    bass_history.push_back(raw_bass);
                    
                    let avg_bass: f32 = bass_history.iter().sum::<f32>() / bass_history.len() as f32;
                    let now = Instant::now();
                    
                    let min_beat_interval = Duration::from_millis(300); 

                    if raw_bass > avg_bass * 1.5 && now.duration_since(last_beat) > min_beat_interval {
                        last_beat = now;
                        if beat_times.len() >= 8 { beat_times.pop_front(); }
                        beat_times.push_back(now);

                        if beat_times.len() >= 4 {
                            let mut total_duration = 0.0;
                            for i in 1..beat_times.len() {
                                total_duration += beat_times[i].duration_since(beat_times[i-1]).as_secs_f32();
                            }
                            let avg_interval = total_duration / (beat_times.len() - 1) as f32;
                            let current_bpm = 60.0 / avg_interval;
                            
                            if current_bpm > 60.0 && current_bpm < 220.0 {
                                store_bpm(current_bpm);
                            }
                        }
                    }

                    let new_bass = if raw_bass > old_bass { raw_bass } else { old_bass * cfg.smoothing_old + raw_bass * cfg.smoothing_new };
                    store_bass(new_bass);

                    let old_mid = get_mid();
                    let raw_mid = mid / cfg.mid_norm_factor;
                    let new_mid = if raw_mid > old_mid { raw_mid } else { old_mid * cfg.smoothing_old + raw_mid * cfg.smoothing_new };
                    store_mid(new_mid);

                    let old_treble = get_treble();
                    let raw_treble = treble / cfg.treble_norm_factor;
                    let new_treble = if raw_treble > old_treble { raw_treble } else { old_treble * cfg.smoothing_old + raw_treble * cfg.smoothing_new };
                    store_treble(new_treble);

                    let old_volume = get_volume();
                    let raw_volume = volume / cfg.volume_norm_factor;
                    let new_volume = if raw_volume > old_volume { raw_volume } else { old_volume * cfg.smoothing_old + raw_volume * cfg.smoothing_new };
                    store_volume(new_volume);

                    let raw_peak_freq = (peak_bin as f32 * sample_rate) / AUDIO_BUFFER_SIZE as f32;
                    let old_peak_freq = get_peak_freq();
                    
                    let new_peak_freq = (old_peak_freq * 0.9) + (raw_peak_freq * 0.1);
                    store_peak_freq(new_peak_freq);

                    idx = 0;
                }
            }
        },
        |err| eprintln!("Audio stream error: {}", err),
        None,
    )?;
    
    Ok(stream)
}
