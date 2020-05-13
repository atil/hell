use gl::types::*;
use image::{DynamicImage, GenericImageView, Rgba};
use rusttype::{point, Font, Scale};
use std::path::Path;

pub fn load_from_file(texture_path: &str) -> u32 {
    let mut texture = 0;
    unsafe {
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        let img = image::open(&Path::new(&texture_path)).expect("Error loading the texture");
        let img = img.flipv();
        let img_data = img.to_bytes();
        if img.color() != image::ColorType::Rgba8 {
            panic!("Image channels isn't RGBA8, instead {:?}", img.color());
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
    texture
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
