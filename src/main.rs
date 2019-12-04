use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::{Window, WindowSurfaceRef};
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::event::Event;
use sdl2::rect::Point;

use crate::sphere::Sphere;
use crate::hittable::Hittable;
use crate::vec3::Vec3;
use crate::ray::Ray;
use crate::camera::Camera;
use crate::renderer::Renderer;
use crate::light::Light;
use std::io;

mod vec3;
mod ray;
mod sphere;
mod hittable;
mod camera;
mod renderer;
mod light;

fn main()
{
    let sdl2_context = sdl2::init().unwrap();
    let video_subsystem = sdl2_context.video().unwrap();

    let window = video_subsystem.window("Raytracer", 400, 300)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut renderer = Renderer::new(canvas); //FIXME: AA does NOT work!

    renderer.add_object(Box::new( Sphere { center: Vec3::new(0.0, 0.0, 3.0), radius: 1.0, color: Vec3::new(255.0, 0.0, 0.0) } ));
    renderer.add_object(Box::new( Sphere { center: Vec3::new(0.0, -101.0, 1.0), radius: 100.0, color: Vec3::new(0.0, 255.0, 0.0) } ));

    renderer.add_light( Light::new(Vec3::new(1.0, 2.0, -2.0), Vec3::new(255.0, 255.0, 255.0)) );

    let mut event_pump = sdl2_context.event_pump().unwrap();
    'running: loop
    {
        //check events (resize, quit, buttons, ...)
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit{..} |
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} => break 'running,
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