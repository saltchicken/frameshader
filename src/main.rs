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
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};

use audio::{build_audio_stream, AudioState};
use config::load_config;
use shader::{compile_shader_program, ShaderProgram};

struct ShaderDef {
    name: String,
    file_name: String,
    static_src: &'static str,
    program: Option<ShaderProgram>,
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

    // Graceful camera degradation (Fixed unwrap for default capture)
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
        
        // Using native OpenCV BGR format directly to bypass CPU conversion
        gl::TexImage2D(
            gl::TEXTURE_2D, 0, gl::RGB as i32, 
            cam_width as i32, cam_height as i32, 
            0, gl::BGR, gl::UNSIGNED_BYTE, ptr::null()
        );
    }

    let vert_src = include_str!("../shaders/vert/shader.vert");
    
    let mut shader_defs = vec![
        ShaderDef { name: "ASCII".into(), file_name: "ascii/static.frag".into(), static_src: include_str!("../shaders/frag/ascii/static.frag"), program: None },
        ShaderDef { name: "Matrix".into(), file_name: "ascii/matrix/matrix.frag".into(), static_src: include_str!("../shaders/frag/ascii/matrix/matrix.frag"), program: None },
        ShaderDef { name: "Color".into(), file_name: "ascii/matrix/color.frag".into(), static_src: include_str!("../shaders/frag/ascii/matrix/color.frag"), program: None },
        // ShaderDef { name: "Flux".into(), file_name: "ascii/matrix/flux.frag".into(), static_src: include_str!("../shaders/frag/ascii/matrix/flux.frag"), program: None },
        // ShaderDef { name: "CRT".into(), file_name: "archived/crt.frag".into(), static_src: include_str!("../shaders/frag/archived/crt.frag"), program: None },
        // ShaderDef { name: "Edge".into(), file_name: "edge.frag".into(), static_src: include_str!("../shaders/frag/edge.frag"), program: None },
        // ShaderDef { name: "VHS".into(), file_name: "archived/vhs.frag".into(), static_src: include_str!("../shaders/frag/archived/vhs.frag"), program: None },
        // ShaderDef { name: "Halftone".into(), file_name: "archived/halftone.frag".into(), static_src: include_str!("../shaders/frag/archived/halftone.frag"), program: None },
        // ShaderDef { name: "Prism".into(), file_name: "archived/prism.frag".into(), static_src: include_str!("../shaders/frag/archived/prism.frag"), program: None },
        // ShaderDef { name: "GlitchSort".into(), file_name: "archived/glitch_sort.frag".into(), static_src: include_str!("../shaders/frag/archived/glitch_sort.frag"), program: None },
        ShaderDef { name: "Vortex".into(), file_name: "vortex.frag".into(), static_src: include_str!("../shaders/frag/vortex.frag"), program: None },
        // ShaderDef { name: "Topography".into(), file_name: "topography.frag".into(), static_src: include_str!("../shaders/frag/topography.frag"), program: None },
        // ShaderDef { name: "Comic".into(), file_name: "archived/comic.frag".into(), static_src: include_str!("../shaders/frag/archived/comic.frag"), program: None },
        // ShaderDef { name: "Thermal".into(), file_name: "archived/thermal.frag".into(), static_src: include_str!("../shaders/frag/archived/thermal.frag"), program: None },
        // ShaderDef { name: "Paint".into(), file_name: "archived/paint.frag".into(), static_src: include_str!("../shaders/frag/archived/paint.frag"), program: None },
        // ShaderDef { name: "NightVision".into(), file_name: "archived/nightvision.frag".into(), static_src: include_str!("../shaders/frag/archived/nightvision.frag"), program: None },
        // ShaderDef { name: "Dither".into(), file_name: "archived/dither.frag".into(), static_src: include_str!("../shaders/frag/archived/dither.frag"), program: None },
        // ShaderDef { name: "Resonance".into(), file_name: "resonance.frag".into(), static_src: include_str!("../shaders/frag/resonance.frag"), program: None },
    ];

    for def in &mut shader_defs {
        let mut loaded_src = def.static_src.to_string();
        
        {
            let path = format!("shaders/frag/{}", def.file_name);
            if let Ok(src) = fs::read_to_string(&path) {
                loaded_src = src;
            }
        }

        if let Some(prog) = compile_shader_program(vert_src, &loaded_src) {
            def.program = Some(prog);
        } else {
            eprintln!("Warning: Failed to compile {} initially.", def.name);
        }
    }

    let mut frame = opencv::core::Mat::default();

    let config_path = "controller.json";
    let (tx, rx) = mpsc::channel();
    
    let mut debouncer = new_debouncer(Duration::from_millis(150), tx)
        .expect("Failed to initialize file watcher debouncer");
    
    if !std::path::Path::new(config_path).exists() {
        let default_config_str = r#"{
// The active shader mode. Options: "ASCII", "Matrix", "Color", "Flux", "CRT", "Edge", "VHS", "Halftone", "Prism", "GlitchSort", "Vortex", "Topography", "Comic", "Thermal", "Paint", "NightVision", "Dither", "Resonance"
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
  "bpm_multiplier": 15.0,
  "sensitivity": 1.5,
  "rain_speed": 0.4,
  "tail_length": 0.3,
  "flicker_speed": 15.0,
  "resonance_ripples": 30.0,
  "vortex_speed": 1.5,
  "vortex_arms": 4.0,
  "vortex_spin": 0.4,
  "vortex_center_size": 0.05,
  "vortex_fov": 0.5,
  "vortex_color_separation": 0.015,
  "vortex_strobe_size": 0.4,
  
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
}"#;
        fs::write(config_path, default_config_str).unwrap();
    }
    
    debouncer.watcher().watch(std::path::Path::new("."), RecursiveMode::NonRecursive)
        .expect("Failed to watch top level directory");
    
    debouncer.watcher().watch(std::path::Path::new("shaders/frag"), RecursiveMode::NonRecursive)
        .expect("Failed to watch shaders directory");

    let mut current_config = load_config(config_path);
    let mut current_mode = get_mode_index(&current_config.mode, &shader_defs);
    
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
                        
                        // Shader Hot Reloading Detection
                        if let Some(ext) = event.path.extension() {
                            if ext == "frag" {
                                for def in &mut shader_defs {
                                    if event.path.ends_with(&def.file_name) {
                                        if let Ok(src) = fs::read_to_string(&event.path) {
                                            if let Some(new_prog) = compile_shader_program(vert_src, &src) {
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
            current_mode = get_mode_index(&current_config.mode, &shader_defs);
            
            if let Ok(mut lock) = shared_audio_config.write() {
                *lock = current_config.audio.clone();
            }
            
            println!("Config reloaded: Active Mode [{}], FPS: [{}], BPM Mult: [{}]", current_config.mode, current_config.playback_fps, current_config.bpm_multiplier);
        }

        let (fb_width, fb_height) = window.get_framebuffer_size();

        unsafe {
            gl::Viewport(0, 0, fb_width, fb_height);
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        let current_bpm = audio_state.get_bpm();

        let target_fps = if current_bpm > 0.0 {
            (current_bpm / 60.0) * current_config.bpm_multiplier 
        } else {
            current_config.playback_fps
        } ;

        let safe_fps = if target_fps > 0.1 { target_fps } else { 0.1 };
        let frame_duration = Duration::from_secs_f32(1.0 / safe_fps);
        let mut update_texture = false;

        if last_video_update.elapsed() >= frame_duration {
            if has_camera {
                if cam.read(&mut frame).unwrap_or(false) && !frame.empty() {
                    // CPU Color conversion eliminated entirely
                    update_texture = true;
                    last_video_update = Instant::now();
                } else if is_file {
                    cam.set(videoio::CAP_PROP_POS_FRAMES, 0.0).unwrap_or_default();
                }
            }
        }

        // Fallback gracefully if the active shader failed to compile
        if let Some(prog) = &shader_defs[current_mode].program {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, video_texture);
                
                if update_texture {
                    // Upload native OpenCV BGR frame directly to GPU
                    gl::TexSubImage2D(
                        gl::TEXTURE_2D, 0, 0, 0, 
                        cam_width, cam_height, 
                        gl::BGR, gl::UNSIGNED_BYTE, 
                        frame.data() as *const _
                    );
                }

                gl::UseProgram(prog.id);
                
                // HashMap-driven Uniform Dispatch
                for (name, &loc) in &prog.uniforms {
                    match name.as_str() {
                        "videoTexture" => gl::Uniform1i(loc, 0),
                        "fontAtlas" => gl::Uniform1i(loc, 1),
                        "resolution" => gl::Uniform2f(loc, fb_width as f32, fb_height as f32),
                        "charSize" => gl::Uniform2f(loc, current_config.font_width, current_config.font_height),
                        "numChars" => gl::Uniform1f(loc, current_config.num_chars),
                        "time" => gl::Uniform1f(loc, glfw.get_time() as f32),
                        "crop" => {
                            let (c_l, c_r, c_t, c_b) = current_config.crop.sanitize();
                            gl::Uniform4f(loc, c_l, c_r, c_t, c_b);
                        },
                        "sensitivity" => gl::Uniform1f(loc, current_config.sensitivity),
                        "rain_speed" => gl::Uniform1f(loc, current_config.rain_speed),
                        "tail_length" => gl::Uniform1f(loc, current_config.tail_length),
                        "flicker_speed" => gl::Uniform1f(loc, current_config.flicker_speed),
                        "resonance_ripples" => gl::Uniform1f(loc, current_config.resonance_ripples),
                        "vortex_speed" => gl::Uniform1f(loc, current_config.vortex_speed),
                        "vortex_arms" => gl::Uniform1f(loc, current_config.vortex_arms),
                        "vortex_spin" => gl::Uniform1f(loc, current_config.vortex_spin),
                        "vortex_center_size" => gl::Uniform1f(loc, current_config.vortex_center_size),
                        "vortex_fov" => gl::Uniform1f(loc, current_config.vortex_fov),
                        "vortex_color_separation" => gl::Uniform1f(loc, current_config.vortex_color_separation),
                        "vortex_strobe_size" => gl::Uniform1f(loc, current_config.vortex_strobe_size),
                        "audio_bass" => gl::Uniform1f(loc, audio_state.get_bass()),
                        "audio_mid" => gl::Uniform1f(loc, audio_state.get_mid()),
                        "audio_treble" => gl::Uniform1f(loc, audio_state.get_treble()),
                        "audio_volume" => gl::Uniform1f(loc, audio_state.get_volume()),
                        "audio_peak_freq" => gl::Uniform1f(loc, audio_state.get_peak_freq()),
                        _ => {} // Ignore unexpected uniforms gracefully
                    }
                }

                gl::BindVertexArray(vao);
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            }
        }
        
        window.swap_buffers();
    }
}

fn get_mode_index(target_mode: &str, defs: &[ShaderDef]) -> usize {
    defs.iter()
        .position(|def| def.name.eq_ignore_ascii_case(target_mode))
        .unwrap_or(0) 
}
