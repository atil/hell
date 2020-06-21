use crate::render;
use crate::shader::*;
use crate::texture;
use crate::ui_batch::*;
use rusttype::Font;

pub struct Ui<'a> {
    batches: Vec<Batch>,
    shader: Shader,
    font: Font<'a>,
}

impl Ui<'_> {
    pub fn init() -> Self {
        let font_data = include_bytes!("../assets/RobotoMono-Regular.ttf");
        let font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");
        let _texture1 = texture::load_from_file("assets/prototype.png");
        let texture2 = texture::create_from_text("Progress", 32.0, &font);

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

    pub fn draw_text(&mut self, text: &str) {
        let rect = Rect::new(-0.9, 0.9, 0.2, 0.2); // TODO: Provide this from the outside
        let texture = texture::create_from_text(text, 32.0, &self.font);

        self.batches.push(Batch::new(vec![rect], texture, true));
    }

    pub unsafe fn draw(&mut self) {
        self.shader.set_used();

        gl::Viewport(
            0,
            0,
            render::SCREEN_SIZE.0 as i32,
            render::SCREEN_SIZE.1 as i32,
        );

        for batch in self.batches.iter() {
            batch.draw();
        }

        self.batches.retain(|b| !b.draw_single_frame);
    }
}
