use crate::camera::{Camera, CropFactor, Focus};
use crate::gfx::material::*;
use crate::gfx::texture::{ConstantTexture, ImageTexture};

use crate::hittables::primitives::*;

use crate::math::vec3::Vec3;
use crate::pathtracer::PathTracer;
use rayon::prelude::*;
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

    frame: u32,
}

impl Renderer {
    pub fn new(width: u32, height: u32, samples: u32, incremental: bool) -> Self {
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
        let pos = Vec3::new(-7.0, 12.0, -7.0);
        let target = Vec3::new(0.0, 5.0, 0.0);

        let f = (Vec3::new(0.707, 9.0, 0.707) - pos).len();
        let fstop = 8;
        let n = 2.0_f32.sqrt().powi(fstop);
        println!("aperture = f/{}", n);

        let camera = Camera::new_physical(
            /*pos: */ pos,
            /*dir: */ target - pos,
            /*w: */ width,
            /*h: */ height,
            /*focus: */ Focus::Distance(f), //if aperture == 0 focus dist is irrelevant
            35.0,
            fstop,
            CropFactor::FULL_FORMAT, //perfect camera => 0 => no DoF ; bigger aperture => stronger DoF
        );

        // https://hdrihaven.com/
        let skybox = Arc::new(ImageTexture::new("res/textures/paul_lobe_haus_4k.hdr"));

        //create the renderer
        let path_tracer = PathTracer::new(width, height, samples, incremental, camera, skybox);

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
            frame: 1,
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
                } => {
                    self.path_tracer.camera.position += 0.1 * self.path_tracer.camera.direction;
                    self.frame = 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    self.path_tracer.camera.position += -0.1 * self.path_tracer.camera.direction;
                    self.frame = 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    self.path_tracer.camera.position += 0.1 * self.path_tracer.camera.right;
                    self.frame = 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    self.path_tracer.camera.position += -0.1 * self.path_tracer.camera.right;
                    self.frame = 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    self.path_tracer.camera.position += 0.1 * self.path_tracer.camera.up;
                    self.frame = 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => {
                    self.path_tracer.camera.position += -0.1 * self.path_tracer.camera.up;
                    self.frame = 1;
                }
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
                    keycode: Some(Keycode::KpPlus),
                    ..
                } => {
                    self.path_tracer
                        .camera
                        .set_focal_length(self.path_tracer.camera.focal_length + 1.0);
                    self.frame = 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::KpMinus),
                    ..
                } => {
                    self.path_tracer
                        .camera
                        .set_focal_length(self.path_tracer.camera.focal_length - 1.0);
                    self.frame = 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::KpMultiply),
                    ..
                } => {
                    self.path_tracer
                        .camera
                        .set_fstop(self.path_tracer.camera.fstop + 1);
                    self.frame = 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::KpDivide),
                    ..
                } => {
                    self.path_tracer
                        .camera
                        .set_fstop(self.path_tracer.camera.fstop - 1);
                    self.frame = 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::KpEnter),
                    ..
                } => {
                    if self.path_tracer.camera.crop_factor == CropFactor::FULL_FORMAT {
                        self.path_tracer.camera.set_crop_factor(CropFactor::APSC);
                    } else if self.path_tracer.camera.crop_factor == CropFactor::APSC {
                        self.path_tracer
                            .camera
                            .set_crop_factor(CropFactor::APSC_CANON);
                    } else if self.path_tracer.camera.crop_factor == CropFactor::APSC_CANON {
                        self.path_tracer
                            .camera
                            .set_crop_factor(CropFactor::FULL_FORMAT);
                    }
                    self.frame = 1;
                }
                _ => {}
            }
        }
    }

    /// creates the scene that will be rendered
    pub fn build_scene(mut self) -> Self {
        //create a 10x10x10 cube of spheres with colorful colors

        self.path_tracer.add_object(Arc::new(Sphere {
            center: Vec3::new(0.0, -1000.0, 0.0),
            radius: 1000.0,
            material: Arc::new(Lambertian::new(
                Arc::new(ConstantTexture::new(Vec3::rgb(5, 50, 10))),
                None,
            )),
        }));

        for x in 0..3i8 {
            for y in 0..3i8 {
                for z in 0..3i8 {
                    let r = (x as f32 * (220.0 / 10.0) + 10.0) as u8;
                    let g = (y as f32 * (220.0 / 10.0) + 10.0) as u8;
                    let b = (z as f32 * (220.0 / 10.0) + 10.0) as u8;

                    let color = Arc::new(ConstantTexture::new(Vec3::rgb(r, g, b)));
                    //let metallic = Metallic::NonMetal;
                    //let refraction = None;

                    self.path_tracer.add_object(Arc::new(Sphere {
                        center: 3.0 * Vec3::new(x as f32, y as f32, z as f32),
                        radius: 1.0,
                        material: Arc::new(Lambertian::new(color, None)),
                    }));
                }
            }
        }

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

        /*
        let texture = Arc::new(ConstantTexture::new(Vec3::new(1.0, 1.0, 1.0)));
        let material = Arc::new(Lambertian::new(texture, None));

        let dragon = Arc::new(Mesh::new("res/models/dragon_tiny.obj"));
        let position = Vec3::new(0.0, 0.0, 0.5);
        let rotation = Quaternion::from_euler(-90.0, -0.0, 45.0);
        let scale = 1.0;

        let transformed_dragon = Arc::new(Transform::new(dragon, position, rotation, scale));

        self.path_tracer.add_object(transformed_dragon);
        */

        //let dragon = Arc::new(Mesh::new("res/models/dragon_tiny.obj"));
        /*let boundary = Arc::new(Sphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: 0.5,
            material: material
        });

        let volume_albedo = Arc::new(ConstantTexture::new(Vec3::rgb(255, 50, 50)));
        let volume_material = Arc::new(Isotropic::new(volume_albedo));
        let volume = ConstantVolume::new(boundary, 2.0, volume_material);

        self.path_tracer.add_object(Arc::new(volume));*/

        // DO NOT CHANGE STUFF AFTER THIS COMMENT

        //creates bvh and leaves the scene immutable (ownership moved to bvh)
        #[cfg(measure_perf)]
        let finalise_time = Instant::now();

        self.path_tracer = self.path_tracer.finalise();

        #[cfg(measure_perf)]
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

        let mut event_pump = self.context.event_pump().unwrap();
        while self.running {
            self.handle_sdl_events(&mut event_pump);

            #[cfg(measure_perf)]
            let render_time = Instant::now();

            let _len = self.color_buffer.len();
            let cb = &mut self.color_buffer;
            let ab = &mut self.albedo_buffer;
            let nb = &mut self.normal_buffer;
            let db = &mut self.depth_buffer;
            let tracer = &self.path_tracer;

            let frame = self.frame;

            cb.par_chunks_mut(3)
                .enumerate()
                .zip(ab.par_chunks_mut(3))
                .zip(nb.par_chunks_mut(3))
                .zip(db.par_chunks_mut(3))
                .for_each_init(
                    || rand::thread_rng(),
                    |rng, ((((index, c), a), n), d)| {
                        tracer.render_pixel(rng, index, frame, c, a, n, d)
                    },
                );

            #[cfg(measure_perf)]
            println!("Render took {:?}", render_time.elapsed());

            #[cfg(measure_perf)]
            let denoise_time = Instant::now();

            //denoise image
            let mut denoise_buffer = vec![0f32; self.color_buffer.len()];
            denoise_filter
                .execute_with_albedo_normal(
                    &self.color_buffer[..],
                    &self.albedo_buffer[..],
                    &self.normal_buffer[..],
                    &mut denoise_buffer[..],
                )
                .expect("failed to denoise image");

            #[cfg(measure_perf)]
            println!("Denoising took {:?}", denoise_time.elapsed());

            let pp_buffer = match &self.display_mode {
                DisplayMode::Denoised => &denoise_buffer,
                DisplayMode::Color => &self.color_buffer,
                DisplayMode::Albedo => &self.albedo_buffer,
                DisplayMode::Normal => &self.normal_buffer,
                DisplayMode::Depth => &self.depth_buffer,
            };

            #[cfg(measure_perf)]
            let convert_time = Instant::now();

            let display_buffer = Self::post_process(pp_buffer);

            #[cfg(measure_perf)]
            println!("Post processing took {:?}", convert_time.elapsed());

            //write pixels
            let mut surface = self.window.surface(&event_pump).unwrap();
            if let Some(pixel_buffer) = surface.without_lock_mut() {
                pixel_buffer.copy_from_slice(&display_buffer[..]);
            }

            #[cfg(measure_perf)]
            {
                println!("Total draw time was: {:?}", render_time.elapsed());
                println!("=========================");
            }

            //"swap" images
            surface.update_window().expect("failed to update windows!");

            //update frame value
            self.frame += 1;
        }
    }
}
