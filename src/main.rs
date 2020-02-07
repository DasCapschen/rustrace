use crate::renderer::Renderer;

mod camera;
mod pathtracer;
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
    pub mod transform;
    pub mod vec3;
    pub mod quat;
}

mod hit;
mod hittables {
    pub mod aabb;
    pub mod bvh;
    pub mod mesh;
    pub mod primitives;
    pub mod volume;
}

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const SAMPLES: u32 = 48;

fn main() {
    Renderer::new(WIDTH, HEIGHT, SAMPLES, true).build_scene().run();
}
