mod audio;
mod config;
mod shader;

use glfw::{Action, Context, Key};
use opencv::{prelude::*, videoio};
use std::env;
use std::ptr;
use std::sync::mpsc;
use std::fs;
use std::time::{Duration, Instant};
use std::sync::{Arc, RwLock};
use std::path::{Path, PathBuf};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};

use audio::{build_audio_stream, AudioState};
use config::load_config;
use shader::{compile_shader_program, ShaderProgram};

struct ShaderDef {
    name: String,
    path: PathBuf,
    program: Option<ShaderProgram>,
}

fn discover_shaders(dir: &Path, vert_src: &str, defs: &mut Vec<ShaderDef>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                discover_shaders(&path, vert_src, defs);
            } else if path.extension().map_or(false, |ext| ext == "frag") {
                let relative_path = path.strip_prefix("shaders/frag").unwrap_or(&path);
                let name = relative_path.with_extension("")
                    .to_string_lossy()
                    .replace('\\', "/");
                
                let src = fs::read_to_string(&path).unwrap_or_default();
                let program = compile_shader_program(vert_src, &src);
                
                if program.is_none() {
                    eprintln!("Warning: Failed to compile {} initially.", name);
                } else {
                    println!("Loaded shader: {}", name);
                }

                defs.push(ShaderDef { name, path, program });
            }
        }
    }
}

