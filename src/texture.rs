use crate::render::TextureHandle;
use gl::types::*;
use image::{GenericImageView};
use std::path::Path;

pub fn load_from_file(texture_path: &str) -> TextureHandle {
    let mut texture_handle = 0;
    unsafe {
        gl::GenTextures(1, &mut texture_handle);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture_handle);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        let img = image::open(&Path::new(&texture_path))
            .expect(format!("Error loading the texture {:?}", texture_path).as_str());
        let img = img.flipv();
        let img_data = img.to_bytes();
        if img.color() != image::ColorType::Rgba8 {
            panic!(
                "Image channels isn't RGBA8, instead {:?} for image file {:?}",
                img.color(),
                texture_path
            );
        }
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            img.width() as i32,
            img.height() as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            &img_data[0] as *const u8 as *const GLvoid,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }
    texture_handle
}
