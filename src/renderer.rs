use crate::camera::Camera;
use crate::gfx::material::{Material, Metallic};
use crate::gfx::texture::{ConstantTexture, ImageTexture};
use crate::hittables::mesh::Mesh;
use crate::math::vec3::Vec3;
use crate::pathtracer::PathTracer;
use scoped_threadpool::Pool;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::Window;
use sdl2::{EventPump, Sdl};
use std::sync::Arc;
use std::time::Instant;

enum DisplayMode {
    Denoised,
    Color,
    Albedo,
    Normal,
    Depth,
}

const GAMMA: f32 = 1.0 / 2.2;

pub struct Renderer {
    width: u32,
    height: u32,

    context: Sdl,
    window: Window,

    path_tracer: PathTracer,
    display_mode: DisplayMode,
    running: bool,

    color_buffer: Vec<f32>,
    albedo_buffer: Vec<f32>,
    normal_buffer: Vec<f32>,
    depth_buffer: Vec<f32>,
}

impl Renderer {
    pub fn new(width: u32, height: u32, samples: u32) -> Self {
        //initialise SDL2
        let context = sdl2::init().unwrap();
        let video_subsystem = context.video().unwrap();

        //hide cursor, lock mouse to window
        //sdl2_context.mouse().set_relative_mouse_mode(true);

        //create a window
        let window = video_subsystem
            .window("Raytracer", width, height)
            .position_centered()
            .build()
            .unwrap();

        //setup the camera here
        let pos = Vec3::new(-1.0, 0.5, 0.0);
        let target = Vec3::new(0.0, 0.0, 0.0);
        let camera = Camera::new(
            /*pos: */ pos,
            /*dir: */ target - pos,
            /*fov: */ 90.0,
            /*w: */ width,
            /*h: */ height,
            /*focus: */ 1.0, //if aperture == 0 focus dist is irrelevant
            /*aperture: */
            0.0, //perfect camera => 0 => no DoF ; bigger aperture => stronger DoF
        );

        // https://hdrihaven.com/
        let skybox = Arc::new(ImageTexture::new("res/textures/paul_lobe_haus_4k.hdr"));

        //create the renderer
        let path_tracer = PathTracer::new(width, height, samples, camera, skybox);

        let buffer_size = (width * height * 3) as usize;
        Renderer {
            width,
            height,
            context,
            window,
            path_tracer,
            display_mode: DisplayMode::Denoised,
            running: false,
            color_buffer: vec![0f32; buffer_size],
            albedo_buffer: vec![0f32; buffer_size],
            normal_buffer: vec![0f32; buffer_size],
            depth_buffer: vec![0f32; buffer_size],
        }
    }

