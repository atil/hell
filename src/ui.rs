use crate::shader::*;
use crate::texture;
use crate::ui_batch::*;
use rusttype::Font;
use std::ffi::CString;

pub struct Ui<'a> {
    batches: Vec<Batch>,
    program: Program,
    font: Font<'a>,
}

impl Ui<'_> {
    pub fn init() -> Self {
        let font_data = include_bytes!("../assets/RobotoMono-Regular.ttf");
        let font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");

        let vert_shader =
            Shader::from_vert_source(&CString::new(include_str!("ui.vert")).unwrap()).unwrap();

        let frag_shader =
            Shader::from_frag_source(&CString::new(include_str!("ui.frag")).unwrap()).unwrap();

        let texture1 = texture::load_from_file("assets/prototype.png");
        let texture2 = texture::create_from_text("Progress", 32.0, &font);

        let program = Program::from_shaders(&[vert_shader, frag_shader]).unwrap();

        let rekt1 = Rect::new(-0.01, 0.01, 0.02, 0.02);
        let rekt2 = Rect::new(-0.4, -0.9, 0.2, 0.1);

        let batches = vec![
            Batch::new(vec![rekt1], texture1, false),
            Batch::new(vec![rekt2], texture2, false),
        ];

        unsafe {
            program.set_i32("texture_ui", 0);
        }

        Self {
            batches: batches,
            program: program,
            font: font,
        }
    }

    pub unsafe fn draw(&mut self) {
        self.program.set_used();

        for batch in self.batches.iter() {
            batch.draw();
        }

        self.batches.retain(|b| !b.draw_single_frame);
    }

    pub fn draw_text(&mut self, text: &str) {
        let rect = Rect::new(-0.9, 0.9, 0.2, 0.2); // TODO: Provide this from the outside
        let texture = texture::create_from_text(text, 32.0, &self.font);

        self.batches.push(Batch::new(vec![rect], texture, true));
    }
}
