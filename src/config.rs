use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct AudioConfig {
    pub bass_bin_start: usize,
    pub bass_bin_end: usize,
    pub mid_bin_start: usize,
    pub mid_bin_end: usize,
    pub treble_bin_start: usize,
    pub treble_bin_end: usize,
    pub smoothing_old: f32,
    pub smoothing_new: f32,
    pub bass_norm_factor: f32,
    pub mid_norm_factor: f32,
    pub treble_norm_factor: f32,
    pub volume_norm_factor: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            bass_bin_start: 1,
            bass_bin_end: 3,
            mid_bin_start: 6,
            mid_bin_end: 22,
            treble_bin_start: 23,
            treble_bin_end: 100,
            smoothing_old: 0.7,
            smoothing_new: 0.3,
            bass_norm_factor: 6.0,
            mid_norm_factor: 34.0,
            treble_norm_factor: 20.0,
            volume_norm_factor: 100.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct CropConfig {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Default for CropConfig {
    fn default() -> Self {
        Self { left: 0.0, right: 0.0, top: 0.0, bottom: 0.0 }
    }
}

impl CropConfig {
    pub fn sanitize(&self) -> (f32, f32, f32, f32) {
        fn clean(mut v: f32) -> f32 {
            if v > 1.0 { v /= 100.0; }
            v.clamp(0.0, 0.49) 
        }
        (clean(self.left), clean(self.right), clean(self.top), clean(self.bottom))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PipelineStage {
    pub name: String,
    #[serde(default)]
    pub params: HashMap<String, f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct ShaderConfig {
    pub pipeline: Vec<PipelineStage>,
    pub globals: HashMap<String, f32>,
    pub crop: CropConfig,
    pub font_width: f32,
    pub font_height: f32,
    pub num_chars: f32,
    pub playback_fps: f32,
    pub bpm_multiplier: f32,
    pub audio: AudioConfig,
}

impl Default for ShaderConfig {
    fn default() -> Self {
        let default_stage = PipelineStage {
            name: "ascii/matrix/flux".to_string(),
            params: HashMap::from([("sensitivity".to_string(), 1.5)]),
        };

        Self {
            pipeline: vec![default_stage],
            globals: HashMap::new(),
            crop: CropConfig::default(),
            font_width: 8.0,
            font_height: 16.0,
            num_chars: 10.0,
            playback_fps: 30.0,
            bpm_multiplier: 15.0,
            audio: AudioConfig::default(),
        }
    }
}

pub fn load_config(path: &str) -> ShaderConfig {
    if let Ok(content) = std::fs::read_to_string(path) {
        match json5::from_str::<ShaderConfig>(&content) {
            Ok(config) => return config,
            Err(e) => eprintln!("Error parsing controller.json: {} - keeping previous state.", e),
        }
    }
    ShaderConfig::default()
}
