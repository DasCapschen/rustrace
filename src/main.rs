use sdl2::event::Event;

use sdl2::keyboard::Keycode;

use crate::light::Light;

use crate::material::Material;
use crate::renderer::Renderer;
use crate::sphere::Sphere;
use crate::vec3::Vec3;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureAccess, TextureCreator};
use sdl2::surface::Surface;
use std::time::{SystemTime, UNIX_EPOCH};

mod camera;
mod hittable;
mod light;
mod material;
mod ray;
mod renderer;
mod sphere;
mod vec3;

const WIDTH: u32 = 300;
const HEIGHT: u32 = 200;

fn main() {
    let sdl2_context = sdl2::init().unwrap();
    let video_subsystem = sdl2_context.video().unwrap();

    let window = video_subsystem
        .window("Raytracer", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut renderer = Renderer::new(WIDTH as i32, HEIGHT as i32, 16);

    let ground_sphere_mat = Material::new(Vec3::rgb(20, 225, 50), 0.0, 1.0);
    let diffuse_sphere_mat = Material::new(Vec3::rgb(225, 80, 30), 0.0, 1.0);
    let sphere1_mat = Material::new(Vec3::rgb(255, 255, 255), 1.0, 0.9);
    let sphere2_mat = Material::new(Vec3::rgb(30, 220, 180), 0.5, 0.1);

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

    renderer.add_light(Light::new(
        Vec3::new(1.0, 2.0, -2.0),
        Vec3::new(255.0, 255.0, 255.0),
    ));

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
                _ => {}
            }
        }

        let mut surface = window.surface(&event_pump).unwrap();

        let start_time = SystemTime::now();
        let pixels = renderer.draw_image();
        let end_time = SystemTime::now();

        if let Some(pixel_buffer) = surface.without_lock_mut() {
            pixel_buffer.copy_from_slice(pixels);
        }

        surface.update_window();
        println!("DRAW! ({:?})", end_time.duration_since(start_time).unwrap());
    }
}
