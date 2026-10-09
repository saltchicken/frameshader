use std::ffi::CString;
use std::fs;
use std::ptr;

pub struct Shader {
    pub id: u32,
}

impl Shader {
    pub fn new(vertex_path: &str, fragment_path: &str) -> Result<Self, String> {
        let v_shader_code = fs::read_to_string(vertex_path).map_err(|e| e.to_string())?;
        let f_shader_code = fs::read_to_string(fragment_path).map_err(|e| e.to_string())?;

        unsafe {
            let vertex = Self::compile_shader(&v_shader_code, gl::VERTEX_SHADER)?;
            let fragment = Self::compile_shader(&f_shader_code, gl::FRAGMENT_SHADER)?;

            let id = gl::CreateProgram();
            gl::AttachShader(id, vertex);
            gl::AttachShader(id, fragment);
            gl::LinkProgram(id);

            // Add glGetProgramiv linking error checks here (like in your C++ code)

            gl::DeleteShader(vertex);
            gl::DeleteShader(fragment);

            Ok(Self { id })
        }
    }

    pub fn use_program(&self) {
        unsafe { gl::UseProgram(self.id); }
    }

    pub fn set_float(&self, name: &str, value: f32) {
        let c_name = CString::new(name).unwrap();
        unsafe {
            let location = gl::GetUniformLocation(self.id, c_name.as_ptr());
            gl::Uniform1f(location, value);
        }
    }

    unsafe fn compile_shader(source: &str, kind: u32) -> Result<u32, String> {
        let shader = gl::CreateShader(kind);
        let c_str = CString::new(source.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);
        
        // Add glGetShaderiv compilation error checks here
        Ok(shader)
    }
}
