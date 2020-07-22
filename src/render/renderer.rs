use crate::object::Object;
use crate::render::directional_light::*;
use crate::render::point_light::*;
use crate::render::shader::*;
use crate::render::skybox::Skybox;
use crate::render::*;
use crate::*;
use cgmath::*;

#[allow(dead_code)] // The glContext needs to be kept alive, even though not being read
pub struct Renderer {
    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,
    directional_light: DirectionalLight,
    skybox: Skybox,
    point_lights: Vec<PointLight>,
    point_light_cubemap_handle: TextureHandle,
    point_light_fbo_handle: BufferHandle,
    world_shader: Shader,
    draw_fbo: BufferHandle,
}

impl Renderer {
    pub fn init(sdl_context: &sdl2::Sdl) -> Self {
        let sdl_video = sdl_context.video().unwrap();
        let gl_attr = sdl_video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(4, 1);
        let window = sdl_video
            .window("Progress.", SCREEN_SIZE.0, SCREEN_SIZE.1)
            .opengl()
            .resizable()
            .build()
            .unwrap();
        sdl_context.mouse().set_relative_mouse_mode(true);

        let gl_context = window.gl_create_context().unwrap();
        gl::load_with(|s| sdl_video.gl_get_proc_address(s) as *const std::os::raw::c_void);

        let projection = cgmath::perspective(
            cgmath::Deg(45.0),
            SCREEN_SIZE.0 as f32 / SCREEN_SIZE.1 as f32,
            NEAR_PLANE,
            FAR_PLANE,
        );
        let directional_light = DirectionalLight::new();

        let point_light1 = PointLight::new(Point3::new(24.0, 2.0, -3.0), 2.0, 0.25, 0);
        let point_light2 =
            PointLight::new(Point3::new(-82.10625, 2.5329967, 57.53004), 1.5, 0.05, 1);
        let point_light3 =
            PointLight::new(Point3::new(-35.36219, 1.324518, 73.998116), 1.5, 0.2, 2);

        let point_light4 = PointLight::new(Point3::new(2.986008, 1.8637276, 50.22367), 1.0, 0.2, 3);

        let point_lights = vec![point_light1, point_light2, point_light3, point_light4];

        let point_light_cubemap_handle =
            unsafe { render::point_light::create_cubemap_array(point_lights.len()) };
        let point_light_fbo_handle = unsafe {
            render::point_light::create_point_light_framebuffer(point_light_cubemap_handle)
        };

        let world_shader = Shader::from_file("src/shaders/triangle.glsl", false)
            .expect("\nProblem loading world shader\n");

        let draw_fbo = unsafe { create_draw_backbuffer() };

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::ClearColor(0.1, 0.05, 0.05, 1.0);

            world_shader.set_used();
            world_shader.set_i32("u_texture0", 0);
            world_shader.set_i32("u_shadowmap_directional", 1);
            world_shader.set_i32("u_shadowmaps_point", 2);
            world_shader.set_i32("u_point_light_count", point_lights.len() as i32);

            for (i, point_light) in point_lights.iter().enumerate() {
                world_shader.set_vec3(
                    format!("u_point_lights[{}].position", i).as_str(),
                    point_light.position.x,
                    point_light.position.y,
                    point_light.position.z,
                );
                world_shader.set_f32(
                    format!("u_point_lights[{}].intensity", i).as_str(),
                    point_light.intensity,
                );
                world_shader.set_f32(
                    format!("u_point_lights[{}].attenuation", i).as_str(),
                    point_light.attenuation,
                );
            }

            world_shader.set_f32("u_far_plane", FAR_PLANE);

            world_shader.set_vec3(
                "u_directional_light_dir",
                directional_light.direction.x,
                directional_light.direction.y,
                directional_light.direction.z,
            );
            world_shader.set_mat4(
                "u_directional_light_vp",
                directional_light.projection * directional_light.view,
            );
            world_shader.set_vec4(
                "u_directional_light_color",
                directional_light.color.x,
                directional_light.color.y,
                directional_light.color.z,
                directional_light.color.w,
            );
            world_shader.set_mat4("u_projection", projection);
            check_gl_error("test1");

            // Zero is reserved for the diffuse texture
            // TODO: Changing the order of these two causes creepy-looking blotch stains
            // Look this up. This might bit us in the back
            // Is this the active texture swap in render()?
            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, point_light_cubemap_handle);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, directional_light.depth_texture_handle);
        }

        Self {
            window: window,
            gl_context: gl_context,
            world_shader: world_shader,

            directional_light: directional_light,
            point_lights: point_lights,
            point_light_cubemap_handle: point_light_cubemap_handle,
            point_light_fbo_handle: point_light_fbo_handle,

            skybox: Skybox::new(projection),
            draw_fbo: draw_fbo,
        }
    }

    pub unsafe fn render(&mut self, objects: &Vec<Object>, player_v: Matrix4<f32>) {
        gl::Disable(gl::CULL_FACE);
        self.directional_light.fill_depth_texture(&objects);

        // Render to point-light cubemap array
        gl::Viewport(0, 0, render::SHADOWMAP_SIZE, render::SHADOWMAP_SIZE);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.point_light_fbo_handle);
        gl::Clear(gl::DEPTH_BUFFER_BIT);
        for point_light in &mut self.point_lights {
            point_light.fill_depth_cubemap(&objects);
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // Render world to backbuffer
        gl::Enable(gl::CULL_FACE);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.draw_fbo);
        self.world_shader.set_used();
        self.world_shader.set_mat4("u_view", player_v);
        gl::Viewport(
            0,
            0,
            DRAW_FRAMEBUFFER_SIZE.0 as i32,
            DRAW_FRAMEBUFFER_SIZE.1 as i32,
        );
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        // Setting the pointlight cubemap for rendering the world
        gl::ActiveTexture(gl::TEXTURE1);
        gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, self.point_light_cubemap_handle);

        for obj in objects {
            self.world_shader.set_mat4("u_model", obj.transform);
            obj.material.draw();
        }

        // Fill the depth==1 fragments with sky texture
        self.skybox.draw(player_v);

        // Render from the draw framebuffer to the default framebuffer (the screen)
        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.draw_fbo);
        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
        gl::BlitFramebuffer(
            0,
            0,
            DRAW_FRAMEBUFFER_SIZE.0 as i32,
            DRAW_FRAMEBUFFER_SIZE.1 as i32,
            0,
            0,
            SCREEN_SIZE.0 as i32,
            SCREEN_SIZE.1 as i32,
            gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT,
            gl::NEAREST,
        );
    }

    pub fn finish_render(&mut self) {
        self.window.gl_swap_window();
    }
}

