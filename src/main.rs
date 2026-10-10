use glfw::{Action, Context, Key};
use opencv::{prelude::*, videoio, imgproc, core};
use std::env;
use std::ffi::CString;
use std::ptr;
use std::sync::mpsc;
use std::fs;
use std::time::Duration;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use serde::{Deserialize, Serialize};

// 1. Define our JSON configuration structure
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)] // Allows partial JSON files (missing fields will use default)
struct ShaderConfig {
    mode: String,
    sensitivity: f32,
    rain_speed: f32,
    tail_length: f32,
    flicker_speed: f32,
}

impl Default for ShaderConfig {
    fn default() -> Self {
        Self {
            mode: "Flux".to_string(),
            sensitivity: 1.5,
            rain_speed: 0.4,
            tail_length: 0.3,
            flicker_speed: 15.0,
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
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    let (mut window, events) = glfw.create_window(1280, 720, "Rust ASCII Shader (Live Config)", glfw::WindowMode::Windowed)
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
    
    // Warning fix: Re-added print statement
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
    ];

    let mut shader_programs = Vec::new();
    for (name, frag_src) in fragment_sources.iter() {
        // Warning fix: Re-added print statement
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
  // The active shader mode. Options: "ASCII", "Matrix", "Color", "Flux"
  "mode": "Flux",
  
  // Live parameters
  "sensitivity": 1.5,
  "rain_speed": 0.4,
  "tail_length": 0.3,
  "flicker_speed": 15.0

  // ==========================================
  // COMMENTED OUT EXAMPLES
  // Uncomment the mode you want and adjust!
  // ==========================================

  /*
  // Basic ASCII (Only uses sensitivity)
  "mode": "ASCII",
  "sensitivity": 1.0,
  */

  /*
  // Classic Matrix (Green rain interacting with brightness)
  "mode": "Matrix",
  "sensitivity": 2.0,
  "rain_speed": 0.3,
  "tail_length": 0.25,
  */

  /*
  // Color Matrix (Rain taking the colors of the camera)
  "mode": "Color",
  "rain_speed": 0.3,
  "tail_length": 0.25,
  */
}"#;
        fs::write(config_path, default_config_str).unwrap();
    }
    
    debouncer.watcher().watch(std::path::Path::new("."), RecursiveMode::NonRecursive)
        .expect("Failed to watch directory");

    // Load initial state
    let mut current_config = load_config(config_path);
    let mut current_mode = get_mode_index(&current_config.mode, &fragment_sources);
    
    println!("Watching controller.json for live updates...");

    // 5. Main Loop
    while !window.should_close() {
        glfw.poll_events();
        
        for (_, event) in glfw::flush_messages(&events) {
            if let glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) = event {
                window.set_should_close(true);
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
            println!("Config reloaded: Active Mode [{}]", current_config.mode);
        }

        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        if cam.read(&mut frame).unwrap_or(false) && !frame.empty() {
            imgproc::cvt_color_def(&frame, &mut rgb_frame, imgproc::COLOR_BGR2RGB).unwrap();

            let prog = &shader_programs[current_mode];

            unsafe {
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, video_texture);
                
                gl::TexSubImage2D(
                    gl::TEXTURE_2D, 0, 0, 0, 
                    cam_width, cam_height, 
                    gl::RGB, gl::UNSIGNED_BYTE, 
                    rgb_frame.data() as *const _
                );

                gl::UseProgram(prog.id);
                
                if prog.loc_video_texture != -1 { gl::Uniform1i(prog.loc_video_texture, 0); }
                if prog.loc_font_atlas != -1 { gl::Uniform1i(prog.loc_font_atlas, 1); }
                if prog.loc_resolution != -1 { gl::Uniform2f(prog.loc_resolution, cam_width as f32, cam_height as f32); }
                if prog.loc_char_size != -1 { gl::Uniform2f(prog.loc_char_size, 8.0, 16.0); }
                if prog.loc_num_chars != -1 { gl::Uniform1f(prog.loc_num_chars, 10.0); }
                if prog.loc_time != -1 { gl::Uniform1f(prog.loc_time, glfw.get_time() as f32); }

                // Live Config Uniforms
                if prog.loc_sensitivity != -1 { gl::Uniform1f(prog.loc_sensitivity, current_config.sensitivity); }
                if prog.loc_rain_speed != -1 { gl::Uniform1f(prog.loc_rain_speed, current_config.rain_speed); }
                if prog.loc_tail_length != -1 { gl::Uniform1f(prog.loc_tail_length, current_config.tail_length); }
                if prog.loc_flicker_speed != -1 { gl::Uniform1f(prog.loc_flicker_speed, current_config.flicker_speed); }

                gl::BindVertexArray(vao);
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            }
        } else if is_file {
            cam.set(videoio::CAP_PROP_POS_FRAMES, 0.0).unwrap();
        } else {
            eprintln!("Camera feed lost!");
            break;
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
