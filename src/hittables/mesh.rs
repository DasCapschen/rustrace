use std::path::Path;

use crate::hittables::primitives::Triangle;
use crate::hittable::{Hittable, HitResult};
use crate::ray::Ray;
use crate::vec3::Vec3;

pub struct Mesh {
    position: Vec3,
    faces: Vec<Triangle>,
}

impl Mesh {
    pub fn new(file: &Path) -> Self {
        //TODO: load file
        Mesh {
            position: Vec3::new(0.0, 0.0, 0.0),
            faces: Vec::new(),
        }
    }
}

//Vec<hittable> implements hittable!
impl Hittable for Mesh {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitResult> {
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
}