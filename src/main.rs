use glfw::{Action, Context, Key};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;
use std::ffi::CString;
use std::fs;
use std::ptr;

fn main() {
    // 1. Initialize Windowing (GLFW)
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    let (mut window, events) = glfw.create_window(1280, 720, "Rust ASCII Shader", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");
    window.make_current();
    window.set_key_polling(true);

    // Load OpenGL function pointers
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    // 2. Initialize Camera (nokhwa)
    println!("Initializing camera at Index 0...");
    
    // Instead of forcing a specific format (which the camera rejected), 
    // we ask nokhwa to look at what the camera supports and pick the highest resolution.
    let mut cam = Camera::new(
        CameraIndex::Index(0),
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution),
    ).expect("Failed to initialize camera at Index 0");
    
    cam.open_stream().expect("Failed to open camera stream");
    
    let actual_format = cam.camera_format();
    println!("Camera stream opened successfully at {}x{} ({} fps)", 
        actual_format.resolution().width(), 
        actual_format.resolution().height(),
        actual_format.frame_rate()
    );

    // 3. Setup Geometry (Fullscreen Quad)
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

    // 4. Load Font Atlas Texture (image crate)
    let img = image::open("font_atlases/dejavu_sans_mono-10-8x16.png")
        .expect("Failed to load font atlas")
        .into_rgba8();
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

    // 5. Setup Camera Video Texture
    let mut video_texture = 0;
    unsafe {
        gl::GenTextures(1, &mut video_texture);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, video_texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        
        // Tell OpenGL not to expect standard 4-byte padding to prevent skewing/crashing 
        // on weird camera resolutions.
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
    }

    // 6. Compile Shaders
    let frag_shaders = [
        "shaders/frag/ascii.frag",
        "shaders/frag/ascii_matrix.frag",
        "shaders/frag/ascii_matrix_color.frag",
        "shaders/frag/ascii_matrix_flux.frag",
    ];

    let mut shader_programs = Vec::new();
    for frag in frag_shaders.iter() {
        println!("Compiling {}...", frag);
        shader_programs.push(compile_shader_program("shaders/vert/shader.vert", frag));
    }

    let mut current_mode = 0; 
    println!("Press SPACE to cycle modes, or 1-4 to select directly.");

    // 7. Main Loop
    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                // Exit on ESC
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                },
                // Cycle modes on Space
                glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
                    current_mode = (current_mode + 1) % shader_programs.len();
                    println!("Switched to: {}", frag_shaders[current_mode]);
                },
                // Jump to specific modes using numbers 1-4
                glfw::WindowEvent::Key(Key::Num1, _, Action::Press, _) => {
                    current_mode = 0;
                    println!("Switched to: {}", frag_shaders[current_mode]);
                },
                glfw::WindowEvent::Key(Key::Num2, _, Action::Press, _) => {
                    current_mode = 1;
                    println!("Switched to: {}", frag_shaders[current_mode]);
                },
                glfw::WindowEvent::Key(Key::Num3, _, Action::Press, _) => {
                    current_mode = 2;
                    println!("Switched to: {}", frag_shaders[current_mode]);
                },
                glfw::WindowEvent::Key(Key::Num4, _, Action::Press, _) => {
                    current_mode = 3;
                    println!("Switched to: {}", frag_shaders[current_mode]);
                },
                _ => {}
            }
        }

        unsafe {
            // Clear the screen to dark gray first
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // Grab frame from webcam and decode it to pure RGB pixels
        if let Ok(frame) = cam.frame() {
            if let Ok(decoded) = frame.decode_image::<RgbFormat>() {
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0);
                    gl::BindTexture(gl::TEXTURE_2D, video_texture);
                    
                    // Upload RGB pixels directly to the GPU
                    gl::TexImage2D(
                        gl::TEXTURE_2D, 0, gl::RGB as i32, 
                        decoded.width() as i32, decoded.height() as i32, 
                        0, gl::RGB, gl::UNSIGNED_BYTE, 
                        decoded.as_raw().as_ptr() as *const _
                    );

                    // Grab the active shader program
                    let active_program = shader_programs[current_mode];
                    gl::UseProgram(active_program);
                    
                    // Use Shader & Set Uniforms
                    set_uniform_1i(active_program, "videoTexture", 0);
                    set_uniform_1i(active_program, "fontAtlas", 1);
                    set_uniform_2f(active_program, "resolution", decoded.width() as f32, decoded.height() as f32);
                    set_uniform_2f(active_program, "charSize", 8.0, 16.0);
                    set_uniform_1f(active_program, "numChars", 10.0);
                    set_uniform_1f(active_program, "sensitivity", 1.5);
                    
                    // Get time from GLFW and pass it to the matrix shaders so the rain animates
                    let time = glfw.get_time() as f32;
                    set_uniform_1f(active_program, "time", time);

                    // Draw
                    gl::BindVertexArray(vao);
                    gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
                }
            }
        }
        window.swap_buffers();
    }
}

// --- Minimal Helper Functions ---

fn compile_shader_program(vert_path: &str, frag_path: &str) -> u32 {
    let v_src = fs::read_to_string(vert_path).expect("Failed to read vertex shader");
    let f_src = fs::read_to_string(frag_path).expect("Failed to read fragment shader");

    unsafe {
        let vertex = gl::CreateShader(gl::VERTEX_SHADER);
        let c_str_v = CString::new(v_src.as_bytes()).unwrap();
        gl::ShaderSource(vertex, 1, &c_str_v.as_ptr(), ptr::null());
        gl::CompileShader(vertex);
        check_shader_compile(vertex, "VERTEX");

        let fragment = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_f = CString::new(f_src.as_bytes()).unwrap();
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

// Helper to print out shader compilation errors
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

unsafe fn set_uniform_1i(program: u32, name: &str, value: i32) {
    let c_name = CString::new(name).unwrap();
    let location = gl::GetUniformLocation(program, c_name.as_ptr());
    if location != -1 {
        gl::Uniform1i(location, value);
    }
}

unsafe fn set_uniform_1f(program: u32, name: &str, value: f32) {
    let c_name = CString::new(name).unwrap();
    let location = gl::GetUniformLocation(program, c_name.as_ptr());
    if location != -1 {
        gl::Uniform1f(location, value);
    }
}

unsafe fn set_uniform_2f(program: u32, name: &str, v1: f32, v2: f32) {
    let c_name = CString::new(name).unwrap();
    let location = gl::GetUniformLocation(program, c_name.as_ptr());
    if location != -1 {
        gl::Uniform2f(location, v1, v2);
    }
}
