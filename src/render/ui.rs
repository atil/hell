use crate::geom::*;
use crate::player::Player;
use crate::render::shader::*;
use crate::render::texture;
use crate::render::ui_batch::*;
use crate::render::*;
use cgmath::*;
use gl::types::*;
use image::{DynamicImage, Rgba};
use rusttype::{point, Font, Scale};

pub struct Ui<'a> {
    batches: Vec<Batch>,
    shader: Shader,
    font: Font<'a>,
}

impl Ui<'_> {
    pub fn init() -> Self {
        let font_data = include_bytes!("../../assets/RobotoMono-Regular.ttf");
        let font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");
        let _texture1 = texture::load_from_file("assets/prototype.png");
        let texture2 = create_from_text("Progress", 32.0, &font);

        let shader =
            Shader::from_file("src/shaders/ui.glsl", false).expect("Error loading ui shader");

        let _rekt1 = Rect::new(-0.01, 0.01, 0.02, 0.02);
        let rekt2 = Rect::new(-0.4, -0.9, 0.2, 0.1);

        let batches = vec![
            // Batch::new(vec![rekt1], texture1, false),
            Batch::new(vec![rekt2], texture2, false),
        ];

        unsafe {
            shader.set_used();
            shader.set_i32("texture_ui", 0);
        }

        Self {
            batches: batches,
            shader: shader,
            font: font,
        }
    }

    fn draw_text(&mut self, text: &str) {
        let rect = Rect::new(-0.9, 0.9, 0.2, 0.2); // TODO: Provide this from the outside
        let texture = create_from_text(text, 32.0, &self.font);

        self.batches.push(Batch::new(vec![rect], texture, true));
    }

    pub unsafe fn draw(&mut self, player: &Player) {
        self.shader.set_used();

        let velocity_string = format!("{:.3}", horz(&player.velocity).magnitude());
        self.draw_text(velocity_string.as_str());

        gl::Viewport(0, 0, SCREEN_SIZE.0 as i32, SCREEN_SIZE.1 as i32);
        for batch in self.batches.iter() {
            batch.draw();
        }
        self.batches.retain(|b| !b.draw_single_frame);
    }
}

fn create_from_text(content: &str, size: f32, font: &Font) -> TextureHandle {
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
