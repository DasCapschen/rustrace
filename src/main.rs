use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use crate::camera::Camera;
use crate::gfx::material::{Material, Metallic};
use crate::gfx::texture::{ConstantTexture, ImageTexture};
use crate::hittables::mesh::Mesh;
use crate::hittables::primitives::{Plane, Sphere};
use crate::math::vec3::Vec3;
use crate::pathtracer::PathTracer;

use std::sync::Arc;
use std::time::{SystemTime, Instant};

use scoped_threadpool::Pool;
use sdl2::EventPump;
use sdl2::video::Window;
use crate::renderer::Renderer;

mod camera;
mod ray;
mod pathtracer;
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

fn main() {
    let mut renderer = Renderer::new(800, 600);
    let mut renderer = renderer.build_scene();
    renderer.run();
}

