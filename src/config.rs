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
    pub video_input: Option<String>,
    pub active_preset: Option<String>,
    pub presets: HashMap<String, Vec<PipelineStage>>,
    pub pipeline: Vec<PipelineStage>,
    pub globals: HashMap<String, f32>,
    pub crop: CropConfig,
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
            video_input: None,
            active_preset: None,
            presets: HashMap::new(),
            pipeline: vec![default_stage],
            globals: HashMap::new(),
            crop: CropConfig::default(),
            playback_fps: 30.0,
            bpm_multiplier: 15.0,
            audio: AudioConfig::default(),
        }
    }
}

impl ShaderConfig {
    pub fn get_active_pipeline(&self) -> &Vec<PipelineStage> {
        if let Some(preset_name) = &self.active_preset {
            if let Some(preset_pipeline) = self.presets.get(preset_name) {
                return preset_pipeline;
            } else {
                eprintln!("Warning: active_preset '{}' not found in presets.", preset_name);
            }
        }
        // Fallback to the default pipeline if no valid preset is active
        &self.pipeline
    }
}

pub fn load_config(path: &str) -> ShaderConfig {
    let mut config = if let Ok(content) = std::fs::read_to_string(path) {
        match json5::from_str::<ShaderConfig>(&content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Error parsing controller.json: {} - keeping previous state.", e);
                ShaderConfig::default()
            }
        }
    } else {
        ShaderConfig::default()
    };

    // Load separate preset files
    let presets_dir = std::path::Path::new("presets");
    if presets_dir.exists() && presets_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(presets_dir) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.is_file() && file_path.extension().map_or(false, |ext| ext == "json") {
                    if let Some(preset_name) = file_path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(content) = std::fs::read_to_string(&file_path) {
                            match json5::from_str::<Vec<PipelineStage>>(&content) {
                                Ok(pipeline) => {
                                    config.presets.insert(preset_name.to_string(), pipeline);
                                }
                                Err(e) => eprintln!("Error parsing preset {}: {}", file_path.display(), e),
                            }
                        }
                    }
                }
            }
        }
    } else {
        // Create the directory if it doesn't exist so the user knows where to put them
        let _ = std::fs::create_dir("presets");
    }

    config
}
