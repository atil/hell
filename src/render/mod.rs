pub mod directional_light;
pub mod material;
pub mod point_light;
pub mod renderer;
pub mod shader;
pub mod skybox;
pub mod texture;
pub mod ui;
pub mod ui_batch;

pub type TextureHandle = u32;
pub type BufferHandle = u32;

pub const SCREEN_SIZE: (u32, u32) = (1280, 720);
pub const SHADOWMAP_SIZE: i32 = 2048;
pub const DRAW_FRAMEBUFFER_SIZE: (u32, u32) = (640, 360);
pub const SIZEOF_FLOAT: usize = std::mem::size_of::<f32>();
pub const FAR_PLANE: f32 = 1000.0;
pub const NEAR_PLANE: f32 = 0.1;

pub unsafe fn check_gl_error(tag: &str) {
    let error = gl::GetError();

    if error != 0 {
        println!("[{0}] error: {1}", tag, error);
    }
}
