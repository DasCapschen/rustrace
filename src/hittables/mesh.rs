use crate::gfx::material::Material;
use crate::gfx::material::Metallic;
use crate::gfx::texture::ConstantTexture;
use std::path::Path;
use std::sync::Arc;

use crate::hit::{Hit, HitResult};
use crate::hittables::aabb::AABB;
use crate::hittables::bvh::BvhTree;
use crate::hittables::primitives::Triangle;
use crate::math::vec3::Vec3;
use crate::ray::Ray;

#[derive(Clone)]
pub struct Mesh {
    position: Vec3,
    faces: BvhTree<Triangle>,
}

impl Mesh {
    pub fn new<P: AsRef<Path>>(file: P) -> Self {
        let (models, _mats) = tobj::load_obj(file.as_ref()).expect("couldn't load file");

        //load material
        let material = Arc::new(Material::new(
            Arc::new(ConstantTexture::new(Vec3::new(0.9, 0.9, 0.9))),
            None,
            Metallic::NonMetal,
            None,
        ));

        // just assume there is only 1 model in the obj!
        let mesh: Vec<Triangle> = models[0]
            .mesh
            .indices
            .chunks(3)
            .map(|chunk| {
                let v1 = Vec3 {
                    x: models[0].mesh.positions[3 * chunk[0] as usize + 0],
                    y: models[0].mesh.positions[3 * chunk[0] as usize + 1],
                    z: models[0].mesh.positions[3 * chunk[0] as usize + 2],
                };

                let v2 = Vec3 {
                    x: models[0].mesh.positions[3 * chunk[1] as usize + 0],
                    y: models[0].mesh.positions[3 * chunk[1] as usize + 1],
                    z: models[0].mesh.positions[3 * chunk[1] as usize + 2],
                };

                let v3 = Vec3 {
                    x: models[0].mesh.positions[3 * chunk[2] as usize + 0],
                    y: models[0].mesh.positions[3 * chunk[2] as usize + 1],
                    z: models[0].mesh.positions[3 * chunk[2] as usize + 2],
                };

                Triangle {
                    llc: v1,
                    span_a: v2 - v1,
                    span_b: v3 - v1,
                    material: material.clone(),
                }
            })
            .collect();

        let bvh = BvhTree::from_hittables(mesh);

        Mesh {
            position: Vec3::new(0.0, 0.0, 0.0),
            faces: bvh,
        }
    }
}

//Vec<Hit> implements hittable!
impl Hit for Mesh {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        //instead of offsetting every face by mesh's position
        //we offset the ray in the opposite direction
        let modified_ray = Ray {
            origin: ray.origin - self.position,
            direction: ray.direction,
        };

        //if we hit, undo the offsetting of the ray and correct the hit position
        if let Some(mut hit) = self.faces.hit(&modified_ray, t_min, t_max) {
            hit.hit_position += self.position;
            return Some(hit);
        }

        None
    }

    fn bounding_box(&self) -> Option<AABB> {
        self.faces.bounding_box()
    }

    fn center(&self) -> Vec3 {
        self.position
    }
}
