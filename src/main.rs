use glfw::{Action, Context, Key};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;
use std::ffi::CString;
use std::ptr;

// Create a struct to hold our compiled programs and their uniform locations
struct ShaderProgram {
    id: u32,
    // Caching these avoids hitting the OpenGL string lookup every frame
    loc_video_texture: i32,
    loc_font_atlas: i32,
    loc_resolution: i32,
    loc_char_size: i32,
    loc_num_chars: i32,
    loc_sensitivity: i32,
    loc_time: i32,
}

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
    
    // Ask nokhwa to look at what the camera supports and pick the highest resolution
    let mut cam = Camera::new(
        CameraIndex::Index(0),
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution),
    ).expect("Failed to initialize camera at Index 0");
    
    cam.open_stream().expect("Failed to open camera stream");
    
    let actual_format = cam.camera_format();
    let cam_width = actual_format.resolution().width();
    let cam_height = actual_format.resolution().height();
    
    println!("Camera stream opened successfully at {}x{} ({} fps)", 
        cam_width, 
        cam_height,
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
    // Note: Since this is read at runtime, ensure the font_atlases directory is present next to the executable.
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

    // 5. Setup Camera Video Texture (OPTIMIZED)
    let mut video_texture = 0;
    unsafe {
        gl::GenTextures(1, &mut video_texture);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, video_texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        
        // Prevent skewing/crashing on weird camera resolutions
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        
        // Allocate empty GPU memory ONCE using the known camera resolution
        gl::TexImage2D(
            gl::TEXTURE_2D, 0, gl::RGB as i32, 
            cam_width as i32, cam_height as i32, 
            0, gl::RGB, gl::UNSIGNED_BYTE, ptr::null()
        );
    }

    // 6. Compile Shaders (PORTABILITY + CACHING)
    // include_str! embeds the shaders directly into the binary at compile time.
    let vert_src = include_str!("../shaders/vert/shader.vert");
    
    // We pair a human-readable name with the baked-in source string
    let fragment_sources = [
        ("ASCII", include_str!("../shaders/frag/ascii.frag")),
        ("Matrix", include_str!("../shaders/frag/ascii_matrix.frag")),
        ("Color", include_str!("../shaders/frag/ascii_matrix_color.frag")),
        ("Flux", include_str!("../shaders/frag/ascii_matrix_flux.frag")),
    ];

    let mut shader_programs = Vec::new();
    for (name, frag_src) in fragment_sources.iter() {
        println!("Compiling {}...", name);
        let id = compile_shader_program(vert_src, frag_src);
        
        // Cache uniform locations at startup
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
            });
        }
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
                    println!("Switched to: {}", fragment_sources[current_mode].0);
                },
                // Idiomatic pattern matching for number keys 1-4
                glfw::WindowEvent::Key(key @ (Key::Num1 | Key::Num2 | Key::Num3 | Key::Num4), _, Action::Press, _) => {
                    current_mode = (key as usize) - (Key::Num1 as usize);
                    println!("Switched to: {}", fragment_sources[current_mode].0);
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
                let prog = &shader_programs[current_mode];

                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0);
                    gl::BindTexture(gl::TEXTURE_2D, video_texture);
                    
                    // PERFORMANCE FIX: Use TexSubImage2D to update existing memory 
                    // rather than reallocating memory every frame.
                    gl::TexSubImage2D(
                        gl::TEXTURE_2D, 0, 0, 0, 
                        decoded.width() as i32, decoded.height() as i32, 
                        gl::RGB, gl::UNSIGNED_BYTE, 
                        decoded.as_raw().as_ptr() as *const _
                    );

                    // Use active shader program
                    gl::UseProgram(prog.id);
                    
                    // Set uniforms using the pre-cached locations
                    // Note: Unused uniforms in specific shaders will silently be ignored by OpenGL (location = -1)
                    if prog.loc_video_texture != -1 { gl::Uniform1i(prog.loc_video_texture, 0); }
                    if prog.loc_font_atlas != -1 { gl::Uniform1i(prog.loc_font_atlas, 1); }
                    if prog.loc_resolution != -1 { gl::Uniform2f(prog.loc_resolution, decoded.width() as f32, decoded.height() as f32); }
                    if prog.loc_char_size != -1 { gl::Uniform2f(prog.loc_char_size, 8.0, 16.0); }
                    if prog.loc_num_chars != -1 { gl::Uniform1f(prog.loc_num_chars, 10.0); }
                    if prog.loc_sensitivity != -1 { gl::Uniform1f(prog.loc_sensitivity, 1.5); }
                    
                    // Pass time to the matrix shaders so the rain animates
                    if prog.loc_time != -1 { gl::Uniform1f(prog.loc_time, glfw.get_time() as f32); }

                    // Draw
                    gl::BindVertexArray(vao);
                    gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
                }
            }
        }
        window.swap_buffers();
    }
}

// --- Helper Functions ---

// Modified to take raw string slices from include_str! instead of file paths
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
