use glfw::{Action, Context, Key};
use opencv::{prelude::*, videoio, imgproc, core};
use std::env;
use std::ffi::CString;
use std::ptr;
use std::sync::mpsc;
use std::fs;
use std::time::{Duration, Instant};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use serde::{Deserialize, Serialize};

// Audio Dependencies
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustfft::{num_complex::Complex, FftPlanner};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};

// --- AUDIO GLOBALS ---
const AUDIO_BUFFER_SIZE: usize = 512;

static BASS_VAL: AtomicU32 = AtomicU32::new(0);
static MID_VAL: AtomicU32 = AtomicU32::new(0);
static TREBLE_VAL: AtomicU32 = AtomicU32::new(0);
static VOLUME_VAL: AtomicU32 = AtomicU32::new(0);
static PEAK_FREQ_VAL: AtomicU32 = AtomicU32::new(0);

fn store_bass(val: f32) { BASS_VAL.store(val.to_bits(), Ordering::Relaxed); }
fn get_bass() -> f32 { f32::from_bits(BASS_VAL.load(Ordering::Relaxed)) }

fn store_mid(val: f32) { MID_VAL.store(val.to_bits(), Ordering::Relaxed); }
fn get_mid() -> f32 { f32::from_bits(MID_VAL.load(Ordering::Relaxed)) }

fn store_treble(val: f32) { TREBLE_VAL.store(val.to_bits(), Ordering::Relaxed); }
fn get_treble() -> f32 { f32::from_bits(TREBLE_VAL.load(Ordering::Relaxed)) }

fn store_volume(val: f32) { VOLUME_VAL.store(val.to_bits(), Ordering::Relaxed); }
fn get_volume() -> f32 { f32::from_bits(VOLUME_VAL.load(Ordering::Relaxed)) }

fn store_peak_freq(val: f32) { PEAK_FREQ_VAL.store(val.to_bits(), Ordering::Relaxed); }
fn get_peak_freq() -> f32 { f32::from_bits(PEAK_FREQ_VAL.load(Ordering::Relaxed)) }