    /// check events (resize, quit, buttons, ...)
    fn handle_sdl_events(&mut self, event_pump: &mut EventPump) {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => self.running = false,
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => self.path_tracer.camera.position += 0.1 * self.path_tracer.camera.forward(),
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => self.path_tracer.camera.position += -0.1 * self.path_tracer.camera.forward(),
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => self.path_tracer.camera.position += 0.1 * self.path_tracer.camera.right(),
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => self.path_tracer.camera.position += -0.1 * self.path_tracer.camera.right(),
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => self.path_tracer.camera.position += 0.1 * self.path_tracer.camera.up(),
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => self.path_tracer.camera.position += -0.1 * self.path_tracer.camera.up(),
                Event::KeyDown {
                    keycode: Some(Keycode::F1),
                    ..
                } => self.display_mode = DisplayMode::Denoised,
                Event::KeyDown {
                    keycode: Some(Keycode::F2),
                    ..
                } => self.display_mode = DisplayMode::Color,
                Event::KeyDown {
                    keycode: Some(Keycode::F3),
                    ..
                } => self.display_mode = DisplayMode::Albedo,
                Event::KeyDown {
                    keycode: Some(Keycode::F4),
                    ..
                } => self.display_mode = DisplayMode::Normal,
                Event::KeyDown {
                    keycode: Some(Keycode::F5),
                    ..
                } => self.display_mode = DisplayMode::Depth,
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => self.path_tracer.debug_index = Some(0),
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => self.path_tracer.debug_index = None,
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    let bvh = self.path_tracer.bvh.as_ref().unwrap();
                    let idx = self.path_tracer.debug_index.unwrap();
                    self.path_tracer.debug_index = Some(bvh.get_left_node_index(idx));
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    let bvh = self.path_tracer.bvh.as_ref().unwrap();
                    let idx = self.path_tracer.debug_index.unwrap();
                    self.path_tracer.debug_index = Some(bvh.get_right_node_index(idx));
                }
                _ => {}
            }
        }
    }

    /// creates the scene that will be rendered
    pub fn build_scene(mut self) -> Self {
        //create a 10x10x10 cube of spheres with colorful colors
        /*
        for x in -10..10i8 {
            for y in -10..10i8 {
                for z in -10..10i8 {
                    let r = (x as f32 * (220.0 / 10.0) + 10.0) as u8;
                    let g = (y as f32 * (220.0 / 10.0) + 10.0) as u8;
                    let b = (z as f32 * (220.0 / 10.0) + 10.0) as u8;

                    let color = Arc::new(ConstantTexture::new(Vec3::rgb(r, g, b)));
                    let metallic = Metallic::NonMetal;
                    let refraction = None;

                    renderer.add_object(Arc::new(Sphere {
                        center: 1.5 * Vec3::new(x as f32, y as f32, z as f32),
                        radius: 0.5,
                        material: Arc::new(Material::new(color, None, metallic, refraction)),
                    }));
                }
            }
        }
        */

        /*
        let checker_dark = Arc::new(ConstantTexture::new(Vec3::new(0.33, 0.33, 0.33)));
        let checker_bright = Arc::new(ConstantTexture::new(Vec3::new(1.0, 1.0, 1.0)));
        let checkered_texture = Arc::new(CheckeredTexture::new(checker_dark, checker_bright));

        let ground_mat = Material::new(checkered_texture, None, Metallic::NonMetal, None);

        renderer.add_object(Arc::new(Plane {
            center: Vec3::new(0.0, 0.0, 0.0),
            span_a: Vec3::new(100.0, 0.0, 0.0),
            span_b: Vec3::new(0.0, 0.0, 100.0),
            material: Arc::new(ground_mat),
        }));
        */

        //let texture = Arc::new(ImageTexture::new("res/textures/globe.jpg"));
        //let normal = Arc::new(ImageTexture::new("res/textures/globeNormal.jpg"));
        /*let metal_params = MetalParameters {
            metallic: Arc::new(ConstantTexture::new(Vec3::rgb(255,255,255))),
            roughness: Arc::new(ConstantTexture::new(Vec3::rgb(10,10,10))),
        };*/

        let texture = Arc::new(ConstantTexture::new(Vec3::new(1.0, 1.0, 1.0)));
        let _material = Arc::new(Material::new(texture, None, Metallic::NonMetal, None));

        /*renderer.add_object(Arc::new(Sphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: 0.5,
            material: material
        }));*/

        self.path_tracer
            .add_object(Arc::new(Mesh::new("res/models/dragon_tiny.obj")));

        // DO NOT CHANGE STUFF AFTER THIS COMMENT

        //creates bvh and leaves the scene immutable (ownership moved to bvh)
        let finalise_time = Instant::now();
        self.path_tracer = self.path_tracer.finalise();
        println!("Finalising took {:?}", finalise_time.elapsed());
        self
    }

    /// does gamma correction and converts f32-RGB to u8-BGRA
    fn post_process(raw: &[f32]) -> Vec<u8> {
        //RGB => BGRA
        raw.chunks(3)
            .map(|chunk| {
                vec![
                    //RGB -> BGR, and gamma correct
                    (chunk[2].powf(GAMMA) * 255.0) as u8,
                    (chunk[1].powf(GAMMA) * 255.0) as u8,
                    (chunk[0].powf(GAMMA) * 255.0) as u8,
                    //add alpha channel
                    0u8,
                ]
            })
            .flatten()
            .collect()
    }

    /// MAIN LOOP
    pub fn run(&mut self) {
        self.running = true;

        //we can't really save any denoiser stuff (really annoying...)
        //create the denoiser stuff
        let denoise_device = oidn::Device::new();
        let mut denoise_filter = oidn::filter::RayTracing::new(&denoise_device);
        denoise_filter
            .set_srgb(false)
            .set_img_dims(self.width as usize, self.height as usize);

        //create thread pool (cannot save this either, annoying...)
        let mut thread_pool = Pool::new(8);

        let mut event_pump = self.context.event_pump().unwrap();
        while self.running {
            self.handle_sdl_events(&mut event_pump);

            let subdiv = thread_pool.thread_count() as usize;
            let len = self.color_buffer.len() / subdiv;

            let render_time = Instant::now();

            //this is a new thread
            thread_pool.scoped(|s| {
                //here, create references to outside things, like a thread setup
                let r = &self.path_tracer;

                //Mutable Slices of our vectors do NOT split the vector itself below!
                let mut curr_color_buf = &mut self.color_buffer[..];
                let mut curr_albedo_buf = &mut self.albedo_buffer[..];
                let mut curr_normal_buf = &mut self.normal_buffer[..];
                let mut curr_depth_buf = &mut self.depth_buffer[..];

                for i in 0..subdiv {
                    //split the buffers into parts for each thread!
                    let (color_slice, color_buf) = curr_color_buf.split_at_mut(len);
                    curr_color_buf = color_buf;
                    let (albedo_slice, albedo_buf) = curr_albedo_buf.split_at_mut(len);
                    curr_albedo_buf = albedo_buf;
                    let (normal_slice, normal_buf) = curr_normal_buf.split_at_mut(len);
                    curr_normal_buf = normal_buf;
                    let (depth_slice, depth_buf) = curr_depth_buf.split_at_mut(len);
                    curr_depth_buf = depth_buf;

                    //this is the actual function of the thread
                    s.execute(move || {
                        r.draw_image(
                            color_slice,
                            albedo_slice,
                            normal_slice,
                            depth_slice,
                            i * len,
                        );
                    });
                }
            });
            println!("Render took {:?}", render_time.elapsed());

            //denoise image
            let denoise_time = Instant::now();
            let mut denoise_buffer = vec![0f32; self.color_buffer.len()];
            denoise_filter
                .execute(
                    &self.color_buffer[..],
                    Some(&self.albedo_buffer[..]),
                    Some(&self.normal_buffer[..]),
                    &mut denoise_buffer[..],
                )
                .expect("failed to denoise image");
            println!("Denoising took {:?}", denoise_time.elapsed());

            let pp_buffer = match &self.display_mode {
                DisplayMode::Denoised => &denoise_buffer,
                DisplayMode::Color => &self.color_buffer,
                DisplayMode::Albedo => &self.albedo_buffer,
                DisplayMode::Normal => &self.normal_buffer,
                DisplayMode::Depth => &self.depth_buffer,
            };

            let convert_time = Instant::now();
            let display_buffer = Self::post_process(pp_buffer);
            println!("Post processing took {:?}", convert_time.elapsed());

            //write pixels
            let mut surface = self.window.surface(&event_pump).unwrap();
            if let Some(pixel_buffer) = surface.without_lock_mut() {
                pixel_buffer.copy_from_slice(&display_buffer[..]);
            }

            println!("Total draw time was: {:?}", render_time.elapsed());
            println!("=========================");

            //"swap" images
            surface.update_window().expect("failed to update windows!");
        }
    }
}
