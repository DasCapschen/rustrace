use sdl2::event::Event;

use sdl2::keyboard::Keycode;







use crate::light::Light;
use crate::material::{Lambertian, Metal};

use crate::renderer::Renderer;
use crate::sphere::Sphere;
use crate::vec3::Vec3;

mod camera;
mod hittable;
mod light;
mod material;
mod ray;
mod renderer;
mod sphere;
mod vec3;

fn main() {
    let sdl2_context = sdl2::init().unwrap();
    let video_subsystem = sdl2_context.video().unwrap();

    let window = video_subsystem
        .window("Raytracer", 400, 300)
        .position_centered()
        .build()
        .unwrap();

    let canvas = window.into_canvas().build().unwrap();
    let mut renderer = Renderer::new(canvas); //FIXME: AA does NOT work!

    let ground_sphere_mat = Box::new(Lambertian::new(Vec3::new(20.0, 225.0, 50.0)));

    let diffuse_sphere_mat = Box::new(Lambertian::new(Vec3::new(225.0, 80.0, 30.0)));

    let sphere1_mat = Box::new(Metal::new(Vec3::new(255.0, 255.0, 255.0), 0.8));

    let sphere2_mat = Box::new(Metal::new(Vec3::new(30.0, 220.0, 180.0), 0.3));

    //diffuse sphere
    renderer.add_object(Box::new(Sphere {
        center: Vec3::new(0.0, 0.0, 3.0),
        radius: 1.0,
        material: diffuse_sphere_mat,
    }));

    //2 metal reflector spheres
    renderer.add_object(Box::new(Sphere {
        center: Vec3::new(-2.1, 0.0, 3.5),
        radius: 1.0,
        material: sphere1_mat,
    }));
    renderer.add_object(Box::new(Sphere {
        center: Vec3::new(2.1, 0.0, 2.5),
        radius: 1.0,
        material: sphere2_mat,
    }));

    //"ground"
    renderer.add_object(Box::new(Sphere {
        center: Vec3::new(0.0, -101.0, 1.0),
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

        renderer.draw_image();
    }

    /*
    loop {
        let ev = event_pump.wait_event();
        match ev {
            Event::Quit{..} |
            Event::KeyDown {keycode: Some(Keycode::Escape), ..} => break,
            _ => {}
        }
    }
    */
}
