use std::path::Path;

use crate::hit::{Hit, HitResult};
use crate::hittables::aabb::AABB;
use crate::hittables::primitives::Triangle;
use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Clone)]
pub struct Mesh {
    position: Vec3,
    faces: Vec<Triangle>,
    bounding_box: Option<AABB>,
}

impl Mesh {
    pub fn new(_file: &Path) -> Self {
        //TODO: load file
        let faces = Vec::new();
        let bounding_box = faces.bounding_box();

        Mesh {
            position: Vec3::new(0.0, 0.0, 0.0),
            faces,
            bounding_box,
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
        self.bounding_box
    }

    fn center(&self) -> Vec3 {
        self.position
    }
}
