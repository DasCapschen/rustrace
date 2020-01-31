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
const SAMPLES: u32 = 16;

fn main() {
    Renderer::new(WIDTH, HEIGHT, SAMPLES).build_scene().run();
}
