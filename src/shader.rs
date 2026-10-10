use std::ffi::CString;
use std::ptr;
use std::collections::HashMap;

pub struct ShaderProgram {
    pub id: u32,
    pub uniforms: HashMap<String, i32>,
}

pub fn compile_shader_program(v_src: &str, f_src: &str) -> Option<ShaderProgram> {
    unsafe {
        let vertex = gl::CreateShader(gl::VERTEX_SHADER);
        let c_str_v = CString::new(v_src).unwrap();
        gl::ShaderSource(vertex, 1, &c_str_v.as_ptr(), ptr::null());
        gl::CompileShader(vertex);
        if !check_shader_compile(vertex, "VERTEX") {
            gl::DeleteShader(vertex);
            return None;
        }

        let fragment = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_f = CString::new(f_src).unwrap();
        gl::ShaderSource(fragment, 1, &c_str_f.as_ptr(), ptr::null());
        gl::CompileShader(fragment);
        if !check_shader_compile(fragment, "FRAGMENT") {
            gl::DeleteShader(vertex);
            gl::DeleteShader(fragment);
            return None;
        }

        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex);
        gl::AttachShader(program, fragment);
        gl::LinkProgram(program);

        gl::DeleteShader(vertex);
        gl::DeleteShader(fragment);
        
        // Dynamically locate all active uniforms
        let mut uniforms = HashMap::new();
        let mut count = 0;
        gl::GetProgramiv(program, gl::ACTIVE_UNIFORMS, &mut count);
        
        for i in 0..count {
            let mut length = 0;
            let mut size = 0;
            let mut type_ = 0;
            let mut name_buf = vec![0u8; 256];
            
            gl::GetActiveUniform(
                program,
                i as u32,
                name_buf.len() as i32,
                &mut length,
                &mut size,
                &mut type_,
                name_buf.as_mut_ptr() as *mut i8,
            );
            
            if length > 0 {
                let name_str = String::from_utf8_lossy(&name_buf[..length as usize]).into_owned();
                let c_name = CString::new(name_str.clone()).unwrap();
                let location = gl::GetUniformLocation(program, c_name.as_ptr());
                uniforms.insert(name_str, location);
            }
        }
        
        Some(ShaderProgram { id: program, uniforms })
    }
}

unsafe fn check_shader_compile(shader: u32, shader_type: &str) -> bool {
    let mut success = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    if success == 0 {
        let mut len = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        let mut buffer: Vec<u8> = vec![0; len as usize];
        gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut i8);
        eprintln!("{} Shader Compilation Error:\n{}", shader_type, String::from_utf8_lossy(&buffer));
        return false;
    }
    true
}