fn get_shader_index(target_name: &str, defs: &[ShaderDef]) -> usize {
    defs.iter()
        .position(|def| def.name.eq_ignore_ascii_case(target_name))
        .unwrap_or(0) 
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    let (mut window, events) = glfw.create_window(1280, 720, "frameshader", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.make_current();
    window.set_key_polling(true);
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    gl::load_with(|s| window.get_proc_address(s) as *const _);

    let args: Vec<String> = env::args().collect();
    let mut is_file = false;
    let mut has_camera = true;

    let mut cam = if args.len() > 1 {
        let input = &args[1];
        if let Ok(cam_index) = input.parse::<i32>() {
            videoio::VideoCapture::new(cam_index, videoio::CAP_ANY)
                .unwrap_or_else(|_| videoio::VideoCapture::default().unwrap())
        } else {
            is_file = true;
            videoio::VideoCapture::from_file(input, videoio::CAP_ANY)
                .unwrap_or_else(|_| videoio::VideoCapture::default().unwrap())
        }
    } else {
        videoio::VideoCapture::new(0, videoio::CAP_ANY)
            .unwrap_or_else(|_| videoio::VideoCapture::default().unwrap())
    };

    if !cam.is_opened().unwrap_or(false) {
        eprintln!("Warning: No input feed available. Running in shader-only mode.");
        has_camera = false;
    }

    let cam_width = if has_camera { cam.get(videoio::CAP_PROP_FRAME_WIDTH).unwrap_or(1280.0) as i32 } else { 1280 };
    let cam_height = if has_camera { cam.get(videoio::CAP_PROP_FRAME_HEIGHT).unwrap_or(720.0) as i32 } else { 720 };
    let fps = if has_camera { cam.get(videoio::CAP_PROP_FPS).unwrap_or(30.0) } else { 30.0 };
    
    if has_camera {
        println!("Input opened successfully at {}x{} ({} fps)", cam_width, cam_height, fps);
    }

    let vertices: [f32; 20] = [
         1.0,  1.0, 0.0,   1.0, 1.0, // Top Right
         1.0, -1.0, 0.0,   1.0, 0.0, // Bottom Right
        -1.0, -1.0, 0.0,   0.0, 0.0, // Bottom Left
        -1.0,  1.0, 0.0,   0.0, 1.0, // Top Left
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

    let mut fbos = [0u32; 2];
    let mut fbo_textures = [0u32; 2];
    let mut current_fb_width = 0;
    let mut current_fb_height = 0;
    unsafe {
        gl::GenFramebuffers(2, fbos.as_mut_ptr());
        gl::GenTextures(2, fbo_textures.as_mut_ptr());
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
            0, gl::BGR, gl::UNSIGNED_BYTE, ptr::null()
        );
    }

    let vert_src = fs::read_to_string("shaders/vert/shader.vert")
        .expect("Failed to read vertex shader from shaders/vert/shader.vert");
    
    let mut shader_defs = Vec::new();
    discover_shaders(Path::new("shaders/frag"), &vert_src, &mut shader_defs);

    let mut frame = opencv::core::Mat::default();

    let config_path = "controller.json";
    let (tx, rx) = mpsc::channel();
    
    let mut debouncer = new_debouncer(Duration::from_millis(150), tx)
        .expect("Failed to initialize file watcher debouncer");
    
    if !Path::new(config_path).exists() {
        let default_config_str = r#"{
  "pipeline": [
    {
      "name": "filters/edge",
      "params": {
        "sensitivity": 1.2
      }
    },
    {
      "name": "warp/vortex",
      "params": {
        "sensitivity": 1.5,
        "vortex_speed": 1.5,
        "vortex_arms": 4.0,
        "vortex_spin": 0.4,
        "vortex_center_size": 0.05,
        "vortex_fov": 0.5,
        "vortex_color_separation": 0.015,
        "vortex_strobe_size": 0.4
      }
    }
  ],
  "globals": {
    "rain_speed": 0.4,
    "tail_length": 0.3,
    "flicker_speed": 15.0,
    "resonance_ripples": 30.0,
    "font_width": 8.0,
    "font_height": 16.0,
    "num_chars": 10.0
  },
  "crop": { "left": 0.0, "right": 0.0, "top": 0.0, "bottom": 0.0 },
  "playback_fps": 30.0,
  "bpm_multiplier": 15.0,
  "audio": {
    "bass_bin_start": 1, "bass_bin_end": 3,
    "mid_bin_start": 6, "mid_bin_end": 22,
    "treble_bin_start": 23, "treble_bin_end": 100,
    "smoothing_old": 0.7, "smoothing_new": 0.3,
    "bass_norm_factor": 6.0, "mid_norm_factor": 34.0,
    "treble_norm_factor": 20.0, "volume_norm_factor": 100.0
  }
}"#;
        fs::write(config_path, default_config_str).unwrap();
    }
    
    debouncer.watcher().watch(Path::new("."), RecursiveMode::NonRecursive).unwrap();
    debouncer.watcher().watch(Path::new("shaders/frag"), RecursiveMode::Recursive).unwrap();

    let mut current_config = load_config(config_path);
    let shared_audio_config = Arc::new(RwLock::new(current_config.audio.clone()));
    let audio_state = Arc::new(AudioState::default());

    let _audio_stream = build_audio_stream(Arc::clone(&shared_audio_config), Arc::clone(&audio_state))
        .map_err(|e| eprintln!("Warning: Failed to start audio stream. ({})", e)).ok();
    
    println!("Watching controller.json & shaders/ for live updates...");
    let mut last_video_update = Instant::now();

    while !window.should_close() {
        glfw.poll_events();
        
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
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

        let mut config_changed = false;
        while let Ok(res) = rx.try_recv() {
            match res {
                Ok(events) => {
                    for event in events {
                        if event.path.ends_with(config_path) {
                            config_changed = true;
                        }
                        
                        if let Some(ext) = event.path.extension() {
                            if ext == "frag" {
                                for def in &mut shader_defs {
                                    if event.path.ends_with(&def.path) {
                                        if let Ok(src) = fs::read_to_string(&event.path) {
                                            if let Some(new_prog) = compile_shader_program(&vert_src, &src) {
                                                def.program = Some(new_prog);
                                                println!("Hot-reloaded shader: {}", def.name);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Watcher error: {:?}", e),
            }
        }

        if config_changed {
            current_config = load_config(config_path);
            
            if let Ok(mut lock) = shared_audio_config.write() {
                *lock = current_config.audio.clone();
            }
            println!("Config reloaded.");
        }

        let (fb_width, fb_height) = window.get_framebuffer_size();

        if fb_width != current_fb_width || fb_height != current_fb_height {
            current_fb_width = fb_width;
            current_fb_height = fb_height;
            unsafe {
                for i in 0..2 {
                    gl::BindFramebuffer(gl::FRAMEBUFFER, fbos[i]);
                    gl::BindTexture(gl::TEXTURE_2D, fbo_textures[i]);
                    gl::TexImage2D(
                        gl::TEXTURE_2D, 0, gl::RGB as i32, 
                        fb_width, fb_height, 
                        0, gl::RGB, gl::UNSIGNED_BYTE, ptr::null()
                    );
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                    
                    gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, fbo_textures[i], 0);
                    
                    if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                        eprintln!("FBO {} is not complete!", i);
                    }
                }
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            }
        }

        let current_bpm = audio_state.get_bpm();
        let target_fps = if current_bpm > 0.0 {
            (current_bpm / 60.0) * current_config.bpm_multiplier 
        } else {
            current_config.playback_fps
        };

        let safe_fps = if target_fps > 0.1 { target_fps } else { 0.1 };
        let frame_duration = Duration::from_secs_f32(1.0 / safe_fps);
        let mut update_texture = false;

        if last_video_update.elapsed() >= frame_duration {
            if has_camera {
                if cam.read(&mut frame).unwrap_or(false) && !frame.empty() {
                    update_texture = true;
                    last_video_update = Instant::now();
                } else if is_file {
                    cam.set(videoio::CAP_PROP_POS_FRAMES, 0.0).unwrap_or_default();
                }
            }
        }

        unsafe {
            if update_texture {
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, video_texture);
                gl::TexSubImage2D(
                    gl::TEXTURE_2D, 0, 0, 0, 
                    cam_width, cam_height, 
                    gl::BGR, gl::UNSIGNED_BYTE, 
                    frame.data() as *const _
                );
            }
        }

        let pipeline_len = current_config.pipeline.len();

        if pipeline_len > 0 && !shader_defs.is_empty() {
            for (i, stage) in current_config.pipeline.iter().enumerate() {
                let is_first = i == 0;
                let is_last = i == pipeline_len - 1;

                let shader_idx = get_shader_index(&stage.name, &shader_defs);
                if let Some(prog) = &shader_defs[shader_idx].program {
                    unsafe {
                        if is_last {
                            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                        } else {
                            gl::BindFramebuffer(gl::FRAMEBUFFER, fbos[i % 2]);
                        }
                        
                        gl::Viewport(0, 0, fb_width, fb_height);
                        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
                        gl::Clear(gl::COLOR_BUFFER_BIT);

                        gl::UseProgram(prog.id);
                        
                        gl::ActiveTexture(gl::TEXTURE0);
                        if is_first {
                            gl::BindTexture(gl::TEXTURE_2D, video_texture);
                        } else {
                            gl::BindTexture(gl::TEXTURE_2D, fbo_textures[(i - 1) % 2]);
                        }

                        gl::ActiveTexture(gl::TEXTURE1);
                        gl::BindTexture(gl::TEXTURE_2D, font_texture);
                        
                        // --- DYNAMIC UNIFORM DISPATCHER ---
                        for (name, &loc) in &prog.uniforms {
                            let param_str = name.as_str();
                            match param_str {
                                // 1. Built-in system textures and variables
                                "videoTexture" => gl::Uniform1i(loc, 0),
                                "fontAtlas" => gl::Uniform1i(loc, 1),
                                "resolution" => gl::Uniform2f(loc, fb_width as f32, fb_height as f32),
                                
                                "charSize" => {
                                    let w = stage.params.get("font_width")
                                        .or_else(|| current_config.globals.get("font_width"))
                                        .unwrap_or(&8.0);
                                    let h = stage.params.get("font_height")
                                        .or_else(|| current_config.globals.get("font_height"))
                                        .unwrap_or(&16.0);
                                    gl::Uniform2f(loc, *w, *h);
                                },
                                "numChars" => {
                                    let n = stage.params.get("num_chars")
                                        .or_else(|| current_config.globals.get("num_chars"))
                                        .unwrap_or(&10.0);
                                    gl::Uniform1f(loc, *n);
                                },
                                
                                "time" => gl::Uniform1f(loc, glfw.get_time() as f32),
                                "is_first_pass" => gl::Uniform1i(loc, if is_first { 1 } else { 0 }),
                                "crop" => {
                                    let (c_l, c_r, c_t, c_b) = current_config.crop.sanitize();
                                    gl::Uniform4f(loc, c_l, c_r, c_t, c_b);
                                },
                                
                                // 2. Audio variables
                                "audio_bass" => gl::Uniform1f(loc, audio_state.get_bass()),
                                "audio_mid" => gl::Uniform1f(loc, audio_state.get_mid()),
                                "audio_treble" => gl::Uniform1f(loc, audio_state.get_treble()),
                                "audio_volume" => gl::Uniform1f(loc, audio_state.get_volume()),
                                "audio_peak_freq" => gl::Uniform1f(loc, audio_state.get_peak_freq()),
                                
                                // 3. Dynamic custom parameters
                                custom_param => {
                                    // Try local stage param first, fallback to global, then default to 0.0
                                    if let Some(&val) = stage.params.get(custom_param) {
                                        gl::Uniform1f(loc, val);
                                    } else if let Some(&val) = current_config.globals.get(custom_param) {
                                        gl::Uniform1f(loc, val);
                                    }
                                }
                            }
                        }

                        gl::BindVertexArray(vao);
                        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
                    }
                }
            }
        } else {
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                gl::Viewport(0, 0, fb_width, fb_height);
                gl::ClearColor(0.1, 0.1, 0.1, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
        }
        
        window.swap_buffers();
    }
}
