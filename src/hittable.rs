use std::fmt::Debug;

use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Debug)]
pub struct HitResult {
    pub ray_param: f64,
    pub hit_position: Vec3,
    pub normal: Vec3,
    pub material: Box<dyn Material>,
}

pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitResult>;
}

// hit a list of hittables
impl Hittable for Vec<Box<dyn Hittable>> {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitResult> {
        let mut closest = t_max;
        let mut result = None;

        for object in self {
            if let Some(hit) = object.hit(ray, t_min, closest) {
                closest = hit.ray_param;
                result = Some(hit);
            }
        }

        result
    }
}
