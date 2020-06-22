use crate::render;
use crate::render::{BufferHandle, TextureHandle};
use crate::shader::*;
use cgmath::*;
use image::GenericImageView;
use std::path::Path;

const VERTEX_DATA: [f32; 108] = [
    -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0,
    -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0,
    -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0,
    1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0,
    -1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, 1.0,
    -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, -1.0,
    -1.0, 1.0, 1.0, -1.0, 1.0,
];

pub struct Skybox {
    vao: BufferHandle,
    cubemap_handle: TextureHandle,
    shader: Shader,
}

impl Skybox {
    pub fn new(projection: Matrix4<f32>) -> Skybox {
        let mut vao = 0;
        let mut vbo = 0;
        let cubemap_handle: TextureHandle;

        let shader = Shader::from_file("src/shaders/skybox.glsl", false)
            .expect("Problem loading skybox shader");

        unsafe {
            gl::GenBuffers(1, &mut vbo);
            gl::GenVertexArrays(1, &mut vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (VERTEX_DATA.len() * render::SIZEOF_FLOAT) as isize,
                VERTEX_DATA.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW,
            );

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (3 * render::SIZEOF_FLOAT) as i32,
                std::ptr::null(),
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            cubemap_handle = load_cubemap_from_file("assets/skybox/gehenna");

            shader.set_used();
            shader.set_mat4("u_projection", projection);
            shader.set_i32("u_skybox", 0);
        }

        Skybox {
            vao: vao,
            cubemap_handle: cubemap_handle,
            shader: shader,
        }
    }

    pub unsafe fn draw(&mut self, player_v: Matrix4<f32>) {
        self.shader.set_used();

        gl::DepthFunc(gl::LEQUAL);
        let skybox_view = Matrix4::from_cols(player_v.x, player_v.y, player_v.z, Vector4::zero());
        self.shader.set_mat4("u_view", skybox_view);
        gl::BindVertexArray(self.vao);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.cubemap_handle);
        gl::DrawArrays(gl::TRIANGLES, 0, 36);
        gl::BindVertexArray(0);
        gl::DepthFunc(gl::LESS);
    }
}

unsafe fn load_cubemap_from_file(cubemap_path: &str) -> TextureHandle {
    let mut cubemap_handle = 0;

    gl::GenTextures(1, &mut cubemap_handle);
    gl::BindTexture(gl::TEXTURE_CUBE_MAP, cubemap_handle);

    load_cubemap_face(
        format!("{}_front.png", cubemap_path).as_str(),
        gl::TEXTURE_CUBE_MAP_POSITIVE_Z,
    );
    load_cubemap_face(
        format!("{}_back.png", cubemap_path).as_str(),
        gl::TEXTURE_CUBE_MAP_NEGATIVE_Z,
    );
    load_cubemap_face(
        format!("{}_left.png", cubemap_path).as_str(),
        gl::TEXTURE_CUBE_MAP_NEGATIVE_X,
    );
    load_cubemap_face(
        format!("{}_right.png", cubemap_path).as_str(),
        gl::TEXTURE_CUBE_MAP_POSITIVE_X,
    );
    load_cubemap_face(
        format!("{}_top.png", cubemap_path).as_str(),
        gl::TEXTURE_CUBE_MAP_POSITIVE_Y,
    );
    load_cubemap_face(
        format!("{}_bottom.png", cubemap_path).as_str(),
        gl::TEXTURE_CUBE_MAP_NEGATIVE_Y,
    );

    gl::TexParameteri(
        gl::TEXTURE_CUBE_MAP,
        gl::TEXTURE_WRAP_S,
        gl::CLAMP_TO_EDGE as i32,
    );
    gl::TexParameteri(
        gl::TEXTURE_CUBE_MAP,
        gl::TEXTURE_WRAP_T,
        gl::CLAMP_TO_EDGE as i32,
    );
    gl::TexParameteri(
        gl::TEXTURE_CUBE_MAP,
        gl::TEXTURE_WRAP_R,
        gl::CLAMP_TO_EDGE as i32,
    );
    gl::TexParameteri(
        gl::TEXTURE_CUBE_MAP,
        gl::TEXTURE_MIN_FILTER,
        gl::LINEAR as i32,
    );
    gl::TexParameteri(
        gl::TEXTURE_CUBE_MAP,
        gl::TEXTURE_MAG_FILTER,
        gl::LINEAR as i32,
    );

    cubemap_handle
}

fn load_cubemap_face(cubemap_face_path: &str, texture_enum: u32) {
    let img = image::open(&Path::new(&cubemap_face_path))
        .expect(format!("Error loading the cubemap face {:?}", cubemap_face_path).as_str());
    // let img = img.flipv();
    if img.color() != image::ColorType::Rgba8 {
        panic!(
            "Image channels isn't RGBA8, instead {:?} for image file {:?}",
            img.color(),
            cubemap_face_path
        );
    }

    unsafe {
        gl::TexImage2D(
            texture_enum,
            0,
            gl::RGBA as i32,
            img.width() as i32,
            img.height() as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            &img.to_bytes()[0] as *const u8 as *const std::ffi::c_void,
        );
    }
}