// 1. Define our JSON configuration structure
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
struct AudioConfig {
    bass_bin_start: usize,
    bass_bin_end: usize,
    mid_bin_start: usize,
    mid_bin_end: usize,
    treble_bin_start: usize,
    treble_bin_end: usize,
    smoothing_old: f32,
    smoothing_new: f32,
    bass_norm_factor: f32,
    mid_norm_factor: f32,
    treble_norm_factor: f32,
    volume_norm_factor: f32,
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
struct CropConfig {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

impl Default for CropConfig {
    fn default() -> Self {
        Self { left: 0.0, right: 0.0, top: 0.0, bottom: 0.0 }
    }
}

impl CropConfig {
    fn sanitize(&self) -> (f32, f32, f32, f32) {
        fn clean(mut v: f32) -> f32 {
            // Auto-correct whole numbers (e.g., 10 -> 0.1)
            if v > 1.0 { v /= 100.0; }
            // Ensure values never drop below 0 or crop more than half the screen (0.49)
            v.clamp(0.0, 0.49) 
        }
        (clean(self.left), clean(self.right), clean(self.top), clean(self.bottom))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)] // Allows partial JSON files (missing fields will use default)
struct ShaderConfig {
    mode: String,
    crop: CropConfig,
    font_width: f32,
    font_height: f32,
    num_chars: f32,
    playback_fps: f32,
    sensitivity: f32,
    rain_speed: f32,
    tail_length: f32,
    flicker_speed: f32,
    audio: AudioConfig,
}

impl Default for ShaderConfig {
    fn default() -> Self {
        Self {
            mode: "Flux".to_string(),
            crop: CropConfig::default(),
            font_width: 8.0,
            font_height: 16.0,
            num_chars: 10.0,
            playback_fps: 30.0,
            sensitivity: 1.5,
            rain_speed: 0.4,
            tail_length: 0.3,
            flicker_speed: 15.0,
            audio: AudioConfig::default(),
        }
    }
}

// 2. Expand the struct to cache our new parameters
struct ShaderProgram {
    id: u32,
    loc_video_texture: i32,
    loc_font_atlas: i32,
    loc_resolution: i32,
    loc_char_size: i32,
    loc_num_chars: i32,
    loc_sensitivity: i32,
    loc_time: i32,
    loc_rain_speed: i32,
    loc_tail_length: i32,
    loc_flicker_speed: i32,
    loc_audio_bass: i32,
    loc_audio_mid: i32,
    loc_audio_treble: i32,
    loc_audio_volume: i32,
    loc_audio_peak_freq: i32,
    loc_crop: i32,
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    // Start as a standard window (you can maximize it and the shaders will now adapt correctly!)
    let (mut window, events) = glfw.create_window(1280, 720, "frameshader", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.make_current();
    window.set_key_polling(true);
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    gl::load_with(|s| window.get_proc_address(s) as *const _);

    let args: Vec<String> = env::args().collect();
    let mut is_file = false;

    let mut cam = if args.len() > 1 {
        let input = &args[1];
        if let Ok(cam_index) = input.parse::<i32>() {
            videoio::VideoCapture::new(cam_index, videoio::CAP_ANY).expect("Failed to open camera")
        } else {
            is_file = true;
            videoio::VideoCapture::from_file(input, videoio::CAP_ANY).expect("Failed to open video file")
        }
    } else {
        videoio::VideoCapture::new(0, videoio::CAP_ANY).expect("Failed to open camera")
    };

    let cam_width = cam.get(videoio::CAP_PROP_FRAME_WIDTH).unwrap() as i32;
    let cam_height = cam.get(videoio::CAP_PROP_FRAME_HEIGHT).unwrap() as i32;
    let fps = cam.get(videoio::CAP_PROP_FPS).unwrap();
    
    println!("Input opened successfully at {}x{} ({} fps)", cam_width, cam_height, fps);

    let vertices: [f32; 20] = [
         1.0,  1.0, 0.0,   1.0, 0.0,
         1.0, -1.0, 0.0,   1.0, 1.0,
        -1.0, -1.0, 0.0,   0.0, 1.0,
        -1.0,  1.0, 0.0,   0.0, 0.0,
    ];
    let indices: [u32; 6] = [0, 1, 3, 1, 2, 3];
    let (mut vao, mut vbo, mut ebo) = (0, 0, 0);

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * 4) as isize, vertices.as_ptr() as *const _, gl::STATIC_DRAW);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (indices.len() * 4) as isize, indices.as_ptr() as *const _, gl::STATIC_DRAW);

        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 20, ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 20, (3 * 4) as *const _);
        gl::EnableVertexAttribArray(1);
    }

    let img = image::open("font_atlases/dejavu_sans_mono-10-8x16.png").expect("Failed to load font atlas").into_rgba8();
    let mut font_texture = 0;
    unsafe {
        gl::GenTextures(1, &mut font_texture);
        gl::ActiveTexture(gl::TEXTURE1);
        gl::BindTexture(gl::TEXTURE_2D, font_texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexImage2D(
            gl::TEXTURE_2D, 0, gl::RGBA as i32, img.width() as i32, img.height() as i32,
            0, gl::RGBA, gl::UNSIGNED_BYTE, img.as_raw().as_ptr() as *const _,
        );
    }

    let mut video_texture = 0;
    unsafe {
        gl::GenTextures(1, &mut video_texture);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, video_texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::TexImage2D(
            gl::TEXTURE_2D, 0, gl::RGB as i32, 
            cam_width as i32, cam_height as i32, 
            0, gl::RGB, gl::UNSIGNED_BYTE, ptr::null()
        );
    }

    let vert_src = include_str!("../shaders/vert/shader.vert");
    let fragment_sources = [
        ("ASCII", include_str!("../shaders/frag/ascii.frag")),
        ("Matrix", include_str!("../shaders/frag/ascii_matrix.frag")),
        ("Color", include_str!("../shaders/frag/ascii_matrix_color.frag")),
        ("Flux", include_str!("../shaders/frag/ascii_matrix_flux.frag")),
        ("CRT", include_str!("../shaders/frag/crt.frag")),
        ("Edge", include_str!("../shaders/frag/edge.frag")),
        ("VHS", include_str!("../shaders/frag/vhs.frag")),
        ("Halftone", include_str!("../shaders/frag/halftone.frag")),
        ("Prism", include_str!("../shaders/frag/prism.frag")),
        ("GlitchSort", include_str!("../shaders/frag/glitch_sort.frag")),
        ("Vortex", include_str!("../shaders/frag/vortex.frag")),
        ("Topography", include_str!("../shaders/frag/topography.frag")),
    ];

    let mut shader_programs = Vec::new();
    for (name, frag_src) in fragment_sources.iter() {
        println!("Compiling {}...", name);
        
        let id = compile_shader_program(vert_src, frag_src);
        unsafe {
            shader_programs.push(ShaderProgram {
                id,
                loc_video_texture: gl::GetUniformLocation(id, c"videoTexture".as_ptr()),
                loc_font_atlas: gl::GetUniformLocation(id, c"fontAtlas".as_ptr()),
                loc_resolution: gl::GetUniformLocation(id, c"resolution".as_ptr()),
                loc_char_size: gl::GetUniformLocation(id, c"charSize".as_ptr()),
                loc_num_chars: gl::GetUniformLocation(id, c"numChars".as_ptr()),
                loc_sensitivity: gl::GetUniformLocation(id, c"sensitivity".as_ptr()),
                loc_time: gl::GetUniformLocation(id, c"time".as_ptr()),
                loc_rain_speed: gl::GetUniformLocation(id, c"rain_speed".as_ptr()),
                loc_tail_length: gl::GetUniformLocation(id, c"tail_length".as_ptr()),
                loc_flicker_speed: gl::GetUniformLocation(id, c"flicker_speed".as_ptr()),
                loc_audio_bass: gl::GetUniformLocation(id, c"audio_bass".as_ptr()),
                loc_audio_mid: gl::GetUniformLocation(id, c"audio_mid".as_ptr()),
                loc_audio_treble: gl::GetUniformLocation(id, c"audio_treble".as_ptr()),
                loc_audio_volume: gl::GetUniformLocation(id, c"audio_volume".as_ptr()),
                loc_audio_peak_freq: gl::GetUniformLocation(id, c"audio_peak_freq".as_ptr()),
                loc_crop: gl::GetUniformLocation(id, c"crop".as_ptr()),
            });
        }
    }

    let mut frame = core::Mat::default();
    let mut rgb_frame = core::Mat::default();

    // 4. Setup File Watcher with Debounce
    let config_path = "controller.json";
    let (tx, rx) = mpsc::channel();
    
    // Set a 250ms debounce window
    let mut debouncer = new_debouncer(Duration::from_millis(250), tx)
        .expect("Failed to initialize file watcher debouncer");
    
    // Create default file if it doesn't exist, using a raw string to preserve comments
    if !std::path::Path::new(config_path).exists() {
        let default_config_str = r#"{
  // The active shader mode. Options: "ASCII", "Matrix", "Color", "Flux", "CRT", "Edge", "VHS", "Halftone"
  "mode": "Flux",
  
  // Crop parameters (0.0 to 1.0 for percentage of edges to remove)
  "crop": {
    "left": 0.0,
    "right": 0.0,
    "top": 0.0,
    "bottom": 0.0
  },
  
  // Font Graphics Options
  "font_width": 8.0,
  "font_height": 16.0,
  "num_chars": 10.0,
  
  // Live parameters
  "playback_fps": 30.0,
  "sensitivity": 1.5,
  "rain_speed": 0.4,
  "tail_length": 0.3,
  "flicker_speed": 15.0,
  
  // Audio reactive parameters
  "audio": {
    "bass_bin_start": 1,
    "bass_bin_end": 3,
    "mid_bin_start": 6,
    "mid_bin_end": 22,
    "treble_bin_start": 23,
    "treble_bin_end": 100,
    "smoothing_old": 0.7,
    "smoothing_new": 0.3,
    "bass_norm_factor": 6.0,
    "mid_norm_factor": 34.0,
    "treble_norm_factor": 20.0,
    "volume_norm_factor": 100.0
  }

  // ==========================================
  // COMMENTED OUT EXAMPLES
  // ==========================================

  /*
  // VHS Glitch Mode
  "mode": "VHS",
  "playback_fps": 30.0,
  "sensitivity": 1.5,
  */

  /*
  // Halftone Dot Mode
  "mode": "Halftone",
  "playback_fps": 30.0,
  "sensitivity": 1.0,
  */
}"#;
        fs::write(config_path, default_config_str).unwrap();
    }
    
    debouncer.watcher().watch(std::path::Path::new("."), RecursiveMode::NonRecursive)
        .expect("Failed to watch directory");

    // Load initial state
    let mut current_config = load_config(config_path);
    let mut current_mode = get_mode_index(&current_config.mode, &fragment_sources);
    
    // Initialize Audio
    let shared_audio_config = Arc::new(RwLock::new(current_config.audio.clone()));
    let _audio_stream = build_audio_stream(Arc::clone(&shared_audio_config))
        .expect("Failed to start audio stream");
    
    println!("Watching controller.json for live updates...");

    let mut last_video_update = Instant::now();

    // 5. Main Loop
    while !window.should_close() {
        glfw.poll_events();
        
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                // Toggle fullscreen on F11
                glfw::WindowEvent::Key(Key::F11, _, Action::Press, _) => {
                    let is_fullscreen = window.with_window_mode(|mode| {
                        match mode {
                            glfw::WindowMode::Windowed => false,
                            glfw::WindowMode::FullScreen(_) => true,
                        }
                    });

                    if is_fullscreen {
                        window.set_monitor(glfw::WindowMode::Windowed, 0, 0, 1280, 720, None);
                    } else {
                        glfw.with_primary_monitor(|_, m| {
                            if let Some(monitor) = m {
                                let mode = monitor.get_video_mode().unwrap();
                                window.set_monitor(
                                    glfw::WindowMode::FullScreen(monitor),
                                    0, 0, mode.width, mode.height, Some(mode.refresh_rate)
                                );
                            }
                        });
                    }
                }
                _ => {}
            }
        }

        // Check if config file was modified (debounced)
        let mut config_changed = false;
        while let Ok(res) = rx.try_recv() {
            match res {
                Ok(events) => {
                    for event in events {
                        if event.path.ends_with(config_path) {
                            config_changed = true;
                        }
                    }
                }
                Err(e) => eprintln!("Watcher error: {:?}", e),
            }
        }

        if config_changed {
            current_config = load_config(config_path);
            current_mode = get_mode_index(&current_config.mode, &fragment_sources);
            
            // Send new audio config to the audio thread
            if let Ok(mut lock) = shared_audio_config.write() {
                *lock = current_config.audio.clone();
            }
            
            println!("Config reloaded: Active Mode [{}], FPS: [{}]", current_config.mode, current_config.playback_fps);
        }

        let (fb_width, fb_height) = window.get_framebuffer_size();

        unsafe {
            gl::Viewport(0, 0, fb_width, fb_height);
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // 1. Check if it's time to read a new video frame
        let safe_fps = if current_config.playback_fps > 0.1 { current_config.playback_fps } else { 0.1 };
        let frame_duration = Duration::from_secs_f32(1.0 / safe_fps);
        let mut update_texture = false;

        if last_video_update.elapsed() >= frame_duration {
            if cam.read(&mut frame).unwrap_or(false) && !frame.empty() {
                imgproc::cvt_color_def(&frame, &mut rgb_frame, imgproc::COLOR_BGR2RGB).unwrap();
                update_texture = true;
                last_video_update = Instant::now();
            } else if is_file {
                // Loop the video if it reaches the end
                cam.set(videoio::CAP_PROP_POS_FRAMES, 0.0).unwrap();
            } else {
                eprintln!("Camera feed lost!");
                break;
            }
        }

        // 2. Render the frame
        let prog = &shader_programs[current_mode];

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, video_texture);
            
            // Only send new pixel data to the GPU if we actually read a new frame
            if update_texture {
                gl::TexSubImage2D(
                    gl::TEXTURE_2D, 0, 0, 0, 
                    cam_width, cam_height, 
                    gl::RGB, gl::UNSIGNED_BYTE, 
                    rgb_frame.data() as *const _
                );
            }

            gl::UseProgram(prog.id);
            
            if prog.loc_video_texture != -1 { gl::Uniform1i(prog.loc_video_texture, 0); }
            if prog.loc_font_atlas != -1 { gl::Uniform1i(prog.loc_font_atlas, 1); }
            
            // Use fb_width and fb_height instead of cam_width and cam_height for correct aspect ratio scaling
            if prog.loc_resolution != -1 { gl::Uniform2f(prog.loc_resolution, fb_width as f32, fb_height as f32); }
            
            // Use variables from config
            if prog.loc_char_size != -1 { gl::Uniform2f(prog.loc_char_size, current_config.font_width, current_config.font_height); }
            if prog.loc_num_chars != -1 { gl::Uniform1f(prog.loc_num_chars, current_config.num_chars); }

            if prog.loc_time != -1 { gl::Uniform1f(prog.loc_time, glfw.get_time() as f32); }

            // Live Config Uniforms
            if prog.loc_crop != -1 {
                let (c_l, c_r, c_t, c_b) = current_config.crop.sanitize();
                gl::Uniform4f(
                    prog.loc_crop,
                    c_l,
                    c_r,
                    c_t,
                    c_b
                );
            }
            
            if prog.loc_sensitivity != -1 { gl::Uniform1f(prog.loc_sensitivity, current_config.sensitivity); }
            if prog.loc_rain_speed != -1 { gl::Uniform1f(prog.loc_rain_speed, current_config.rain_speed); }
            if prog.loc_tail_length != -1 { gl::Uniform1f(prog.loc_tail_length, current_config.tail_length); }
            if prog.loc_flicker_speed != -1 { gl::Uniform1f(prog.loc_flicker_speed, current_config.flicker_speed); }
            
            // Live Audio Uniforms
            if prog.loc_audio_bass != -1 { gl::Uniform1f(prog.loc_audio_bass, get_bass()); }
            if prog.loc_audio_mid != -1 { gl::Uniform1f(prog.loc_audio_mid, get_mid()); }
            if prog.loc_audio_treble != -1 { gl::Uniform1f(prog.loc_audio_treble, get_treble()); }
            if prog.loc_audio_volume != -1 { gl::Uniform1f(prog.loc_audio_volume, get_volume()); }
            if prog.loc_audio_peak_freq != -1 { gl::Uniform1f(prog.loc_audio_peak_freq, get_peak_freq()); }

            gl::BindVertexArray(vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
        }
        
        window.swap_buffers();
    }
}

