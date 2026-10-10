use glfw::{Action, Context, Key};
use opencv::{prelude::*, videoio, imgproc, core};
use std::env;
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
    
    // Enable VSync so the video loop doesn't render as fast as the CPU allows
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    // Load OpenGL function pointers
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    // 2. Initialize Video Input (Camera or File)
    let args: Vec<String> = env::args().collect();
    let mut is_file = false;

    let mut cam = if args.len() > 1 {
        let input = &args[1];
        
        // Try to parse the argument as a number first (for camera index)
        if let Ok(cam_index) = input.parse::<i32>() {
            println!("Opening camera index {}...", cam_index);
            videoio::VideoCapture::new(cam_index, videoio::CAP_ANY).expect("Failed to open camera")
        } else {
            // If it's not a number, treat it as a file path
            println!("Opening video file {}...", input);
            is_file = true;
            videoio::VideoCapture::from_file(input, videoio::CAP_ANY).expect("Failed to open video file")
        }
    } else {
        println!("No input specified. Defaulting to camera 0...");
        videoio::VideoCapture::new(0, videoio::CAP_ANY).expect("Failed to open camera")
    };

    let cam_width = cam.get(videoio::CAP_PROP_FRAME_WIDTH).unwrap() as i32;
    let cam_height = cam.get(videoio::CAP_PROP_FRAME_HEIGHT).unwrap() as i32;
    let fps = cam.get(videoio::CAP_PROP_FPS).unwrap();
    
    println!("Input opened successfully at {}x{} ({} fps)", cam_width, cam_height, fps);

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

    // 5. Setup Camera Video Texture (OPTIMIZED)
    let mut video_texture = 0;
    unsafe {
        gl::GenTextures(1, &mut video_texture);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, video_texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        
        // Prevent skewing/crashing on weird resolutions
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        
        // Allocate empty GPU memory ONCE using the known video resolution
        gl::TexImage2D(
            gl::TEXTURE_2D, 0, gl::RGB as i32, 
            cam_width as i32, cam_height as i32, 
            0, gl::RGB, gl::UNSIGNED_BYTE, ptr::null()
        );
    }

    // 6. Compile Shaders (PORTABILITY + CACHING)
    let vert_src = include_str!("../shaders/vert/shader.vert");
    
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

    // Pre-allocate Mats for OpenCV to prevent allocating memory inside the hot loop
    let mut frame = core::Mat::default();
    let mut rgb_frame = core::Mat::default();

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

        // Grab frame from video file or camera
        if cam.read(&mut frame).unwrap_or(false) && !frame.empty() {
            // Convert OpenCV's default BGR to RGB for OpenGL (Using OpenCV 5.0 default arguments fix)
            imgproc::cvt_color_def(&frame, &mut rgb_frame, imgproc::COLOR_BGR2RGB).unwrap();

            let prog = &shader_programs[current_mode];

            unsafe {
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, video_texture);
                
                // Use TexSubImage2D to update existing memory 
                gl::TexSubImage2D(
                    gl::TEXTURE_2D, 0, 0, 0, 
                    cam_width, cam_height, 
                    gl::RGB, gl::UNSIGNED_BYTE, 
                    rgb_frame.data() as *const _
                );

                // Use active shader program
                gl::UseProgram(prog.id);
                
                // Set uniforms using the pre-cached locations
                if prog.loc_video_texture != -1 { gl::Uniform1i(prog.loc_video_texture, 0); }
                if prog.loc_font_atlas != -1 { gl::Uniform1i(prog.loc_font_atlas, 1); }
                if prog.loc_resolution != -1 { gl::Uniform2f(prog.loc_resolution, cam_width as f32, cam_height as f32); }
                if prog.loc_char_size != -1 { gl::Uniform2f(prog.loc_char_size, 8.0, 16.0); }
                if prog.loc_num_chars != -1 { gl::Uniform1f(prog.loc_num_chars, 10.0); }
                if prog.loc_sensitivity != -1 { gl::Uniform1f(prog.loc_sensitivity, 1.5); }
                
                // Pass time to the matrix shaders so the rain animates
                if prog.loc_time != -1 { gl::Uniform1f(prog.loc_time, glfw.get_time() as f32); }

                // Draw
                gl::BindVertexArray(vao);
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            }
        } else if is_file {
            // Video file ended. Loop back to frame 0.
            cam.set(videoio::CAP_PROP_POS_FRAMES, 0.0).unwrap();
        } else {
            // Camera stream died or disconnected. Exit the loop.
            eprintln!("Camera feed lost!");
            break;
        }
        
        window.swap_buffers();
    }
}

// --- Helper Functions ---

// Takes raw string slices from include_str!
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
