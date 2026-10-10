use std::ffi::CString;
use std::ptr;

pub struct ShaderProgram {
    pub id: u32,
    pub loc_video_texture: i32,
    pub loc_font_atlas: i32,
    pub loc_resolution: i32,
    pub loc_char_size: i32,
    pub loc_num_chars: i32,
    pub loc_sensitivity: i32,
    pub loc_time: i32,
    pub loc_rain_speed: i32,
    pub loc_tail_length: i32,
    pub loc_flicker_speed: i32,
    pub loc_resonance_ripples: i32,
    pub loc_vortex_speed: i32,
    pub loc_vortex_arms: i32,
    pub loc_vortex_spin: i32,
    pub loc_vortex_center_size: i32,
    pub loc_vortex_fov: i32,
    pub loc_vortex_color_separation: i32,
    pub loc_vortex_strobe_size: i32,
    pub loc_audio_bass: i32,
    pub loc_audio_mid: i32,
    pub loc_audio_treble: i32,
    pub loc_audio_volume: i32,
    pub loc_audio_peak_freq: i32,
    pub loc_crop: i32,
}

pub fn compile_shader_program(v_src: &str, f_src: &str) -> u32 {
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