// --- Helper Functions ---

fn load_config(path: &str) -> ShaderConfig {
    if let Ok(content) = fs::read_to_string(path) {
        // Use JSON5 parser so we can support comments
        match json5::from_str::<ShaderConfig>(&content) {
            Ok(config) => return config,
            Err(e) => eprintln!("Error parsing controller.json: {} - keeping previous state.", e),
        }
    }
    ShaderConfig::default()
}

fn get_mode_index(target_mode: &str, sources: &[(&str, &str)]) -> usize {
    sources.iter()
        .position(|&(name, _)| name.eq_ignore_ascii_case(target_mode))
        .unwrap_or(0) 
}

fn compile_shader_program(v_src: &str, f_src: &str) -> u32 {
    unsafe {
        let vertex = gl::CreateShader(gl::VERTEX_SHADER);
        let c_str_v = CString::new(v_src).unwrap();
        gl::ShaderSource(vertex, 1, &c_str_v.as_ptr(), ptr::null());
        gl::CompileShader(vertex);
        check_shader_compile(vertex, "VERTEX");

        let fragment = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_f = CString::new(f_src).unwrap();
        gl::ShaderSource(fragment, 1, &c_str_f.as_ptr(), ptr::null());
        gl::CompileShader(fragment);
        check_shader_compile(fragment, "FRAGMENT");

        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex);
        gl::AttachShader(program, fragment);
        gl::LinkProgram(program);

        gl::DeleteShader(vertex);
        gl::DeleteShader(fragment);
        
        program
    }
}

