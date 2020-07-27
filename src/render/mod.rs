mod directional_light;
pub mod material;
mod point_light;
pub mod renderer;
mod shader;
mod skybox;
mod texture;
pub mod ui;
mod ui_batch;

type TextureHandle = u32;
type BufferHandle = u32;

const SCREEN_SIZE: (u32, u32) = (1280, 720);
const SHADOWMAP_SIZE: i32 = 2048;
const DRAW_FRAMEBUFFER_SIZE: (u32, u32) = (640, 360);
const SIZEOF_FLOAT: usize = std::mem::size_of::<f32>();
const FAR_PLANE: f32 = 1000.0; // TODO: Check if it's OK to use with shadowmaps as well
const NEAR_PLANE: f32 = 0.1;

unsafe fn check_gl_error(tag: &str) {
    let error = gl::GetError();

    if error != 0 {
        println!("[{0}] error: {1}", tag, error);
    }
}
