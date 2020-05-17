use cgmath::{Matrix, Matrix4};
use gl;
use gl::types::*;
use std;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::prelude::*;

const VERSION: &'static str = "#version 420 core\r\n";
const DEFINE_VERTEX: &'static str = "#define VERTEX\r\n";
const DEFINE_FRAGMENT: &'static str = "#define FRAGMENT\r\n";

pub struct Program {
    id: gl::types::GLuint,
}

impl Program {
    pub fn from_shader(path: &str) -> Result<Program, String> {
        let mut shader_file = File::open(path).expect("Unable to open shader file");
        let mut shader_text = String::new();
        shader_file
            .read_to_string(&mut shader_text)
            .expect("Unable to read shader file");

        let program_id = unsafe { gl::CreateProgram() };
        let vert_id = Program::create_shader(&shader_text, gl::VERTEX_SHADER, DEFINE_VERTEX)?;
        let frag_id = Program::create_shader(&shader_text, gl::FRAGMENT_SHADER, DEFINE_FRAGMENT)?;
        unsafe {
            let mut success: GLint = 1;
            gl::AttachShader(program_id, vert_id);
            gl::AttachShader(program_id, frag_id);
            gl::LinkProgram(program_id);
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);

            if success == 0 {
                let mut len: GLint = 0;
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);

                let error = create_whitespace_cstring_with_len(len as usize);
                gl::GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut GLchar,
                );

                return Err(error.to_string_lossy().into_owned());
            }

            gl::DetachShader(program_id, vert_id);
            gl::DetachShader(program_id, frag_id);
        }

        Ok(Program { id: program_id })
    }

    fn create_shader(source: &str, shader_type: GLenum, define: &str) -> Result<GLuint, String> {
        let id = unsafe { gl::CreateShader(shader_type) };
        let mut success: GLint = 1;

        unsafe {
            let shader_text = format!("{}{}{}", VERSION, define, source);
            let cstr: &CStr = &CString::new(shader_text).unwrap();
            gl::ShaderSource(id, 1, &cstr.as_ptr(), std::ptr::null());
            gl::CompileShader(id);
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);

            if success == 0 {
                let mut len: GLint = 0;
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);

                let error = create_whitespace_cstring_with_len(len as usize);
                gl::GetShaderInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar);

                return Err(error.to_string_lossy().into_owned());
            }
        }

        Ok(id)
    }

    pub unsafe fn set_used(&self) {
        gl::UseProgram(self.id);
    }

    pub unsafe fn set_matrix(&self, name: &str, matrix: Matrix4<f32>) {
        gl::ProgramUniformMatrix4fv(self.id, self.loc(name), 1, gl::FALSE, matrix.as_ptr());
    }

    pub unsafe fn set_i32(&self, name: &str, i: i32) {
        gl::Uniform1i(self.loc(name), i);
    }

    pub unsafe fn set_vec3(&self, name: &str, vec: [f32; 3]) {
        gl::Uniform3f(self.loc(name), vec[0], vec[1], vec[2]);
    }
    unsafe fn loc(&self, name: &str) -> i32 {
        gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr())
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    buffer.extend([b' '].iter().cycle().take(len));
    unsafe { CString::from_vec_unchecked(buffer) }
}
