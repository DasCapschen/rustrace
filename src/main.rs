use sdl2::event::Event;

use sdl2::keyboard::Keycode;

use crate::hittables::primitives::{Sphere};
use crate::material::Material;
use crate::renderer::Renderer;
use crate::vec3::Vec3;

use std::sync::Arc;
use std::time::SystemTime;

use scoped_threadpool::Pool;

mod camera;
mod material;
mod ray;
mod renderer;
mod vec3;

mod hittable;
mod hittables {
    pub mod aabb;
    pub mod bvh;
    pub mod mesh;
    pub mod primitives;
}

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

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

    //create the actual raytracer
    let mut renderer = Renderer::new(WIDTH as i32, HEIGHT as i32, 4);

    //create a 10x10x10 cube of spheres with colorful colors
    for x in 0..10u8 {
        for y in 0..10u8 {
            for z in 0..10u8 {
                let r = (x as f64 * (220.0 / 10.0) + 10.0) as u8;
                let g = (y as f64 * (220.0 / 10.0) + 10.0) as u8;
                let b = (z as f64 * (220.0 / 10.0) + 10.0) as u8;

                renderer.add_object(Arc::new(Sphere {
                    center: 1.5 * Vec3::new(x as f64, y as f64, z as f64),
                    radius: 0.5,
                    material: Material::new(Vec3::rgb(r, g, b), 0.0, 0.0, 0.0),
                }));
            }
        }
    }

    //create some materials
    /*let ground_mat = Material::new(Vec3::rgb(100, 200, 30), 0.0, 1.0, 0.0);
    let diffuse_sphere_mat = Material::new(Vec3::rgb(200, 75, 75), 0.0, 1.0, 0.0);
    let sphere1_mat = Material::new(Vec3::rgb(200, 150, 50), 1.0, 0.75, 0.0);
    let sphere2_mat = Material::new(Vec3::rgb(200, 200, 200), 1.0, 0.0, 1.5);

    //diffuse sphere
    renderer.add_object(Arc::new(Sphere {
        center: Vec3::new(0.0, 0.0, 3.0),
        radius: 1.0,
        material: diffuse_sphere_mat,
    }));

    //2 metal reflector spheres
    renderer.add_object(Arc::new(Sphere {
        center: Vec3::new(-2.0, 0.0, 3.0),
        radius: 1.0,
        material: sphere1_mat,
    }));
    renderer.add_object(Arc::new(Sphere {
        center: Vec3::new(2.0, 0.0, 3.0),
        radius: 1.0,
        material: sphere2_mat,
    }));

    //"ground"
    //BVH current does not support infinite planes!

    renderer.add_object(Arc::new(Plane {
        center: Vec3::new(0.0, -3.0, 0.0),
        span_a: Vec3::new(0.0, 0.0, 50.0), //swap span_a and span_b to flip normal
        span_b: Vec3::new(50.0, 0.0, 0.0),
        infinite: false,
        material: ground_mat,
    }));

    /*
    renderer.add_object(Arc::new(Sphere {
        center: Vec3::new(0.0, -103.0, 0.0),
        radius: 100.0,
        material: ground_mat,
    }));
    */

    //albedo > 1 => emits light ;
    //let light_material = Material::new(Vec3::new(5.0, 5.0, 5.0), 0.0, 0.0, 0.0);

    //create a light
    /*renderer.add_object(Arc::new(Sphere {
        center: Vec3::new(200.0, 500.0, -1000.0),
        radius: 750.0,
        material: light_material,
    }));*/
    */

    let mut pool = Pool::new(40);

    let mut denoise_device = oidn::Device::new();
    let mut denoise_filter = oidn::filter::RayTracing::new(&mut denoise_device);
    denoise_filter
        .set_srgb(true)
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
                    let dx = xrel as f64 / WIDTH as f64;
                    let dy = yrel as f64 / HEIGHT as f64;
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

        //render the image
        let start_time = SystemTime::now();

        //w*h, RGB
        let mut render_buffer = vec![0f32; (WIDTH * HEIGHT * 3) as usize];
        let subdiv = pool.thread_count() as usize;
        let len = render_buffer.len() / subdiv;

        //this is a new thread
        pool.scoped(|s| {
            //here, create references to outside things
            //like a thread setup
            let r = &renderer;
            let mut curr_buf = &mut render_buffer[..];
            for i in 0..subdiv {
                let (slice, buf) = curr_buf.split_at_mut(len);
                curr_buf = buf;

                s.execute(move || {
                    //this is the actual function of the thread
                    r.draw_image(slice, i * len);
                });
            }
        });

        //denoise image
        let mut denoise_buffer = vec![0f32; render_buffer.len()];
        denoise_filter.execute(&render_buffer[..], &mut denoise_buffer[..]);

        //RGB => BGRA
        let bgra_buffer: Vec<Vec<u8>> = denoise_buffer
            .chunks(3)
            .map(|chunk| {
                vec![
                    (chunk[2] * 255.0) as u8,
                    (chunk[1] * 255.0) as u8,
                    (chunk[0] * 255.0) as u8,
                    0u8,
                ]
            })
            .collect();
        let flat_buffer: Vec<u8> = bgra_buffer.into_iter().flatten().collect();

        //write pixels
        let mut surface = window.surface(&event_pump).unwrap();
        if let Some(pixel_buffer) = surface.without_lock_mut() {
            pixel_buffer.copy_from_slice(&flat_buffer[..]);
        }

        /*
        surface.save_bmp("/home/captncaps/denoised.bmp");

        //RGB => BGRA, every pixel doubled
        let mut bgra_buffer: Vec<Vec<u8>> = render_buffer.chunks(3).map(|chunk|
            vec![(chunk[2]* 255.0) as u8, (chunk[1]* 255.0) as u8, (chunk[0]* 255.0) as u8, 0u8]
        ).collect();

        //flatten
        let flat_buffer: Vec<u8> = bgra_buffer.into_iter().flatten().collect();

        //write pixels
        let mut surface = window.surface(&event_pump).unwrap();
        if let Some(pixel_buffer) = surface.without_lock_mut() {
            pixel_buffer.copy_from_slice(&flat_buffer[..]);
        }

        surface.save_bmp("/home/captncaps/noisy.bmp");
        */

        let end_time = SystemTime::now();
        println!("DRAW! ({:?})", end_time.duration_since(start_time).unwrap());

        //"swap" images
        surface.update_window().expect("failed to update windows!");
    }
}
