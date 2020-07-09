use crate::render;
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
const DEFINE_GEOMETRY: &'static str = "#define GEOMETRY\r\n";

pub struct Shader {
    id: u32,
}

impl Shader {
    pub fn from_file(path: &str, has_geom: bool) -> Result<Shader, String> {
        let mut shader_file =
            File::open(path).expect(format!("Unable to open shader file: {}", path).as_str());
        let mut shader_text = String::new();
        shader_file
            .read_to_string(&mut shader_text)
            .expect("Unable to read shader file");

        let shader_id = unsafe { gl::CreateProgram() };
        let vert_id = Shader::from_source(&shader_text, gl::VERTEX_SHADER, DEFINE_VERTEX)?;
        let frag_id = Shader::from_source(&shader_text, gl::FRAGMENT_SHADER, DEFINE_FRAGMENT)?;
        let geom_id = {
            if has_geom {
                Shader::from_source(&shader_text, gl::GEOMETRY_SHADER, DEFINE_GEOMETRY)?
            } else {
                0
            }
        };
        unsafe {
            let mut success: GLint = 1;
            gl::AttachShader(shader_id, vert_id);
            gl::AttachShader(shader_id, frag_id);
            if has_geom {
                gl::AttachShader(shader_id, geom_id);
            }

            gl::LinkProgram(shader_id);
            gl::GetProgramiv(shader_id, gl::LINK_STATUS, &mut success);

            if success == 0 {
                let mut len: i32 = 0;
                gl::GetProgramiv(shader_id, gl::INFO_LOG_LENGTH, &mut len);

                let error = create_whitespace_cstring_with_len(len as usize);
                gl::GetProgramInfoLog(
                    shader_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut GLchar,
                );

                return Err(error.to_string_lossy().into_owned());
            }

            gl::DetachShader(shader_id, vert_id);
            gl::DetachShader(shader_id, frag_id);
            if has_geom {
                gl::DetachShader(shader_id, geom_id);
            }
        }

        Ok(Shader { id: shader_id })
    }

    fn from_source(source: &str, shader_type: GLenum, define: &str) -> Result<GLuint, String> {
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

    pub unsafe fn set_mat4(&self, name: &str, matrix: Matrix4<f32>) {
        let cstr = CString::new(name).unwrap();
        let loc = gl::GetUniformLocation(self.id, cstr.as_ptr());

        gl::ProgramUniformMatrix4fv(self.id, loc, 1, gl::FALSE, matrix.as_ptr());
        render::check_gl_error(format!("{} mat4", name).as_str());
    }

    pub unsafe fn set_i32(&self, name: &str, i: i32) {
        let cstr = CString::new(name).unwrap();
        let loc = gl::GetUniformLocation(self.id, cstr.as_ptr());

        gl::Uniform1i(loc, i);
        render::check_gl_error(format!("{} i32", name).as_str());
    }

    pub unsafe fn set_f32(&self, name: &str, f: f32) {
        let cstr = CString::new(name).unwrap();
        let loc = gl::GetUniformLocation(self.id, cstr.as_ptr());

        gl::Uniform1f(loc, f);
        render::check_gl_error(format!("{} f32", name).as_str());
    }

    pub unsafe fn set_vec3(&self, name: &str, f0: f32, f1: f32, f2: f32) {
        let cstr = CString::new(name).unwrap();
        let loc = gl::GetUniformLocation(self.id, cstr.as_ptr());

        gl::Uniform3f(loc, f0, f1, f2);
        render::check_gl_error(format!("{} vec3", name).as_str());
    }

    pub unsafe fn set_vec4(&self, name: &str, f0: f32, f1: f32, f2: f32, f3: f32) {
        let cstr = CString::new(name).unwrap();
        let loc = gl::GetUniformLocation(self.id, cstr.as_ptr());

        gl::Uniform4f(loc, f0, f1, f2, f3);
        render::check_gl_error(format!("{} vec4", name).as_str());
    }
}

impl Drop for Shader {
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
