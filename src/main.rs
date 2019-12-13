use sdl2::event::Event;

use sdl2::keyboard::Keycode;

use crate::material::Material;
use crate::renderer::Renderer;
use crate::sphere::Sphere;
use crate::vec3::Vec3;

use std::time::SystemTime;

mod camera;
mod hittable;
mod material;
mod ray;
mod renderer;
mod sphere;
mod vec3;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

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

    //create the actual raytracer
    let mut renderer = Renderer::new((WIDTH / 2) as i32, (HEIGHT / 2) as i32, 64);

    //create some materials
    let ground_sphere_mat = Material::new(Vec3::rgb(100, 200, 30), 0.0, 1.0, 0.0);
    let diffuse_sphere_mat = Material::new(Vec3::rgb(200, 75, 75), 0.0, 1.0, 0.0);
    let sphere1_mat = Material::new(Vec3::rgb(200, 150, 50), 1.0, 1.0, 0.0);
    let sphere2_mat = Material::new(Vec3::rgb(200, 200, 200), 1.0, 0.0, 1.5);

    //diffuse sphere
    renderer.add_object(Box::new(Sphere {
        center: Vec3::new(0.0, 0.0, 3.0),
        radius: 1.0,
        material: diffuse_sphere_mat,
    }));

    //2 metal reflector spheres
    renderer.add_object(Box::new(Sphere {
        center: Vec3::new(-2.0, 0.0, 3.0),
        radius: 1.0,
        material: sphere1_mat,
    }));
    renderer.add_object(Box::new(Sphere {
        center: Vec3::new(2.0, 0.0, 3.0),
        radius: 1.0,
        material: sphere2_mat,
    }));

    //"ground"
    renderer.add_object(Box::new(Sphere {
        center: Vec3::new(0.0, -100.9, 1.0),
        radius: 100.0,
        material: ground_sphere_mat,
    }));

    //albedo > 1 => emits light ;
    let light_material = Material::new(Vec3::new(1000.0, 1000.0, 1000.0), 0.0, 0.0, 0.0);

    //create a light
    renderer.add_object(Box::new(Sphere {
        center: Vec3::new(2.0, 1.5, 0.0),
        radius: 0.5,
        material: light_material,
    }));

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
                Event::MouseMotion { xrel, yrel, .. } => {
                    let dx = xrel as f64 / WIDTH as f64;
                    let dy = yrel as f64 / HEIGHT as f64;
                    renderer.camera.direction += Vec3::new(dx, -dy, 0.0);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => renderer.camera.position += Vec3::new(0.0, 0.0, 0.1),
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => renderer.camera.position += Vec3::new(0.0, 0.0, -0.1),
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => renderer.camera.position += Vec3::new(0.1, 0.0, 0.0),
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => renderer.camera.position += Vec3::new(-0.1, 0.0, 0.0),
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => renderer.camera.position += Vec3::new(0.0, 0.1, 0.0),
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => renderer.camera.position += Vec3::new(0.0, -0.1, 0.0),
                _ => {}
            }
        }

        //render the image
        let start_time = SystemTime::now();
        let pixels = renderer.draw_image();
        let end_time = SystemTime::now();
        //println!("DRAW! ({:?})", end_time.duration_since(start_time).unwrap());

        let mut surface = window.surface(&event_pump).unwrap();

        //write pixels
        if let Some(pixel_buffer) = surface.without_lock_mut() {
            pixel_buffer.copy_from_slice(pixels);
        }

        //"swap" images
        surface.update_window().expect("failed to update windows!");
    }
}