unsafe fn create_draw_backbuffer() -> u32 {
    let mut draw_fbo = 0;
    gl::GenFramebuffers(1, &mut draw_fbo);
    gl::BindFramebuffer(gl::FRAMEBUFFER, draw_fbo);

    let mut draw_texture_handle = 0;
    gl::GenTextures(1, &mut draw_texture_handle);
    gl::BindTexture(gl::TEXTURE_2D, draw_texture_handle);
    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::RGB as i32,
        DRAW_FRAMEBUFFER_SIZE.0 as i32,
        DRAW_FRAMEBUFFER_SIZE.1 as i32,
        0,
        gl::RGB,
        gl::UNSIGNED_BYTE,
        std::ptr::null(),
    );
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

    gl::FramebufferTexture2D(
        gl::FRAMEBUFFER,
        gl::COLOR_ATTACHMENT0,
        gl::TEXTURE_2D,
        draw_texture_handle,
        0,
    );

    let mut draw_rbo = 0;
    gl::GenRenderbuffers(1, &mut draw_rbo);
    gl::BindRenderbuffer(gl::RENDERBUFFER, draw_rbo);
    gl::RenderbufferStorage(
        gl::RENDERBUFFER,
        gl::DEPTH24_STENCIL8,
        DRAW_FRAMEBUFFER_SIZE.0 as i32,
        DRAW_FRAMEBUFFER_SIZE.1 as i32,
    );
    gl::FramebufferRenderbuffer(
        gl::FRAMEBUFFER,
        gl::DEPTH_STENCIL_ATTACHMENT,
        gl::RENDERBUFFER,
        draw_rbo,
    );

    let err = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
    if err != gl::FRAMEBUFFER_COMPLETE {
        println!("Problem while doing framebuffer stuff: {}", err);
    }
    gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

    draw_fbo
}
