use crate::render;
use gl::types::*;
use image::{DynamicImage, GenericImageView, Rgba};
use rusttype::{point, Font, Scale};
use std::path::Path;

pub fn load_from_file(texture_path: &str) -> u32 {
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

pub fn create_from_text(content: &str, size: f32, font: &Font) -> u32 {
    let scale = Scale::uniform(size);
    let color = (255, 0, 255);
    let v_metrics = font.v_metrics(scale);
    let glyphs: Vec<_> = font
        .layout(content, scale, point(20.0, 20.0 + v_metrics.ascent))
        .collect();
    let glyphs_height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;
    let glyphs_width = {
        let min_x = glyphs
            .first()
            .map(|g| g.pixel_bounding_box().unwrap().min.x)
            .unwrap();
        let max_x = glyphs
            .last()
            .map(|g| g.pixel_bounding_box().unwrap().max.x)
            .unwrap();
        (max_x - min_x) as u32
    };

    // TODO: What are those 40's?
    let mut img = DynamicImage::new_rgba8(glyphs_width + 40, glyphs_height + 40).to_rgba();
    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                img.put_pixel(
                    x + bounding_box.min.x as u32,
                    y + bounding_box.min.y as u32,
                    Rgba([color.0, color.1, color.2, (v * 255.0) as u8]),
                )
            });
        }
    }

    let mut texture = 0;
    unsafe {
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        img = image::imageops::flip_vertical(&img);
        let width = img.width();
        let height = img.height();
        let img_data = img.into_raw();
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            width as i32,
            height as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            &img_data[0] as *const u8 as *const GLvoid,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }
    texture
}

pub unsafe fn create_depth_texture() -> u32 {
    let mut depth_texture_handle = 0;
    gl::GenTextures(1, &mut depth_texture_handle);
    gl::BindTexture(gl::TEXTURE_2D, depth_texture_handle);
    gl::TexParameteri(
        gl::TEXTURE_2D,
        gl::TEXTURE_WRAP_S,
        gl::CLAMP_TO_BORDER as i32,
    );
    gl::TexParameteri(
        gl::TEXTURE_2D,
        gl::TEXTURE_WRAP_T,
        gl::CLAMP_TO_BORDER as i32,
    );
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

    let border_color = vec![1.0, 1.0, 1.0, 1.0];
    gl::TexParameterfv(
        gl::TEXTURE_2D,
        gl::TEXTURE_BORDER_COLOR,
        border_color.as_ptr(),
    );

    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::DEPTH_COMPONENT as i32,
        render::SHADOWMAP_SIZE as i32,
        render::SHADOWMAP_SIZE as i32,
        0,
        gl::DEPTH_COMPONENT,
        gl::FLOAT,
        std::ptr::null(),
    );

    depth_texture_handle
}

pub unsafe fn load_cubemap_from_file(cubemap_path: &str) -> u32 {
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

fn load_cubemap_face(cubemap_face_path: &str, texture_enum: GLenum) {
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
            &img.to_bytes()[0] as *const u8 as *const GLvoid,
        );
    }
}
