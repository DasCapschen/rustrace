use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use crate::camera::Camera;
use crate::gfx::material::{Material, Metallic};
use crate::gfx::texture::{ConstantTexture, ImageTexture};
use crate::hittables::mesh::Mesh;
use crate::hittables::primitives::{Plane, Sphere};
use crate::math::vec3::Vec3;
use crate::renderer::Renderer;

use std::sync::Arc;
use std::time::SystemTime;

use scoped_threadpool::Pool;

mod camera;
mod ray;
mod renderer;

mod gfx {
    pub mod material;
    pub mod texture;
}

mod math {
    pub mod mat3;
    pub mod onb;
    pub mod pdf;
    pub mod vec3;
}

mod hit;
mod hittables {
    pub mod aabb;
    pub mod bvh;
    pub mod mesh;
    pub mod primitives;
}

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const GAMMA: f32 = 1.0 / 2.2;

fn main() {
    //initialise SDL2
    let sdl2_context = sdl2::init().unwrap();
    let video_subsystem = sdl2_context.video().unwrap();

    //create a window
    let window = video_subsystem
        .window("Raytracer", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    //hide cursor, lock mouse to window
    //sdl2_context.mouse().set_relative_mouse_mode(true);

    // https://hdrihaven.com/
    let skybox = Arc::new(ImageTexture::new("res/textures/paul_lobe_haus_4k.hdr"));

    let pos = Vec3::new(-1.0, 0.5, 0.0);
    let target = Vec3::new(0.0, 0.0, 0.0);
    let camera = Camera::new(
        /*pos: */ pos,
        /*dir: */ target - pos,
        /*fov: */ 90.0,
        /*w: */ WIDTH as i32,
        /*h: */ HEIGHT as i32,
        /*focus: */ 1.0,    //if aperture == 0 focus dist is irrelevant
        /*aperture: */ 0.0, //perfect camera => 0 => no DoF ; bigger aperture => stronger DoF
    );

    //create the actual raytracer
    let mut renderer = Renderer::new(WIDTH as i32, HEIGHT as i32, 1, camera, skybox);

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
    let material = Arc::new(Material::new(texture, None, Metallic::NonMetal, None));

    /*renderer.add_object(Arc::new(Sphere {
        center: Vec3::new(0.0, 0.0, 0.0),
        radius: 0.5,
        material: material
    }));*/

    renderer.add_object(Arc::new(Mesh::new("res/models/dragon.obj")));

    //creates bvh and leaves the renderer immutable
    let start = SystemTime::now();

    let mut renderer = renderer.finalise(); //finalise builds bvh and flushes scene!

    let end = SystemTime::now();
    println!("Finalising took {:?}", end.duration_since(start).unwrap());

    let mut pool = Pool::new(8);

    let mut color_buffer = vec![0f32; (WIDTH * HEIGHT * 3) as usize];
    let mut albedo_buffer = vec![0f32; color_buffer.len()];
    let mut normal_buffer = vec![0f32; color_buffer.len()];

    let denoise_device = oidn::Device::new();
    let mut denoise_filter = oidn::filter::RayTracing::new(&denoise_device);
    denoise_filter
        .set_srgb(false)
        .set_img_dims(WIDTH as usize, HEIGHT as usize);

    //main loop
    let mut event_pump = sdl2_context.event_pump().unwrap();
    'running: loop {
        //check events (resize, quit, buttons, ...)
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                /*Event::MouseMotion { xrel, yrel, .. } => {
                    let dx = xrel as f32 / WIDTH as f32;
                    let dy = yrel as f32 / HEIGHT as f32;
                    //this allows rotation in the positive half-space!
                    renderer.camera.direction += Vec3::new(dx, -dy, 0.0);
                }*/
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => renderer.camera.position += 0.1 * renderer.camera.forward(),
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => renderer.camera.position += -0.1 * renderer.camera.forward(),
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => renderer.camera.position += 0.1 * renderer.camera.right(),
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => renderer.camera.position += -0.1 * renderer.camera.right(),
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => renderer.camera.position += 0.1 * renderer.camera.up(),
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => renderer.camera.position += -0.1 * renderer.camera.up(),
                _ => {}
            }
        }

        let subdiv = pool.thread_count() as usize;
        let len = color_buffer.len() / subdiv;

        let render_start_time = SystemTime::now();

        //this is a new thread
        pool.scoped(|s| {
            //here, create references to outside things
            //like a thread setup
            let r = &renderer;
            let mut curr_color_buf = &mut color_buffer[..];
            let mut curr_albedo_buf = &mut albedo_buffer[..];
            let mut curr_normal_buf = &mut normal_buffer[..];
            for i in 0..subdiv {
                let (color_slice, color_buf) = curr_color_buf.split_at_mut(len);
                curr_color_buf = color_buf;
                let (albedo_slice, albedo_buf) = curr_albedo_buf.split_at_mut(len);
                curr_albedo_buf = albedo_buf;
                let (normal_slice, normal_buf) = curr_normal_buf.split_at_mut(len);
                curr_normal_buf = normal_buf;

                s.execute(move || {
                    //this is the actual function of the thread
                    r.draw_image(color_slice, albedo_slice, normal_slice, i * len);
                });
            }
        });

        let render_end_time = SystemTime::now();
        println!(
            "Render took {:?}",
            render_end_time.duration_since(render_start_time).unwrap()
        );

        let denoise_start_time = SystemTime::now();

        //denoise image
        let mut denoise_buffer = vec![0f32; color_buffer.len()];
        match denoise_filter.execute(
            &color_buffer[..],
            Some(&albedo_buffer[..]),
            Some(&normal_buffer[..]),
            &mut denoise_buffer[..],
        ) {
            Ok(_) => {}
            Err(err) => {
                panic!("{:?}", err);
            }
        }

        let denoise_end_time = SystemTime::now();
        println!(
            "Denoising took {:?}",
            denoise_end_time.duration_since(denoise_start_time).unwrap()
        );

        let convert_start_time = SystemTime::now();
        //RGB => BGRA
        let bgra_buffer: Vec<u8> = denoise_buffer
            .chunks(3)
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
            .collect();

        let convert_end_time = SystemTime::now();
        println!(
            "Post processing took {:?}",
            convert_end_time.duration_since(convert_start_time).unwrap()
        );

        //write pixels
        let mut surface = window.surface(&event_pump).unwrap();
        if let Some(pixel_buffer) = surface.without_lock_mut() {
            pixel_buffer.copy_from_slice(&bgra_buffer[..]);
        }

        println!(
            "Total draw time was: {:?}",
            convert_end_time.duration_since(render_start_time).unwrap()
        );
        println!("=========================");

        //"swap" images
        surface.update_window().expect("failed to update windows!");
    }
}