unsafe fn check_shader_compile(shader: u32, shader_type: &str) {
    let mut success = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    if success == 0 {
        let mut len = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        let mut buffer: Vec<u8> = vec![0; len as usize];
        gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut i8);
        panic!("{} Shader Compilation Error:\n{}", shader_type, String::from_utf8_lossy(&buffer));
    }
}

// --- Audio Functions ---
fn build_audio_stream(shared_config: Arc<RwLock<AudioConfig>>) -> Result<cpal::Stream, anyhow::Error> {
    let host = cpal::default_host();
    let device = host.default_input_device().ok_or_else(|| anyhow::anyhow!("No input device found"))?;
    let config = device.default_input_config()?;
    
    println!("Audio Client initialized.");

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => run_stream::<f32>(&device, &config.into(), shared_config),
        cpal::SampleFormat::I16 => run_stream::<i16>(&device, &config.into(), shared_config),
        _ => anyhow::bail!("Unsupported sample format"),
    }?;
    
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
                    
                    // Analyze up to Nyquist limit for volume and peak freq
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

                    let peak_freq = (peak_bin as f32 * sample_rate) / AUDIO_BUFFER_SIZE as f32;
                    store_peak_freq(peak_freq);

                    idx = 0;
                }
            }
        },
        |err| eprintln!("Audio stream error: {}", err),
        None,
    )?;
    
    stream.play()?;
    Ok(stream)
}
