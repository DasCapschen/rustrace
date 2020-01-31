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
    Renderer::new(800, 600).build_scene().run();
}